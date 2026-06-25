use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::SubtitleCue;
use std::path::Path;

#[async_trait(?Send)]
pub trait SomaTtsProvider: Send + Sync {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError>;
}

#[derive(Debug, Clone)]
pub struct TtsResult {
    pub audio_file: String,
    pub audio_duration: f64,
    pub subtitle_cues: Vec<SubtitleCue>,
}
