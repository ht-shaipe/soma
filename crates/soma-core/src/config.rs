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
    /// Whisper 语音识别模型配置（缺失时使用默认值）
    #[serde(default)]
    pub whisper: WhisperSection,
    /// 网络代理配置（缺失时使用默认值）
    #[serde(default)]
    pub proxy: ProxySection,
    /// Azure 语音服务配置（缺失时使用默认值）
    #[serde(default)]
    pub azure: AzureSection,
    /// SiliconFlow TTS 配置（缺失时使用默认值）
    #[serde(default)]
    pub siliconflow: SiliconflowSection,
    /// ElevenLabs TTS 配置（缺失时使用默认值）
    #[serde(default)]
    pub elevenlabs: ElevenlabsSection,
    /// 火山引擎 TTS 配置（缺失时使用默认值）
    #[serde(default)]
    pub volcengine: VolcengineSection,
    /// 科大讯飞 TTS 配置（缺失时使用默认值）
    #[serde(default)]
    pub xfyun: XfyunSection,
    /// Fish-Speech S2 TTS 配置（缺失时使用默认值）
    #[serde(default)]
    pub fishspeech: FishspeechSection,
    /// UI 界面与发布相关配置（缺失时使用默认值）
    #[serde(default)]
    pub ui: UiSection,
    /// 数字人口播视频生成配置
    #[serde(default)]
    pub digital_human: DigitalHumanSection,
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
    /// 通义千问 API 基础 URL
    pub qwen_base_url: Option<String>,
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

    /// 豆包（Doubao/火山引擎）API 密钥
    pub doubao_api_key: Option<String>,
    /// 豆包 API 基础 URL
    pub doubao_base_url: Option<String>,
    /// 豆包 模型名称
    pub doubao_model_name: Option<String>,

    /// 混元（Hunyuan/腾讯）API 密钥
    pub hunyuan_api_key: Option<String>,
    /// 混元 API 基础 URL
    pub hunyuan_base_url: Option<String>,
    /// 混元 模型名称
    pub hunyuan_model_name: Option<String>,

    /// 智谱（Zhipu/BigModel）API 密钥
    pub zhipu_api_key: Option<String>,
    /// 智谱 API 基础 URL
    pub zhipu_base_url: Option<String>,
    /// 智谱 模型名称
    pub zhipu_model_name: Option<String>,

    /// 文心（Wenxin/百度）API 密钥
    pub wenxin_api_key: Option<String>,
    /// 文心 API 基础 URL
    pub wenxin_base_url: Option<String>,
    /// 文心 模型名称
    pub wenxin_model_name: Option<String>,
    /// 文心 Secret Key（用于获取 access_token）
    pub wenxin_secret_key: Option<String>,

    /// 讯飞（Xunfei/Spark）API 密钥
    pub xunfei_api_key: Option<String>,
    /// 讯飞 API 基础 URL
    pub xunfei_base_url: Option<String>,
    /// 讯飞 模型名称
    pub xunfei_model_name: Option<String>,

    /// Pollinations API 密钥
    pub pollinations_api_key: Option<String>,
    /// Pollinations 基础 URL
    pub pollinations_base_url: Option<String>,
    /// Pollinations 模型名称
    pub pollinations_model_name: Option<String>,

    /// LiteLLM 代理模型名称（通过 LiteLLM 代理统一调用多种 LLM）
    pub litellm_model_name: Option<String>,

    /// Cloudflare Workers AI Account ID
    pub cloudflare_account_id: Option<String>,

    /// 是否启用 G4F（免费 GPT 接口）
    pub enable_g4f: Option<bool>,
    /// G4F 模型名称
    pub g4f_model_name: Option<String>,

    // ── 字幕 ──
    /// 字幕服务提供者，如 "edge"、"whisper"
    pub subtitle_provider: Option<String>,
    /// 是否在旁白文案中注入 Fish-Speech 风格的情感标签（如 [whisper] [excited]）
    ///
    /// 开启后 LLM 生成旁白时会插入情感标签；使用 fishspeech 引擎合成时标签生效，
    /// 其他引擎会在合成前自动剥离标签。
    pub narration_emotion_tags: Option<bool>,

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

    // ── AI 视频生成 ──
    /// 智谱 AI 视频生成 API 密钥
    pub zhipu_video_api_key: Option<String>,
    /// 智谱 AI 视频生成模型名称
    pub zhipu_video_model: Option<String>,
    /// 可灵 Access Key
    pub kling_access_key: Option<String>,
    /// 可灵 Secret Key
    pub kling_secret_key: Option<String>,
    /// 可灵视频生成模型名称
    pub kling_video_model: Option<String>,
    /// MiniMax 视频生成 API 密钥
    pub minimax_video_api_key: Option<String>,
    /// MiniMax 视频生成模型名称
    pub minimax_video_model: Option<String>,
    /// AI 视频生成超时时间（秒），默认 300
    pub video_gen_timeout: Option<u64>,
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

/// Fish-Speech S2 TTS 语音合成配置
///
/// 对应 TOML 配置文件的 `[fishspeech]` 段。
/// 同时兼容自部署的 S2 API Server（如 `http://gpu-host:8080`）
/// 与 Fish Audio 云端 API（`https://api.fish.audio`，需配合平台 API Key）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FishspeechSection {
    /// API 服务基础 URL，自部署如 "http://192.168.1.100:8080"，
    /// 云端为 "https://api.fish.audio"
    pub base_url: Option<String>,
    /// API 密钥（自部署服务未启用 --api-key 时可留空；云端必填）
    pub api_key: Option<String>,
    /// 默认参考音色 ID（服务端 references 目录名或 fish.audio 平台的音色 ID），留空使用模型默认音色
    pub reference_id: Option<String>,
    /// 音频格式：wav / pcm / mp3 / opus，默认 "mp3"
    pub format: Option<String>,
    /// 是否对中英文文本做数字归一化以提升稳定性，默认 true
    pub normalize: Option<bool>,
    /// 请求超时时间（秒），默认 120
    pub timeout: Option<u64>,
}

impl FishspeechSection {
    /// 获取 API 基础 URL，默认 Fish Audio 云端 "https://api.fish.audio"
    pub fn get_base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or("https://api.fish.audio")
    }

    /// 获取 API 密钥（可为空，自部署未启用鉴权时无需填写）
    pub fn get_api_key(&self) -> &str {
        self.api_key.as_deref().unwrap_or("")
    }

    /// 获取默认参考音色 ID（可为空）
    pub fn get_reference_id(&self) -> &str {
        self.reference_id.as_deref().unwrap_or("")
    }

    /// 获取音频格式，默认 "mp3"
    pub fn get_format(&self) -> &str {
        self.format.as_deref().unwrap_or("mp3")
    }

    /// 是否启用文本数字归一化，默认 true
    pub fn get_normalize(&self) -> bool {
        self.normalize.unwrap_or(true)
    }

    /// 获取请求超时时间（秒），默认 120
    pub fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(120)
    }
}

/// 数字人口播视频生成配置段
///
/// 配置第三方数字人服务（首期 HeyGen）的 API 密钥、超时、轮询等参数，
/// 以及敏感词库路径。对应 TOML 配置文件的 `[digital_human]` 段。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DigitalHumanSection {
    /// 数字人服务提供者名称，默认 "heygen"
    pub provider: Option<String>,
    /// 数字人服务 API 密钥
    pub api_key: Option<String>,
    /// 数字人服务基础 URL
    pub base_url: Option<String>,
    /// 数字人模型名称
    pub model: Option<String>,
    /// 单次任务超时时间（秒），默认 300
    pub timeout: Option<u64>,
    /// 轮询间隔（秒），默认 5
    pub poll_interval: Option<u64>,
    /// 最大重试次数，默认 3
    pub max_retries: Option<u32>,
    /// 敏感词库文件路径，默认 "resource/sensitive_words.txt"
    pub sensitive_words_path: Option<String>,
    /// SadTalker 本地数字人专属配置
    #[serde(default)]
    pub sadtalker: SadTalkerConfig,
    /// EchoMimicV3-Flash 本地数字人专属配置
    #[serde(default)]
    pub echomimic_v3: EchoMimicV3Config,
    /// 声音克隆 TTS 专属配置
    #[serde(default)]
    pub voice_clone: VoiceCloneConfig,
    /// 分段生成策略配置
    #[serde(default)]
    pub segment: SegmentConfig,
    /// HeyGem/Duix.Avatar 数字人配置
    #[serde(default)]
    pub heygem: HeyGemConfig,
    /// Live2D 卡通数字人配置
    #[serde(default)]
    pub live2d: Live2DConfig,
}

/// Live2D 卡通数字人提供商配置
///
/// 对应 TOML 配置文件的 `[digital_human.live2d]` 子段。
/// 纯 CPU 渲染，零 GPU 成本，通过 Python 子进程调用 live2d-py。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Live2DConfig {
    /// Live2D 模型存储目录，默认 "./storage/live2d_models"
    pub models_dir: Option<String>,
    /// Python 解释器路径，默认 "python3"
    pub python_path: Option<String>,
    /// 渲染脚本路径，默认 "resource/live2d_runner.py"
    pub script_path: Option<String>,
    /// 渲染帧率，默认 30，clamp [24, 60]
    pub fps: Option<u32>,
    /// 输出视频宽度，默认 1080，clamp [256, 3840]
    pub width: Option<u32>,
    /// 输出视频高度，默认 1920，clamp [256, 3840]
    pub height: Option<u32>,
    /// 默认模型 ID，留空时需在任务参数中指定
    pub default_model: Option<String>,
    /// 单次渲染超时时间（秒），默认 600
    pub timeout: Option<u64>,
    /// 最大重试次数，默认 3
    pub max_retries: Option<u32>,
    /// 渲染线程数，默认 1，clamp 到 CPU 核数
    pub render_threads: Option<u32>,
    /// 是否执行环境预检，默认 true
    pub preflight_check: Option<bool>,
}

impl Live2DConfig {
    /// 获取 Live2D 模型存放目录（缺省 `storage/live2d_models`）
    pub fn get_models_dir(&self) -> &str {
        self.models_dir.as_deref().unwrap_or("./storage/live2d_models")
    }
    /// 获取 Live2D 渲染所用 Python 解释器路径
    pub fn get_python_path(&self) -> &str {
        self.python_path.as_deref().unwrap_or("python3")
    }
    /// 获取 Live2D 渲染脚本路径
    pub fn get_script_path(&self) -> &str {
        self.script_path.as_deref().unwrap_or("resource/live2d_runner.py")
    }
    /// 获取 Live2D 渲染帧率（FPS）
    pub fn get_fps(&self) -> u32 {
        self.fps.unwrap_or(30).clamp(24, 60)
    }
    /// 获取 Live2D 渲染画面宽度（像素）
    pub fn get_width(&self) -> u32 {
        self.width.unwrap_or(1080).clamp(256, 3840)
    }
    /// 获取 Live2D 渲染画面高度（像素）
    pub fn get_height(&self) -> u32 {
        self.height.unwrap_or(1920).clamp(256, 3840)
    }
    /// 获取默认 Live2D 模型标识
    pub fn get_default_model(&self) -> &str {
        self.default_model.as_deref().unwrap_or("")
    }
    /// 获取 Live2D 渲染超时（秒）
    pub fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(600)
    }
    /// 获取 Live2D 渲染失败重试次数
    pub fn get_max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }
    /// 获取 Live2D 渲染线程数
    pub fn get_render_threads(&self) -> u32 {
        let requested = self.render_threads.unwrap_or(1);
        let max = std::thread::available_parallelism().map(|n| n.get() as u32).unwrap_or(1);
        requested.clamp(1, max)
    }
    /// 获取是否启用渲染前环境预检
    pub fn get_preflight_check(&self) -> bool {
        self.preflight_check.unwrap_or(true)
    }
}

/// HeyGem/Duix.Avatar HTTP 数字人提供商配置
///
/// 对应 TOML 配置文件的 `[digital_human.heygem]` 子段。
/// 通过 HTTP API 调用 Docker 部署的 HeyGem 服务。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeyGemConfig {
    /// TTS 服务基础 URL，默认 "http://127.0.0.1:18180"
    pub tts_base_url: Option<String>,
    /// 视频合成服务基础 URL，默认 "http://127.0.0.1:8383"
    pub video_base_url: Option<String>,
    /// 单次任务超时时间（秒），默认 600
    pub timeout: Option<u64>,
    /// 轮询间隔（秒），默认 5
    pub poll_interval: Option<u64>,
    /// 最大重试次数，默认 3
    pub max_retries: Option<u32>,
    /// 最大并发数，默认 2
    pub max_concurrent: Option<u32>,
    /// TTS top_p 参数，默认 0.7
    pub top_p: Option<f64>,
    /// TTS temperature 参数，默认 0.7
    pub temperature: Option<f64>,
    /// TTS repetition_penalty 参数，默认 1.5
    pub repetition_penalty: Option<f64>,
    /// 商户资产存储目录，默认 "./storage/heygem_assets"
    pub assets_dir: Option<String>,
    /// 是否自动覆盖已有资产，默认 false
    pub auto_overwrite: Option<bool>,
    /// 是否执行环境预检，默认 true
    pub preflight_check: Option<bool>,
}

impl HeyGemConfig {
    /// 获取 HeyGem TTS 服务地址
    pub fn get_tts_base_url(&self) -> &str {
        self.tts_base_url.as_deref().unwrap_or("http://127.0.0.1:18180")
    }
    /// 获取 HeyGem 视频合成服务地址
    pub fn get_video_base_url(&self) -> &str {
        self.video_base_url.as_deref().unwrap_or("http://127.0.0.1:8383")
    }
    /// 获取 Live2D 渲染超时（秒）
    pub fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(600)
    }
    /// 获取 HeyGem 任务轮询间隔（秒）
    pub fn get_poll_interval(&self) -> u64 {
        self.poll_interval.unwrap_or(5)
    }
    /// 获取 Live2D 渲染失败重试次数
    pub fn get_max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }
    /// 获取 HeyGem 最大并发任务数
    pub fn get_max_concurrent(&self) -> u32 {
        self.max_concurrent.unwrap_or(2)
    }
    /// 获取 HeyGem TTS 采样 top_p 参数
    pub fn get_top_p(&self) -> f64 {
        self.top_p.unwrap_or(0.7)
    }
    /// 获取 HeyGem TTS 采样温度
    pub fn get_temperature(&self) -> f64 {
        self.temperature.unwrap_or(0.7)
    }
    /// 获取 HeyGem TTS 重复惩罚系数
    pub fn get_repetition_penalty(&self) -> f64 {
        self.repetition_penalty.unwrap_or(1.5)
    }
    /// 获取 HeyGem 商户资产目录（静默视频/参考音频）
    pub fn get_assets_dir(&self) -> &str {
        self.assets_dir.as_deref().unwrap_or("./storage/heygem_assets")
    }
    /// 获取合成产物是否自动覆盖同名文件
    pub fn get_auto_overwrite(&self) -> bool {
        self.auto_overwrite.unwrap_or(false)
    }
    /// 获取是否启用渲染前环境预检
    pub fn get_preflight_check(&self) -> bool {
        self.preflight_check.unwrap_or(true)
    }
}

/// SadTalker 本地数字人提供商配置
///
/// 对应 TOML 配置文件的 `[digital_human.sadtalker]` 子段。
/// 所有字段为 `Option<T>`，缺失时使用 getter 默认值。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SadTalkerConfig {
    /// Python 虚拟环境目录路径
    pub env_path: Option<String>,
    /// Python 解释器路径，默认 "python3"
    pub python_path: Option<String>,
    /// 封装脚本路径，默认 "{env_path}/sadtalker_runner.py"
    pub script_path: Option<String>,
    /// 模型权重目录路径
    pub model_path: Option<String>,
    /// 推理设备，默认 "cpu"，可选 "cuda"
    pub device: Option<String>,
    /// 是否使用 still 模式（仅生成上半身）
    pub still_mode: Option<bool>,
    /// 是否使用全图增强
    pub full_enhancer: Option<bool>,
    /// 生成视频尺寸，默认 256
    pub batch_size: Option<u32>,
    /// 输出分辨率，默认 256
    pub size: Option<u32>,
    /// 头部姿态风格，默认 0
    pub pose_style: Option<u32>,
    /// 表情缩放系数，默认 1.0
    pub exp_scale: Option<f64>,
    /// 单次推理超时时间（秒），默认 900
    pub timeout: Option<u64>,
    /// 最大并发数，默认 1（串行）
    pub max_concurrent: Option<u32>,
    /// 是否在提交任务前执行环境预检，默认 true
    pub preflight_check: Option<bool>,
}

impl SadTalkerConfig {
    /// 获取环境目录路径，未配置返回空串
    pub fn get_env_path(&self) -> &str {
        self.env_path.as_deref().unwrap_or("")
    }

    /// 获取 Python 解释器路径，默认 "python3"
    pub fn get_python_path(&self) -> String {
        self.python_path.clone().unwrap_or_else(|| "python3".to_string())
    }

    /// 获取封装脚本路径，默认 "{env_path}/sadtalker_runner.py"
    pub fn get_script_path(&self) -> String {
        if let Some(ref s) = self.script_path {
            return s.clone();
        }
        let env = self.get_env_path();
        if env.is_empty() {
            "sadtalker_runner.py".to_string()
        } else {
            format!("{}/sadtalker_runner.py", env)
        }
    }

    /// 获取模型权重目录路径，未配置返回空串
    pub fn get_model_path(&self) -> &str {
        self.model_path.as_deref().unwrap_or("")
    }

    /// 获取推理设备，默认 "cpu"
    pub fn get_device(&self) -> &str {
        self.device.as_deref().unwrap_or("cpu")
    }

    /// 获取单次推理超时时间（秒），默认 900
    pub fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(900)
    }

    /// 获取最大并发数，默认 1
    pub fn get_max_concurrent(&self) -> u32 {
        self.max_concurrent.unwrap_or(1)
    }

    /// 是否执行环境预检，默认 true
    pub fn get_preflight_check(&self) -> bool {
        self.preflight_check.unwrap_or(true)
    }

    /// 获取 still 模式，默认 false
    pub fn get_still_mode(&self) -> bool {
        self.still_mode.unwrap_or(false)
    }

    /// 获取全图增强，默认 false
    pub fn get_full_enhancer(&self) -> bool {
        self.full_enhancer.unwrap_or(false)
    }

    /// 获取 batch size，默认 2
    pub fn get_batch_size(&self) -> u32 {
        self.batch_size.unwrap_or(2)
    }

    /// 获取输出尺寸，默认 256
    pub fn get_size(&self) -> u32 {
        self.size.unwrap_or(256)
    }

    /// 获取姿态风格，默认 0
    pub fn get_pose_style(&self) -> u32 {
        self.pose_style.unwrap_or(0)
    }

    /// 获取表情缩放，默认 1.0
    pub fn get_exp_scale(&self) -> f64 {
        self.exp_scale.unwrap_or(1.0)
    }
}

/// EchoMimicV3-Flash 本地数字人提供商配置
///
/// 对应 TOML 配置文件的 `[digital_human.echomimic_v3]` 子段。
/// 所有字段为 `Option<T>`，缺失时使用 getter 默认值。
/// 基于 Wan2.1-Fun-V1.1-1.3B 视频扩散模型，GPU 推理，8 步 Flash 快速生成。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EchoMimicV3Config {
    /// EchoMimicV3 运行环境目录路径
    pub env_path: Option<String>,
    /// Python 解释器路径，默认 "python3"
    pub python_path: Option<String>,
    /// 推理封装脚本路径，默认 "{env_path}/echomimic_v3_runner.py"
    pub script_path: Option<String>,
    /// 模型权重目录路径
    pub model_path: Option<String>,
    /// 推理设备，默认 "cuda"（强制 GPU）
    pub device: Option<String>,
    /// 输出分辨率，默认 768（可选 512）
    pub resolution: Option<u32>,
    /// Flash 推理步数，默认 8
    pub infer_steps: Option<u32>,
    /// 推理配置文件路径，默认 "{env_path}/config/prompts/flash.yaml"
    pub config_path: Option<String>,
    /// GPU 显存限制（GB），默认 24
    pub gpu_memory_limit: Option<u32>,
    /// 单次推理超时时间（秒），默认 600
    pub timeout: Option<u64>,
    /// 最大并发数，默认 1（串行）
    pub max_concurrent: Option<u32>,
    /// 最大重试次数，默认 3
    pub max_retries: Option<u32>,
    /// 是否在提交任务前执行环境预检，默认 true
    pub preflight_check: Option<bool>,
    /// 模型下载源，默认 "modelscope"（可选 "huggingface"）
    pub model_source: Option<String>,
}

impl EchoMimicV3Config {
    /// 获取环境目录路径，未配置返回空串
    pub fn get_env_path(&self) -> &str {
        self.env_path.as_deref().unwrap_or("")
    }

    /// 获取 Python 解释器路径，默认 "python3"
    pub fn get_python_path(&self) -> String {
        self.python_path.clone().unwrap_or_else(|| "python3".to_string())
    }

    /// 获取推理封装脚本路径，默认 "{env_path}/echomimic_v3_runner.py"
    pub fn get_script_path(&self) -> String {
        if let Some(ref s) = self.script_path {
            return s.clone();
        }
        let env = self.get_env_path();
        if env.is_empty() {
            "echomimic_v3_runner.py".to_string()
        } else {
            format!("{}/echomimic_v3_runner.py", env)
        }
    }

    /// 获取模型权重目录路径，未配置返回空串
    pub fn get_model_path(&self) -> &str {
        self.model_path.as_deref().unwrap_or("")
    }

    /// 获取推理设备，默认 "cuda"（强制 GPU）
    pub fn get_device(&self) -> &str {
        self.device.as_deref().unwrap_or("cuda")
    }

    /// 获取输出分辨率，默认 768
    pub fn get_resolution(&self) -> u32 {
        self.resolution.unwrap_or(768)
    }

    /// 获取 Flash 推理步数，默认 8
    pub fn get_infer_steps(&self) -> u32 {
        self.infer_steps.unwrap_or(8)
    }

    /// 获取推理配置文件路径，默认 "{env_path}/config/prompts/flash.yaml"
    pub fn get_config_path(&self) -> String {
        if let Some(ref s) = self.config_path {
            return s.clone();
        }
        let env = self.get_env_path();
        if env.is_empty() {
            "config/prompts/flash.yaml".to_string()
        } else {
            format!("{}/config/prompts/flash.yaml", env)
        }
    }

    /// 获取 GPU 显存限制（GB），默认 24
    pub fn get_gpu_memory_limit(&self) -> u32 {
        self.gpu_memory_limit.unwrap_or(24)
    }

    /// 获取单次推理超时时间（秒），默认 600
    pub fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(600)
    }

    /// 获取最大并发数，默认 1
    pub fn get_max_concurrent(&self) -> u32 {
        self.max_concurrent.unwrap_or(1)
    }

    /// 获取最大重试次数，默认 3
    pub fn get_max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }

    /// 是否执行环境预检，默认 true
    pub fn get_preflight_check(&self) -> bool {
        self.preflight_check.unwrap_or(true)
    }

    /// 获取模型下载源，默认 "modelscope"
    pub fn get_model_source(&self) -> &str {
        self.model_source.as_deref().unwrap_or("modelscope")
    }
}

/// 声音克隆 TTS 提供商配置
///
/// 对应 TOML 配置文件的 `[digital_human.voice_clone]` 子段。
/// 所有字段为 `Option<T>`，缺失时使用 getter 默认值。
/// 通过 SSH 远程调用 GPU 服务器上的声音克隆模型（GPT-SoVITS/CosyVoice/Fish-Speech）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceCloneConfig {
    /// SSH 主机地址，必填
    pub ssh_host: Option<String>,
    /// SSH 端口，默认 22
    pub ssh_port: Option<u16>,
    /// SSH 用户名，默认 "root"
    pub ssh_user: Option<String>,
    /// SSH 私钥本地路径，必填
    pub ssh_key_path: Option<String>,
    /// GPU 服务器上运行环境根目录，必填
    pub remote_env_path: Option<String>,
    /// 远程 Python 解释器路径，默认 "python3"
    pub remote_python_path: Option<String>,
    /// 远程推理封装脚本路径，默认 "{remote_env_path}/voice_clone_runner.py"
    pub remote_script_path: Option<String>,
    /// 远程模型权重目录，必填
    pub remote_model_path: Option<String>,
    /// 推理设备，默认 "cuda"（强制 GPU）
    pub device: Option<String>,
    /// 默认克隆模型，默认 "gpt_sovits"（可选 "cosyvoice"/"fish_speech"）
    pub default_clone_model: Option<String>,
    /// GPU 显存上限（GB），默认 24
    pub gpu_memory_limit: Option<u32>,
    /// 单次合成超时时间（秒），默认 300
    pub timeout: Option<u64>,
    /// 最大并发数，默认 1（串行）
    pub max_concurrent: Option<u32>,
    /// 最大重试次数，默认 3
    pub max_retries: Option<u32>,
    /// 是否在合成前执行环境预检，默认 true
    pub preflight_check: Option<bool>,
}

impl VoiceCloneConfig {
    /// 获取 SSH 主机地址，未配置返回空串
    pub fn get_ssh_host(&self) -> &str {
        self.ssh_host.as_deref().unwrap_or("")
    }

    /// 获取 SSH 端口，默认 22
    pub fn get_ssh_port(&self) -> u16 {
        self.ssh_port.unwrap_or(22)
    }

    /// 获取 SSH 用户名，默认 "root"
    pub fn get_ssh_user(&self) -> &str {
        self.ssh_user.as_deref().unwrap_or("root")
    }

    /// 获取 SSH 私钥路径，未配置返回空串
    pub fn get_ssh_key_path(&self) -> &str {
        self.ssh_key_path.as_deref().unwrap_or("")
    }

    /// 获取远程环境目录路径，未配置返回空串
    pub fn get_remote_env_path(&self) -> &str {
        self.remote_env_path.as_deref().unwrap_or("")
    }

    /// 获取远程 Python 解释器路径，默认 "python3"
    pub fn get_remote_python_path(&self) -> String {
        self.remote_python_path.clone().unwrap_or_else(|| "python3".to_string())
    }

    /// 获取远程推理封装脚本路径，默认 "{remote_env_path}/voice_clone_runner.py"
    pub fn get_remote_script_path(&self) -> String {
        if let Some(ref s) = self.remote_script_path {
            return s.clone();
        }
        let env = self.get_remote_env_path();
        if env.is_empty() {
            "voice_clone_runner.py".to_string()
        } else {
            format!("{}/voice_clone_runner.py", env)
        }
    }

    /// 获取远程模型权重目录路径，未配置返回空串
    pub fn get_remote_model_path(&self) -> &str {
        self.remote_model_path.as_deref().unwrap_or("")
    }

    /// 获取推理设备，默认 "cuda"（强制 GPU）
    pub fn get_device(&self) -> &str {
        self.device.as_deref().unwrap_or("cuda")
    }

    /// 获取默认克隆模型，默认 "gpt_sovits"
    pub fn get_default_clone_model(&self) -> &str {
        self.default_clone_model.as_deref().unwrap_or("gpt_sovits")
    }

    /// 获取 GPU 显存限制（GB），默认 24
    pub fn get_gpu_memory_limit(&self) -> u32 {
        self.gpu_memory_limit.unwrap_or(24)
    }

    /// 获取单次合成超时时间（秒），默认 300
    pub fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(300)
    }

    /// 获取最大并发数，默认 1
    pub fn get_max_concurrent(&self) -> u32 {
        self.max_concurrent.unwrap_or(1)
    }

    /// 获取最大重试次数，默认 3
    pub fn get_max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }

    /// 是否执行环境预检，默认 true
    pub fn get_preflight_check(&self) -> bool {
        self.preflight_check.unwrap_or(true)
    }
}

/// 分段生成策略配置
///
/// 对应 TOML 配置文件的 `[digital_human.segment]` 子段。
/// 控制 EchoMimicV3 长文案分段生成的参数。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SegmentConfig {
    /// 单段最大时长（秒），默认 5.0
    pub segment_max_duration: Option<f64>,
    /// 单段最大字符数，默认 25
    pub segment_max_chars: Option<usize>,
    /// 分段失败重试次数，默认 3
    pub segment_retry_count: Option<u32>,
}

impl SegmentConfig {
    /// 获取单段最大时长（秒），默认 5.0
    pub fn get_segment_max_duration(&self) -> f64 {
        self.segment_max_duration.unwrap_or(5.0)
    }

    /// 获取单段最大字符数，默认 25
    pub fn get_segment_max_chars(&self) -> usize {
        self.segment_max_chars.unwrap_or(25)
    }

    /// 获取分段失败重试次数，默认 3
    pub fn get_segment_retry_count(&self) -> u32 {
        self.segment_retry_count.unwrap_or(3)
    }
}

impl DigitalHumanSection {
    /// 获取提供者名称，默认 "heygen"
    pub fn get_provider(&self) -> &str {
        self.provider.as_deref().unwrap_or("heygen")
    }

    /// 获取单次任务超时时间（秒），默认 300
    pub fn get_timeout(&self) -> u64 {
        self.timeout.unwrap_or(300)
    }

    /// 获取轮询间隔（秒），默认 5
    pub fn get_poll_interval(&self) -> u64 {
        self.poll_interval.unwrap_or(5)
    }

    /// 获取最大重试次数，默认 3
    pub fn get_max_retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }

    /// 获取敏感词库文件路径，默认 "resource/sensitive_words.txt"
    pub fn get_sensitive_words_path(&self) -> &str {
        self.sensitive_words_path
            .as_deref()
            .unwrap_or("resource/sensitive_words.txt")
    }
}

/// 火山引擎（字节豆包）TTS 语音合成配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolcengineSection {
    /// 火山引擎语音合成 App ID
    pub app_id: Option<String>,
    /// 火山引擎访问令牌
    pub access_token: Option<String>,
    /// 火山引擎集群名称，默认 "volcano_tts"
    pub cluster: Option<String>,
}

/// 科大讯飞 TTS 语音合成配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct XfyunSection {
    /// 讯飞开放平台 App ID
    pub app_id: Option<String>,
    /// 讯飞 API Key
    pub api_key: Option<String>,
    /// 讯飞 API Secret
    pub api_secret: Option<String>,
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

    /// 获取服务监听端口号，默认 8090
    pub fn get_listen_port(&self) -> u16 {
        self.app.port.unwrap_or(8090)
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

    /// 获取是否在旁白文案中注入 Fish-Speech 风格情感标签，默认 false
    pub fn get_narration_emotion_tags(&self) -> bool {
        self.app.narration_emotion_tags.unwrap_or(false)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_echomimic_v3_config_default() {
        let config = EchoMimicV3Config::default();
        assert_eq!(config.get_env_path(), "");
        assert_eq!(config.get_python_path(), "python3");
        assert_eq!(config.get_model_path(), "");
        assert_eq!(config.get_device(), "cuda");
        assert_eq!(config.get_resolution(), 768);
        assert_eq!(config.get_infer_steps(), 8);
        assert_eq!(config.get_gpu_memory_limit(), 24);
        assert_eq!(config.get_timeout(), 600);
        assert_eq!(config.get_max_concurrent(), 1);
        assert_eq!(config.get_max_retries(), 3);
        assert!(config.get_preflight_check());
        assert_eq!(config.get_model_source(), "modelscope");
    }

    #[test]
    fn test_echomimic_v3_config_script_path_default() {
        let config = EchoMimicV3Config {
            env_path: Some("/opt/EchoMimicV3".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_script_path(), "/opt/EchoMimicV3/echomimic_v3_runner.py");
        assert_eq!(config.get_config_path(), "/opt/EchoMimicV3/config/prompts/flash.yaml");
    }

    #[test]
    fn test_echomimic_v3_config_script_path_no_env() {
        let config = EchoMimicV3Config::default();
        assert_eq!(config.get_script_path(), "echomimic_v3_runner.py");
        assert_eq!(config.get_config_path(), "config/prompts/flash.yaml");
    }

    #[test]
    fn test_echomimic_v3_config_custom_values() {
        let config = EchoMimicV3Config {
            env_path: Some("/env".to_string()),
            python_path: Some("/env/bin/python".to_string()),
            script_path: Some("/custom/runner.py".to_string()),
            model_path: Some("/models".to_string()),
            device: Some("cuda:1".to_string()),
            resolution: Some(512),
            infer_steps: Some(4),
            config_path: Some("/custom/flash.yaml".to_string()),
            gpu_memory_limit: Some(16),
            timeout: Some(1200),
            max_concurrent: Some(2),
            max_retries: Some(5),
            preflight_check: Some(false),
            model_source: Some("huggingface".to_string()),
        };
        assert_eq!(config.get_python_path(), "/env/bin/python");
        assert_eq!(config.get_script_path(), "/custom/runner.py");
        assert_eq!(config.get_model_path(), "/models");
        assert_eq!(config.get_device(), "cuda:1");
        assert_eq!(config.get_resolution(), 512);
        assert_eq!(config.get_infer_steps(), 4);
        assert_eq!(config.get_config_path(), "/custom/flash.yaml");
        assert_eq!(config.get_gpu_memory_limit(), 16);
        assert_eq!(config.get_timeout(), 1200);
        assert_eq!(config.get_max_concurrent(), 2);
        assert_eq!(config.get_max_retries(), 5);
        assert!(!config.get_preflight_check());
        assert_eq!(config.get_model_source(), "huggingface");
    }

    #[test]
    fn test_digital_human_section_echomimic_v3_default() {
        let section = DigitalHumanSection::default();
        assert_eq!(section.echomimic_v3.get_device(), "cuda");
        assert_eq!(section.echomimic_v3.get_resolution(), 768);
    }

    #[test]
    fn test_digital_human_section_with_echomimic_v3_toml() {
        let toml_str = r#"
provider = "echomimic_v3"

[echomimic_v3]
env_path = "/opt/EchoMimicV3"
model_path = "/models/echomimic"
device = "cuda:0"
resolution = 512
infer_steps = 4
timeout = 900
"#;
        let config: DigitalHumanSection = toml::from_str(toml_str).unwrap();
        let emv3 = &config.echomimic_v3;
        assert_eq!(config.get_provider(), "echomimic_v3");
        assert_eq!(emv3.get_env_path(), "/opt/EchoMimicV3");
        assert_eq!(emv3.get_model_path(), "/models/echomimic");
        assert_eq!(emv3.get_device(), "cuda:0");
        assert_eq!(emv3.get_resolution(), 512);
        assert_eq!(emv3.get_infer_steps(), 4);
        assert_eq!(emv3.get_timeout(), 900);
    }

    #[test]
    fn test_digital_human_section_without_echomimic_v3_toml() {
        let toml_str = r#"
provider = "heygen"
api_key = "test-key"
"#;
        let config: DigitalHumanSection = toml::from_str(toml_str).unwrap();
        assert_eq!(config.get_provider(), "heygen");
        assert_eq!(config.echomimic_v3.get_device(), "cuda");
        assert_eq!(config.echomimic_v3.get_resolution(), 768);
    }

    #[test]
    fn test_digital_human_sadtalker_and_echomimic_v3_independent() {
        let toml_str = r#"
provider = "echomimic_v3"

[sadtalker]
env_path = "/sadtalker/env"
model_path = "/sadtalker/models"
device = "cpu"
size = 256

[echomimic_v3]
env_path = "/echomimic/env"
model_path = "/echomimic/models"
device = "cuda:0"
resolution = 768
"#;
        let config: DigitalHumanSection = toml::from_str(toml_str).unwrap();
        let sadtalker = &config.sadtalker;
        let emv3 = &config.echomimic_v3;
        assert_eq!(sadtalker.get_env_path(), "/sadtalker/env");
        assert_eq!(sadtalker.get_model_path(), "/sadtalker/models");
        assert_eq!(sadtalker.get_device(), "cpu");
        assert_eq!(sadtalker.get_size(), 256);
        assert_eq!(emv3.get_env_path(), "/echomimic/env");
        assert_eq!(emv3.get_model_path(), "/echomimic/models");
        assert_eq!(emv3.get_device(), "cuda:0");
        assert_eq!(emv3.get_resolution(), 768);
    }

    #[test]
    fn test_voice_clone_config_default() {
        let config = VoiceCloneConfig::default();
        assert_eq!(config.get_ssh_host(), "");
        assert_eq!(config.get_ssh_port(), 22);
        assert_eq!(config.get_ssh_user(), "root");
        assert_eq!(config.get_ssh_key_path(), "");
        assert_eq!(config.get_remote_env_path(), "");
        assert_eq!(config.get_remote_python_path(), "python3");
        assert_eq!(config.get_remote_model_path(), "");
        assert_eq!(config.get_device(), "cuda");
        assert_eq!(config.get_default_clone_model(), "gpt_sovits");
        assert_eq!(config.get_gpu_memory_limit(), 24);
        assert_eq!(config.get_timeout(), 300);
        assert_eq!(config.get_max_concurrent(), 1);
        assert_eq!(config.get_max_retries(), 3);
        assert!(config.get_preflight_check());
    }

    #[test]
    fn test_voice_clone_config_custom_values() {
        let toml_str = r#"
ssh_host = "connect.example.com"
ssh_port = 26322
ssh_user = "root"
ssh_key_path = "~/.ssh/id_rsa"
remote_env_path = "/root/voice_clone"
remote_python_path = "/root/venv/bin/python"
remote_model_path = "/root/models"
device = "cuda:0"
default_clone_model = "cosyvoice"
gpu_memory_limit = 16
timeout = 120
max_concurrent = 2
max_retries = 5
preflight_check = false
"#;
        let config: VoiceCloneConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.get_ssh_host(), "connect.example.com");
        assert_eq!(config.get_ssh_port(), 26322);
        assert_eq!(config.get_ssh_key_path(), "~/.ssh/id_rsa");
        assert_eq!(config.get_remote_env_path(), "/root/voice_clone");
        assert_eq!(config.get_remote_python_path(), "/root/venv/bin/python");
        assert_eq!(config.get_remote_model_path(), "/root/models");
        assert_eq!(config.get_device(), "cuda:0");
        assert_eq!(config.get_default_clone_model(), "cosyvoice");
        assert_eq!(config.get_gpu_memory_limit(), 16);
        assert_eq!(config.get_timeout(), 120);
        assert_eq!(config.get_max_concurrent(), 2);
        assert_eq!(config.get_max_retries(), 5);
        assert!(!config.get_preflight_check());
    }

    #[test]
    fn test_voice_clone_config_remote_script_path_default() {
        let config = VoiceCloneConfig {
            remote_env_path: Some("/root/voice_clone".into()),
            ..Default::default()
        };
        assert_eq!(
            config.get_remote_script_path(),
            "/root/voice_clone/voice_clone_runner.py"
        );
    }

    #[test]
    fn test_voice_clone_config_remote_script_path_explicit() {
        let config = VoiceCloneConfig {
            remote_script_path: Some("/custom/runner.py".into()),
            ..Default::default()
        };
        assert_eq!(config.get_remote_script_path(), "/custom/runner.py");
    }

    #[test]
    fn test_digital_human_section_with_voice_clone_toml() {
        let toml_str = r#"
provider = "echomimic_v3"

[voice_clone]
ssh_host = "connect.example.com"
ssh_port = 26322
remote_env_path = "/root/voice_clone"
remote_model_path = "/root/models"
"#;
        let config: DigitalHumanSection = toml::from_str(toml_str).unwrap();
        let vc = &config.voice_clone;
        assert_eq!(vc.get_ssh_host(), "connect.example.com");
        assert_eq!(vc.get_ssh_port(), 26322);
        assert_eq!(vc.get_remote_env_path(), "/root/voice_clone");
        assert_eq!(vc.get_remote_model_path(), "/root/models");
        assert_eq!(vc.get_device(), "cuda");
        assert_eq!(vc.get_default_clone_model(), "gpt_sovits");
    }

    #[test]
    fn test_digital_human_section_without_voice_clone_toml() {
        let toml_str = r#"provider = "heygen""#;
        let config: DigitalHumanSection = toml::from_str(toml_str).unwrap();
        assert_eq!(config.voice_clone.get_ssh_host(), "");
        assert_eq!(config.voice_clone.get_device(), "cuda");
    }
}
