use async_trait::async_trait;
use soma_core::error::SomaError;
use crate::edge_tts::{generate_subtitle_cues_from_text, get_audio_duration};
use crate::provider::{SomaTtsProvider, TtsResult};
use std::path::Path;

pub struct ElevenlabsTts {
    api_key: String,
    model_id: String,
}

impl ElevenlabsTts {
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
        _rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        if self.api_key.is_empty() {
            return Err(SomaError::Tts("ElevenLabs API key not set".into()));
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        let url = format!("https://api.elevenlabs.io/v1/text-to-speech/{}", voice_id);
        let payload = serde_json::json!({
            "text": text,
            "model_id": self.model_id,
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "audio/mpeg")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(SomaError::Tts(format!("ElevenLabs TTS failed: {}", resp.status())));
        }

        let bytes = resp.bytes().await.map_err(|e| SomaError::Http(e.to_string()))?;
        let output_str = output_path.to_string_lossy().to_string();
        std::fs::write(output_path, &bytes).map_err(SomaError::Io)?;

        let audio_duration = get_audio_duration(&output_str)?;
        let cues = generate_subtitle_cues_from_text(text, audio_duration);

        Ok(TtsResult {
            audio_file: output_str,
            audio_duration,
            subtitle_cues: cues,
        })
    }
}
