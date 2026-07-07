/// 视频生成流水线完整实现模块
///
/// 实现6步视频生成流水线（对应7步业务流程）：
/// 1. generate_intent    - 需求理解：LLM 解析/提炼用户意图
/// 2. generate_script    - 文案/剧情：LLM 创作扩展脚本文案
/// 3. generate_storyboard - 分镜脚本+提示词：LLM 拆分场景并生成视觉提示词
/// 4. get_video_materials - 素材生成：根据分镜视觉提示词获取图片/视频素材
/// 5. generate_audio_and_subtitle - 音频生成：TTS语音 + 字幕
/// 6. generate_final_videos - 合成输出：FFmpeg 拼接
///
/// 支持通过 stop_at 参数在任意步骤后停止，方便调试和预览中间结果。

use soma_core::error::SomaError;
use soma_core::models::{StoryboardScene, TaskStatus, VideoParams};
use crate::state::{self, TaskUpdateData};
use crate::Config;

pub fn run_task(task_id: &str, params: &VideoParams, stop_at: &str) -> Result<(), SomaError> {
    let conf = Config::get();

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
            generate_intent(task_id, params, &conf)?
        }
    } else {
        generate_intent(task_id, params, &conf)?
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
            let s = generate_script(task_id, params, &intent, &conf)?;
            state::update_task_data(task_id, &TaskUpdateData {
                script: Some(s.clone()),
                ..Default::default()
            });
            s
        }
    } else {
        let s = generate_script(task_id, params, &intent, &conf)?;
        state::update_task_data(task_id, &TaskUpdateData {
            script: Some(s.clone()),
            ..Default::default()
        });
        s
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
                generate_storyboard_from_script(task_id, params, &video_script, &intent, &conf)?
            }
        } else if params.video_source.as_deref() == Some("local") {
            vec![]
        } else {
            generate_storyboard_from_script(task_id, params, &video_script, &intent, &conf)?
        }
    } else if params.video_source.as_deref() == Some("local") {
        vec![]
    } else {
        generate_storyboard_from_script(task_id, params, &video_script, &intent, &conf)?
    };

    let video_terms: Vec<String> = if let Some(ref t) = task_data {
        if let Some(ref terms) = t.terms {
            if !terms.is_empty() {
                terms.clone()
            } else {
                storyboard.iter().map(|s| s.visual_prompt.clone()).collect()
            }
        } else {
            storyboard.iter().map(|s| s.visual_prompt.clone()).collect()
        }
    } else {
        storyboard.iter().map(|s| s.visual_prompt.clone()).collect()
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

    state::update_task(task_id, None, Some(30));

    // ===== 第4步：素材生成 =====
    let materials = if let Some(ref t) = task_data {
        if let Some(ref mats) = t.materials {
            if !mats.is_empty() {
                mats.clone()
            } else {
                get_video_materials(task_id, params, &video_terms, 0.0, &conf)?
            }
        } else {
            get_video_materials(task_id, params, &video_terms, 0.0, &conf)?
        }
    } else {
        get_video_materials(task_id, params, &video_terms, 0.0, &conf)?
    };
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
    let narration_text: String = if storyboard.is_empty() {
        video_script.clone()
    } else {
        storyboard.iter()
            .map(|s| s.narration.as_str())
            .collect::<Vec<&str>>()
            .join(" ")
    };

    let (audio_file, audio_duration) = if let Some(ref t) = task_data {
        if let Some(ref af) = t.audio_file {
            if !af.is_empty() && std::path::Path::new(af).exists() {
                let dur = t.audio_duration.unwrap_or(0.0);
                (af.clone(), dur)
            } else {
                generate_audio(task_id, params, &narration_text, &conf)?
            }
        } else {
            generate_audio(task_id, params, &narration_text, &conf)?
        }
    } else {
        generate_audio(task_id, params, &narration_text, &conf)?
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
                generate_subtitle(task_id, params, &narration_text, &audio_file, &conf)?
            }
        } else {
            generate_subtitle(task_id, params, &narration_text, &audio_file, &conf)?
        }
    } else {
        generate_subtitle(task_id, params, &narration_text, &audio_file, &conf)?
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
    let (final_videos, combined_videos) = generate_final_videos(task_id, params, &materials, &audio_file, &subtitle_path, &conf)?;

    let ui = &conf.app.ui;
    if super::upload::is_configured(ui) && ui.upload_post_auto_upload.unwrap_or(false) {
        if let Some(ref platforms) = ui.upload_post_platforms {
            if !platforms.is_empty() && !final_videos.is_empty() {
                let title = video_script.chars().take(100).collect::<String>();
                let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Upload(e.to_string()))?;
                if let Err(e) = rt.block_on(
                    super::upload::upload_video(&final_videos[0], &title, platforms, ui, None)
                ) {
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

/// 第1步：需求理解 — 提炼用户简短描述为结构化创作参数
///
/// 对应设计文档①需求理解。返回包含 theme/style/mood 等字段的 JSON。
fn generate_intent(_task_id: &str, params: &VideoParams, conf: &Config) -> Result<serde_json::Value, SomaError> {
    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");
    let language = params.video_language.as_deref().unwrap_or("");
    let aspect_ratio = params.video_aspect.as_deref().unwrap_or("9:16");

    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Llm(e.to_string()))?;
    retry(3, || {
        let fut = super::llm::generate_intent(provider, &params.video_subject, language, aspect_ratio, conf);
        rt.block_on(fut)
    })
}

/// 第2步：文案/剧情 — 生成视频脚本
///
/// 若 params 中已提供脚本内容则直接使用，否则调用 LLM 生成。
fn generate_script(_task_id: &str, params: &VideoParams, intent: &serde_json::Value, conf: &Config) -> Result<String, SomaError> {
    let script = params.video_script.trim().to_string();
    if !script.is_empty() {
        return Ok(script);
    }

    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");
    let language = params.video_language.as_deref().unwrap_or("");
    let paragraph_number = params.get_paragraph_number();
    let prompt = params.video_script_prompt.as_deref().unwrap_or("");
    let system_prompt = if params.get_use_custom_system_prompt() {
        params.custom_system_prompt.as_deref().unwrap_or("")
    } else {
        ""
    };

    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Llm(e.to_string()))?;
    retry(5, || {
        let fut = super::llm::generate_script(provider, &params.video_subject, intent, language, paragraph_number, prompt, system_prompt, conf);
        rt.block_on(fut)
    })
}

/// 第3步：分镜脚本 + 提示词生成
///
/// 若用户已提供关键词(video_terms)，转换为简单分镜；
/// 否则调用 LLM 将脚本拆分为分镜场景，同时生成视觉提示词。
fn generate_storyboard_from_script(_task_id: &str, params: &VideoParams, script: &str, intent: &serde_json::Value, conf: &Config) -> Result<Vec<StoryboardScene>, SomaError> {
    if let Some(ref terms) = params.video_terms {
        let term_list = if let Some(arr) = terms.as_array() {
            arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<String>>()
        } else if let Some(s) = terms.as_str() {
            s.split(&[',', '，'][..]).map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect()
        } else {
            vec![]
        };

        if !term_list.is_empty() {
            let clip_dur = params.get_clip_duration();
            let scenes: Vec<StoryboardScene> = term_list.iter().enumerate().map(|(i, term)| {
                StoryboardScene {
                    scene_id: (i + 1) as u32,
                    duration: Some(clip_dur),
                    narration: script.to_string(),
                    visual_desc: None,
                    visual_prompt: term.clone(),
                    camera_movement: Some("static".to_string()),
                    transition: if i + 1 < term_list.len() { Some("cut".to_string()) } else { None },
                    text_overlay: None,
                    mood: intent.get("mood").and_then(|v| v.as_str()).map(String::from),
                }
            }).collect();
            return Ok(scenes);
        }
    }

    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");
    let clip_duration = params.get_clip_duration();

    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Llm(e.to_string()))?;
    retry(3, || {
        let fut = super::llm::generate_storyboard(provider, &params.video_subject, script, clip_duration, intent, conf);
        rt.block_on(fut)
    })
}

/// 第5步：使用 TTS 生成语音音频
///
/// 将脚本文本通过 TTS 转换为 MP3 音频文件，并返回文件路径和时长。
pub fn generate_audio(task_id: &str, params: &VideoParams, script: &str, conf: &crate::Config) -> Result<(String, f64), SomaError> {
    let task_audio_dir = soma_core::utils::task_dir(task_id);
    let audio_file = task_audio_dir.join("audio.mp3").to_string_lossy().to_string();

    let voice_name = params.get_voice_name();
    let rate = params.get_voice_rate();

    if let Some(parent) = std::path::Path::new(&audio_file).parent() {
        std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Tts(e.to_string()))?;
    let result = if soma_tts::voices::is_siliconflow_voice(voice_name) {
        let sf_key = conf.app.siliconflow.api_key.as_deref().unwrap_or("");
        let tts = soma_tts::siliconflow_tts::SiliconflowTts::new(sf_key);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(&tts, script, voice_name, rate, std::path::Path::new(&audio_file));
            rt.block_on(fut)
        })
    } else if soma_tts::voices::is_elevenlabs_voice(voice_name) {
        let el_key = conf.app.elevenlabs.api_key.as_deref().unwrap_or("");
        let el_model = conf.app.elevenlabs.model_id.as_deref().unwrap_or("eleven_multilingual_v2");
        let tts = soma_tts::elevenlabs_tts::ElevenlabsTts::new(el_key, el_model);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(&tts, script, voice_name, rate, std::path::Path::new(&audio_file));
            rt.block_on(fut)
        })
    } else if soma_tts::voices::is_mimo_voice(voice_name) {
        let mimo_key = conf.app.app.mimo_api_key.as_deref().unwrap_or("");
        let mimo_base = conf.app.app.mimo_base_url.as_deref().unwrap_or("");
        let mimo_model = conf.app.app.mimo_tts_model_name.as_deref().unwrap_or("");
        let mimo_style = conf.app.app.mimo_tts_style_prompt.as_deref().unwrap_or("");
        let tts = soma_tts::mimo_tts::MimoTts::new(mimo_key, mimo_base, mimo_model, mimo_style);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(&tts, script, voice_name, rate, std::path::Path::new(&audio_file));
            rt.block_on(fut)
        })
    } else if soma_tts::voices::is_gemini_voice(voice_name) {
        let gemini_key = conf.app.app.gemini_api_key.as_deref().unwrap_or("");
        let gemini_model = conf.app.app.gemini_model_name.as_deref().unwrap_or("gemini-2.5-flash-preview-tts");
        let tts = soma_tts::gemini_tts::GeminiTts::new(gemini_key, "", gemini_model);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(&tts, script, voice_name, rate, std::path::Path::new(&audio_file));
            rt.block_on(fut)
        })
    } else if soma_tts::voices::is_azure_voice(voice_name) {
        let azure_key = conf.app.azure.speech_key.as_deref().unwrap_or("");
        let azure_region = conf.app.azure.speech_region.as_deref().unwrap_or("eastasia");
        let tts = soma_tts::azure_tts::AzureTts::new(azure_key, azure_region);
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(&tts, script, voice_name, rate, std::path::Path::new(&audio_file));
            rt.block_on(fut)
        })
    } else {
        let tts = soma_tts::edge_tts::EdgeTts::new(conf.app.get_edge_tts_timeout());
        retry(3, || {
            let fut = soma_tts::provider::SomaTtsProvider::synthesize(&tts, script, voice_name, rate, std::path::Path::new(&audio_file));
            rt.block_on(fut)
        })
    }.map_err(|e| {
        state::update_task(task_id, Some(TaskStatus::Failed.as_i32()), None);
        e
    })?;

    Ok((result.audio_file, result.audio_duration))
}

/// 第5步（续）：生成字幕文件
///
/// 根据字幕提供商配置，选择 Whisper 或 Edge TTS 方式生成 SRT 字幕文件。
fn generate_subtitle(task_id: &str, params: &VideoParams, script: &str, audio_file: &str, conf: &crate::Config) -> Result<String, SomaError> {
    if !params.get_subtitle_enabled() {
        return Ok(String::new());
    }

    let task_dir = soma_core::utils::task_dir(task_id);
    let subtitle_path = task_dir.join("subtitle.srt").to_string_lossy().to_string();

    let subtitle_provider = conf.app.get_subtitle_provider();
    if subtitle_provider == "whisper" {
        let ws = &conf.app.whisper;
        soma_tts::subtitle::generate_whisper_subtitle(
            audio_file, &subtitle_path,
            ws.model_size.as_deref().unwrap_or("base"),
            ws.device.as_deref().unwrap_or("cpu"),
            ws.compute_type.as_deref().unwrap_or("int8"),
        )?;
        let mut cues = soma_tts::subtitle::file_to_subtitles(&subtitle_path);
        soma_tts::subtitle::correct_subtitle(&mut cues, script);
        soma_tts::subtitle::create_subtitle_file(&cues, &subtitle_path)?;
    } else {
        let ffmpeg = soma_video::Ffmpeg::new(&conf.app.get_ffmpeg_binary(), params.get_n_threads(), params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()));
        let audio_dur = ffmpeg.get_audio_duration(audio_file)?;
        let cues = soma_tts::edge_tts::generate_subtitle_cues_from_text(script, audio_dur);
        soma_tts::subtitle::create_subtitle_file(&cues, &subtitle_path)?;
    }

    Ok(subtitle_path)
}

/// 第4步：获取视频素材
///
/// 根据视频来源配置，从本地目录或在线素材网站下载/获取视频素材文件。
/// 使用分镜的视觉提示词作为搜索关键词。
pub fn get_video_materials(task_id: &str, params: &VideoParams, terms: &[String], audio_duration: f64, conf: &crate::Config) -> Result<Vec<String>, SomaError> {
    let source = params.video_source.as_deref().unwrap_or("pexels");
    let aspect = params.get_video_aspect();
    let clip_dur = params.get_clip_duration();
    let portrait = params.portrait_image.as_deref();

    if portrait.is_some() && source != "local" {
        let ai_source = if source == "cogvideox" || source == "kling" || source == "minimax" {
            source.to_string()
        } else {
            resolve_portrait_ai_source(&conf.app.app)
        };
        let portrait_url = resolve_portrait_url(portrait.unwrap_or(""), &conf);
        let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Stock(e.to_string()))?;
        return rt.block_on(
            soma_stock::generate_ai_videos(
                task_id, terms, &ai_source, &aspect, clip_dur, &conf.app, Some(portrait_url.as_str()),
            )
        );
    }

    if source == "local" {
        let ffmpeg = soma_video::Ffmpeg::new(&conf.app.get_ffmpeg_binary(), params.get_n_threads(), params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()));
        let materials = params.video_materials.as_deref().unwrap_or(&[]);
        return ffmpeg.preprocess_local_materials(materials, clip_dur, &aspect);
    }

    if source == "cogvideox" || source == "kling" || source == "minimax" {
        let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Stock(e.to_string()))?;
        return rt.block_on(
            soma_stock::generate_ai_videos(
                task_id, terms, source, &aspect, clip_dur, &conf.app, None,
            )
        );
    }

    let pexels_keys = conf.app.app.pexels_api_keys.clone().unwrap_or_default();
    let pixabay_keys = conf.app.app.pixabay_api_keys.clone().unwrap_or_default();
    let coverr_keys = conf.app.app.coverr_api_keys.clone().unwrap_or_default();
    let material_dir = conf.app.app.material_directory.clone().unwrap_or_default();

    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Stock(e.to_string()))?;
    rt.block_on(
        soma_stock::download_videos(
            task_id, terms, source, &aspect,
            audio_duration * params.get_video_count() as f64,
            clip_dur, &pexels_keys, &pixabay_keys, &coverr_keys, &material_dir,
        )
    )
}

/// 第6步：合成最终视频
///
/// 对每个视频副本执行：拼接素材 → 添加 BGM → 添加音频和字幕，
/// 生成最终视频文件。进度从 70% 递增到 100%。
fn generate_final_videos(
    task_id: &str,
    params: &VideoParams,
    materials: &[String],
    audio_file: &str,
    subtitle_path: &str,
    conf: &crate::Config,
) -> Result<(Vec<String>, Vec<String>), SomaError> {
    let ffmpeg = soma_video::Ffmpeg::new(&conf.app.get_ffmpeg_binary(), params.get_n_threads(), params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()));
    let composer = soma_video::VideoComposer::new(ffmpeg);
        let aspect = params.get_video_aspect();
        let clip_dur = params.get_clip_duration();
        let video_count = params.get_video_count();
        let transition_mode = params.video_transition_mode.as_deref().unwrap_or("none");

    let mut final_videos = Vec::new();
    let mut combined_videos = Vec::new();
    let mut progress = 70u32;

    for i in 1..=video_count {
        let task_dir_path = soma_core::utils::task_dir(task_id);
        let combined_path = task_dir_path.join(format!("combined-{}.mp4", i)).to_string_lossy().to_string();
        let final_path = task_dir_path.join(format!("final-{}.mp4", i)).to_string_lossy().to_string();

        composer.combine_videos(materials, audio_file, &combined_path, &aspect, clip_dur, transition_mode)?;

        progress += (30 / video_count / 2).max(1);
        state::update_task(task_id, None, Some(progress));

        let bgm_file = composer.get_bgm_file(
            params.bgm_type.as_deref().unwrap_or("random"),
            params.bgm_file.as_deref().unwrap_or(""),
        );

        let mut final_params = params.clone();
        if !bgm_file.is_empty() {
            final_params.bgm_file = Some(bgm_file);
        }

        composer.generate_video(&combined_path, audio_file, subtitle_path, &final_path, &final_params)?;

        let current_path = if let Some(ref wm) = params.video_watermark {
            if !wm.is_empty() && std::path::Path::new(wm).exists() {
                let wm_path = task_dir_path.join(format!("watermarked-{}.mp4", i)).to_string_lossy().to_string();
                composer.ffmpeg().add_watermark(&final_path, wm, &wm_path)?;
                wm_path
            } else {
                final_path.clone()
            }
        } else {
            final_path.clone()
        };

        let final_path_with_intro_outro = if params.video_intro.is_some() || params.video_outro.is_some() {
            let intro = params.video_intro.as_deref().unwrap_or("");
            let outro = params.video_outro.as_deref().unwrap_or("");
            let has_intro = !intro.is_empty() && std::path::Path::new(intro).exists();
            let has_outro = !outro.is_empty() && std::path::Path::new(outro).exists();
            if has_intro || has_outro {
                let io_path = task_dir_path.join(format!("final-io-{}.mp4", i)).to_string_lossy().to_string();
                let mut segments: Vec<&str> = Vec::new();
                if has_intro { segments.push(intro); }
                segments.push(&current_path);
                if has_outro { segments.push(outro); }
                composer.ffmpeg().concat_videos(&segments, &io_path)?;
                io_path
            } else {
                current_path.clone()
            }
        } else {
            current_path.clone()
        };

        progress += (30 / video_count / 2).max(1);
        state::update_task(task_id, None, Some(progress));

        final_videos.push(final_path_with_intro_outro);
        combined_videos.push(combined_path);
    }

    Ok((final_videos, combined_videos))
}

fn retry<F, T>(max_retries: usize, f: F) -> Result<T, SomaError>
where
    F: Fn() -> Result<T, SomaError>,
{
    let mut last_err = None;
    for attempt in 0..=max_retries {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                if attempt < max_retries {
                    let delay = std::time::Duration::from_millis(500 * (1 << attempt) as u64);
                    std::thread::sleep(delay);
                }
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| SomaError::Llm("重试全部失败且无错误记录".into())))
}

fn resolve_portrait_ai_source(conf: &soma_core::config::AppSection) -> String {
    let zhipu_key = conf.zhipu_video_api_key.as_deref().or(conf.zhipu_api_key.as_deref()).unwrap_or("");
    if !zhipu_key.is_empty() {
        return "cogvideox".to_string();
    }
    let kling_ak = conf.kling_access_key.as_deref().unwrap_or("");
    let kling_sk = conf.kling_secret_key.as_deref().unwrap_or("");
    if !kling_ak.is_empty() && !kling_sk.is_empty() {
        return "kling".to_string();
    }
    let minimax_key = conf.minimax_video_api_key.as_deref().unwrap_or("");
    if !minimax_key.is_empty() {
        return "minimax".to_string();
    }
    "cogvideox".to_string()
}

fn resolve_portrait_url(path: &str, conf: &crate::Config) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    let endpoint = conf.app.get_endpoint();
    if endpoint.is_empty() {
        return path.to_string();
    }
    let storage_path = conf.app.get_storage_path();
    let storage_abs = std::path::Path::new(storage_path).canonicalize().unwrap_or_else(|_| std::path::PathBuf::from(storage_path));
    let portrait_abs = std::path::Path::new(path).canonicalize().unwrap_or_else(|_| std::path::PathBuf::from(path));
    if let Ok(rel) = portrait_abs.strip_prefix(&storage_abs) {
        let rel_str = rel.to_string_lossy();
        return format!("{}/storage/{}", endpoint.trim_end_matches('/'), rel_str);
    }
    path.to_string()
}
