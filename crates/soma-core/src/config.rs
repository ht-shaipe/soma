use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::error::SomaError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub app: AppSection,
    pub whisper: WhisperSection,
    pub proxy: ProxySection,
    pub azure: AzureSection,
    pub siliconflow: SiliconflowSection,
    pub elevenlabs: ElevenlabsSection,
    pub ui: UiSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSection {
    pub video_source: Option<String>,
    pub hide_config: Option<bool>,
    pub edge_tts_timeout: Option<f64>,
    pub tls_verify: Option<bool>,

    pub pexels_api_keys: Option<Vec<String>>,
    pub pixabay_api_keys: Option<Vec<String>>,
    pub coverr_api_keys: Option<Vec<String>>,

    pub llm_provider: Option<String>,
    pub openai_api_key: Option<String>,
    pub openai_base_url: Option<String>,
    pub openai_model_name: Option<String>,

    pub deepseek_api_key: Option<String>,
    pub deepseek_base_url: Option<String>,
    pub deepseek_model_name: Option<String>,

    pub qwen_api_key: Option<String>,
    pub qwen_model_name: Option<String>,

    pub gemini_api_key: Option<String>,
    pub gemini_model_name: Option<String>,

    pub moonshot_api_key: Option<String>,
    pub moonshot_base_url: Option<String>,
    pub moonshot_model_name: Option<String>,

    pub azure_api_key: Option<String>,
    pub azure_base_url: Option<String>,
    pub azure_model_name: Option<String>,
    pub azure_api_version: Option<String>,

    pub ollama_base_url: Option<String>,
    pub ollama_model_name: Option<String>,

    pub groq_api_key: Option<String>,
    pub groq_model_name: Option<String>,
    pub groq_base_url: Option<String>,

    pub grok_api_key: Option<String>,
    pub grok_model_name: Option<String>,
    pub grok_base_url: Option<String>,

    pub minimax_api_key: Option<String>,
    pub minimax_base_url: Option<String>,
    pub minimax_model_name: Option<String>,

    pub evolink_api_key: Option<String>,
    pub evolink_base_url: Option<String>,
    pub evolink_model_name: Option<String>,

    pub mimo_api_key: Option<String>,
    pub mimo_base_url: Option<String>,
    pub mimo_model_name: Option<String>,
    pub mimo_tts_model_name: Option<String>,
    pub mimo_tts_style_prompt: Option<String>,

    pub aihubmix_api_key: Option<String>,
    pub aihubmix_base_url: Option<String>,
    pub aihubmix_model_name: Option<String>,

    pub aimlapi_api_key: Option<String>,
    pub aimlapi_base_url: Option<String>,
    pub aimlapi_model_name: Option<String>,

    pub oneapi_api_key: Option<String>,
    pub oneapi_base_url: Option<String>,
    pub oneapi_model_name: Option<String>,

    pub modelscope_api_key: Option<String>,
    pub modelscope_base_url: Option<String>,
    pub modelscope_model_name: Option<String>,

    pub pollinations_api_key: Option<String>,
    pub pollinations_base_url: Option<String>,
    pub pollinations_model_name: Option<String>,

    pub litellm_model_name: Option<String>,

    pub enable_g4f: Option<bool>,
    pub g4f_model_name: Option<String>,

    pub subtitle_provider: Option<String>,

    pub imagemagick_path: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub video_codec: Option<String>,

    pub endpoint: Option<String>,
    pub material_directory: Option<String>,

    pub enable_redis: Option<bool>,
    pub redis_host: Option<String>,
    pub redis_port: Option<u32>,
    pub redis_db: Option<u32>,
    pub redis_password: Option<String>,

    pub max_concurrent_tasks: Option<usize>,
    pub max_queued_tasks: Option<usize>,

    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub storage_path: Option<String>,
    pub concurrent_tasks: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhisperSection {
    pub model_size: Option<String>,
    pub device: Option<String>,
    pub compute_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxySection {
    pub http: Option<String>,
    pub https: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AzureSection {
    pub speech_key: Option<String>,
    pub speech_region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SiliconflowSection {
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElevenlabsSection {
    pub api_key: Option<String>,
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiSection {
    pub hide_log: Option<bool>,
    pub subtitle_position: Option<String>,
    pub custom_position: Option<f64>,

    pub upload_post_enabled: Option<bool>,
    pub upload_post_api_key: Option<String>,
    pub upload_post_username: Option<String>,
    pub upload_post_platforms: Option<Vec<String>>,
    pub upload_post_auto_upload: Option<bool>,
    pub upload_post_youtube_privacy_status: Option<String>,
}

impl AppConfig {
    pub fn load_toml(path: &str) -> Result<AppConfig, SomaError> {
        let content = fs::read_to_string(path).map_err(SomaError::Io)?;
        toml::from_str(&content).map_err(|e| SomaError::Config(e.to_string()))
    }

    pub fn get_proxy_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        if let Some(ref http) = self.proxy.http {
            map.insert("http".to_string(), http.clone());
        }
        if let Some(ref https) = self.proxy.https {
            map.insert("https".to_string(), https.clone());
        }
        map
    }

    pub fn get_listen_host(&self) -> &str {
        self.app.host.as_deref().unwrap_or("0.0.0.0")
    }

    pub fn get_listen_port(&self) -> u16 {
        self.app.port.unwrap_or(8080)
    }

    pub fn get_storage_path(&self) -> &str {
        self.app.storage_path.as_deref().unwrap_or("./storage")
    }

    pub fn get_max_concurrent_tasks(&self) -> usize {
        self.app.max_concurrent_tasks.unwrap_or(5)
    }

    pub fn get_max_queued_tasks(&self) -> usize {
        self.app.max_queued_tasks.unwrap_or(100)
    }

    pub fn get_subtitle_provider(&self) -> &str {
        self.app.subtitle_provider.as_deref().unwrap_or("edge")
    }

    pub fn get_edge_tts_timeout(&self) -> Option<f64> {
        self.app.edge_tts_timeout.filter(|&t| t > 0.0)
    }

    pub fn get_tls_verify(&self) -> bool {
        self.app.tls_verify.unwrap_or(true)
    }

    pub fn get_video_codec(&self) -> &str {
        self.app.video_codec.as_deref().unwrap_or("libx264")
    }

    pub fn get_endpoint(&self) -> &str {
        self.app.endpoint.as_deref().unwrap_or("")
    }

    pub fn get_ffmpeg_binary(&self) -> String {
        if let Some(ref path) = self.app.ffmpeg_path {
            if Path::new(path).exists() {
                return path.clone();
            }
        }
        if let Ok(which) = which::which("ffmpeg") {
            return which.to_string_lossy().to_string();
        }
        "ffmpeg".to_string()
    }
}
