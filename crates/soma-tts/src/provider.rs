//! TTS 服务提供者抽象与合成结果定义
//!
//! 定义了所有 TTS 引擎必须实现的 [`SomaTtsProvider`] trait，
//! 以及合成完成后的返回结构 [`TtsResult`]。

use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::SubtitleCue;
use std::path::Path;

/// TTS 服务提供者 trait
///
/// 所有 TTS 引擎（EdgeTTS、SiliconFlow、ElevenLabs、MiMo 等）均需实现此 trait。
/// 使用 `?Send` 标记是因为部分引擎的异步运行时不要求 `Send`。
#[async_trait(?Send)]
pub trait SomaTtsProvider: Send + Sync {
    /// 将文本合成为语音文件
    ///
    /// - `text` - 待合成的文本内容
    /// - `voice` - 语音名称（可能包含引擎前缀，如 `siliconflow:xxx`）
    /// - `rate` - 语速倍率（1.0 为正常语速）
    /// - `output_path` - 音频文件输出路径
    ///
    /// 返回 [`TtsResult`]，包含音频文件路径、时长和字幕时间轴。
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError>;
}

/// TTS 合成结果
#[derive(Debug, Clone)]
pub struct TtsResult {
    /// 生成的音频文件路径
    pub audio_file: String,
    /// 音频时长（秒）
    pub audio_duration: f64,
    /// 根据文本自动生成的字幕时间轴列表
    pub subtitle_cues: Vec<SubtitleCue>,
}
