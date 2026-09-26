//! EdgeTTS 引擎实现
//!
//! 基于 Microsoft Edge 在线 TTS 服务（通过 `edge-tts` 命令行工具）进行语音合成。
//! 同时提供静音音频生成、音频时长探测和基于文本的字幕时间轴生成功能。

use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::SubtitleCue;
use soma_core::utils;
use crate::provider::{SomaTtsProvider, TtsResult};
use crate::voices;
use std::path::Path;

/// EdgeTTS 语音合成器
///
/// 通过调用系统安装的 `edge-tts` CLI 工具完成语音合成。
pub struct EdgeTts {
    /// 命令超时时间（秒），为 None 时不设超时
    timeout: Option<f64>,
}

impl EdgeTts {
    /// 创建 EdgeTts 实例
    ///
    /// - `timeout` - 可选的命令执行超时时间（秒）
    pub fn new(timeout: Option<f64>) -> Self {
        Self { timeout }
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for EdgeTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        // 无语音模式：生成对应时长的静音音频
        if voices::is_no_voice(voice) {
            return self.synthesize_silent(text, output_path).await;
        }

        // 解析语音名称，去除性别后缀等
        let voice_name = voices::parse_voice_name(voice);
        // 将语速倍率转换为 edge-tts 所需的百分比格式（如 +50%、-20%）
        let rate_str = voices::convert_rate_to_percent(rate);

        // 确保输出目录存在
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        // 构造超时参数（如果设置了超时）
        let timeout_args: Vec<String> = self.timeout
            .map(|t| vec![format!("--timeout={}", t as u64)])
            .unwrap_or_default();

        // 调用 edge-tts 命令行工具进行语音合成
        let output_str = output_path.to_string_lossy().to_string();
        let result = tokio::process::Command::new("edge-tts")
            .args([
                "--voice", &voice_name,
                "--rate", &rate_str,
                "--text", text,
                "--write-media", &output_str,
            ])
            .args(&timeout_args)
            .output()
            .await
            .map_err(|e| SomaError::Tts(format!("edge-tts command failed: {}", e)))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SomaError::Tts(format!("edge-tts failed: {}", stderr)));
        }

        // 获取音频时长并生成字幕时间轴
        let audio_duration = get_audio_duration(&output_str)?;
        let cues = generate_subtitle_cues_from_text(text, audio_duration);

        Ok(TtsResult {
            audio_file: output_str,
            audio_duration,
            subtitle_cues: cues,
        })
    }
}

impl EdgeTts {
    /// 生成静音音频文件
    ///
    /// 当语音设置为 "no-voice" 时调用，使用 ffmpeg 生成指定时长的静音 MP3 文件。
    /// 时长根据文本长度估算（参见 [`voices::estimate_no_voice_duration`]）。
    async fn synthesize_silent(&self, text: &str, output_path: &Path) -> Result<TtsResult, SomaError> {
        // 根据文本长度估算无语音时的音频时长
        let duration = voices::estimate_no_voice_duration(text);
        let output_str = output_path.to_string_lossy().to_string();

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        // 使用 ffmpeg 的 anullsrc 滤镜生成静音音频
        let ffmpeg = "ffmpeg";
        let result = tokio::process::Command::new(ffmpeg)
            .args([
                "-y", "-f", "lavfi",
                "-i", "anullsrc=r=44100:cl=mono",
                "-t", &format!("{:.3}", duration),
                "-codec:a", "libmp3lame",
                "-q:a", "4",
                &output_str,
            ])
            .output()
            .await
            .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg silent audio failed: {}", e)))?;

        if !result.status.success() {
            return Err(SomaError::Ffmpeg("failed to generate silent audio".into()));
        }

        // 即使是静音音频也生成字幕时间轴，便于视频对齐
        let cues = generate_subtitle_cues_from_text(text, duration);
        Ok(TtsResult {
            audio_file: output_str,
            audio_duration: duration,
            subtitle_cues: cues,
        })
    }
}

/// 使用 ffprobe 获取音频文件时长
///
/// 通过调用 `ffprobe` 命令解析音频文件的 format duration 信息。
///
/// - `audio_path` - 音频文件路径
///
/// 返回音频时长（秒）。
pub fn get_audio_duration(audio_path: &str) -> Result<f64, SomaError> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            audio_path,
        ])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;

    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    duration_str.parse::<f64>()
        .map_err(|e| SomaError::Tts(format!("failed to parse audio duration: {}", e)))
}

/// 根据文本内容按标点分句，并按字符数比例分配字幕时间轴
///
/// 算法逻辑：
/// 1. 将文本按标点符号分割为句子列表
/// 2. 统计所有句子的总字符数
/// 3. 按每句字符数占总字符数的比例分配音频时长
/// 4. 最后一句取剩余时长，确保不超出总时长
///
/// - `text` - 原始文本
/// - `audio_duration` - 音频总时长（秒）
///
/// 返回字幕时间轴列表 [`SubtitleCue`]。
pub fn generate_subtitle_cues_from_text(text: &str, audio_duration: f64) -> Vec<SubtitleCue> {
    let sentences = utils::split_string_by_punctuations(text);
    if sentences.is_empty() {
        return vec![SubtitleCue {
            index: 1,
            start_ms: 0,
            end_ms: (audio_duration * 10000.0) as u64,
            text: text.to_string(),
        }];
    }

    let max_line_width: usize = 14;
    let mut split_sentences: Vec<String> = Vec::new();
    for sentence in &sentences {
        if sentence.trim().is_empty() {
            continue;
        }
        let width: usize = sentence.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum();
        if width <= max_line_width * 2 {
            split_sentences.push(sentence.clone());
        } else {
            let mut chunk = String::new();
            let mut chunk_width: usize = 0;
            for ch in sentence.chars() {
                let cw = if ch.is_ascii() { 1 } else { 2 };
                if chunk_width + cw > max_line_width * 2 && !chunk.is_empty() {
                    split_sentences.push(chunk.trim().to_string());
                    chunk.clear();
                    chunk_width = 0;
                }
                chunk.push(ch);
                chunk_width += cw;
            }
            if !chunk.trim().is_empty() {
                split_sentences.push(chunk.trim().to_string());
            }
        }
    }

    if split_sentences.is_empty() {
        return vec![];
    }

    let total_chars: usize = split_sentences.iter().map(|s| s.chars().count()).sum();
    if total_chars == 0 {
        return vec![];
    }

    let audio_duration_100ns = (audio_duration * 10_000_000.0) as u64;
    let mut cues = Vec::new();
    let mut current_offset: u64 = 0;

    for (i, sentence) in split_sentences.iter().enumerate() {
        if sentence.trim().is_empty() {
            continue;
        }
        let sentence_chars = sentence.chars().count();
        let sentence_duration = if i == split_sentences.len() - 1 {
            audio_duration_100ns.saturating_sub(current_offset)
        } else {
            ((audio_duration_100ns as f64) * (sentence_chars as f64 / total_chars as f64)).max(1.0) as u64
        };
        let sentence_end = (current_offset + sentence_duration).min(audio_duration_100ns);

        cues.push(SubtitleCue {
            index: (cues.len() + 1) as u32,
            start_ms: current_offset / 10_000,
            end_ms: sentence_end / 10_000,
            text: sentence.clone(),
        });

        current_offset = sentence_end;
    }

    cues
}

/// 根据字幕时间轴生成 SRT 格式字幕文件
///
/// - `cues` - 字幕时间轴列表
/// - `output_path` - SRT 文件输出路径
pub fn create_subtitle_file(cues: &[SubtitleCue], output_path: &str) -> Result<(), SomaError> {
    crate::subtitle::create_subtitle_file(cues, output_path)
}
