//! 视频合成功能点（素材 + 音频 + 字幕 → 成品视频）
//!
//! 覆盖原流水线第 6 步，同时可作为独立的视频后期入口：
//! 对每个视频副本执行 拼接素材 → 添加 BGM → 合成音频/字幕，
//! 再按需叠加水印与片头/片尾。

use crate::context::FeatureContext;
use crate::descriptor::{FeatureKind, FeatureMeta};
use crate::envelope::ArtifactKind;
use crate::feature::TypedFeature;
use crate::progress::{FeatureProgress, ProgressReporter};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soma_core::config::AppConfig;
use soma_core::error::SomaError;
use soma_core::models::VideoParams;

/// 视频合成（纯函数，供功能点与宿主兼容层复用）
///
/// `on_progress` 以百分比（0~99）上报合成进度，每个副本的两个子阶段各上报一次。
///
/// 返回 (最终视频列表, 拼接中间视频列表)。
pub fn compose_videos(
    conf: &AppConfig,
    params: &VideoParams,
    materials: &[String],
    audio_file: &str,
    subtitle_path: &str,
    output_dir: &str,
    on_progress: &dyn Fn(u32),
) -> Result<(Vec<String>, Vec<String>), SomaError> {
    let codec = params.get_video_encoder().unwrap_or(conf.get_video_codec());
    let ffmpeg = soma_video::Ffmpeg::new(&conf.get_ffmpeg_binary(), params.get_n_threads(), codec);
    let composer = soma_video::VideoComposer::new(ffmpeg);
    let aspect = params.get_video_aspect();
    let clip_dur = params.get_clip_duration();
    let video_count = params.get_video_count();
    let transition_mode = params.video_transition_mode.as_deref().unwrap_or("FadeIn");

    let out_dir = std::path::Path::new(output_dir);
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir).map_err(SomaError::Io)?;
    }

    let mut final_videos = Vec::new();
    let mut combined_videos = Vec::new();
    let mut progress = 0u32;
    let total_steps = (video_count * 2).max(1);
    let step_progress = (100 / total_steps).max(1);

    for i in 1..=video_count {
        let combined_path = out_dir
            .join(format!("combined-{}.mp4", i))
            .to_string_lossy()
            .to_string();
        let final_path = out_dir
            .join(format!("final-{}.mp4", i))
            .to_string_lossy()
            .to_string();

        progress = (progress + step_progress).min(99);
        on_progress(progress);

        composer.combine_videos(
            materials,
            audio_file,
            &combined_path,
            &aspect,
            clip_dur,
            transition_mode,
        )?;

        progress = (progress + step_progress).min(99);
        on_progress(progress);

        let bgm_file = composer.get_bgm_file(
            params.bgm_type.as_deref().unwrap_or("random"),
            params.bgm_file.as_deref().unwrap_or(""),
        );

        let mut final_params = params.clone();
        if !bgm_file.is_empty() {
            final_params.bgm_file = Some(bgm_file);
        }

        composer.generate_video(
            &combined_path,
            audio_file,
            subtitle_path,
            &final_path,
            &final_params,
        )?;

        // 水印（右下角，半透明）
        let current_path = if let Some(ref wm) = params.video_watermark {
            if !wm.is_empty() && std::path::Path::new(wm).exists() {
                let wm_path = out_dir
                    .join(format!("watermarked-{}.mp4", i))
                    .to_string_lossy()
                    .to_string();
                composer.ffmpeg().add_watermark(&final_path, wm, &wm_path)?;
                wm_path
            } else {
                final_path.clone()
            }
        } else {
            final_path.clone()
        };

        // 片头/片尾拼接
        let final_path_with_intro_outro =
            if params.video_intro.is_some() || params.video_outro.is_some() {
                let intro = params.video_intro.as_deref().unwrap_or("");
                let outro = params.video_outro.as_deref().unwrap_or("");
                let has_intro = !intro.is_empty() && std::path::Path::new(intro).exists();
                let has_outro = !outro.is_empty() && std::path::Path::new(outro).exists();
                if has_intro || has_outro {
                    let io_path = out_dir
                        .join(format!("final-io-{}.mp4", i))
                        .to_string_lossy()
                        .to_string();
                    let mut segments: Vec<&str> = Vec::new();
                    if has_intro {
                        segments.push(intro);
                    }
                    segments.push(&current_path);
                    if has_outro {
                        segments.push(outro);
                    }
                    composer.ffmpeg().concat_videos(&segments, &io_path)?;
                    io_path
                } else {
                    current_path.clone()
                }
            } else {
                current_path.clone()
            };

        final_videos.push(final_path_with_intro_outro);
        combined_videos.push(combined_path);
    }

    Ok((final_videos, combined_videos))
}

/// video.compose 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoComposeInput {
    /// 视频参数（字幕样式 / BGM / 水印 / 片头片尾 / 编码器等）
    pub params: VideoParams,
    /// 素材文件路径列表
    pub materials: Vec<String>,
    /// 配音音频文件路径
    pub audio_file: String,
    /// 字幕文件路径；空字符串表示不叠加字幕
    #[serde(default)]
    pub subtitle_path: String,
    /// 输出目录；缺省写入产物目录
    #[serde(default)]
    pub output_dir: Option<String>,
}

/// video.compose 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VideoComposeOutput {
    /// 最终成品视频路径列表（含水印/片头片尾处理）
    pub final_videos: Vec<String>,
    /// 拼接中间视频路径列表（未叠字幕/BGM 前的 combined 文件）
    pub combined_videos: Vec<String>,
}

/// 视频合成：素材 + 音频 + 字幕 → 成品视频（可脱离任务独立调用）
pub struct VideoComposeFeature;

impl TypedFeature for VideoComposeFeature {
    type Input = VideoComposeInput;
    type Output = VideoComposeOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "video.compose".into(),
            name: "视频合成".into(),
            description: "拼接素材、混音 BGM、叠加字幕/水印/片头片尾，合成成品视频".into(),
            kind: FeatureKind::VideoCompose,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: VideoComposeInput,
        progress: &dyn ProgressReporter,
    ) -> Result<VideoComposeOutput, SomaError> {
        let output_dir = match input.output_dir {
            Some(ref d) if !d.is_empty() => d.clone(),
            _ => ctx.work_dir().to_string_lossy().to_string(),
        };
        let feature_id = ctx.feature_id().to_string();
        let run_id = ctx.run_id().to_string();
        let on_progress = |percent: u32| {
            progress
                .report(FeatureProgress::new(&feature_id, &run_id, percent).with_step("compose"));
        };

        let (final_videos, combined_videos) = compose_videos(
            ctx.config(),
            &input.params,
            &input.materials,
            &input.audio_file,
            &input.subtitle_path,
            &output_dir,
            &on_progress,
        )?;

        for path in &final_videos {
            let name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            ctx.add_artifact(name, path, ArtifactKind::Video);
        }
        Ok(VideoComposeOutput {
            final_videos,
            combined_videos,
        })
    }
}
