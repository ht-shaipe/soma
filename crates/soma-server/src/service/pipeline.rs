/// 视频生成流水线编排器
///
/// 0.1.2 起各步骤已功能点化（见 soma-feature::features），
/// 本模块退化为"功能点的默认编排"：
/// 1. llm.intent        - 需求理解：LLM 解析/提炼用户意图
/// 2. llm.script        - 文案/剧情：LLM 创作扩展脚本文案
/// 3. llm.storyboard    - 分镜脚本+提示词：LLM 拆分场景并生成视觉提示词
///    llm.narration     - 旁白文案：脚本转口语化朗读文本
/// 4. material.generate - 素材生成：本地/在线/AI 视频素材
/// 5. tts.synthesize + subtitle.generate - 音频与字幕
/// 6. video.compose     - 合成输出：FFmpeg 拼接
///
/// 编排器职责：
/// - 任务上下文管理（断点续跑：已有步骤产物直接复用）
/// - stop_at 断点控制（在任意步骤后停止，方便调试和预览中间结果）
/// - 功能点输出 → TaskInfo 字段的回写
///
/// 兼容说明：generate_audio / generate_audio_to / get_video_materials
/// 保留原签名供数字人服务与 handler 层调用，内部委托功能点层实现。

use soma_core::error::SomaError;
use soma_core::models::{StoryboardScene, TaskStatus, VideoParams, AiVideoSegmentLog};
use soma_feature::features::compose::VideoComposeOutput;
use soma_feature::features::llm::{
    derive_terms, LlmIntentOutput, LlmNarrationOutput, LlmScriptOutput, LlmStoryboardOutput,
};
use soma_feature::features::materials::MaterialGenerateOutput;
use soma_feature::features::subtitle::SubtitleGenerateOutput;
use soma_feature::features::tts::TtsSynthesizeOutput;
use soma_feature::progress::{FeatureProgress, FnReporter, NoopProgress};
use crate::state::{self, TaskUpdateData};
use crate::Config;

pub fn run_task(task_id: &str, params: &VideoParams, stop_at: &str) -> Result<(), SomaError> {
    let conf = Config::get();
    let task_dir = soma_core::utils::task_dir(task_id);
    let task_dir_str = task_dir.to_string_lossy().to_string();

    state::update_task(task_id, Some(TaskStatus::Processing.as_i32()), Some(2));

    // 读取已有任务数据，跳过已完成的步骤
    let task_data = state::get_task(task_id);

    // ===== 第1步：需求理解 =====
    let intent = if let Some(ref t) = task_data {
        if t.script.is_some() || t.storyboard.is_some() {
            let mut intent_val = serde_json::json!({});
            if let Some(ref style) = params.intent_style {
                intent_val["style"] = serde_json::Value::String(style.clone());
            }
            if let Some(ref mood) = params.intent_mood {
                intent_val["mood"] = serde_json::Value::String(mood.clone());
            }
            state::update_task(task_id, None, Some(5));
            intent_val
        } else {
            let out: LlmIntentOutput = serde_json::from_value(super::registry::run_feature(
                "llm.intent",
                task_id,
                serde_json::json!({
                    "video_subject": params.video_subject,
                    "language": params.video_language,
                    "aspect_ratio": params.video_aspect,
                    "style": params.intent_style,
                    "mood": params.intent_mood,
                }),
                &conf.app,
                &NoopProgress,
            )?)?;
            out.intent
        }
    } else {
        let out: LlmIntentOutput = serde_json::from_value(super::registry::run_feature(
            "llm.intent",
            task_id,
            serde_json::json!({
                "video_subject": params.video_subject,
                "language": params.video_language,
                "aspect_ratio": params.video_aspect,
                "style": params.intent_style,
                "mood": params.intent_mood,
            }),
            &conf.app,
            &NoopProgress,
        )?)?;
        out.intent
    };
    state::update_task(task_id, None, Some(5));

    if stop_at == "intent" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            script: Some(intent.to_string()),
            ..Default::default()
        });
        return Ok(());
    }

    // ===== 第2步：文案/剧情 =====
    let video_script = if let Some(ref t) = task_data {
        if let Some(ref script) = t.script {
            script.clone()
        } else {
            step_script(task_id, params, &intent, &conf)?
        }
    } else {
        step_script(task_id, params, &intent, &conf)?
    };

    if stop_at == "script" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            ..Default::default()
        });
        return Ok(());
    }

    state::update_task(task_id, None, Some(15));

    // ===== 第3步：分镜脚本 + 提示词 =====
    let storyboard = if let Some(ref t) = task_data {
        if let Some(ref sb) = t.storyboard {
            if !sb.is_empty() {
                sb.clone()
            } else if params.video_source.as_deref() == Some("local") {
                vec![]
            } else {
                step_storyboard(task_id, params, &video_script, &intent, &conf)?
            }
        } else if params.video_source.as_deref() == Some("local") {
            vec![]
        } else {
            step_storyboard(task_id, params, &video_script, &intent, &conf)?
        }
    } else if params.video_source.as_deref() == Some("local") {
        vec![]
    } else {
        step_storyboard(task_id, params, &video_script, &intent, &conf)?
    };

    let video_terms: Vec<String> = if let Some(ref t) = task_data {
        if let Some(ref terms) = t.terms {
            if !terms.is_empty() {
                terms.clone()
            } else {
                derive_terms(&storyboard)
            }
        } else {
            derive_terms(&storyboard)
        }
    } else {
        derive_terms(&storyboard)
    };

    state::update_task_data(task_id, &TaskUpdateData {
        terms: Some(video_terms.clone()),
        storyboard: Some(storyboard.clone()),
        ..Default::default()
    });

    if stop_at == "terms" || stop_at == "storyboard" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            ..Default::default()
        });
        return Ok(());
    }

    state::update_task(task_id, None, Some(25));

    // ===== 第3.5步：旁白文案生成 =====
    let narration_text = if let Some(ref t) = task_data {
        if let Some(ref n) = t.narration {
            if !n.is_empty() {
                log::info!("task {}: 使用已有旁白文案 ({}字)", task_id, n.len());
                n.clone()
            } else {
                log::info!("task {}: 开始生成旁白文案...", task_id);
                step_narration(task_id, params, &video_script, &storyboard, &conf)?
            }
        } else {
            log::info!("task {}: 开始生成旁白文案(narration为None)...", task_id);
            step_narration(task_id, params, &video_script, &storyboard, &conf)?
        }
    } else {
        log::info!("task {}: 开始生成旁白文案(无task_data)...", task_id);
        step_narration(task_id, params, &video_script, &storyboard, &conf)?
    };
    log::info!("task {}: 旁白文案生成完成 ({}字)", task_id, narration_text.len());
    state::update_task_data(task_id, &TaskUpdateData {
        narration: Some(narration_text.clone()),
        ..Default::default()
    });

    if stop_at == "narration" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            ..Default::default()
        });
        return Ok(());
    }

    state::update_task(task_id, None, Some(30));

    // ===== 第4步：素材生成 =====
    let (materials, ai_video_logs) = if let Some(ref t) = task_data {
        if let Some(ref mats) = t.materials {
            if !mats.is_empty() {
                (mats.clone(), vec![])
            } else {
                step_materials(task_id, params, &video_terms, 0.0, &conf, &task_dir_str)?
            }
        } else {
            step_materials(task_id, params, &video_terms, 0.0, &conf, &task_dir_str)?
        }
    } else {
        step_materials(task_id, params, &video_terms, 0.0, &conf, &task_dir_str)?
    };
    if !ai_video_logs.is_empty() {
        state::update_task_data(task_id, &TaskUpdateData {
            ai_video_logs: Some(ai_video_logs),
            ..Default::default()
        });
    }
    state::update_task_data(task_id, &TaskUpdateData {
        materials: Some(materials.clone()),
        ..Default::default()
    });

    if stop_at == "materials" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            ..Default::default()
        });
        return Ok(());
    }

    state::update_task(task_id, None, Some(55));

    // ===== 第5步：音频生成 + 字幕 =====
    let tts_text = if narration_text.is_empty() {
        if storyboard.is_empty() {
            video_script.clone()
        } else {
            storyboard.iter()
                .map(|s| s.narration.as_str())
                .collect::<Vec<&str>>()
                .join(" ")
        }
    } else {
        narration_text.clone()
    };

    let (audio_file, audio_duration) = if let Some(ref t) = task_data {
        if let Some(ref af) = t.audio_file {
            if !af.is_empty() && std::path::Path::new(af).exists() {
                let dur = t.audio_duration.unwrap_or(0.0);
                (af.clone(), dur)
            } else {
                step_tts(task_id, params, &tts_text, &conf, &task_dir)?
            }
        } else {
            step_tts(task_id, params, &tts_text, &conf, &task_dir)?
        }
    } else {
        step_tts(task_id, params, &tts_text, &conf, &task_dir)?
    };
    state::update_task_data(task_id, &TaskUpdateData {
        audio_file: Some(audio_file.clone()),
        audio_duration: Some(audio_duration),
        ..Default::default()
    });

    if stop_at == "audio" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            ..Default::default()
        });
        return Ok(());
    }

    let subtitle_path = if let Some(ref t) = task_data {
        if let Some(ref sp) = t.subtitle_path {
            if !sp.is_empty() && std::path::Path::new(sp).exists() {
                sp.clone()
            } else {
                step_subtitle(task_id, params, &tts_text, &audio_file, &conf, &task_dir)?
            }
        } else {
            step_subtitle(task_id, params, &tts_text, &audio_file, &conf, &task_dir)?
        }
    } else {
        step_subtitle(task_id, params, &tts_text, &audio_file, &conf, &task_dir)?
    };
    state::update_task_data(task_id, &TaskUpdateData {
        subtitle_path: Some(subtitle_path.clone()),
        ..Default::default()
    });

    if stop_at == "subtitle" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            ..Default::default()
        });
        return Ok(());
    }

    state::update_task(task_id, None, Some(70));

    // ===== 第6步：合成最终视频 =====
    // 功能点内部进度（0~99）映射到任务的 70~99 区间
    let task_id_for_progress = task_id.to_string();
    let compose_reporter = FnReporter(move |p: FeatureProgress| {
        let mapped = 70 + (p.percent.min(100) as u32) * 29 / 100;
        state::update_task(&task_id_for_progress, None, Some(mapped));
    });
    let out: VideoComposeOutput = serde_json::from_value(super::registry::run_feature(
        "video.compose",
        task_id,
        serde_json::json!({
            "params": params,
            "materials": materials,
            "audio_file": audio_file,
            "subtitle_path": subtitle_path,
            "output_dir": task_dir_str,
        }),
        &conf.app,
        &compose_reporter,
    )?)?;
    let final_videos = out.final_videos;
    let combined_videos = out.combined_videos;

    let ui = &conf.app.ui;
    if super::upload::is_configured(ui) && ui.upload_post_auto_upload.unwrap_or(false) {
        if let Some(ref platforms) = ui.upload_post_platforms {
            if !platforms.is_empty() && !final_videos.is_empty() {
                let title = video_script.chars().take(100).collect::<String>();
                if let Err(e) = block_on_upload(&final_videos[0], &title, platforms, ui) {
                    log::error!("自动发布失败: {:?}", e);
                }
            }
        }
    }

    state::update_task_data(task_id, &TaskUpdateData {
        state: Some(TaskStatus::Completed.as_i32()),
        progress: Some(100),
        script: Some(video_script),
        terms: Some(video_terms),
        storyboard: Some(storyboard),
        videos: Some(final_videos),
        combined_videos: Some(combined_videos),
        ..Default::default()
    });

    Ok(())
}

/// 在阻塞上下文中执行自动发布（upload 服务为 async）
fn block_on_upload(
    video: &str,
    title: &str,
    platforms: &[String],
    ui: &soma_core::config::UiSection,
) -> Result<(), SomaError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| SomaError::Upload(format!("tokio runtime 创建失败: {}", e)))?;
    rt.block_on(async {
        super::upload::upload_video(video, title, platforms, ui, None)
            .await
            .map(|_| ())
    })
}

/// 第2步：文案/剧情（llm.script 功能点）
///
/// 与原实现一致：用户随任务提交的脚本（video_script 非空）直接使用，跳过 LLM。
fn step_script(task_id: &str, params: &VideoParams, intent: &serde_json::Value, conf: &Config) -> Result<String, SomaError> {
    let provided = params.video_script.trim().to_string();
    let script = if !provided.is_empty() {
        provided
    } else {
        let out: LlmScriptOutput = serde_json::from_value(super::registry::run_feature(
            "llm.script",
            task_id,
            serde_json::json!({
                "video_subject": params.video_subject,
                "intent": intent,
                "language": params.video_language,
                "paragraph_number": params.get_paragraph_number(),
                "script_prompt": params.video_script_prompt,
                "system_prompt": params.custom_system_prompt,
                "use_custom_system_prompt": params.get_use_custom_system_prompt(),
            }),
            &conf.app,
            &NoopProgress,
        )?)?;
        out.script
    };
    state::update_task_data(task_id, &TaskUpdateData {
        script: Some(script.clone()),
        ..Default::default()
    });
    Ok(script)
}

/// 第3步：分镜脚本（llm.storyboard 功能点）
fn step_storyboard(task_id: &str, params: &VideoParams, script: &str, intent: &serde_json::Value, conf: &Config) -> Result<Vec<StoryboardScene>, SomaError> {
    // 用户已提供关键词（数组或逗号分隔字符串）时，作为视觉提示词的约束传入
    let user_terms: Vec<String> = match &params.video_terms {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        Some(serde_json::Value::String(s)) => s
            .split(&[',', '，'][..])
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect(),
        _ => vec![],
    };
    let out: LlmStoryboardOutput = serde_json::from_value(super::registry::run_feature(
        "llm.storyboard",
        task_id,
        serde_json::json!({
            "video_subject": params.video_subject,
            "script": script,
            "clip_duration": params.get_clip_duration(),
            "intent": intent,
            "user_terms": user_terms,
        }),
        &conf.app,
        &NoopProgress,
    )?)?;
    Ok(out.storyboard)
}

/// 第3.5步：旁白文案（llm.narration 功能点）
fn step_narration(task_id: &str, params: &VideoParams, script: &str, storyboard: &[StoryboardScene], conf: &Config) -> Result<String, SomaError> {
    let out: LlmNarrationOutput = serde_json::from_value(super::registry::run_feature(
        "llm.narration",
        task_id,
        serde_json::json!({
            "script": script,
            "storyboard": storyboard,
            "style": params.intent_style,
            "mood": params.intent_mood,
            "language": params.video_language,
        }),
        &conf.app,
        &NoopProgress,
    )?)?;
    Ok(out.narration)
}

/// 第4步：素材生成（material.generate 功能点）
///
/// 目录解析与原实现一致：
/// - AI 视频保存到任务目录
/// - 在线素材按配置 material_directory（空→缓存目录 / "task"→任务目录 / 自定义路径）
fn step_materials(task_id: &str, params: &VideoParams, terms: &[String], audio_duration: f64, conf: &Config, task_dir_str: &str) -> Result<(Vec<String>, Vec<AiVideoSegmentLog>), SomaError> {
    let source = params.video_source.as_deref().unwrap_or("pexels");
    let material_dir = conf.app.app.material_directory.clone().unwrap_or_default();
    let is_ai_branch = (params.portrait_image.is_some() && source != "local")
        || matches!(source, "cogvideox" | "kling" | "minimax");

    let (ai_output_dir, download_dir) = if is_ai_branch {
        (Some(task_dir_str.to_string()), None)
    } else if source == "local" {
        (None, None)
    } else if material_dir == "task" {
        (None, Some(task_dir_str.to_string()))
    } else if material_dir.is_empty() {
        (None, None)
    } else {
        (None, Some(material_dir.clone()))
    };

    let out: MaterialGenerateOutput = serde_json::from_value(super::registry::run_feature(
        "material.generate",
        task_id,
        serde_json::json!({
            "terms": terms,
            "source": params.video_source,
            "aspect": params.video_aspect,
            "clip_duration": params.get_clip_duration(),
            "portrait_image": params.portrait_image,
            "local_materials": params.video_materials.clone().unwrap_or_default(),
            "audio_duration": audio_duration,
            "video_count": params.get_video_count(),
            "n_threads": params.get_n_threads(),
            "video_encoder": params.video_encoder,
            "ai_output_dir": ai_output_dir,
            "download_dir": download_dir,
        }),
        &conf.app,
        &NoopProgress,
    )?)?;
    Ok((out.materials, out.ai_video_logs))
}

/// 第5步（音频）：TTS 合成（tts.synthesize 功能点），输出到任务目录 audio.mp3
fn step_tts(task_id: &str, params: &VideoParams, tts_text: &str, conf: &Config, task_dir: &std::path::Path) -> Result<(String, f64), SomaError> {
    let out: TtsSynthesizeOutput = serde_json::from_value(super::registry::run_feature(
        "tts.synthesize",
        task_id,
        serde_json::json!({
            "text": tts_text,
            "voice_name": params.voice_name,
            "voice_rate": params.voice_rate,
            "tts_provider": params.tts_provider,
            "clone_reference_audio": params.clone_reference_audio,
            "clone_reference_text": params.clone_reference_text,
            "clone_model": params.clone_model,
            "merchant_id": params.merchant_id,
            "output_path": task_dir.join("audio.mp3").to_string_lossy().to_string(),
        }),
        &conf.app,
        &NoopProgress,
    )?)?;
    Ok((out.audio_file, out.audio_duration))
}

/// 第5步（字幕）：字幕生成（subtitle.generate 功能点），输出到任务目录 subtitle.srt
fn step_subtitle(task_id: &str, params: &VideoParams, tts_text: &str, audio_file: &str, conf: &Config, task_dir: &std::path::Path) -> Result<String, SomaError> {
    let out: SubtitleGenerateOutput = serde_json::from_value(super::registry::run_feature(
        "subtitle.generate",
        task_id,
        serde_json::json!({
            "audio_file": audio_file,
            "text": tts_text,
            "subtitle_enabled": params.get_subtitle_enabled(),
            "n_threads": params.get_n_threads(),
            "video_encoder": params.video_encoder,
            "output_path": task_dir.join("subtitle.srt").to_string_lossy().to_string(),
        }),
        &conf.app,
        &NoopProgress,
    )?)?;
    Ok(out.subtitle_path)
}

/// 兼容入口：TTS 合成到任务目录（数字人服务与 handler 层使用）
///
/// 行为与原实现一致：失败时同步把任务标记为 Failed。
pub fn generate_audio(task_id: &str, params: &VideoParams, script: &str, conf: &crate::Config) -> Result<(String, f64), SomaError> {
    let task_audio_dir = soma_core::utils::task_dir(task_id);
    let audio_file = task_audio_dir.join("audio.mp3").to_string_lossy().to_string();
    generate_audio_to(task_id, params, script, &audio_file, conf)
}

/// 兼容入口：TTS 合成到指定路径（数字人服务使用）
///
/// 行为与原实现一致：失败时同步把任务标记为 Failed。
pub fn generate_audio_to(task_id: &str, params: &VideoParams, script: &str, output_path: &str, conf: &crate::Config) -> Result<(String, f64), SomaError> {
    let result = soma_feature::features::tts::synthesize_to(
        &conf.app,
        script,
        params.get_voice_name(),
        params.get_voice_rate(),
        params.get_tts_provider(),
        params.clone_reference_audio.as_deref().unwrap_or(""),
        params.clone_reference_text.as_deref().unwrap_or(""),
        params.clone_model.as_deref().unwrap_or(""),
        params.get_merchant_id(),
        output_path,
    );
    result.map_err(|e| {
        state::update_task(task_id, Some(TaskStatus::Failed.as_i32()), None);
        e
    })
}

/// 兼容入口：素材生成（handler 层的独立素材获取使用）
///
/// 目录解析与原实现一致（AI 视频保存到任务目录，在线素材按配置解析）。
pub fn get_video_materials(task_id: &str, params: &VideoParams, terms: &[String], audio_duration: f64, conf: &crate::Config) -> Result<Vec<String>, SomaError> {
    let task_dir = soma_core::utils::task_dir(task_id);
    let (materials, _) = step_materials(task_id, params, terms, audio_duration, conf, &task_dir.to_string_lossy())?;
    Ok(materials)
}
