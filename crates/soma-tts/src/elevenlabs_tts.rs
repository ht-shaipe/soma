//! ElevenLabs TTS 引擎实现
//!
//! 基于 ElevenLabs 云端 TTS API 进行语音合成。
//! 支持多语言模型（默认 eleven_multilingual_v2）。

use async_trait::async_trait;
use soma_core::error::SomaError;
use crate::edge_tts::{generate_subtitle_cues_from_text, get_audio_duration};
use crate::provider::{SomaTtsProvider, TtsResult};
use std::path::Path;

/// ElevenLabs TTS 语音合成器
///
/// 使用 ElevenLabs 平台的文本转语音 API 进行合成。
pub struct ElevenlabsTts {
    /// ElevenLabs API 密钥
    api_key: String,
    /// 模型 ID，默认为 `eleven_multilingual_v2`
    model_id: String,
}

impl ElevenlabsTts {
    /// 创建 ElevenlabsTts 实例
    ///
    /// - `api_key` - ElevenLabs 平台的 API 密钥
    /// - `model_id` - 模型 ID，为空时默认使用 `eleven_multilingual_v2`
    pub fn new(api_key: &str, model_id: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model_id: if model_id.is_empty() { "eleven_multilingual_v2".into() } else { model_id.into() },
        }
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for ElevenlabsTts {
    async fn synthesize(
        &self,
        text: &str,
        voice_id: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        if self.api_key.is_empty() {
            return Err(SomaError::Tts("ElevenLabs API key not set".into()));
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{}", voice_id);
        // rate 偏移量映射到 stability: rate>1 → stability低(更快), rate<1 → stability高(更慢)
        // stability 范围 0.0~1.0，默认 0.5
        let stability = (0.5 + (1.0 - rate as f64) * 0.3).clamp(0.0, 1.0);
        let similarity_boost = 0.75;
        let payload = serde_json::json!({
            "text": text,
            "model_id": self.model_id,
            "voice_settings": {
                "stability": stability,
                "similarity_boost": similarity_boost,
            }
        });

        // 发送 HTTP POST 请求，使用 xi-api-key 头进行认证
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "audio/mpeg")  // 请求返回 MP3 格式音频
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        // 检查 API 响应状态
        if !resp.status().is_success() {
            return Err(SomaError::Tts(format!("ElevenLabs TTS failed: {}", resp.status())));
        }

        // 将返回的音频数据写入文件
        let bytes = resp.bytes().await.map_err(|e| SomaError::Http(e.to_string()))?;
        let output_str = output_path.to_string_lossy().to_string();
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
