use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::SubtitleCue;
use crate::edge_tts::generate_subtitle_cues_from_text;
use crate::provider::{SomaTtsProvider, TtsResult};
use std::path::Path;

pub struct SiliconflowTts {
    api_key: String,
}

impl SiliconflowTts {
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
        if self.api_key.is_empty() {
            return Err(SomaError::Tts("SiliconFlow API key not set".into()));
        }

        let gain = (rate - 1.0).max(-10.0).min(10.0);
        let url = "https://api.siliconflow.cn/v1/audio/speech";

        let payload = serde_json::json!({
            "model": "FunAudioLLM/CosyVoice2-0.5B",
            "input": text,
            "voice": voice,
            "response_format": "mp3",
            "sample_rate": 32000,
            "stream": false,
            "speed": rate,
            "gain": gain,
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(SomaError::Tts(format!("SiliconFlow TTS failed: {} - {}", status, body)));
        }

        let bytes = resp.bytes().await.map_err(|e| SomaError::Http(e.to_string()))?;
        let output_str = output_path.to_string_lossy().to_string();
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }
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

fn get_audio_duration(audio_path: &str) -> Result<f64, SomaError> {
    let output = std::process::Command::new("ffprobe")
        .args(&["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", audio_path])
        .output()
        .map_err(|e| SomaError::Ffmpeg(format!("ffprobe failed: {}", e)))?;
    String::from_utf8_lossy(&output.stdout).trim().parse::<f64>()
        .map_err(|e| SomaError::Tts(format!("parse duration failed: {}", e)))
}
