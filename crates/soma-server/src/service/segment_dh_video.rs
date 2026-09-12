//! 数字人口播视频分段生成流程
//!
//! 当文案较长时（TTS 音频 > 6 秒），将文案切分为多个短句，
//! 逐段生成 TTS 音频 + EchoMimicV3 口播视频，最后拼接为完整视频。
//!
//! 流程：切句 → 循环(每句TTS+口播) → FFmpeg concat → 叠字幕

use std::path::Path;
use soma_core::error::SomaError;
use soma_core::models::{DigitalHumanParams, VideoParams};
use crate::state;
use crate::Config;

pub const SINGLE_GEN_FRAME_LIMIT: u32 = 150;
pub const SINGLE_GEN_DURATION_LIMIT: f64 = 6.0;
pub const CONCAT_DURATION_TOLERANCE: f64 = 0.5;

/// 单段处理结果
pub struct SegmentResult {
    pub audio_path: String,
    pub video_path: String,
    pub audio_duration: f64,
}

/// 分段流程整体结果
pub struct SegmentOutcome {
    pub portrait_video_path: String,
    pub audio_file: String,
    pub audio_duration: f64,
    pub segment_count: u32,
}

/// 在片段内部按字符数切分，优先在空格处断开
pub fn split_long_segment(text: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 {
        return vec![text.to_string()];
    }
    let chars: Vec<char> = text.chars().collect();
    let mut result: Vec<String> = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + max_chars).min(chars.len());
        if end == chars.len() {
            result.push(chars[start..end].iter().collect());
            break;
        }

        let mut break_pos = end;
        for i in (start + 1..=end).rev() {
            if chars[i - 1].is_whitespace() {
                break_pos = i;
                break;
            }
        }
        result.push(chars[start..break_pos].iter().collect());
        start = break_pos;
        while start < chars.len() && chars[start].is_whitespace() {
            start += 1;
        }
    }
    result.into_iter().filter(|s| !s.trim().is_empty()).collect()
}

/// 将文案按标点和字符数切分为多个短句
pub fn split_into_segments(text: &str, max_chars: usize) -> Vec<String> {
    let raw = soma_core::utils::split_string_by_punctuations(text);
    let mut result = Vec::new();
    for seg in raw {
        let trimmed = seg.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().count() <= max_chars {
            result.push(trimmed.to_string());
        } else {
            for sub in split_long_segment(trimmed, max_chars) {
                if !sub.trim().is_empty() {
                    result.push(sub);
                }
            }
        }
    }
    result
}

/// 分段生成主入口
pub fn run_segment_flow(
    task_id: &str,
    params: &DigitalHumanParams,
    portrait_path: &Path,
    narration_text: &str,
    conf: &Config,
) -> Result<SegmentOutcome, SomaError> {
    let text = narration_text.trim();
    if text.is_empty() {
        return Err(SomaError::Tts("文案不能为空".into()));
    }

    let task_dir = soma_core::utils::task_dir(task_id);
    let seg_conf = &conf.app.digital_human.segment;
    let max_chars = seg_conf.get_segment_max_chars();
    let max_dur = seg_conf.get_segment_max_duration();

    let segments = split_into_segments(text, max_chars);
    let n = segments.len() as u32;
    log::info!("分段生成: task_id={}, segments={}", task_id, n);

    state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
        segment_count: Some(n),
        ..Default::default()
    });

    let video_params = build_video_params(params);
    let ffmpeg = soma_video::Ffmpeg::new(
        &conf.app.get_ffmpeg_binary(),
        video_params.get_n_threads(),
        video_params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()),
    );

    let audio_mp3 = task_dir.join("audio.mp3").to_string_lossy().to_string();
    let portrait_video_mp4 = task_dir.join("portrait_video.mp4").to_string_lossy().to_string();
    let final_mp4 = task_dir.join("final.mp4").to_string_lossy().to_string();

    if n == 1 {
        let (af, dur) = crate::service::pipeline::generate_audio_to(
            task_id, &video_params, &segments[0], &audio_mp3, conf,
        )?;
        if dur <= SINGLE_GEN_DURATION_LIMIT {
            log::info!("快速路径: task_id={}, dur={:.2}s", task_id, dur);
            crate::service::digital_human::generate_portrait_video_stage(
                task_id, params, portrait_path, &af, &portrait_video_mp4, conf, None,
            )?;

            let subtitle_path = if params.subtitle_enabled.unwrap_or(false) {
                crate::service::digital_human::generate_subtitle_stage(
                    task_id, &video_params, narration_text, &af, conf,
                ).unwrap_or_default()
            } else {
                String::new()
            };

            crate::service::digital_human::compose_final_video_stage(
                task_id, &video_params, &portrait_video_mp4, &af, &subtitle_path, &final_mp4, conf,
            )?;

            return Ok(SegmentOutcome {
                portrait_video_path: portrait_video_mp4,
                audio_file: af,
                audio_duration: dur,
                segment_count: 1,
            });
        }
    }

    let mut seg_audios: Vec<String> = Vec::new();
    let mut seg_durations: Vec<f64> = Vec::new();

    for (i, seg_text) in segments.iter().enumerate() {
        let idx = i + 1;
        let progress = 10 + (40 - 10) * i as u32 / n;
        state::update_dh_task(task_id, None, Some(progress));

        let seg_mp3 = task_dir.join(format!("segment_{}.mp3", idx)).to_string_lossy().to_string();
        let dur = generate_segment_audio(task_id, &video_params, seg_text, idx, &seg_mp3, conf)?;
        seg_audios.push(seg_mp3);
        seg_durations.push(dur);
        log::info!("TTS 分段 {}/{} 完成: dur={:.2}s", idx, n, dur);
    }

    let mut seg_videos: Vec<String> = Vec::new();
    let mut total_dur = 0.0f64;

    for (i, _seg_text) in segments.iter().enumerate() {
        let idx = i + 1;
        let progress = 40 + (85 - 40) * i as u32 / n;
        state::update_dh_task(task_id, None, Some(progress));

        let seg_mp4 = task_dir.join(format!("portrait_{}.mp4", idx)).to_string_lossy().to_string();
        let dur = seg_durations[i];

        generate_segment_video(
            task_id, params, portrait_path, &seg_audios[i], &seg_mp4,
            idx, dur, conf,
        )?;

        total_dur += dur;
        seg_videos.push(seg_mp4.clone());

        state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
            current_segment: Some(idx as u32),
            segment_audio_files: Some(seg_audios[..=i].to_vec()),
            segment_video_files: Some(seg_videos.clone()),
            ..Default::default()
        });
    }


    state::update_dh_task(task_id, None, Some(85));
    concat_portrait_segments(&seg_videos, &portrait_video_mp4, &ffmpeg)?;

    state::update_dh_task(task_id, None, Some(90));
    ffmpeg.concat_audios(&seg_audios, &audio_mp3)?;
    let audio_duration = ffmpeg.get_audio_duration(&audio_mp3).unwrap_or(total_dur);

    let subtitle_path = if params.subtitle_enabled.unwrap_or(false) {
        crate::service::digital_human::generate_subtitle_stage(
            task_id, &video_params, narration_text, &audio_mp3, conf,
        ).unwrap_or_default()
    } else {
        String::new()
    };

    crate::service::digital_human::compose_final_video_stage(
        task_id, &video_params, &portrait_video_mp4, &audio_mp3, &subtitle_path, &final_mp4, conf,
    )?;

    state::update_dh_task_data(task_id, &state::DhTaskUpdateData {
        portrait_video_path: Some(portrait_video_mp4.clone()),
        audio_file: Some(audio_mp3.clone()),
        audio_duration: Some(audio_duration),
        final_video_path: Some(final_mp4.clone()),
        state: Some(soma_core::models::TaskStatus::Completed.as_i32()),
        progress: Some(100),
        subtitle_path: if subtitle_path.is_empty() { None } else { Some(subtitle_path) },
        ..Default::default()
    });

    log::info!("分段生成完成: task_id={}, segments={}, dur={:.2}s", task_id, n, audio_duration);

    Ok(SegmentOutcome {
        portrait_video_path: portrait_video_mp4,
        audio_file: audio_mp3,
        audio_duration,
        segment_count: n,
    })
}

fn generate_segment_audio(
    task_id: &str,
    video_params: &VideoParams,
    seg_text: &str,
    index: usize,
    seg_mp3: &str,
    conf: &Config,
) -> Result<f64, SomaError> {
    let task_dir = soma_core::utils::task_dir(task_id);
    let ffmpeg = soma_video::Ffmpeg::new(
        &conf.app.get_ffmpeg_binary(),
        video_params.get_n_threads(),
        video_params.get_video_encoder().unwrap_or_else(|| conf.app.get_video_codec()),
    );

    if Path::new(seg_mp3).exists() {
        if let Ok(dur) = ffmpeg.get_audio_duration(seg_mp3) {
            if dur > 0.0 {
                return Ok(dur);
            }
        }
    }

    let (_, mut dur) = crate::service::pipeline::generate_audio_to(
        task_id, video_params, seg_text, seg_mp3, conf,
    ).map_err(|e| SomaError::Tts(format!("TTS 合成失败（分段 {}）: {}", index, e)))?;

    if dur > SINGLE_GEN_DURATION_LIMIT && seg_text.chars().count() > 2 {
        log::warn!("分段 {} 音频 {:.2}s 超限，二次切分重生成", index, dur);
        let sub_texts = split_long_segment(seg_text, seg_text.chars().count() / 2);
        let mut sub_audios: Vec<String> = Vec::new();
        let mut sub_dur = 0.0f64;
        for (j, sub) in sub_texts.iter().enumerate() {
            let sub_mp3 = task_dir.join(format!("segment_{}_{}.mp3", index, j + 1)).to_string_lossy().to_string();
            let (_, d) = crate::service::pipeline::generate_audio_to(
                task_id, video_params, sub, &sub_mp3, conf,
            ).map_err(|e| SomaError::Tts(format!("TTS 二次切分失败（分段 {}-{}）: {}", index, j + 1, e)))?;
            sub_audios.push(sub_mp3);
            sub_dur += d;
        }
        ffmpeg.concat_audios(&sub_audios, seg_mp3)?;
        dur = ffmpeg.get_audio_duration(seg_mp3).unwrap_or(sub_dur);
        for sub in &sub_audios {
            let _ = std::fs::remove_file(sub);
        }
    }

    Ok(dur)
}

fn generate_segment_video(
    task_id: &str,
    params: &DigitalHumanParams,
    portrait_path: &Path,
    seg_mp3: &str,
    seg_mp4: &str,
    index: usize,
    dur: f64,
    conf: &Config,
) -> Result<(), SomaError> {
    let task_dir = soma_core::utils::task_dir(task_id);
    let ffmpeg = soma_video::Ffmpeg::new(
        &conf.app.get_ffmpeg_binary(),
        params.n_threads.unwrap_or(4),
        params.video_encoder.as_deref().unwrap_or_else(|| conf.app.get_video_codec()),
    );

    if !Path::new(seg_mp4).exists() {
        let preview: String = std::fs::read_to_string(seg_mp3).unwrap_or_default().chars().take(50).collect();
        log::info!("口播分段 {} 开始: task_id={}", index, task_id);

        crate::service::digital_human::generate_portrait_video_stage(
            task_id, params, portrait_path, seg_mp3, seg_mp4, conf, Some(index),
        ).map_err(|e| SomaError::VideoGen(format!("口播视频生成失败（分段 {}）: {}", index, e)))?;
    }

    let video_dur = ffmpeg.get_video_duration(seg_mp4).unwrap_or(0.0);
    let provider = conf.app.digital_human.get_provider();
    if provider != "live2d" && video_dur < dur {
        let pad = dur - video_dur;
        log::info!("分段 {} 补冻结帧: video={:.2}s audio={:.2}s pad={:.2}s", index, video_dur, dur, pad);
        let padded_mp4 = task_dir.join(format!("portrait_{}_padded.mp4", index)).to_string_lossy().to_string();
        let result = std::process::Command::new(&conf.app.get_ffmpeg_binary())
            .args(&[
                "-y", "-i", seg_mp4,
                "-vf", &format!("tpad=stop_mode=clone:stop_duration={:.3}", pad),
                "-c:v", &conf.app.get_video_codec(),
                "-pix_fmt", "yuv420p",
                "-an",
                &padded_mp4,
            ])
            .output()
            .map_err(SomaError::Io)?;
        if result.status.success() && Path::new(&padded_mp4).exists() {
            let _ = std::fs::rename(&padded_mp4, seg_mp4);
        } else {
            log::warn!("分段 {} 冻结帧补齐失败，使用原视频", index);
        }
    }

    log::info!("口播分段 {} 完成: task_id={}, dur={:.2}s", index, task_id, dur);
    Ok(())
}

fn concat_portrait_segments(
    segments: &[String],
    output_path: &str,
    ffmpeg: &soma_video::Ffmpeg,
) -> Result<(), SomaError> {
    for (i, seg) in segments.iter().enumerate() {
        if !Path::new(seg).exists() {
            return Err(SomaError::VideoGen(format!(
                "分段 {} 口播视频文件缺失，无法拼接", i + 1
            )));
        }
    }
    ffmpeg.concat_clips(segments, output_path)?;

    let actual = ffmpeg.get_video_duration(output_path).unwrap_or(0.0);
    let mut expected = 0.0;
    for seg in segments {
        expected += ffmpeg.get_video_duration(seg).unwrap_or(0.0);
    }
    if (actual - expected).abs() > CONCAT_DURATION_TOLERANCE {
        log::warn!("拼接时长偏差: actual={:.2}s, expected={:.2}s", actual, expected);
    }
    Ok(())
}

fn build_video_params(params: &DigitalHumanParams) -> VideoParams {
    crate::service::digital_human::dh_params_to_video_params(params)
}