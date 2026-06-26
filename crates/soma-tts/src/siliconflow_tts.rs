//! SiliconFlow TTS 引擎实现
//!
//! 基于 SiliconFlow 云端 TTS API（CosyVoice2 模型）进行语音合成。
//! API 文档：https://api.siliconflow.cn/v1/audio/speech

use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::SubtitleCue;
use crate::edge_tts::generate_subtitle_cues_from_text;
use crate::provider::{SomaTtsProvider, TtsResult};
use std::path::Path;

/// SiliconFlow TTS 语音合成器
///
/// 使用 SiliconFlow 平台的 CosyVoice2-0.5B 模型进行语音合成。
pub struct SiliconflowTts {
    /// SiliconFlow API 密钥
    api_key: String,
}

impl SiliconflowTts {
    /// 创建 SiliconflowTts 实例
    ///
    /// - `api_key` - SiliconFlow 平台的 API 密钥
    pub fn new(api_key: &str) -> Self {
        Self { api_key: api_key.to_string() }
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for SiliconflowTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        // 检查 API 密钥是否已设置
        if self.api_key.is_empty() {
            return Err(SomaError::Tts("SiliconFlow API key not set".into()));
        }

        // 将语速倍率转换为增益值，范围限制在 [-10, 10]
        let gain = (rate - 1.0).max(-10.0).min(10.0);
        let url = "https://api.siliconflow.cn/v1/audio/speech";

        // 构造 SiliconFlow TTS API 请求体
        let payload = serde_json::json!({
            "model": "FunAudioLLM/CosyVoice2-0.5B",  // 使用 CosyVoice2 模型
            "input": text,
            "voice": voice,
            "response_format": "mp3",   // 输出 MP3 格式
            "sample_rate": 32000,       // 采样率 32kHz
            "stream": false,            // 非流式返回完整音频
            "speed": rate,              // 语速倍率
            "gain": gain,               // 音频增益
        });

        // 发送 HTTP POST 请求到 SiliconFlow API
        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        // 检查 API 响应状态
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SomaError::Tts(format!("SiliconFlow TTS failed: {} - {}", status, body)));
        }

        // 将返回的音频数据写入文件
        let bytes = resp.bytes().await.map_err(|e| SomaError::Http(e.to_string()))?;
        let output_str = output_path.to_string_lossy().to_string();
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        std::fs::write(output_path, &bytes).map_err(SomaError::Io)?;

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

/// 使用 ffprobe 获取音频文件时长（秒）
fn get_audio_duration(audio_path: &str) -> Result<f64, SomaError> {
    let output = std::process::Command::new("ffprobe")
        .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", audio_path])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
    String::from_utf8_lossy(&output.stdout).trim().parse::<f64>()
        .map_err(|e| SomaError::Tts(format!("parse duration failed: {}", e)))
}
