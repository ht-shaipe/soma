/// soma-video 视频处理 crate
///
/// 提供基于 FFmpeg 的视频剪辑、拼接、转场特效、字幕渲染和音频合成等功能。
/// 核心结构体包括 [`Ffmpeg`]（FFmpeg 命令行封装）和 [`VideoComposer`]（视频合成编排器）。
pub mod ffmpeg;
pub mod compose;
pub mod effects;

pub use ffmpeg::Ffmpeg;
pub use compose::VideoComposer;
