//! 配置模块
//!
//! 定义应用所有配置项的结构体，支持从 TOML 文件加载配置，
//! 并提供各配置项的带默认值访问方法。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::error::SomaError;

/// 应用总配置，对应 TOML 配置文件的顶层结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// 应用基本配置（视频源、LLM、字幕等）
    pub app: AppSection,
    /// Whisper 语音识别模型配置
    pub whisper: WhisperSection,
    /// 网络代理配置
    pub proxy: ProxySection,
    /// Azure 语音服务配置
    pub azure: AzureSection,
    /// SiliconFlow TTS 配置
    pub siliconflow: SiliconflowSection,
    /// ElevenLabs TTS 配置
    pub elevenlabs: ElevenlabsSection,
    /// UI 界面与发布相关配置
    pub ui: UiSection,
}

/// 应用基本配置段，涵盖视频源、LLM 提供者、字幕、存储等核心参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSection {
    // ── 视频素材源 ──
    /// 视频素材来源，如 "pexels"、"pixabay"、"coverr"
    pub video_source: Option<String>,
    /// 是否隐藏配置页面
    pub hide_config: Option<bool>,
    /// Edge TTS 请求超时时间（秒），0 或 None 表示不限制
    pub edge_tts_timeout: Option<f64>,
    /// 是否验证 TLS 证书，默认 true
    pub tls_verify: Option<bool>,

    /// Pexels API 密钥列表，支持多 Key 轮换
    pub pexels_api_keys: Option<Vec<String>>,
    /// Pixabay API 密钥列表
    pub pixabay_api_keys: Option<Vec<String>>,
    /// Coverr API 密钥列表
    pub coverr_api_keys: Option<Vec<String>>,

    // ── LLM 提供者 ──
    /// 当前使用的 LLM 提供者名称，如 "openai"、"deepseek"、"qwen" 等
    pub llm_provider: Option<String>,

    /// OpenAI API 密钥
    pub openai_api_key: Option<String>,
    /// OpenAI API 基础 URL
    pub openai_base_url: Option<String>,
    /// OpenAI 模型名称
    pub openai_model_name: Option<String>,

    /// DeepSeek API 密钥
    pub deepseek_api_key: Option<String>,
    /// DeepSeek API 基础 URL
    pub deepseek_base_url: Option<String>,
    /// DeepSeek 模型名称
    pub deepseek_model_name: Option<String>,

    /// 通义千问 API 密钥
    pub qwen_api_key: Option<String>,
    /// 通义千问模型名称
    pub qwen_model_name: Option<String>,

    /// Gemini API 密钥
    pub gemini_api_key: Option<String>,
    /// Gemini 模型名称
    pub gemini_model_name: Option<String>,

    /// Moonshot API 密钥
    pub moonshot_api_key: Option<String>,
    /// Moonshot API 基础 URL
    pub moonshot_base_url: Option<String>,
    /// Moonshot 模型名称
    pub moonshot_model_name: Option<String>,

    /// Azure OpenAI API 密钥
    pub azure_api_key: Option<String>,
    /// Azure OpenAI 端点 URL
    pub azure_base_url: Option<String>,
    /// Azure OpenAI 部署的模型名称
    pub azure_model_name: Option<String>,
    /// Azure OpenAI API 版本号
    pub azure_api_version: Option<String>,

    /// Ollama 服务基础 URL（本地部署）
    pub ollama_base_url: Option<String>,
    /// Ollama 模型名称
    pub ollama_model_name: Option<String>,

    /// Groq API 密钥
    pub groq_api_key: Option<String>,
    /// Groq 模型名称
    pub groq_model_name: Option<String>,
    /// Groq API 基础 URL
    pub groq_base_url: Option<String>,

    /// Grok API 密钥
    pub grok_api_key: Option<String>,
    /// Grok 模型名称
    pub grok_model_name: Option<String>,
    /// Grok API 基础 URL
    pub grok_base_url: Option<String>,

    /// MiniMax API 密钥
    pub minimax_api_key: Option<String>,
    /// MiniMax API 基础 URL
    pub minimax_base_url: Option<String>,
    /// MiniMax 模型名称
    pub minimax_model_name: Option<String>,

    /// Evolink API 密钥
    pub evolink_api_key: Option<String>,
    /// Evolink API 基础 URL
    pub evolink_base_url: Option<String>,
    /// Evolink 模型名称
    pub evolink_model_name: Option<String>,

    /// Mimo API 密钥
    pub mimo_api_key: Option<String>,
    /// Mimo API 基础 URL
    pub mimo_base_url: Option<String>,
    /// Mimo 模型名称
    pub mimo_model_name: Option<String>,
    /// Mimo TTS 语音合成模型名称
    pub mimo_tts_model_name: Option<String>,
    /// Mimo TTS 语音风格提示词
    pub mimo_tts_style_prompt: Option<String>,

    /// AIHubMix API 密钥
    pub aihubmix_api_key: Option<String>,
    /// AIHubMix API 基础 URL
    pub aihubmix_base_url: Option<String>,
    /// AIHubMix 模型名称
    pub aihubmix_model_name: Option<String>,

    /// AIMLAPI API 密钥
    pub aimlapi_api_key: Option<String>,
    /// AIMLAPI API 基础 URL
    pub aimlapi_base_url: Option<String>,
    /// AIMLAPI 模型名称
    pub aimlapi_model_name: Option<String>,

    /// OneAPI API 密钥（兼容 OpenAI 格式的聚合网关）
    pub oneapi_api_key: Option<String>,
    /// OneAPI 基础 URL
    pub oneapi_base_url: Option<String>,
    /// OneAPI 模型名称
    pub oneapi_model_name: Option<String>,

    /// ModelScope API 密钥
    pub modelscope_api_key: Option<String>,
    /// ModelScope 基础 URL
    pub modelscope_base_url: Option<String>,
    /// ModelScope 模型名称
    pub modelscope_model_name: Option<String>,

    /// Pollinations API 密钥
    pub pollinations_api_key: Option<String>,
    /// Pollinations 基础 URL
    pub pollinations_base_url: Option<String>,
    /// Pollinations 模型名称
    pub pollinations_model_name: Option<String>,

    /// LiteLLM 代理模型名称（通过 LiteLLM 代理统一调用多种 LLM）
    pub litellm_model_name: Option<String>,

    /// 是否启用 G4F（免费 GPT 接口）
    pub enable_g4f: Option<bool>,
    /// G4F 模型名称
    pub g4f_model_name: Option<String>,

    // ── 字幕 ──
    /// 字幕服务提供者，如 "edge"、"whisper"
    pub subtitle_provider: Option<String>,

    // ── 工具路径 ──
    /// ImageMagick 可执行文件路径
    pub imagemagick_path: Option<String>,
    /// FFmpeg 可执行文件路径
    pub ffmpeg_path: Option<String>,
    /// 视频编码器，如 "libx264"、"libx265"
    pub video_codec: Option<String>,

    // ── 网络 / 存储 ──
    /// 服务外部访问端点 URL，用于生成资源的公网访问地址
    pub endpoint: Option<String>,
    /// 本地素材目录路径
    pub material_directory: Option<String>,

    // ── Redis ──
    /// 是否启用 Redis 作为任务存储后端
    pub enable_redis: Option<bool>,
    /// Redis 主机地址
    pub redis_host: Option<String>,
    /// Redis 端口号
    pub redis_port: Option<u32>,
    /// Redis 数据库编号
    pub redis_db: Option<u32>,
    /// Redis 认证密码
    pub redis_password: Option<String>,

    // ── 并发控制 ──
    /// 最大并发任务数
    pub max_concurrent_tasks: Option<usize>,
    /// 最大排队等待任务数
    pub max_queued_tasks: Option<usize>,

    // ── 服务监听 ──
    /// 服务名称
    pub name: Option<String>,
    /// 监听主机地址
    pub host: Option<String>,
    /// 监听端口号
    pub port: Option<u16>,
    /// 存储目录路径
    pub storage_path: Option<String>,
    /// 并发任务数（旧字段，建议使用 max_concurrent_tasks）
    pub concurrent_tasks: Option<usize>,
}

/// Whisper 语音识别模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhisperSection {
    /// 模型大小，如 "tiny"、"base"、"small"、"medium"、"large"
    pub model_size: Option<String>,
    /// 推理设备，如 "cpu"、"cuda"
    pub device: Option<String>,
    /// 计算精度，如 "int8"、"float16"
    pub compute_type: Option<String>,
}

/// 网络代理配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxySection {
    /// HTTP 代理地址
    pub http: Option<String>,
    /// HTTPS 代理地址
    pub https: Option<String>,
}

/// Azure 语音服务配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AzureSection {
    /// Azure 语音服务订阅密钥
    pub speech_key: Option<String>,
    /// Azure 语音服务区域，如 "eastasia"
    pub speech_region: Option<String>,
}

/// SiliconFlow TTS 语音合成配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SiliconflowSection {
    /// SiliconFlow API 密钥
    pub api_key: Option<String>,
}

/// ElevenLabs TTS 语音合成配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElevenlabsSection {
    /// ElevenLabs API 密钥
    pub api_key: Option<String>,
    /// ElevenLabs 语音模型 ID
    pub model_id: Option<String>,
}

/// UI 界面与视频发布相关配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiSection {
    /// 是否隐藏日志面板
    pub hide_log: Option<bool>,
    /// 字幕位置，如 "top"、"bottom"、"custom"
    pub subtitle_position: Option<String>,
    /// 自定义字幕垂直位置比例（0.0~1.0），仅当 subtitle_position 为 "custom" 时生效
    pub custom_position: Option<f64>,

    /// 是否启用视频上传发布功能
    pub upload_post_enabled: Option<bool>,
    /// 上传发布 API 密钥
    pub upload_post_api_key: Option<String>,
    /// 上传发布用户名
    pub upload_post_username: Option<String>,
    /// 发布目标平台列表，如 ["youtube", "tiktok"]
    pub upload_post_platforms: Option<Vec<String>>,
    /// 是否在视频生成完成后自动上传
    pub upload_post_auto_upload: Option<bool>,
    /// YouTube 视频隐私状态，如 "public"、"private"、"unlisted"
    pub upload_post_youtube_privacy_status: Option<String>,
}

impl AppConfig {
    /// 从 TOML 文件路径加载配置
    ///
    /// # 参数
    /// - `path`: TOML 配置文件路径
    ///
    /// # 返回
    /// 解析成功返回 `AppConfig`，失败返回 `SomaError::Io` 或 `SomaError::Config`
    pub fn load_toml(path: &str) -> Result<AppConfig, SomaError> {
        let content = fs::read_to_string(path).map_err(SomaError::Io)?;
        toml::from_str(&content).map_err(|e| SomaError::Config(e.to_string()))
    }

    /// 获取代理配置的 HashMap，键为协议名（"http"/"https"），值为代理地址
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

    /// 获取服务监听主机地址，默认 "0.0.0.0"
    pub fn get_listen_host(&self) -> &str {
        self.app.host.as_deref().unwrap_or("0.0.0.0")
    }

    /// 获取服务监听端口号，默认 8080
    pub fn get_listen_port(&self) -> u16 {
        self.app.port.unwrap_or(8080)
    }

    /// 获取存储目录路径，默认 "./storage"
    pub fn get_storage_path(&self) -> &str {
        self.app.storage_path.as_deref().unwrap_or("./storage")
    }

    /// 获取最大并发任务数，默认 5
    pub fn get_max_concurrent_tasks(&self) -> usize {
        self.app.max_concurrent_tasks.unwrap_or(5)
    }

    /// 获取最大排队任务数，默认 100
    pub fn get_max_queued_tasks(&self) -> usize {
        self.app.max_queued_tasks.unwrap_or(100)
    }

    /// 获取字幕服务提供者名称，默认 "edge"（Edge TTS）
    pub fn get_subtitle_provider(&self) -> &str {
        self.app.subtitle_provider.as_deref().unwrap_or("edge")
    }

    /// 获取 Edge TTS 超时时间，仅当值大于 0 时有效
    pub fn get_edge_tts_timeout(&self) -> Option<f64> {
        self.app.edge_tts_timeout.filter(|&t| t > 0.0)
    }

    /// 获取是否验证 TLS 证书，默认 true
    pub fn get_tls_verify(&self) -> bool {
        self.app.tls_verify.unwrap_or(true)
    }

    /// 获取视频编码器名称，默认 "libx264"
    pub fn get_video_codec(&self) -> &str {
        self.app.video_codec.as_deref().unwrap_or("libx264")
    }

    /// 获取服务外部访问端点 URL，默认空字符串（表示相对路径）
    pub fn get_endpoint(&self) -> &str {
        self.app.endpoint.as_deref().unwrap_or("")
    }

    /// 获取 FFmpeg 可执行文件路径
    ///
    /// 优先使用配置中的 `ffmpeg_path`（需文件存在），
    /// 其次通过 `which` 查找系统 PATH 中的 ffmpeg，
    /// 最后回退到 "ffmpeg" 字符串
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
