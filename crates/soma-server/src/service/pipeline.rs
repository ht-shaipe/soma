/// 视频生成流水线完整实现模块
///
/// 实现6步视频生成流水线的完整执行逻辑：
/// 1. generate_script   - 生成视频脚本（若未提供则调用 LLM 生成）
/// 2. generate_terms    - 提取素材搜索关键词（本地素材时跳过）
/// 3. generate_audio    - 使用 Edge TTS 将脚本转为语音
/// 4. generate_subtitle - 根据音频和脚本生成字幕文件
/// 5. get_video_materials - 下载/获取视频素材（支持本地和在线素材源）
/// 6. generate_final_videos - 合成最终视频（拼接素材 + 音频 + 字幕 + BGM）
///
/// 支持通过 stop_at 参数在任意步骤后停止，方便调试和预览中间结果。

use soma_core::error::SomaError;
use soma_core::models::{TaskStatus, VideoParams};
use crate::state::{self, TaskUpdateData};
use crate::Config;

/// 执行完整的视频生成任务
///
/// 按照6步流水线依次执行，每步完成后更新任务进度。
/// 通过 stop_at 参数可在指定步骤后停止，支持 "script"、"terms"、"audio"、
/// "subtitle"、"materials" 等值，方便调试和逐步预览。
///
/// 参数：
/// - `task_id`: 任务唯一 ID
/// - `params`: 视频生成参数
/// - `stop_at`: 停止步骤名称（空字符串表示执行全部步骤）
///
/// 返回：成功返回 Ok(())，任一步骤失败返回 SomaError
pub fn run_task(task_id: &str, params: &VideoParams, stop_at: &str) -> Result<(), SomaError> {
    let conf = Config::get();

    // 标记任务为处理中，初始进度 5%
    state::update_task(task_id, Some(TaskStatus::Processing.as_i32()), Some(5));

    // ===== 第1步：生成视频脚本 =====
    let video_script = generate_script(task_id, params)?;
    state::update_task(task_id, None, Some(10));

    // 若指定停在脚本步骤，保存脚本并完成
    if stop_at == "script" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            script: Some(video_script.clone()),
            ..Default::default()
        });
        return Ok(());
    }

    // ===== 第2步：提取素材搜索关键词 =====
    // 若视频来源为本地素材，则跳过关键词提取
    let video_terms = if params.video_source.as_deref() != Some("local") {
        let terms = generate_terms(task_id, params, &video_script)?;
        state::update_task_data(task_id, &TaskUpdateData {
            terms: Some(terms.clone()),
            ..Default::default()
        });
        terms
    } else {
        vec![]
    };

    // 若指定停在关键词步骤，保存脚本和关键词并完成
    if stop_at == "terms" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            script: Some(video_script.clone()),
            terms: Some(video_terms.clone()),
            ..Default::default()
        });
        return Ok(());
    }

    state::update_task(task_id, None, Some(20));

    // ===== 第3步：生成语音音频 =====
    let (audio_file, audio_duration) = generate_audio(task_id, params, &video_script, &conf)?;
    state::update_task_data(task_id, &TaskUpdateData {
        audio_file: Some(audio_file.clone()),
        audio_duration: Some(audio_duration),
        ..Default::default()
    });

    // 若指定停在音频步骤，完成
    if stop_at == "audio" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            ..Default::default()
        });
        return Ok(());
    }

    state::update_task(task_id, None, Some(30));

    // ===== 第4步：生成字幕文件 =====
    let subtitle_path = generate_subtitle(task_id, params, &video_script, &audio_file, &conf)?;
    state::update_task_data(task_id, &TaskUpdateData {
        subtitle_path: Some(subtitle_path.clone()),
        ..Default::default()
    });

    // 若指定停在字幕步骤，完成
    if stop_at == "subtitle" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            ..Default::default()
        });
        return Ok(());
    }

    state::update_task(task_id, None, Some(40));

    // ===== 第5步：获取视频素材 =====
    let materials = get_video_materials(task_id, params, &video_terms, audio_duration, &conf)?;
    state::update_task_data(task_id, &TaskUpdateData {
        materials: Some(materials.clone()),
        ..Default::default()
    });

    // 若指定停在素材步骤，完成
    if stop_at == "materials" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            ..Default::default()
        });
        return Ok(());
    }

    state::update_task(task_id, None, Some(50));

    // ===== 第6步：合成最终视频 =====
    let (final_videos, combined_videos) = generate_final_videos(task_id, params, &materials, &audio_file, &subtitle_path, &conf)?;

    // ===== 第7步：自动跨平台发布（如果配置了） =====
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

    // 全部完成，更新任务状态和数据
    state::update_task_data(task_id, &TaskUpdateData {
        state: Some(TaskStatus::Completed.as_i32()),
        progress: Some(100),
        script: Some(video_script),
        terms: Some(video_terms),
        videos: Some(final_videos),
        combined_videos: Some(combined_videos),
        ..Default::default()
    });

    Ok(())
}

/// 生成视频脚本
///
/// 若 params 中已提供脚本内容则直接使用，否则调用 LLM 异步生成。
/// 由于此函数在同步线程中调用，使用 tokio::Runtime::new() 创建临时运行时。
///
/// 参数：
/// - `_task_id`: 任务 ID（保留参数，当前未使用）
/// - `params`: 视频参数，可能包含预设脚本
///
/// 返回：脚本文本内容
fn generate_script(_task_id: &str, params: &VideoParams) -> Result<String, SomaError> {
    let script = params.video_script.trim().to_string();
    if !script.is_empty() {
        return Ok(script);
    }

    let conf = Config::get();
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
        let fut = super::llm::generate_script(provider, &params.video_subject, language, paragraph_number, prompt, system_prompt, &conf);
        rt.block_on(fut)
    })
}

/// 提取素材搜索关键词
///
/// 若 params 中已提供关键词（JSON 数组或逗号分隔字符串）则直接解析使用，
/// 否则调用 LLM 从脚本中提取。关键词数量根据 match_materials_to_script 选项决定。
///
/// 参数：
/// - `_task_id`: 任务 ID（保留参数）
/// - `params`: 视频参数，可能包含预设关键词
/// - `script`: 视频脚本内容
///
/// 返回：关键词列表
fn generate_terms(_task_id: &str, params: &VideoParams, script: &str) -> Result<Vec<String>, SomaError> {
    // 尝试使用用户预设的关键词
    if let Some(ref terms) = params.video_terms {
        // JSON 数组格式
        if let Some(arr) = terms.as_array() {
            return Ok(arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());
        }
        // 逗号分隔字符串格式
        if let Some(s) = terms.as_str() {
            return Ok(s.split(&[',', '，'][..]).map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect());
        }
    }

    // 用户未提供关键词，调用 LLM 提取
    let conf = Config::get();
    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");
    // 根据脚本匹配选项决定关键词数量：匹配模式8个，普通模式5个
    let amount = if params.match_materials_to_script.unwrap_or(false) { 8 } else { 5 };

    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Llm(e.to_string()))?;
    retry(5, || {
        let fut = super::llm::generate_terms(provider, &params.video_subject, script, amount, &conf);
        rt.block_on(fut)
    })
}

/// 使用 Edge TTS 生成语音音频
///
/// 将视频脚本通过 Edge TTS 转换为 MP3 音频文件，并返回文件路径和时长。
/// 生成失败时将任务状态标记为 Failed。
///
/// 参数：
/// - `task_id`: 任务 ID
/// - `params`: 视频参数（包含语音名称和语速设置）
/// - `script`: 要转换的脚本文本
/// - `conf`: 全局配置（包含 TTS 超时设置）
///
/// 返回：(音频文件路径, 音频时长秒数) 元组
fn generate_audio(task_id: &str, params: &VideoParams, script: &str, conf: &crate::Config) -> Result<(String, f64), SomaError> {
    let task_audio_dir = soma_core::utils::task_dir(task_id);
    let audio_file = task_audio_dir.join("audio.mp3").to_string_lossy().to_string();

    let voice_name = params.get_voice_name();
    let rate = params.get_voice_rate();

    if let Some(parent) = std::path::Path::new(&audio_file).parent() {
        std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
    }

    // 根据语音名称前缀路由到对应 TTS 引擎
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
        // 默认使用 EdgeTTS
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

/// 生成字幕文件
///
/// 根据字幕提供商配置，选择 Whisper 或 Edge TTS 方式生成 SRT 字幕文件。
/// 若用户未启用字幕，直接返回空字符串。
///
/// 参数：
/// - `task_id`: 任务 ID
/// - `params`: 视频参数（包含字幕开关和线程数设置）
/// - `script`: 视频脚本文本
/// - `audio_file`: 音频文件路径
/// - `conf`: 全局配置（包含字幕提供商和 FFmpeg 配置）
///
/// 返回：字幕文件路径（未启用字幕时为空字符串）
fn generate_subtitle(task_id: &str, params: &VideoParams, script: &str, audio_file: &str, conf: &crate::Config) -> Result<String, SomaError> {
    // 检查字幕是否启用
    if !params.get_subtitle_enabled() {
        return Ok(String::new());
    }

    let task_dir = soma_core::utils::task_dir(task_id);
    let subtitle_path = task_dir.join("subtitle.srt").to_string_lossy().to_string();

    let subtitle_provider = conf.app.get_subtitle_provider();
    if subtitle_provider == "whisper" {
        // Whisper 模式：先识别音频生成字幕，再用脚本文本校正
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
        // Edge TTS 模式：根据文本和音频时长均匀分配字幕时间轴
        let ffmpeg = soma_video::Ffmpeg::new(&conf.app.get_ffmpeg_binary(), params.get_n_threads(), params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()));
        let audio_dur = ffmpeg.get_audio_duration(audio_file)?;
        let cues = soma_tts::edge_tts::generate_subtitle_cues_from_text(script, audio_dur);
        soma_tts::subtitle::create_subtitle_file(&cues, &subtitle_path)?;
    }

    Ok(subtitle_path)
}

/// 获取视频素材
///
/// 根据视频来源配置，从本地目录或在线素材网站（Pexels、Pixabay、Coverr）
/// 下载/获取视频素材文件。本地素材会通过 FFmpeg 预处理为统一格式。
///
/// 参数：
/// - `task_id`: 任务 ID（在线下载时用于创建任务目录）
/// - `params`: 视频参数（包含素材来源、宽高比、片段时长等设置）
/// - `terms`: 搜索关键词列表
/// - `audio_duration`: 音频时长（用于计算所需素材总时长）
/// - `conf`: 全局配置（包含 API Key 和素材目录设置）
///
/// 返回：素材文件路径列表
fn get_video_materials(task_id: &str, params: &VideoParams, terms: &[String], audio_duration: f64, conf: &crate::Config) -> Result<Vec<String>, SomaError> {
    let source = params.video_source.as_deref().unwrap_or("pexels");
    let aspect = params.get_video_aspect();
    let clip_dur = params.get_clip_duration();

    if source == "local" {
        // 本地素材模式：通过 FFmpeg 预处理本地素材文件
        let ffmpeg = soma_video::Ffmpeg::new(&conf.app.get_ffmpeg_binary(), params.get_n_threads(), params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()));
        let materials = params.video_materials.as_deref().unwrap_or(&[]);
        return ffmpeg.preprocess_local_materials(materials, clip_dur, &aspect);
    }

    // 在线素材模式：读取各素材网站的 API Key
    let pexels_keys = conf.app.app.pexels_api_keys.clone().unwrap_or_default();
    let pixabay_keys = conf.app.app.pixabay_api_keys.clone().unwrap_or_default();
    let coverr_keys = conf.app.app.coverr_api_keys.clone().unwrap_or_default();
    let material_dir = conf.app.app.material_directory.clone().unwrap_or_default();

    // 异步下载在线素材（总时长 = 音频时长 × 视频数量）
    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Stock(e.to_string()))?;
    rt.block_on(
        soma_stock::download_videos(
            task_id, terms, source, &aspect,
            audio_duration * params.get_video_count() as f64,
            clip_dur, &pexels_keys, &pixabay_keys, &coverr_keys, &material_dir,
        )
    )
}

/// 合成最终视频
///
/// 对每个视频副本执行：拼接素材 → 添加 BGM → 添加音频和字幕，
/// 生成最终视频文件。进度从 50% 递增到 100%。
///
/// 参数：
/// - `task_id`: 任务 ID
/// - `params`: 视频参数（包含宽高比、片段时长、BGM 类型、视频数量等）
/// - `materials`: 视频素材文件路径列表
/// - `audio_file`: 音频文件路径
/// - `subtitle_path`: 字幕文件路径
/// - `conf`: 全局配置（包含 FFmpeg 路径和编解码设置）
///
/// 返回：(最终视频路径列表, 合成视频路径列表) 元组
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
    let mut progress = 50u32;

    // 为每个视频副本执行合成流程
    for i in 1..=video_count {
        let task_dir_path = soma_core::utils::task_dir(task_id);
        let combined_path = task_dir_path.join(format!("combined-{}.mp4", i)).to_string_lossy().to_string();
        let final_path = task_dir_path.join(format!("final-{}.mp4", i)).to_string_lossy().to_string();

        // 步骤1：拼接素材视频片段为合成视频
        composer.combine_videos(materials, audio_file, &combined_path, &aspect, clip_dur, transition_mode)?;

        progress += (50 / video_count / 2).max(1);
        state::update_task(task_id, None, Some(progress));

        // 步骤2：获取 BGM 文件（随机或指定）
        let bgm_file = composer.get_bgm_file(
            params.bgm_type.as_deref().unwrap_or("random"),
            params.bgm_file.as_deref().unwrap_or(""),
        );

        let mut final_params = params.clone();
        if !bgm_file.is_empty() {
            final_params.bgm_file = Some(bgm_file);
        }

        // 步骤3：生成最终视频（合成视频 + 音频 + 字幕 + BGM）
        composer.generate_video(&combined_path, audio_file, subtitle_path, &final_path, &final_params)?;

        // 步骤4：叠加水印（如果配置了）
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

        // 步骤5：拼接片头/片尾（如果配置了）
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

        progress += (50 / video_count / 2).max(1);
        state::update_task(task_id, None, Some(progress));

        final_videos.push(final_path_with_intro_outro);
        combined_videos.push(combined_path);
    }

    Ok((final_videos, combined_videos))
}

/// 通用重试包装函数
///
/// 对可能失败的操作进行多次重试，每次失败后等待指数退避时间。
/// 适用于 LLM API 调用（网络波动、限流）和 TTS 合成等不稳定操作。
///
/// - `max_retries`: 最大重试次数（不含首次执行）
/// - `f`: 待执行的操作闭包
///
/// 返回：首次成功的结果，或最后一次的错误
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
