use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use soma_core::error::SomaError;
use crate::edge_tts::{generate_subtitle_cues_from_text, get_audio_duration};
use crate::provider::{SomaTtsProvider, TtsResult};
use std::path::Path;

pub struct MimoTts {
    api_key: String,
    base_url: String,
    model_name: String,
    style_prompt: String,
}

impl MimoTts {
    pub fn new(api_key: &str, base_url: &str, model_name: &str, style_prompt: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: if base_url.is_empty() { "https://api.xiaomimimo.com/v1".into() } else { base_url.into() },
            model_name: if model_name.is_empty() { "mimo-v2.5-tts".into() } else { model_name.into() },
            style_prompt: if style_prompt.is_empty() { "请用自然、清晰、适合短视频旁白的语气朗读。".into() } else { style_prompt.into() },
        }
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for MimoTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        _rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        if self.api_key.is_empty() {
            return Err(SomaError::Tts("MiMo API key not set".into()));
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        let payload = serde_json::json!({
            "model": self.model_name,
            "messages": [
                {"role": "user", "content": self.style_prompt},
                {"role": "assistant", "content": text}
            ],
            "audio": {"format": "mp3", "voice": voice}
        });

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

        if !resp.status().is_success() {
            return Err(SomaError::Tts(format!("MiMo TTS failed: {}", resp.status())));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| SomaError::Http(e.to_string()))?;
        let audio_data = body.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("audio"))
            .and_then(|a| a.get("data"))
            .and_then(|d| d.as_str())
            .ok_or_else(|| SomaError::Tts("MiMo TTS: no audio data in response".into()))?;

        let audio_bytes = base64::engine::general_purpose::STANDARD.decode(audio_data).map_err(|e| SomaError::Tts(format!("base64 decode failed: {}", e)))?;
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
