//! Azure Cognitive Services TTS 引擎实现
//!
//! 通过 Azure Speech Service REST API v1 进行语音合成。
//! 支持 V2 语音（如 zh-CN-XiaoxiaoNeural-V2），自动检测 V2 后缀并映射到正确 API。
//! 返回的音频格式为 MP3（通过请求 audio-24khz-96kbitrate-mono-mp3 output 格式）。

use crate::edge_tts::{generate_subtitle_cues_from_text, get_audio_duration};
use crate::provider::{SomaTtsProvider, TtsResult};
use crate::voices;
use async_trait::async_trait;
use soma_core::error::SomaError;
use std::path::Path;

/// Azure TTS 语音合成器
///
/// 通过 Azure Speech Service REST API 进行语音合成。
pub struct AzureTts {
    /// Azure 语音服务订阅密钥
    speech_key: String,
    /// Azure 语音服务区域（如 "eastasia"）
    speech_region: String,
}

impl AzureTts {
    /// 创建 AzureTts 实例
    ///
    /// - `speech_key` - Azure 语音服务订阅密钥
    /// - `speech_region` - Azure 语音服务区域
    pub fn new(speech_key: &str, speech_region: &str) -> Self {
        Self {
            speech_key: speech_key.to_string(),
            speech_region: speech_region.to_string(),
        }
    }

    /// 构造 Azure TTS REST API URL
    fn tts_url(&self) -> String {
        format!(
            "https://{}.tts.speech.microsoft.com/cognitiveservices/v1",
            self.speech_region
        )
    }

    /// 构造 SSML 请求体
    ///
    /// Azure TTS 需要 SSML 格式指定语音和语速。
    /// 语速通过 `<prosody>` 标签的 rate 属性控制，格式为百分比（如 "+50%"）。
    fn build_ssml(voice_name: &str, text: &str, rate: f32) -> String {
        let rate_str = voices::convert_rate_to_percent(rate);
        format!(
            r#"<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='en-US'>
<voice name='{}'>
<prosody rate='{}'>
{}
</prosody>
</voice>
</speak>"#,
            voice_name, rate_str, text
        )
    }
}

#[async_trait(?Send)]
impl SomaTtsProvider for AzureTts {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError> {
        if voices::is_no_voice(voice) {
            let duration = voices::estimate_no_voice_duration(text);
            let output_str = output_path.to_string_lossy().to_string();
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
            }
            let result = tokio::process::Command::new("ffmpeg")
                .args([
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    "anullsrc=r=44100:cl=mono",
                    "-t",
                    &format!("{:.3}", duration),
                    "-codec:a",
                    "libmp3lame",
                    "-q:a",
                    "4",
                    &output_str,
                ])
                .output()
                .await
                .map_err(|e| SomaError::Ffmpeg(format!("ffmpeg silent audio failed: {}", e)))?;
            if !result.status.success() {
                return Err(SomaError::Ffmpeg("failed to generate silent audio".into()));
            }
            let cues = generate_subtitle_cues_from_text(text, duration);
            return Ok(TtsResult {
                audio_file: output_str,
                audio_duration: duration,
                subtitle_cues: cues,
            });
        }

        // 检测 V2 语音，去除 -V2 后缀用于 API 调用
        let voice_name = if let Some(v2_name) = voices::is_azure_v2_voice(voice) {
            v2_name
        } else {
            voices::parse_voice_name(voice)
        };

        let ssml = Self::build_ssml(&voice_name, text, rate);

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(SomaError::Io)?;
        }

        let url = self.tts_url();
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Ocp-Apim-Subscription-Key", &self.speech_key)
            .header("Content-Type", "application/ssml+xml")
            .header(
                "X-Microsoft-OutputFormat",
                "audio-24khz-96kbitrate-mono-mp3",
            )
            .header("User-Agent", "soma-tts")
            .body(ssml)
            .send()
            .await
            .map_err(|e| SomaError::Tts(format!("Azure TTS request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(SomaError::Tts(format!(
                "Azure TTS API error: {} - {}",
                status, body
            )));
        }

        let audio_data = response
            .bytes()
            .await
            .map_err(|e| SomaError::Tts(format!("Azure TTS read response failed: {}", e)))?;

        let output_str = output_path.to_string_lossy().to_string();
        std::fs::write(&output_str, &audio_data)
            .map_err(|e| SomaError::Tts(format!("Azure TTS write audio failed: {}", e)))?;

        let audio_duration = get_audio_duration(&output_str)?;
        let cues = generate_subtitle_cues_from_text(text, audio_duration);

        Ok(TtsResult {
            audio_file: output_str,
            audio_duration,
            subtitle_cues: cues,
        })
    }
}
