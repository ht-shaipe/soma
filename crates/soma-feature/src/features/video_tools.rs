//! 视频处理功能点（原子能力）
//!
//! 将原一键合成的每个环节拆分为可独立调用的原语：
//! 拼接对齐（concat）→ 渲染（render）→ 水印（watermark）→ 片头片尾拼接（join），
//! 另含转场、裁切缩放与媒体信息探测，可自由组合成自定义后期链。

use crate::context::FeatureContext;
use crate::descriptor::{FeatureKind, FeatureMeta};
use crate::envelope::ArtifactKind;
use crate::feature::TypedFeature;
use crate::progress::{FeatureProgress, ProgressReporter};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soma_core::config::AppConfig;
use soma_core::error::SomaError;
use soma_core::models::{VideoAspect, VideoParams};

/// 构建与全局配置一致的 FFmpeg 实例
fn build_ffmpeg(conf: &AppConfig, n_threads: u32, encoder: Option<&str>) -> soma_video::Ffmpeg {
    let codec = encoder.unwrap_or(conf.get_video_codec());
    soma_video::Ffmpeg::new(&conf.get_ffmpeg_binary(), n_threads, codec)
}

/// 解析输出路径：显式指定优先，否则写入产物目录（文件名取 output_name）
fn resolve_output(
    ctx: &FeatureContext,
    output_path: &Option<String>,
    default_name: &str,
) -> String {
    match output_path {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => ctx.artifact_path(default_name).to_string_lossy().to_string(),
    }
}

fn register_video(ctx: &FeatureContext, path: &str) {
    let name = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    ctx.add_artifact(name, path, ArtifactKind::Video);
}

// ============ video.concat：素材拼接对齐音频 ============

/// video.concat 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoConcatInput {
    /// 素材视频文件路径列表
    pub materials: Vec<String>,
    /// 对齐的音频文件路径（总时长基准）
    pub audio_file: String,
    /// 画幅比例（"16:9"/"9:16"/"1:1"），缺省 9:16
    #[serde(default)]
    pub aspect: Option<String>,
    /// 单片段最长时长（秒），缺省 4
    #[serde(default)]
    pub clip_duration: Option<u32>,
    /// 转场模式（FadeIn/FadeOut/None/Shuffle 等），缺省 FadeIn
    #[serde(default)]
    pub transition_mode: Option<String>,
    /// 输出文件路径；缺省写入产物目录 combined.mp4
    #[serde(default)]
    pub output_path: Option<String>,
}

/// video.concat 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VideoConcatOutput {
    /// 拼接后的视频文件路径
    pub video: String,
}

/// 素材拼接：多段素材按转场模式拼接并与音频时长对齐
pub struct VideoConcatFeature;

impl TypedFeature for VideoConcatFeature {
    type Input = VideoConcatInput;
    type Output = VideoConcatOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "video.concat".into(),
            name: "素材拼接".into(),
            description: "多段素材按转场模式拼接，并与音频时长对齐".into(),
            kind: FeatureKind::VideoTool,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: VideoConcatInput,
        progress: &dyn ProgressReporter,
    ) -> Result<VideoConcatOutput, SomaError> {
        if input.materials.is_empty() {
            return Err(SomaError::Config("素材列表为空".into()));
        }
        let conf = ctx.config();
        let aspect = input
            .aspect
            .as_deref()
            .and_then(VideoAspect::from_str)
            .unwrap_or(VideoAspect::Portrait);
        let output_path = resolve_output(ctx, &input.output_path, "combined.mp4");
        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        progress.report(
            FeatureProgress::new(ctx.feature_id(), ctx.run_id(), 30)
                .with_step("concat")
                .with_message(format!("拼接 {} 段素材", input.materials.len())),
        );
        let composer = soma_video::VideoComposer::new(build_ffmpeg(
            conf,
            2,
            None,
        ));
        composer.combine_videos(
            &input.materials,
            &input.audio_file,
            &output_path,
            &aspect,
            input.clip_duration.unwrap_or(4),
            input.transition_mode.as_deref().unwrap_or("FadeIn"),
        )?;
        register_video(ctx, &output_path);
        Ok(VideoConcatOutput { video: output_path })
    }
}

// ============ video.render：渲染（音频+字幕+BGM） ============

/// video.render 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoRenderInput {
    /// 输入视频（通常是 video.concat 的产物）
    pub video: String,
    /// 配音音频文件路径
    pub audio_file: String,
    /// 字幕文件路径（SRT）；空表示不加字幕
    #[serde(default)]
    pub subtitle_path: String,
    /// 视频参数（字幕样式 / BGM / 音量等，与任务参数同构）
    pub params: VideoParams,
    /// 输出文件路径；缺省写入产物目录 final.mp4
    #[serde(default)]
    pub output_path: Option<String>,
}

/// video.render 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VideoRenderOutput {
    /// 渲染后的视频文件路径
    pub video: String,
}

/// 视频渲染：为拼接视频叠加配音、字幕与 BGM，输出成品
pub struct VideoRenderFeature;

impl TypedFeature for VideoRenderFeature {
    type Input = VideoRenderInput;
    type Output = VideoRenderOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "video.render".into(),
            name: "视频渲染".into(),
            description: "为视频合成配音、字幕与 BGM（支持全部字幕样式参数）".into(),
            kind: FeatureKind::VideoTool,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: VideoRenderInput,
        progress: &dyn ProgressReporter,
    ) -> Result<VideoRenderOutput, SomaError> {
        let conf = ctx.config();
        let output_path = resolve_output(ctx, &input.output_path, "final.mp4");
        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        progress.report(
            FeatureProgress::new(ctx.feature_id(), ctx.run_id(), 40)
                .with_step("render")
                .with_message("渲染音频与字幕"),
        );
        let composer = soma_video::VideoComposer::new(build_ffmpeg(
            conf,
            input.params.get_n_threads(),
            input.params.get_video_encoder(),
        ));
        let mut final_params = input.params.clone();
        let bgm_file = composer.get_bgm_file(
            input.params.bgm_type.as_deref().unwrap_or("none"),
            input.params.bgm_file.as_deref().unwrap_or(""),
        );
        if !bgm_file.is_empty() {
            final_params.bgm_file = Some(bgm_file);
        }
        composer.generate_video(
            &input.video,
            &input.audio_file,
            &input.subtitle_path,
            &output_path,
            &final_params,
        )?;
        register_video(ctx, &output_path);
        Ok(VideoRenderOutput { video: output_path })
    }
}

// ============ video.watermark：水印叠加 ============

/// video.watermark 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoWatermarkInput {
    /// 输入视频路径
    pub video: String,
    /// 水印图片路径（叠加在右下角，半透明）
    pub watermark: String,
    /// 输出文件路径；缺省写入产物目录 watermarked.mp4
    #[serde(default)]
    pub output_path: Option<String>,
}

/// video.watermark 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VideoWatermarkOutput {
    /// 加水印后的视频文件路径
    pub video: String,
}

/// 水印叠加：为视频右下角叠加半透明图片水印
pub struct VideoWatermarkFeature;

impl TypedFeature for VideoWatermarkFeature {
    type Input = VideoWatermarkInput;
    type Output = VideoWatermarkOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "video.watermark".into(),
            name: "水印叠加".into(),
            description: "为视频右下角叠加半透明图片水印".into(),
            kind: FeatureKind::VideoTool,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: VideoWatermarkInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<VideoWatermarkOutput, SomaError> {
        if !std::path::Path::new(&input.watermark).exists() {
            return Err(SomaError::Config(format!("水印图片不存在: {}", input.watermark)));
        }
        let conf = ctx.config();
        let output_path = resolve_output(ctx, &input.output_path, "watermarked.mp4");
        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        let ffmpeg = build_ffmpeg(conf, 2, None);
        ffmpeg.add_watermark(&input.video, &input.watermark, &output_path)?;
        register_video(ctx, &output_path);
        Ok(VideoWatermarkOutput { video: output_path })
    }
}

// ============ video.join：多段视频纯拼接 ============

/// video.join 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoJoinInput {
    /// 视频片段路径列表（按顺序拼接，如 片头/正片/片尾）
    pub videos: Vec<String>,
    /// 输出文件路径；缺省写入产物目录 joined.mp4
    #[serde(default)]
    pub output_path: Option<String>,
}

/// video.join 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VideoJoinOutput {
    /// 拼接后的视频文件路径
    pub video: String,
}

/// 视频拼接：多段视频无损顺序合并（不转码对齐，适合片头片尾拼接）
pub struct VideoJoinFeature;

impl TypedFeature for VideoJoinFeature {
    type Input = VideoJoinInput;
    type Output = VideoJoinOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "video.join".into(),
            name: "视频合并".into(),
            description: "多段视频按顺序合并（如片头/正片/片尾）".into(),
            kind: FeatureKind::VideoTool,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: VideoJoinInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<VideoJoinOutput, SomaError> {
        if input.videos.is_empty() {
            return Err(SomaError::Config("视频片段列表为空".into()));
        }
        let conf = ctx.config();
        let output_path = resolve_output(ctx, &input.output_path, "joined.mp4");
        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        let segments: Vec<&str> = input.videos.iter().map(|s| s.as_str()).collect();
        let ffmpeg = build_ffmpeg(conf, 2, None);
        ffmpeg.concat_videos(&segments, &output_path)?;
        register_video(ctx, &output_path);
        Ok(VideoJoinOutput { video: output_path })
    }
}

// ============ video.transition：转场特效 ============

/// video.transition 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoTransitionInput {
    /// 输入视频路径
    pub video: String,
    /// 转场类型：FadeIn（淡入）/ FadeOut（淡出），其他值跳过
    pub transition: String,
    /// 转场时长（秒），缺省 1
    #[serde(default)]
    pub duration: Option<f64>,
    /// 输出文件路径；缺省写入产物目录 transition.mp4
    #[serde(default)]
    pub output_path: Option<String>,
}

/// video.transition 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VideoTransitionOutput {
    /// 处理后的视频文件路径
    pub video: String,
}

/// 转场特效：为视频添加淡入/淡出效果
pub struct VideoTransitionFeature;

impl TypedFeature for VideoTransitionFeature {
    type Input = VideoTransitionInput;
    type Output = VideoTransitionOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "video.transition".into(),
            name: "转场特效".into(),
            description: "为视频添加淡入/淡出转场效果".into(),
            kind: FeatureKind::VideoTool,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: VideoTransitionInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<VideoTransitionOutput, SomaError> {
        let conf = ctx.config();
        let output_path = resolve_output(ctx, &input.output_path, "transition.mp4");
        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        let ffmpeg = build_ffmpeg(conf, 2, None);
        ffmpeg.add_transition(
            &input.video,
            &output_path,
            &input.transition,
            input.duration.unwrap_or(1.0),
        )?;
        register_video(ctx, &output_path);
        Ok(VideoTransitionOutput { video: output_path })
    }
}

// ============ video.clip_resize：裁切缩放 ============

/// video.clip_resize 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoClipResizeInput {
    /// 输入视频路径
    pub video: String,
    /// 目标画幅比例（"16:9"/"9:16"/"1:1"），缺省 9:16
    #[serde(default)]
    pub aspect: Option<String>,
    /// 截取起点（秒），缺省 0
    #[serde(default)]
    pub start: Option<f64>,
    /// 截取时长（秒），缺省到视频结尾（以 3600s 上限兜底）
    #[serde(default)]
    pub duration: Option<f64>,
    /// 是否去除音轨（原语固定 -an）
    #[serde(default)]
    pub strip_audio: Option<bool>,
    /// 输出文件路径；缺省写入产物目录 resized.mp4
    #[serde(default)]
    pub output_path: Option<String>,
}

/// video.clip_resize 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VideoClipResizeOutput {
    /// 处理后的视频文件路径
    pub video: String,
    /// 实际分辨率 (宽, 高)
    pub resolution: (u32, u32),
}

/// 裁切缩放：按目标画幅智能裁切并缩放（背景模糊填充）
pub struct VideoClipResizeFeature;

impl TypedFeature for VideoClipResizeFeature {
    type Input = VideoClipResizeInput;
    type Output = VideoClipResizeOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "video.clip_resize".into(),
            name: "裁切缩放".into(),
            description: "按目标画幅裁切缩放视频（模糊背景填充），支持区间截取".into(),
            kind: FeatureKind::VideoTool,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: VideoClipResizeInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<VideoClipResizeOutput, SomaError> {
        let conf = ctx.config();
        let aspect = input
            .aspect
            .as_deref()
            .and_then(VideoAspect::from_str)
            .unwrap_or(VideoAspect::Portrait);
        let (width, height) = aspect_resolution(&aspect);
        let output_path = resolve_output(ctx, &input.output_path, "resized.mp4");
        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        let ffmpeg = build_ffmpeg(conf, 2, None);
        let duration = match input.duration {
            Some(d) => d,
            None => (ffmpeg.get_video_duration(&input.video)? - input.start.unwrap_or(0.0)).max(0.1),
        };
        ffmpeg.clip_and_resize(
            &input.video,
            &output_path,
            width,
            height,
            input.start.unwrap_or(0.0),
            duration,
        )?;
        if !input.strip_audio.unwrap_or(false) {
            // clip_and_resize 原语固定去除音轨；此处保持行为一致并如实告知
            log::warn!("clip_resize 输出不含音轨（底层原语固定 -an）");
        }
        register_video(ctx, &output_path);
        Ok(VideoClipResizeOutput {
            video: output_path,
            resolution: (width, height),
        })
    }
}

// ============ video.info：媒体信息探测 ============

/// video.info 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct VideoInfoInput {
    /// 视频文件路径
    pub video: String,
}

/// video.info 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct VideoInfoOutput {
    /// 时长（秒）
    pub duration: f64,
    /// 宽度（像素）
    pub width: u32,
    /// 高度（像素）
    pub height: u32,
    /// 画幅比例（"16:9"/"9:16"/"1:1" 或比例值）
    pub aspect: String,
}

/// 媒体信息探测：读取视频时长与分辨率
pub struct VideoInfoFeature;

impl TypedFeature for VideoInfoFeature {
    type Input = VideoInfoInput;
    type Output = VideoInfoOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "video.info".into(),
            name: "媒体信息".into(),
            description: "探测视频的时长与分辨率".into(),
            kind: FeatureKind::VideoTool,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: VideoInfoInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<VideoInfoOutput, SomaError> {
        if !std::path::Path::new(&input.video).exists() {
            return Err(SomaError::Config(format!("视频文件不存在: {}", input.video)));
        }
        let ffmpeg = build_ffmpeg(ctx.config(), 1, None);
        let duration = ffmpeg.get_video_duration(&input.video)?;
        let (width, height) = ffmpeg.get_video_resolution(&input.video)?;
        let aspect = detect_aspect(width, height);
        Ok(VideoInfoOutput { duration, width, height, aspect })
    }
}

/// 按画幅比例枚举取分辨率
fn aspect_resolution(aspect: &VideoAspect) -> (u32, u32) {
    aspect.to_resolution()
}

/// 根据宽高推断画幅比例描述
fn detect_aspect(width: u32, height: u32) -> String {
    if height > width {
        "9:16".into()
    } else if width > height {
        "16:9".into()
    } else {
        "1:1".into()
    }
}
