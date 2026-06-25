use soma_core::error::SomaError;
use soma_core::models::{TaskStatus, VideoParams};
use crate::state::{self, TaskUpdateData};
use crate::Config;

pub fn run_task(task_id: &str, params: &VideoParams, stop_at: &str) -> Result<(), SomaError> {
    let conf = Config::get();

    state::update_task(task_id, Some(TaskStatus::Processing.as_i32()), Some(5));

    let video_script = generate_script(task_id, params)?;
    state::update_task(task_id, None, Some(10));

    if stop_at == "script" {
        state::update_task_data(task_id, &TaskUpdateData {
            state: Some(TaskStatus::Completed.as_i32()),
            progress: Some(100),
            script: Some(video_script.clone()),
            ..Default::default()
        });
        return Ok(());
    }

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

    let (audio_file, audio_duration) = generate_audio(task_id, params, &video_script, &conf)?;
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

    state::update_task(task_id, None, Some(30));

    let subtitle_path = generate_subtitle(task_id, params, &video_script, &audio_file, &conf)?;
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

    state::update_task(task_id, None, Some(40));

    let materials = get_video_materials(task_id, params, &video_terms, audio_duration, &conf)?;
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

    state::update_task(task_id, None, Some(50));

    let (final_videos, combined_videos) = generate_final_videos(task_id, params, &materials, &audio_file, &subtitle_path, &conf)?;

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
    let system_prompt = params.custom_system_prompt.as_deref().unwrap_or("");

    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Llm(e.to_string()))?;
    rt.block_on(super::llm::generate_script(provider, &params.video_subject, language, paragraph_number, prompt, system_prompt, &conf))
}

fn generate_terms(_task_id: &str, params: &VideoParams, script: &str) -> Result<Vec<String>, SomaError> {
    if let Some(ref terms) = params.video_terms {
        if let Some(arr) = terms.as_array() {
            return Ok(arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());
        }
        if let Some(s) = terms.as_str() {
            return Ok(s.split(&[',', '，'][..]).map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect());
        }
    }

    let conf = Config::get();
    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");
    let amount = if params.match_materials_to_script.unwrap_or(false) { 8 } else { 5 };

    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Llm(e.to_string()))?;
    rt.block_on(super::llm::generate_terms(provider, &params.video_subject, script, amount, &conf))
}

fn generate_audio(task_id: &str, params: &VideoParams, script: &str, conf: &crate::Config) -> Result<(String, f64), SomaError> {
    let task_audio_dir = soma_core::utils::task_dir(task_id);
    let audio_file = task_audio_dir.join("audio.mp3").to_string_lossy().to_string();

    let voice_name = params.get_voice_name();
    let rate = params.get_voice_rate();

    let tts = soma_tts::edge_tts::EdgeTts::new(conf.app.get_edge_tts_timeout());

    let rt = tokio::runtime::Runtime::new().map_err(|e| SomaError::Tts(e.to_string()))?;
    let result = rt.block_on(
        soma_tts::provider::SomaTtsProvider::synthesize(&tts, script, voice_name, rate, std::path::Path::new(&audio_file))
    ).map_err(|e| {
        state::update_task(task_id, Some(TaskStatus::Failed.as_i32()), None);
        e
    })?;

    Ok((result.audio_file, result.audio_duration))
}

fn generate_subtitle(task_id: &str, params: &VideoParams, script: &str, audio_file: &str, conf: &crate::Config) -> Result<String, SomaError> {
    if !params.get_subtitle_enabled() {
        return Ok(String::new());
    }

    let task_dir = soma_core::utils::task_dir(task_id);
    let subtitle_path = task_dir.join("subtitle.srt").to_string_lossy().to_string();

    let subtitle_provider = conf.app.get_subtitle_provider();
    if subtitle_provider == "whisper" {
        soma_tts::subtitle::generate_whisper_subtitle(audio_file, &subtitle_path)?;
        let mut cues = soma_tts::subtitle::file_to_subtitles(&subtitle_path);
        soma_tts::subtitle::correct_subtitle(&mut cues, script);
        soma_tts::subtitle::create_subtitle_file(&cues, &subtitle_path)?;
    } else {
        let ffmpeg = soma_video::Ffmpeg::new(&conf.app.get_ffmpeg_binary(), params.get_n_threads(), conf.app.get_video_codec());
        let audio_dur = ffmpeg.get_audio_duration(audio_file)?;
        let cues = soma_tts::edge_tts::generate_subtitle_cues_from_text(script, audio_dur);
        soma_tts::subtitle::create_subtitle_file(&cues, &subtitle_path)?;
    }

    Ok(subtitle_path)
}

fn get_video_materials(task_id: &str, params: &VideoParams, terms: &[String], audio_duration: f64, conf: &crate::Config) -> Result<Vec<String>, SomaError> {
    let source = params.video_source.as_deref().unwrap_or("pexels");
    let aspect = params.get_video_aspect();
    let clip_dur = params.get_clip_duration();

    if source == "local" {
        let ffmpeg = soma_video::Ffmpeg::new(&conf.app.get_ffmpeg_binary(), params.get_n_threads(), conf.app.get_video_codec());
        let materials = params.video_materials.as_deref().unwrap_or(&[]);
        return ffmpeg.preprocess_local_materials(materials, clip_dur, &aspect);
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

fn generate_final_videos(
    task_id: &str,
    params: &VideoParams,
    materials: &[String],
    audio_file: &str,
    subtitle_path: &str,
    conf: &crate::Config,
) -> Result<(Vec<String>, Vec<String>), SomaError> {
    let ffmpeg = soma_video::Ffmpeg::new(&conf.app.get_ffmpeg_binary(), params.get_n_threads(), conf.app.get_video_codec());
    let composer = soma_video::VideoComposer::new(ffmpeg);
    let aspect = params.get_video_aspect();
    let clip_dur = params.get_clip_duration();
    let video_count = params.get_video_count();

    let mut final_videos = Vec::new();
    let mut combined_videos = Vec::new();
    let mut progress = 50u32;

    for i in 1..=video_count {
        let task_dir_path = soma_core::utils::task_dir(task_id);
        let combined_path = task_dir_path.join(format!("combined-{}.mp4", i)).to_string_lossy().to_string();
        let final_path = task_dir_path.join(format!("final-{}.mp4", i)).to_string_lossy().to_string();

        composer.combine_videos(materials, audio_file, &combined_path, &aspect, clip_dur)?;

        progress += (50 / video_count / 2).max(1);
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

        progress += (50 / video_count / 2).max(1);
        state::update_task(task_id, None, Some(progress));

        final_videos.push(final_path);
        combined_videos.push(combined_path);
    }

    Ok((final_videos, combined_videos))
}
