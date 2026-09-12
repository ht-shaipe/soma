//! 火山引擎（字节豆包）TTS 引擎实现
//!
//! 基于火山引擎语音合成 API 进行 TTS 合成和语音克隆。
//! API 文档：https://www.volcengine.com/docs/6561

use async_trait::async_trait;
use soma_core::error::SomaError;
use crate::edge_tts::{generate_subtitle_cues_from_text, get_audio_duration};
use crate::provider::{SomaTtsProvider, TtsResult};
use std::path::Path;

/// 火山引擎 TTS 语音合成器
pub struct VolcengineTts {
    /// 火山引擎 App ID
    app_id: String,
    /// 访问令牌
    access_token: String,
    /// 集群名称
    cluster: String,
}

impl VolcengineTts {
    pub fn new(app_id: &str, access_token: &str, cluster: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            access_token: access_token.to_string(),
            cluster: if cluster.is_empty() { "volcano_tts".into() } else { cluster.into() },
        }
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for VolcengineTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        if self.app_id.is_empty() || self.access_token.is_empty() {
            return Err(SomaError::Tts("火山引擎 App ID 或 Access Token 未设置".into()));
        }

        let voice_id = crate::voices::extract_volcengine_voice(voice).unwrap_or_else(|| voice.to_string());
        let reqid = uuid::Uuid::new_v4().to_string();

        let payload = serde_json::json!({
            "app": {
                "appid": self.app_id,
                "token": self.access_token,
                "cluster": self.cluster,
            },
            "audio": {
                "voice_type": voice_id,
                "encoding": "mp3",
                "rate": 24000,
                "speed_ratio": rate,
            },
            "request": {
                "reqid": reqid,
                "text": text,
                "operation": "query",
            }
        });

        let client = reqwest::Client::new();
        let resp = client
            .post("https://openspeech.bytedance.com/api/v1/tts")
            .header("Authorization", format!("Bearer;{}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SomaError::Tts(format!("火山引擎 TTS 失败: {} - {}", status, body)));
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::Tts(format!("解析火山引擎响应失败: {}", e)))?;

        let code = resp_json.get("code").and_then(|v| v.as_i64()).unwrap_or(0);
        if code != 3000 {
            let message = resp_json.get("message").and_then(|v| v.as_str()).unwrap_or("unknown");
            return Err(SomaError::Tts(format!("火山引擎 TTS 错误: code={}, message={}", code, message)));
        }

        let data_b64 = resp_json
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SomaError::Tts("火山引擎响应缺少音频数据".into()))?;

        let audio_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            data_b64,
        ).map_err(|e| SomaError::Tts(format!("Base64 解码失败: {}", e)))?;

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
        let output_str = output_path.to_string_lossy().to_string();
        std::fs::write(output_path, &audio_bytes).map_err(SomaError::Io)?;

        let audio_duration = get_audio_duration(&output_str)?;
        let cues = generate_subtitle_cues_from_text(text, audio_duration);

        Ok(TtsResult {
            audio_file: output_str,
            audio_duration,
            subtitle_cues: cues,
        })
    }
}
