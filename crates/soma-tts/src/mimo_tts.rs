//! MiMo TTS 引擎实现
//!
//! 基于小米 MiMo 平台的 TTS API 进行语音合成。
//! MiMo API 以聊天补全接口形式提供 TTS 服务，音频数据以 Base64 编码返回。

use crate::edge_tts::{generate_subtitle_cues_from_text, get_audio_duration};
use crate::provider::{SomaTtsProvider, TtsResult};
use async_trait::async_trait;
use base64::Engine;
use soma_core::error::SomaError;
use std::path::Path;

/// MiMo TTS 语音合成器
///
/// 使用小米 MiMo 平台的 TTS 能力，通过 Chat Completions API 获取语音合成结果。
pub struct MimoTts {
    /// MiMo API 密钥
    api_key: String,
    /// API 基础 URL，默认为 `https://api.xiaomimimo.com/v1`
    base_url: String,
    /// 模型名称，默认为 `mimo-v2.5-tts`
    model_name: String,
    /// 风格提示词，用于指导语音合成的语气和风格，
    /// 默认为 "请用自然、清晰、适合短视频旁白的语气朗读。"
    style_prompt: String,
}

impl MimoTts {
    /// 创建 MimoTts 实例
    ///
    /// - `api_key` - MiMo 平台 API 密钥
    /// - `base_url` - API 基础 URL，为空时使用默认地址
    /// - `model_name` - 模型名称，为空时使用 `mimo-v2.5-tts`
    /// - `style_prompt` - 风格提示词，为空时使用默认提示词
    pub fn new(api_key: &str, base_url: &str, model_name: &str, style_prompt: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: if base_url.is_empty() {
                "https://api.xiaomimimo.com/v1".into()
            } else {
                base_url.into()
            },
            model_name: if model_name.is_empty() {
                "mimo-v2.5-tts".into()
            } else {
                model_name.into()
            },
            style_prompt: if style_prompt.is_empty() {
                "请用自然、清晰、适合短视频旁白的语气朗读。".into()
            } else {
                style_prompt.into()
            },
        }
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for MimoTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        _rate: f32, // MiMo API 不直接支持语速参数，此处忽略
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        // 检查 API 密钥是否已设置
        if self.api_key.is_empty() {
            return Err(SomaError::Tts("MiMo API key not set".into()));
        }

        // 确保输出目录存在
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        // 构造 MiMo Chat Completions 请求体
        // user 角色携带风格提示词，assistant 角色携带待合成文本
        // audio 字段指定输出格式和语音
        let payload = serde_json::json!({
            "model": self.model_name,
            "messages": [
                {"role": "user", "content": self.style_prompt},
                {"role": "assistant", "content": text}
            ],
            "audio": {"format": "mp3", "voice": voice}
        });

        // 发送 HTTP POST 请求到 MiMo Chat Completions API
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        // 检查 API 响应状态
        if !resp.status().is_success() {
            return Err(SomaError::Tts(format!(
                "MiMo TTS failed: {}",
                resp.status()
            )));
        }

        // 从响应 JSON 中提取 Base64 编码的音频数据
        // 路径：choices[0].message.audio.data
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;
        let audio_data = body
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("audio"))
            .and_then(|a| a.get("data"))
            .and_then(|d| d.as_str())
            .ok_or_else(|| SomaError::Tts("MiMo TTS: no audio data in response".into()))?;

        // Base64 解码音频数据并写入文件
        let audio_bytes = base64::engine::general_purpose::STANDARD
            .decode(audio_data)
            .map_err(|e| SomaError::Tts(format!("base64 decode failed: {}", e)))?;
        let output_str = output_path.to_string_lossy().to_string();
        std::fs::write(output_path, &audio_bytes).map_err(SomaError::Io)?;

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
