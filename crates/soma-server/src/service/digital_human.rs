//! 数字人口播视频生成流水线
//!
//! 编排"配音音频 → 口播视频 → 字幕与背景音乐合成"三阶段，
//! 进度按 30/90/100 上报。复用现有 TTS、字幕、FFmpeg 合成能力。
//!
//! 与 `pipeline` 模块的区别：输入为单张人像照片 + 一段文案，
//! 输出为口播视频（人物开口说话，口型与配音同步），不经过 LLM 文案改写。

use soma_core::error::SomaError;
use soma_core::models::{TaskStatus, VideoParams, DigitalHumanParams};
use crate::state;
use crate::Config;

/// 将数字人参数转换为视频参数，用于复用 TTS 和视频合成能力
pub fn dh_params_to_video_params(params: &DigitalHumanParams) -> VideoParams {
    VideoParams {
        video_subject: "digital_human".to_string(),
        video_script: params.narration_text.clone(),
        video_terms: None,
        video_aspect: params.video_aspect.clone(),
        video_concat_mode: None,
        video_transition_mode: None,
        video_clip_duration: None,
        match_materials_to_script: None,
        video_count: Some(1),
        video_source: None,
        video_materials: None,
        custom_audio_file: None,
        video_language: params.video_language.clone(),
        voice_name: params.voice_name.clone(),
        voice_volume: params.voice_volume,
        voice_rate: params.voice_rate,
        bgm_type: params.bgm_type.clone(),
        bgm_file: params.bgm_file.clone(),
        bgm_volume: params.bgm_volume,
        subtitle_enabled: params.subtitle_enabled,
        subtitle_position: params.subtitle_position.clone(),
        custom_position: params.custom_position,
        font_name: params.font_name.clone(),
        text_fore_color: params.text_fore_color.clone(),
        text_background_color: None,
        rounded_subtitle_background: None,
        font_size: params.font_size,
        stroke_color: params.stroke_color.clone(),
        stroke_width: params.stroke_width,
        paragraph_number: None,
        n_threads: params.n_threads,
        video_script_prompt: None,
        custom_system_prompt: None,
        use_custom_system_prompt: None,
        video_encoder: params.video_encoder.clone(),
        video_watermark: None,
        video_intro: None,
        video_outro: None,
        portrait_image: Some(params.portrait_image.clone()),
        intent_style: None,
        intent_mood: None,
        intent_audience: None,
        tts_provider: params.tts_provider.clone(),
        clone_reference_audio: params.clone_reference_audio.clone(),
        clone_reference_text: params.clone_reference_text.clone(),
        clone_model: params.clone_model.clone(),
        merchant_id: params.merchant_id.clone(),
    }
}


/// 数字人口播视频生成流水线主入口
///
/// 三阶段顺序执行：
/// 1. 配音音频生成（0~30%）：TTS 合成 narration_text
/// 2. 口播视频生成（30~90%）：调用第三方数字人服务（照片+音频→口播视频）
/// 3. 字幕与背景音乐合成（90~100%）：FFmpeg 合并视频+音频+字幕+BGM
pub fn run_task(task_id: &str, params: &DigitalHumanParams) -> Result<(), SomaError> {
    let conf = Config::get();

    log::info!(
        "数字人任务开始: task_id={}, portrait={}, text_len={}",
        task_id, params.portrait_image, params.narration_text.chars().count()
    );

    state::update_dh_task(task_id, Some(TaskStatus::Processing.as_i32()), Some(0));

    let task_dir = soma_core::utils::task_dir(task_id);
    let provider = conf.app.digital_human.get_provider();

    let portrait_path = if provider == "heygem" {
        let merchant_id = params.get_merchant_id().ok_or_else(|| {
            let msg = "HeyGem 数字人需要商户标识（merchant_id）".to_string();
            state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
                state: Some(TaskStatus::Failed.as_i32()),
                error_message: Some(msg.clone()),
                ..Default::default()
            });
            SomaError::Config(msg)
        })?;
        let hg_conf = &conf.app.digital_human.heygem;
        let asset_store = crate::service::heygem_merchant::MerchantAssetStore::new(
            hg_conf.get_assets_dir(),
        );
        if !asset_store.check_ready(merchant_id)? {
            let msg = format!("商户 {} 模型未就绪，请先完成模型训练", merchant_id);
            state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
                state: Some(TaskStatus::Failed.as_i32()),
                error_message: Some(msg.clone()),
                ..Default::default()
            });
            return Err(SomaError::VideoGen(msg));
        }
        let asset = asset_store.get_asset(merchant_id)?;
        state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
            merchant_id: Some(merchant_id.to_string()),
            ..Default::default()
        });
        std::path::PathBuf::from(asset.silent_video_path)
    } else {
        let p = soma_core::utils::storage_dir("portraits", false)
            .join(&params.portrait_image);
        if !p.exists() {
            let msg = "人像照片不存在，请重新上传".to_string();
            log::error!("数字人任务失败: task_id={}, {}", task_id, msg);
            state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
                state: Some(TaskStatus::Failed.as_i32()),
                error_message: Some(msg.clone()),
                ..Default::default()
            });
            return Err(SomaError::Stock(msg));
        }
        p
    };

    if provider == "echomimic_v3" || provider == "heygem" {
        match crate::service::segment_dh_video::run_segment_flow(
            task_id, params, &portrait_path, &params.narration_text, &conf,
        ) {
            Ok(outcome) => {
                log::info!("数字人任务完成: task_id={}, final={}", task_id, task_dir.join("final.mp4").display());
                let _ = outcome;
                return Ok(());
            }
            Err(e) => {
                let msg = format!("{:?}", e);
                state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
                    state: Some(TaskStatus::Failed.as_i32()),
                    error_message: Some(msg.clone()),
                    ..Default::default()
                });
                return Err(e);
            }
        }
    }


    let portrait_video_path = task_dir.join("portrait_video.mp4").to_string_lossy().to_string();
    let final_video_path = task_dir.join("final.mp4").to_string_lossy().to_string();

    let video_params = dh_params_to_video_params(params);

    let existing = state::get_dh_task(task_id);

    // 阶段1：配音音频生成（0~30%）
    let audio_file = if let Some(ref t) = existing {
        if let Some(ref af) = t.audio_file {
            if std::path::Path::new(af).exists() {
                log::info!("数字人任务跳过阶段1（音频已存在）: task_id={}", task_id);
                af.clone()
            } else {
                generate_audio_stage(task_id, params, &video_params, &conf)?
            }
        } else {
            generate_audio_stage(task_id, params, &video_params, &conf)?
        }
    } else {
        generate_audio_stage(task_id, params, &video_params, &conf)?
    };

    let audio_duration = soma_video::Ffmpeg::new(
        &conf.app.get_ffmpeg_binary(),
        video_params.get_n_threads(),
        video_params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()),
    )
    .get_audio_duration(&audio_file)
    .unwrap_or(0.0);

    state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
        audio_file: Some(audio_file.clone()),
        audio_duration: Some(audio_duration),
        progress: Some(30),
        ..Default::default()
    });

    // 阶段2：口播视频生成（30~90%）
    let portrait_video_path = if let Some(ref t) = existing {
        if let Some(ref pv) = t.portrait_video_path {
            if std::path::Path::new(pv).exists() {
                log::info!("数字人任务跳过阶段2（口播视频已存在）: task_id={}", task_id);
                pv.clone()
            } else {
                generate_portrait_video_stage(
                    task_id, params, &portrait_path, &audio_file, &portrait_video_path, &conf, None,
                )?
            }
        } else {
            generate_portrait_video_stage(
                task_id, params, &portrait_path, &audio_file, &portrait_video_path, &conf, None,
            )?
        }
    } else {
        generate_portrait_video_stage(
            task_id, params, &portrait_path, &audio_file, &portrait_video_path, &conf, None,
        )?
    };

    state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
        portrait_video_path: Some(portrait_video_path.clone()),
        progress: Some(90),
        ..Default::default()
    });

    // 阶段3：字幕与背景音乐合成（90~100%）
    let subtitle_path = if params.subtitle_enabled.unwrap_or(false) {
        generate_subtitle_stage(task_id, &video_params, &params.narration_text, &audio_file, &conf)?
    } else {
        String::new()
    };

    compose_final_video_stage(
        task_id, &video_params, &portrait_video_path, &audio_file, &subtitle_path, &final_video_path, &conf,
    )?;

    state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
        state: Some(TaskStatus::Completed.as_i32()),
        progress: Some(100),
        final_video_path: Some(final_video_path.clone()),
        subtitle_path: if subtitle_path.is_empty() { None } else { Some(subtitle_path) },
        ..Default::default()
    });

    log::info!("数字人任务完成: task_id={}, final={}", task_id, final_video_path);
    Ok(())
}

/// 阶段1：配音音频生成
fn generate_audio_stage(
    task_id: &str,
    params: &DigitalHumanParams,
    video_params: &VideoParams,
    conf: &Config,
) -> Result<String, SomaError> {
    log::info!("数字人任务阶段1（音频生成）: task_id={}", task_id);
    state::update_dh_task(task_id, None, Some(10));

    let text = params.narration_text.trim();
    if text.is_empty() {
        let msg = "文案不能为空".to_string();
        state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
            state: Some(TaskStatus::Failed.as_i32()),
            error_message: Some(msg.clone()),
            ..Default::default()
        });
        return Err(SomaError::Tts(msg));
    }

    let (audio_file, audio_duration) =
        crate::service::pipeline::generate_audio(task_id, video_params, text, conf)?;

    state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
        audio_file: Some(audio_file.clone()),
        audio_duration: Some(audio_duration),
        progress: Some(30),
        ..Default::default()
    });

    log::info!("数字人任务阶段1完成: task_id={}, audio={}", task_id, audio_file);
    Ok(audio_file)
}

/// 阶段2：口播视频生成（调用第三方数字人服务）
pub fn generate_portrait_video_stage(
    task_id: &str,
    params: &DigitalHumanParams,
    portrait_path: &std::path::Path,
    audio_file: &str,
    save_path: &str,
    conf: &Config,
    segment_index: Option<usize>,
) -> Result<String, SomaError> {
    let seg_label = match segment_index {
        Some(idx) => format!("segment={}", idx),
        None => "segment=whole".to_string(),
    };
    log::info!("数字人任务阶段2（口播视频生成）: task_id={}, {}", task_id, seg_label);
    state::update_dh_task(task_id, None, Some(50));

    let aspect_str = match params.get_video_aspect() {
        soma_core::models::VideoAspect::Landscape => "16:9",
        soma_core::models::VideoAspect::Portrait => "9:16",
        soma_core::models::VideoAspect::Square => "1:1",
    };

    let dh_conf = &conf.app.digital_human;
    let provider = soma_stock::digital_human::create_provider(dh_conf)?;

    let gen_params = soma_stock::digital_human::DhVideoGenParams {
        portrait_path: portrait_path.to_string_lossy().to_string(),
        audio_path: audio_file.to_string(),
        aspect_ratio: aspect_str.to_string(),
        model: params.video_encoder.clone(),
    };

    log::info!("数字人任务提交第三方: task_id={}", task_id);
    let third_task_id = block_on_async(provider.create_task(&gen_params))??;

    let timeout = dh_conf.get_timeout();
    let poll_interval = dh_conf.get_poll_interval();

    let status = block_on_async(soma_stock::digital_human::poll_until_done(
        &*provider, &third_task_id, timeout, poll_interval,
    ))??;

    match status {
        soma_stock::digital_human::DhVideoGenStatus::Success { video_url } => {
            log::info!("数字人任务第三方完成: task_id={}, video_url={}", task_id, video_url);
            block_on_async(provider.download_video(&video_url, save_path))??;
            log::info!("数字人任务阶段2完成: task_id={}, saved={}", task_id, save_path);
            Ok(save_path.to_string())
        }
        soma_stock::digital_human::DhVideoGenStatus::Failed { message } => {
            let msg = format!("口播视频生成失败: {}", message);
            state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
                state: Some(TaskStatus::Failed.as_i32()),
                error_message: Some(msg.clone()),
                ..Default::default()
            });
            Err(SomaError::VideoGen(msg))
        }
        soma_stock::digital_human::DhVideoGenStatus::Processing => {
            let msg = "视频生成超时，请重试".to_string();
            state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
                state: Some(TaskStatus::Failed.as_i32()),
                error_message: Some(msg.clone()),
                ..Default::default()
            });
            Err(SomaError::VideoGen(msg))
        }
    }
}

/// 阶段3：字幕生成
pub fn generate_subtitle_stage(
    task_id: &str,
    video_params: &VideoParams,
    text: &str,
    audio_file: &str,
    conf: &Config,
) -> Result<String, SomaError> {
    log::info!("数字人任务阶段3（字幕生成）: task_id={}", task_id);

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
        soma_tts::subtitle::correct_subtitle(&mut cues, text);
        soma_tts::subtitle::create_subtitle_file(&cues, &subtitle_path)?;
    } else {
        let ffmpeg = soma_video::Ffmpeg::new(
            &conf.app.get_ffmpeg_binary(),
            video_params.get_n_threads(),
            video_params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()),
        );
        let audio_dur = ffmpeg.get_audio_duration(audio_file)?;
        let cues = soma_tts::edge_tts::generate_subtitle_cues_from_text(text, audio_dur);
        soma_tts::subtitle::create_subtitle_file(&cues, &subtitle_path)?;
    }

    state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
        subtitle_path: Some(subtitle_path.clone()),
        ..Default::default()
    });

    Ok(subtitle_path)
}

/// 阶段3：最终视频合成（合并口播视频+配音+字幕+BGM）
pub fn compose_final_video_stage(
    task_id: &str,
    video_params: &VideoParams,
    portrait_video_path: &str,
    audio_file: &str,
    subtitle_path: &str,
    final_path: &str,
    conf: &Config,
) -> Result<(), SomaError> {
    log::info!("数字人任务阶段3（最终合成）: task_id={}", task_id);
    state::update_dh_task(task_id, None, Some(95));

    let ffmpeg = soma_video::Ffmpeg::new(
        &conf.app.get_ffmpeg_binary(),
        video_params.get_n_threads(),
        video_params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()),
    );
    let composer = soma_video::VideoComposer::new(ffmpeg);

    let bgm_file = composer.get_bgm_file(
        video_params.bgm_type.as_deref().unwrap_or("none"),
        video_params.bgm_file.as_deref().unwrap_or(""),
    );

    let mut final_params = video_params.clone();
    if !bgm_file.is_empty() {
        final_params.bgm_file = Some(bgm_file);
    }

    composer.generate_video(portrait_video_path, audio_file, subtitle_path, final_path, &final_params)?;

    log::info!("数字人任务阶段3完成: task_id={}, final={}", task_id, final_path);
    Ok(())
}

/// 阻塞执行异步 Future
fn block_on_async<F>(fut: F) -> Result<F::Output, SomaError>
where
    F: std::future::Future,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| SomaError::Llm(format!("tokio runtime 创建失败: {}", e)))?;
    let local = tokio::task::LocalSet::new();
    Ok(local.block_on(&rt, fut))
}

#[cfg(test)]
mod tests {
    use super::*;
    use soma_core::models::{DigitalHumanParams, VideoAspect};

    /// 通过 JSON 构造 DigitalHumanParams，方便测试
    fn make_params(json: &str) -> DigitalHumanParams {
        serde_json::from_str(json).expect("JSON 解析失败")
    }

    /// 测试基本参数转换：subject/script/aspect/portrait
    #[test]
    fn test_dh_params_to_video_params_basic() {
        let params = make_params(r#"{
            "portrait_image": "test.jpg",
            "narration_text": "测试文案",
            "video_aspect": "16:9"
        }"#);
        let vp = dh_params_to_video_params(&params);
        assert_eq!(vp.video_subject, "digital_human");
        assert_eq!(vp.video_script, "测试文案");
        assert_eq!(vp.video_aspect, Some("16:9".to_string()));
        assert_eq!(vp.portrait_image, Some("test.jpg".to_string()));
        assert_eq!(vp.video_count, Some(1));
    }

    /// 测试参数转换：voice 相关字段
    #[test]
    fn test_dh_params_to_video_params_voice_fields() {
        let params = make_params(r#"{
            "portrait_image": "p.jpg",
            "narration_text": "x",
            "voice_name": "zh-CN-XiaoxiaoNeural",
            "voice_rate": 1.5,
            "voice_volume": 0.8,
            "video_language": "zh-CN"
        }"#);
        let vp = dh_params_to_video_params(&params);
        assert_eq!(vp.voice_name, Some("zh-CN-XiaoxiaoNeural".to_string()));
        assert_eq!(vp.voice_rate, Some(1.5));
        assert_eq!(vp.voice_volume, Some(0.8));
        assert_eq!(vp.video_language, Some("zh-CN".to_string()));
    }

    /// 测试参数转换：bgm 相关字段
    #[test]
    fn test_dh_params_to_video_params_bgm_fields() {
        let params = make_params(r#"{
            "portrait_image": "p.jpg",
            "narration_text": "x",
            "bgm_type": "random",
            "bgm_file": "/path/to/bgm.mp3",
            "bgm_volume": 0.5
        }"#);
        let vp = dh_params_to_video_params(&params);
        assert_eq!(vp.bgm_type, Some("random".to_string()));
        assert_eq!(vp.bgm_file, Some("/path/to/bgm.mp3".to_string()));
        assert_eq!(vp.bgm_volume, Some(0.5));
    }

    /// 测试参数转换：字幕相关字段
    #[test]
    fn test_dh_params_to_video_params_subtitle_fields() {
        let params = make_params(r##"{
            "portrait_image": "p.jpg",
            "narration_text": "x",
            "subtitle_enabled": true,
            "subtitle_position": "bottom",
            "custom_position": 0.8,
            "font_name": "Arial",
            "font_size": 24,
            "text_fore_color": "#FFFFFF",
            "stroke_color": "#000000",
            "stroke_width": 2.0
        }"##);
        let vp = dh_params_to_video_params(&params);
        assert_eq!(vp.subtitle_enabled, Some(true));
        assert_eq!(vp.subtitle_position, Some("bottom".to_string()));
        assert_eq!(vp.custom_position, Some(0.8));
        assert_eq!(vp.font_name, Some("Arial".to_string()));
        assert_eq!(vp.font_size, Some(24));
        assert_eq!(vp.text_fore_color, Some("#FFFFFF".to_string()));
        assert_eq!(vp.stroke_color, Some("#000000".to_string()));
        assert_eq!(vp.stroke_width, Some(2.0));
    }

    /// 测试参数转换：编码器与线程数
    #[test]
    fn test_dh_params_to_video_params_encoder_threads() {
        let params = make_params(r#"{
            "portrait_image": "p.jpg",
            "narration_text": "x",
            "video_encoder": "libx264",
            "n_threads": 4
        }"#);
        let vp = dh_params_to_video_params(&params);
        assert_eq!(vp.video_encoder, Some("libx264".to_string()));
        assert_eq!(vp.n_threads, Some(4));
    }

    /// 测试参数转换：默认值（仅必填字段）
    #[test]
    fn test_dh_params_to_video_params_minimal() {
        let params = make_params(r#"{
            "portrait_image": "min.jpg",
            "narration_text": "最小文案"
        }"#);
        let vp = dh_params_to_video_params(&params);
        assert_eq!(vp.video_subject, "digital_human");
        assert_eq!(vp.video_script, "最小文案");
        assert_eq!(vp.portrait_image, Some("min.jpg".to_string()));
        assert_eq!(vp.video_count, Some(1));
        assert!(vp.voice_name.is_none());
        assert!(vp.video_aspect.is_none());
        assert!(vp.bgm_type.is_none());
        assert!(vp.subtitle_enabled.is_none());
    }


    /// 测试 DigitalHumanParams::get_video_aspect 默认竖屏
    #[test]
    fn test_dh_params_default_aspect() {
        let params = make_params(r#"{
            "portrait_image": "p.jpg",
            "narration_text": "x"
        }"#);
        assert_eq!(params.get_video_aspect(), VideoAspect::Portrait);
    }

    /// 测试 DigitalHumanParams::get_video_aspect 横屏
    #[test]
    fn test_dh_params_landscape_aspect() {
        let params = make_params(r#"{
            "portrait_image": "p.jpg",
            "narration_text": "x",
            "video_aspect": "16:9"
        }"#);
        assert_eq!(params.get_video_aspect(), VideoAspect::Landscape);
    }

    /// 测试 DigitalHumanParams::get_tts_provider 默认 edge
    #[test]
    fn test_dh_params_default_tts_provider() {
        let params = make_params(r#"{
            "portrait_image": "p.jpg",
            "narration_text": "x"
        }"#);
        assert_eq!(params.get_tts_provider(), "edge");
    }

    /// 测试 DigitalHumanParams::get_tts_provider 自定义
    #[test]
    fn test_dh_params_custom_tts_provider() {
        let params = make_params(r#"{
            "portrait_image": "p.jpg",
            "narration_text": "x",
            "tts_provider": "azure"
        }"#);
        assert_eq!(params.get_tts_provider(), "azure");
    }
}