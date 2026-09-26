//! 科大讯飞 TTS 引擎实现
//!
//! 基于讯飞开放平台语音合成 API 进行 TTS 合成。
//! 使用 HMAC-SHA256 签名进行认证。
//! API 文档：https://www.xfyun.cn/doc/tts/OnlineTTS/API.html

use async_trait::async_trait;
use soma_core::error::SomaError;
use crate::edge_tts::{generate_subtitle_cues_from_text, get_audio_duration};
use crate::provider::{SomaTtsProvider, TtsResult};
use std::path::Path;
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// 科大讯飞 TTS 语音合成器
pub struct XfyunTts {
    /// 讯飞开放平台 App ID
    app_id: String,
    /// 讯飞 API Key
    api_key: String,
    /// 讯飞 API Secret
    api_secret: String,
}

impl XfyunTts {
    pub fn new(app_id: &str, api_key: &str, api_secret: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            api_key: api_key.to_string(),
            api_secret: api_secret.to_string(),
        }
    }

    /// 生成讯飞 API 鉴权头
    ///
    /// 使用 HMAC-SHA256 算法对 host + date + request-line 进行签名，
    /// 返回 (authorization, date) 元组用于 HTTP 请求头。
    fn build_auth(&self, host: &str, path: &str, method: &str) -> (String, String) {
        let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        let request_line = format!("{} {} HTTP/1.1", method, path);
        let signature_origin = format!("host: {}\ndate: {}\n{}", host, date, request_line);

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC key length error");
        mac.update(signature_origin.as_bytes());
        let signature = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            mac.finalize().into_bytes(),
        );

        let authorization = format!(
            r#"api_key="{}", algorithm="hmac-sha256", headers="host date request-line", signature="{}""#,
            self.api_key, signature
        );

        (authorization, date)
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for XfyunTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        if self.app_id.is_empty() || self.api_key.is_empty() || self.api_secret.is_empty() {
            return Err(SomaError::Tts("讯飞 App ID / API Key / API Secret 未设置".into()));
        }

        let vcn = crate::voices::extract_xfyun_voice(voice).unwrap_or_else(|| voice.to_string());
        let text_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            text.as_bytes(),
        );

        let speed = ((rate * 50.0).round() as i32).clamp(0, 100);

        let payload = serde_json::json!({
            "header": {
                "app_id": self.app_id,
                "status": 3
            },
            "parameter": {
                "tts": {
                    "vcn": vcn,
                    "audio_params": {
                        "aue": "lame",
                        "speed": speed,
                        "volume": 50
                    }
                }
            },
            "payload": {
                "tts": {
                    "text": text_b64
                }
            }
        });

        let host = "tts-api.xfyun.cn";
        let path = "/v2/tts";
        let (authorization, date) = self.build_auth(host, path, "GET");

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("https://{}{}", host, path))
            .header("Authorization", authorization)
            .header("Date", date)
            .header("Host", host)
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SomaError::Tts(format!("讯飞 TTS 失败: {} - {}", status, body)));
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::Tts(format!("解析讯飞响应失败: {}", e)))?;

        let code = resp_json
            .get("header")
            .and_then(|h| h.get("code"))
            .and_then(|v| v.as_i64())
            .unwrap_or(-1);
        if code != 0 {
            let message = resp_json
                .get("header")
                .and_then(|h| h.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            return Err(SomaError::Tts(format!("讯飞 TTS 错误: code={}, message={}", code, message)));
        }

        let audio_b64 = resp_json
            .get("payload")
            .and_then(|p| p.get("tts"))
            .and_then(|t| t.get("audio"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| SomaError::Tts("讯飞响应缺少音频数据".into()))?;

        let audio_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            audio_b64,
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
