//! 数据模型模块
//!
//! 定义视频生成任务的核心数据结构，包括任务状态枚举、视频参数、
//! 字幕时间轴、素材信息等，以及相关常量。

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;

/// 任务状态枚举
///
/// 数值与数据库/接口中的状态码对应：-1=失败，0=草稿，1=完成，4=处理中
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// 任务失败
    Failed = -1,
    /// 任务草稿（配置阶段，尚未开始执行）
    Draft = 0,
    /// 任务完成
    Completed = 1,
    /// 排队等待中
    Queued = 2,
    /// 任务暂停
    Paused = 3,
    /// 任务处理中
    Processing = 4,
}

impl TaskStatus {
    /// 从 i32 数值转换为 TaskStatus，未匹配值默认返回 Processing
    pub fn from_i32(v: i32) -> Self {
        match v {
            -1 => TaskStatus::Failed,
            0 => TaskStatus::Draft,
            1 => TaskStatus::Completed,
            2 => TaskStatus::Queued,
            3 => TaskStatus::Paused,
            _ => TaskStatus::Processing,
        }
    }

    /// 转换为 i32 数值
    pub fn as_i32(&self) -> i32 {
        match self {
            TaskStatus::Failed => -1,
            TaskStatus::Draft => 0,
            TaskStatus::Completed => 1,
            TaskStatus::Queued => 2,
            TaskStatus::Paused => 3,
            TaskStatus::Processing => 4,
        }
    }
}

/// 视频素材拼接模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VideoConcatMode {
    /// 随机顺序拼接
    Random,
    /// 按顺序拼接
    Sequential,
}

/// 视频片段之间的转场模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoTransitionMode {
    /// 无转场
    None,
    /// 随机转场
    Shuffle,
    /// 淡入
    FadeIn,
    /// 淡出
    FadeOut,
    /// 滑入
    SlideIn,
    /// 滑出
    SlideOut,
}

/// 视频画面宽高比
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VideoAspect {
    /// 横屏 16:9（1920×1080）
    #[serde(rename = "16:9")]
    Landscape,
    /// 竖屏 9:16（1080×1920）
    #[serde(rename = "9:16")]
    Portrait,
    /// 方形 1:1（1080×1080）
    #[serde(rename = "1:1")]
    Square,
}

impl VideoAspect {
    /// 从字符串（如 "16:9"）解析为 VideoAspect，无法识别则返回 None
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "16:9" => Some(VideoAspect::Landscape),
            "9:16" => Some(VideoAspect::Portrait),
            "1:1" => Some(VideoAspect::Square),
            _ => None,
        }
    }

    /// 转换为字符串表示（如 "16:9"）
    pub fn as_str(&self) -> &'static str {
        match self {
            VideoAspect::Landscape => "16:9",
            VideoAspect::Portrait => "9:16",
            VideoAspect::Square => "1:1",
        }
    }

    /// 获取对应的像素分辨率 (宽, 高)
    pub fn to_resolution(&self) -> (u32, u32) {
        match self {
            VideoAspect::Landscape => (1920, 1080),
            VideoAspect::Portrait => (1080, 1920),
            VideoAspect::Square => (1080, 1080),
        }
    }

    /// 获取方向描述字符串，用于素材搜索 API 参数
    pub fn orientation(&self) -> &'static str {
        match self {
            VideoAspect::Landscape => "landscape",
            VideoAspect::Portrait => "portrait",
            VideoAspect::Square => "square",
        }
    }
}

/// 分镜脚本的单个场景
///
/// 对应设计文档③分镜脚本中的完整镜头制作指令。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StoryboardScene {
    /// 场景编号（从 1 开始）
    pub scene_id: u32,
    /// 该镜头时长（秒）
    #[serde(default)]
    pub duration: Option<u32>,
    /// 此镜头对应的旁白文本
    pub narration: String,
    /// 画面中文描述（用于预览和理解）
    #[serde(default)]
    pub visual_desc: Option<String>,
    /// 画面英文提示词（用于 AI 视频生成或素材搜索）
    /// 结构公式：主体描述 + 环境/场景 + 光线/色彩 + 风格/质量 + 镜头参数
    pub visual_prompt: String,
    /// 素材搜索英文关键词（1-3个单词，简短精准，适合Pexels/Pixabay搜索）
    #[serde(default)]
    pub search_keyword: Option<String>,
    /// 镜头运动方式：push_in / pull_out / pan_left / pan_right / tilt_up / tilt_down / static / zoom / tracking / aerial / close_up
    #[serde(default)]
    pub camera_movement: Option<String>,
    /// 与下一场景的转场方式：cut / fade / dissolve / slide / zoom
    #[serde(default)]
    pub transition: Option<String>,
    /// 字幕叠加文本（若与旁白不同时使用，如"春 · 起始"等装饰性字幕）
    #[serde(default)]
    pub text_overlay: Option<String>,
    /// 情绪基调：如"温暖、宁静"、"紧张、悬疑"等
    #[serde(default)]
    pub mood: Option<String>,
}

/// 素材信息，记录单个视频素材的来源和属性
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MaterialInfo {
    /// 素材提供者，如 "pexels"、"pixabay"、"coverr"、"local"，默认 "pexels"
    #[serde(default = "default_provider")]
    pub provider: String,
    /// 素材下载 URL 或本地文件路径
    #[serde(default)]
    pub url: String,
    /// 素材视频时长（秒）
    #[serde(default)]
    pub duration: f64,
}

/// MaterialInfo 中 provider 字段的默认值
fn default_provider() -> String {
    "pexels".to_string()
}

/// 视频生成任务参数，由用户提交时传入
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VideoParams {
    /// 视频主题/标题
    pub video_subject: String,
    /// 视频脚本文案（可由 LLM 生成或用户手动指定）
    #[serde(default)]
    pub video_script: String,
    /// 视频搜索关键词（JSON 格式，每段脚本对应的关键词）
    pub video_terms: Option<serde_json::Value>,
    /// 画面宽高比，如 "16:9"、"9:16"、"1:1"
    pub video_aspect: Option<String>,
    /// 素材拼接模式，"random" 或 "sequential"
    pub video_concat_mode: Option<String>,
    /// 转场模式，"none"/"shuffle"/"FadeIn" 等
    pub video_transition_mode: Option<String>,
    /// 单个素材片段时长（秒）
    pub video_clip_duration: Option<u32>,
    /// 是否按脚本段落匹配素材（而非随机匹配）
    pub match_materials_to_script: Option<bool>,
    /// 生成视频数量
    pub video_count: Option<u32>,
    /// 视频素材来源提供者
    pub video_source: Option<String>,
    /// 用户自定义素材列表
    pub video_materials: Option<Vec<MaterialInfo>>,
    /// 自定义音频文件路径（替代 TTS 生成的语音）
    pub custom_audio_file: Option<String>,
    /// 视频脚本语言代码，如 "zh-CN"、"en-US"
    pub video_language: Option<String>,
    /// TTS 语音名称，如 "zh-CN-XiaoxiaoNeural"
    pub voice_name: Option<String>,
    /// 语音音量（0.0~2.0）
    pub voice_volume: Option<f32>,
    /// 语音语速倍率
    pub voice_rate: Option<f32>,
    /// 背景音乐类型，如 "random"、"none"
    pub bgm_type: Option<String>,
    /// 自定义背景音乐文件路径
    pub bgm_file: Option<String>,
    /// 背景音乐音量（0.0~1.0）
    pub bgm_volume: Option<f32>,
    /// 是否启用字幕叠加
    pub subtitle_enabled: Option<bool>,
    /// 字幕位置，"top"/"bottom"/"custom"
    pub subtitle_position: Option<String>,
    /// 自定义字幕垂直位置比例（0.0~1.0）
    pub custom_position: Option<f64>,
    /// 字幕字体名称
    pub font_name: Option<String>,
    /// 字幕文字前景色
    pub text_fore_color: Option<String>,
    /// 字幕文字背景色（可为字符串或 JSON 渐变配置）
    pub text_background_color: Option<serde_json::Value>,
    /// 是否使用圆角字幕背景
    pub rounded_subtitle_background: Option<bool>,
    /// 字幕字号（像素）
    pub font_size: Option<u32>,
    /// 字幕描边颜色
    pub stroke_color: Option<String>,
    /// 字幕描边宽度
    pub stroke_width: Option<f32>,
    /// 脚本分段数（将脚本切分为 N 段分别匹配素材）
    pub paragraph_number: Option<u32>,
    /// FFmpeg 合成线程数
    pub n_threads: Option<u32>,
    /// 生成脚本的 LLM 提示词
    pub video_script_prompt: Option<String>,
    /// 自定义 LLM 系统提示词
    pub custom_system_prompt: Option<String>,
    /// 是否使用自定义系统提示词
    pub use_custom_system_prompt: Option<bool>,
    /// 视频编码器名称（如 libx264, h264_videotoolbox, h264_nvenc），为空时使用全局配置
    pub video_encoder: Option<String>,
    /// 水印图片路径（叠加在视频右下角，半透明）
    pub video_watermark: Option<String>,
    /// 片头视频文件路径（拼接到最终视频最前面）
    pub video_intro: Option<String>,
    /// 片尾视频文件路径（拼接到最终视频最后面）
    pub video_outro: Option<String>,
    /// 人像图片路径或 URL（用于口播视频生成，图生视频模式）
    pub portrait_image: Option<String>,
    /// 意图解析 - 风格
    pub intent_style: Option<String>,
    /// 意图解析 - 情绪基调
    pub intent_mood: Option<String>,
    /// 意图解析 - 目标受众
    pub intent_audience: Option<String>,
    /// TTS 提供者名称（透传自 DigitalHumanParams.tts_provider）
    #[serde(default)]
    pub tts_provider: Option<String>,
    /// 参考音频路径（透传自 DigitalHumanParams.clone_reference_audio）
    #[serde(default)]
    pub clone_reference_audio: Option<String>,
    /// 参考文本（透传自 DigitalHumanParams.clone_reference_text）
    #[serde(default)]
    pub clone_reference_text: Option<String>,
    /// 克隆模型选择（透传自 DigitalHumanParams.clone_model）
    #[serde(default)]
    pub clone_model: Option<String>,
    /// 商户标识（透传自 DigitalHumanParams.merchant_id，HeyGem 多商户模式）
    #[serde(default)]
    pub merchant_id: Option<String>,
    /// Live2D 模型标识（透传自 DigitalHumanParams.live2d_model_id）
    #[serde(default)]
    pub live2d_model_id: Option<String>,
}

impl VideoParams {
    /// 获取视频宽高比，无法识别时默认竖屏 9:16
    pub fn get_video_aspect(&self) -> VideoAspect {
        self.video_aspect
            .as_deref()
            .and_then(VideoAspect::from_str)
            .unwrap_or(VideoAspect::Portrait)
    }

    /// 获取 TTS 提供者名称，默认 "edge"
    pub fn get_tts_provider(&self) -> &str {
        self.tts_provider.as_deref().unwrap_or("edge")
    }

    /// 获取商户标识，未提供返回 None
    pub fn get_merchant_id(&self) -> Option<&str> {
        self.merchant_id.as_deref()
    }

    /// 获取素材拼接模式，默认随机
    pub fn get_concat_mode(&self) -> VideoConcatMode {
        match self.video_concat_mode.as_deref().unwrap_or("random") {
            "sequential" => VideoConcatMode::Sequential,
            _ => VideoConcatMode::Random,
        }
    }

    /// 获取单个素材片段时长（秒），默认 4 秒
    pub fn get_clip_duration(&self) -> u32 {
        self.video_clip_duration.unwrap_or(4)
    }

    /// 获取生成视频数量，默认 1
    pub fn get_video_count(&self) -> u32 {
        self.video_count.unwrap_or(1)
    }

    /// 获取语音名称，默认空字符串（由 TTS 提供者自行选择默认语音）
    pub fn get_voice_name(&self) -> &str {
        self.voice_name.as_deref().unwrap_or("")
    }

    /// 获取语音语速倍率，默认 1.0
    pub fn get_voice_rate(&self) -> f32 {
        self.voice_rate.unwrap_or(1.0)
    }

    /// 获取语音音量，默认 1.0
    pub fn get_voice_volume(&self) -> f32 {
        self.voice_volume.unwrap_or(1.0)
    }

    /// 获取背景音乐音量，默认 0.2
    pub fn get_bgm_volume(&self) -> f32 {
        self.bgm_volume.unwrap_or(0.2)
    }

    /// 获取是否启用字幕叠加，默认 true
    pub fn get_subtitle_enabled(&self) -> bool {
        self.subtitle_enabled.unwrap_or(true)
    }

    /// 获取 FFmpeg 合成线程数，默认 2
    pub fn get_n_threads(&self) -> u32 {
        self.n_threads.unwrap_or(2)
    }

    /// 获取视频编码器名称，为空时返回 None（由调用方决定回退策略）
    pub fn get_video_encoder(&self) -> Option<&str> {
        self.video_encoder.as_deref()
    }

    /// 是否使用自定义系统提示词
    pub fn get_use_custom_system_prompt(&self) -> bool {
        self.use_custom_system_prompt.unwrap_or(false)
    }

    /// 获取脚本分段数，限制在 1~10 之间，默认 1
    pub fn get_paragraph_number(&self) -> u32 {
        self.paragraph_number.unwrap_or(1).clamp(1, 10)
    }

    /// 判断是否为无语音模式（voice_name 为 "no-voice" 或 "none" 时不生成语音）
    pub fn is_no_voice(&self) -> bool {
        let name = self.get_voice_name().to_lowercase();
        name == "no-voice" || name == "none"
    }
}

/// 任务信息，记录单个视频生成任务的完整生命周期数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// 任务唯一 ID
    pub task_id: String,
    /// 任务生成参数
    pub params: VideoParams,
    /// 当前任务状态码（对应 TaskStatus 的 i32 值）
    pub state: i32,
    /// 任务进度百分比（0~100）
    pub progress: u32,
    /// LLM 生成的脚本文案
    pub script: Option<String>,
    /// 脚本各段对应的搜索关键词
    pub terms: Option<Vec<String>>,
    /// 分镜脚本场景列表
    pub storyboard: Option<Vec<StoryboardScene>>,
    /// 旁白文案（口语化的朗读文本，由脚本转换而来，用于 TTS 语音生成）
    pub narration: Option<String>,
    /// 生成的语音文件路径
    pub audio_file: Option<String>,
    /// 语音时长（秒）
    pub audio_duration: Option<f64>,
    /// 字幕文件路径
    pub subtitle_path: Option<String>,
    /// 下载的素材文件路径列表
    pub materials: Option<Vec<String>>,
    /// 每段脚本合成的视频文件路径列表
    pub videos: Option<Vec<String>>,
    /// 最终合并后的视频文件路径列表
    pub combined_videos: Option<Vec<String>>,
    /// AI 视频生成逐段日志（每段脚本/分镜的提示词和生成状态）
    pub ai_video_logs: Option<Vec<AiVideoSegmentLog>>,
    /// 错误信息（任务失败时填充）
    pub error_message: Option<String>,
    /// 任务创建时间（UTC）
    pub created_at: DateTime<Utc>,
    /// 任务最后更新时间（UTC）
    pub updated_at: DateTime<Utc>,
}

impl TaskInfo {
    /// 创建新任务，初始状态为 Processing、进度 0
    pub fn new(task_id: String, params: VideoParams) -> Self {
        Self::with_status(task_id, params, TaskStatus::Processing)
    }

    /// 创建新任务，指定初始状态
    pub fn with_status(task_id: String, params: VideoParams, status: TaskStatus) -> Self {
        let now = Utc::now();
        Self {
            task_id,
            params,
            state: status.as_i32(),
            progress: 0,
            script: None,
            terms: None,
            storyboard: None,
            narration: None,
            audio_file: None,
            audio_duration: None,
            subtitle_path: None,
            materials: None,
            videos: None,
            combined_videos: None,
            ai_video_logs: None,
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新任务状态和/或进度，同时刷新 updated_at 时间戳
    pub fn update(&mut self, state: Option<i32>, progress: Option<u32>) {
        if let Some(s) = state {
            self.state = s;
        }
        if let Some(p) = progress {
            self.progress = p;
        }
        self.updated_at = Utc::now();
    }

    /// 判断任务是否已失败
    pub fn is_failed(&self) -> bool {
        self.state == TaskStatus::Failed.as_i32()
    }

    /// 判断任务是否已完成
    pub fn is_completed(&self) -> bool {
        self.state == TaskStatus::Completed.as_i32()
    }
}

/// AI 视频生成单段日志，记录每个分镜场景的提示词和生成状态
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AiVideoSegmentLog {
    /// 场景编号（对应 StoryboardScene.scene_id）
    pub scene_id: u32,
    /// 提交给 AI 视频生成 API 的英文提示词（即 visual_prompt）
    pub prompt: String,
    /// 生成状态："pending" / "submitted" / "processing" / "success" / "failed" / "timeout"
    pub status: String,
    /// 状态补充信息（如失败原因、AI 任务 ID 等）
    #[serde(default)]
    pub message: Option<String>,
}

/// 字幕时间轴条目，对应 SRT 格式的一条字幕
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleCue {
    /// 字幕序号（从 1 开始）
    pub index: u32,
    /// 字幕起始时间（毫秒）
    pub start_ms: u64,
    /// 字幕结束时间（毫秒）
    pub end_ms: u64,
    /// 字幕文本内容
    pub text: String,
}

/// 标点符号列表，用于脚本分段拆分
///
/// 包含中英文常见标点及阿拉伯语标点
pub const PUNCTUATIONS: &[&str] = &[
    "?", ",", ".", "、", ";", ":", "!", "…", "？", "，", "。", "；", "：", "！", "...",
    "،", "؛", "؟",
];

/// 支持的视频文件扩展名
pub const FILE_TYPE_VIDEOS: &[&str] = &["mp4", "mov", "mkv", "webm"];
/// 支持的图片文件扩展名
pub const FILE_TYPE_IMAGES: &[&str] = &["jpg", "jpeg", "png", "bmp"];

/// 数字人口播视频生成任务参数，由用户提交时传入
///
/// 与 `VideoParams` 区别：输入为单张人像照片 + 一段文案，
/// 输出为口播视频（人物开口说话，口型与配音同步），
/// 不经过 LLM 文案改写，文案原样朗读。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DigitalHumanParams {
    /// 人像照片文件名（位于 `storage/portraits/` 目录下），必填
    pub portrait_image: String,
    /// 口播文案文本，必填，原样朗读不经过 LLM 改写，长度 ≤ 1000 字
    pub narration_text: String,
    /// TTS 语音名称，如 "zh-CN-XiaoxiaoNeural"
    #[serde(default)]
    pub voice_name: Option<String>,
    /// 语音语速倍率
    #[serde(default)]
    pub voice_rate: Option<f32>,
    /// 语音音量（0.0~2.0）
    #[serde(default)]
    pub voice_volume: Option<f32>,
    /// 文案语言代码，如 "zh-CN"、"en-US"
    #[serde(default)]
    pub video_language: Option<String>,
    /// 画面宽高比，如 "16:9"、"9:16"、"1:1"
    #[serde(default)]
    pub video_aspect: Option<String>,
    /// 是否启用字幕叠加
    #[serde(default)]
    pub subtitle_enabled: Option<bool>,
    /// 字幕位置，"top"/"bottom"/"custom"
    #[serde(default)]
    pub subtitle_position: Option<String>,
    /// 自定义字幕垂直位置比例（0.0~1.0）
    #[serde(default)]
    pub custom_position: Option<f64>,
    /// 背景音乐类型，如 "random"、"none"
    #[serde(default)]
    pub bgm_type: Option<String>,
    /// 自定义背景音乐文件路径
    #[serde(default)]
    pub bgm_file: Option<String>,
    /// 背景音乐音量（0.0~1.0）
    #[serde(default)]
    pub bgm_volume: Option<f32>,
    /// 字幕字体名称
    #[serde(default)]
    pub font_name: Option<String>,
    /// 字幕字号（像素）
    #[serde(default)]
    pub font_size: Option<u32>,
    /// 字幕文字前景色
    #[serde(default)]
    pub text_fore_color: Option<String>,
    /// 字幕描边颜色
    #[serde(default)]
    pub stroke_color: Option<String>,
    /// 字幕描边宽度
    #[serde(default)]
    pub stroke_width: Option<f32>,
    /// TTS 提供者名称，如 "edge"、"siliconflow"、"elevenlabs"、"mimo"、"azure"、"gemini"
    #[serde(default)]
    pub tts_provider: Option<String>,
    /// 视频编码器名称（如 libx264, h264_videotoolbox, h264_nvenc）
    #[serde(default)]
    pub video_encoder: Option<String>,
    /// FFmpeg 合成线程数
    #[serde(default)]
    pub n_threads: Option<u32>,
    /// 参考音频文件路径（tts_provider = "voice_clone" 时必填，指向 storage/voice_clone_refs/ 目录下文件）
    #[serde(default)]
    pub clone_reference_audio: Option<String>,
    /// 参考音频对应文本（可选，长度 ≤ 100 字）
    #[serde(default)]
    pub clone_reference_text: Option<String>,
    /// 克隆模型选择（可选，取值 gpt_sovits/cosyvoice/fish_speech，未指定时使用配置默认）
    #[serde(default)]
    pub clone_model: Option<String>,
    /// 商户标识（HeyGem 多商户模式，用于隔离数字人模型资产）
    #[serde(default)]
    pub merchant_id: Option<String>,
    /// Live2D 模型标识（指定使用哪个 Live2D 卡通模型）
    #[serde(default)]
    pub live2d_model_id: Option<String>,
}

impl DigitalHumanParams {
    /// 获取视频宽高比，无法识别时默认竖屏 9:16
    pub fn get_video_aspect(&self) -> VideoAspect {
        self.video_aspect
            .as_deref()
            .and_then(VideoAspect::from_str)
            .unwrap_or(VideoAspect::Portrait)
    }

    /// 获取 TTS 提供者名称，默认 "edge"
    pub fn get_tts_provider(&self) -> &str {
        self.tts_provider.as_deref().unwrap_or("edge")
    }

    /// 获取参考音频文件路径，未提供返回空串
    pub fn get_clone_reference_audio(&self) -> &str {
        self.clone_reference_audio.as_deref().unwrap_or("")
    }

    /// 获取参考音频对应文本，未提供返回空串
    pub fn get_clone_reference_text(&self) -> &str {
        self.clone_reference_text.as_deref().unwrap_or("")
    }

    /// 获取克隆模型选择，未提供返回空串（使用配置默认值）
    pub fn get_clone_model(&self) -> &str {
        self.clone_model.as_deref().unwrap_or("")
    }

    /// 获取商户标识，未提供返回 None
    pub fn get_merchant_id(&self) -> Option<&str> {
        self.merchant_id.as_deref()
    }

    /// 获取 Live2D 模型标识，未提供返回 None
    pub fn get_live2d_model_id(&self) -> Option<&str> {
        self.live2d_model_id.as_deref()
    }
}

/// 数字人口播视频任务信息，记录单个口播任务的完整生命周期数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalHumanTaskInfo {
    /// 任务唯一 ID
    pub task_id: String,
    /// 任务生成参数
    pub params: DigitalHumanParams,
    /// 当前任务状态码（对应 TaskStatus 的 i32 值）
    pub state: i32,
    /// 任务进度百分比（0~100），按阶段上报：0→30（音频）→90（口播视频）→100（合成）
    pub progress: u32,
    /// 生成的语音文件路径
    pub audio_file: Option<String>,
    /// 语音时长（秒）
    pub audio_duration: Option<f64>,
    /// 字幕文件路径
    pub subtitle_path: Option<String>,
    /// 口播视频文件路径（第三方数字人服务生成）
    pub portrait_video_path: Option<String>,
    /// 最终合成视频文件路径
    pub final_video_path: Option<String>,
    /// 错误信息（任务失败时填充）
    pub error_message: Option<String>,
    /// 分段总数
    #[serde(default)]
    pub segment_count: Option<u32>,
    /// 当前分段序号
    #[serde(default)]
    pub current_segment: Option<u32>,
    /// 各分段音频文件路径
    #[serde(default)]
    pub segment_audio_files: Option<Vec<String>>,
    /// 各分段口播视频文件路径
    #[serde(default)]
    pub segment_video_files: Option<Vec<String>>,
    /// 商户标识（HeyGem 多商户模式）
    #[serde(default)]
    pub merchant_id: Option<String>,
    /// HeyGem 视频合成任务编码
    #[serde(default)]
    pub heygem_task_code: Option<String>,
    /// Live2D 模型标识
    #[serde(default)]
    pub live2d_model_id: Option<String>,
    /// Live2D 渲染临时帧目录
    #[serde(default)]
    pub frames_dir: Option<String>,
    /// 任务创建时间（UTC）
    pub created_at: DateTime<Utc>,
    /// 任务最后更新时间（UTC）
    pub updated_at: DateTime<Utc>,
}

/// 商户数字人模型资产状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssetStatus {
    Untrained,
    Ready,
    Training,
}

/// 商户数字人模型资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantAsset {
    pub merchant_id: String,
    pub silent_video_path: String,
    pub reference_audio: String,
    pub reference_text: String,
    pub trained_at: DateTime<Utc>,
    pub asset_status: AssetStatus,
}

/// Live2D 模型状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Live2DModelStatus {
    Available,
    Deleting,
}

/// Live2D 模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Live2DModel {
    pub model_id: String,
    pub model3_json: String,
    pub textures: Vec<String>,
    pub motions: Vec<String>,
    pub expressions: Vec<String>,
    pub package_size_mb: f64,
    pub uploaded_at: DateTime<Utc>,
    pub model_status: Live2DModelStatus,
}

/// Live2D 口型参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Live2DMouthParams {
    pub mouth_open_y: f32,
    pub mouth_form: f32,
    pub mouth_open_x: f32,
}

/// viseme（音素）到 Live2D 口型参数的映射表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisemeMapping {
    pub mappings: std::collections::HashMap<String, Live2DMouthParams>,
}

impl Default for VisemeMapping {
    fn default() -> Self {
        let mut m = std::collections::HashMap::new();
        // 中文拼音音素 → Live2D 口型参数
        m.insert("a".into(), Live2DMouthParams { mouth_open_y: 1.0, mouth_form: 0.0, mouth_open_x: 0.5 });
        m.insert("o".into(), Live2DMouthParams { mouth_open_y: 0.8, mouth_form: -0.8, mouth_open_x: 0.3 });
        m.insert("e".into(), Live2DMouthParams { mouth_open_y: 0.6, mouth_form: -0.3, mouth_open_x: 0.4 });
        m.insert("i".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: 0.8, mouth_open_x: 0.2 });
        m.insert("u".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.9, mouth_open_x: 0.1 });
        m.insert("v".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.8, mouth_open_x: 0.1 });
        m.insert("b".into(), Live2DMouthParams { mouth_open_y: 0.1, mouth_form: 0.0, mouth_open_x: 0.3 });
        m.insert("p".into(), Live2DMouthParams { mouth_open_y: 0.1, mouth_form: 0.0, mouth_open_x: 0.3 });
        m.insert("m".into(), Live2DMouthParams { mouth_open_y: 0.1, mouth_form: 0.0, mouth_open_x: 0.3 });
        m.insert("f".into(), Live2DMouthParams { mouth_open_y: 0.2, mouth_form: -0.5, mouth_open_x: 0.2 });
        m.insert("d".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: 0.3, mouth_open_x: 0.3 });
        m.insert("t".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: 0.3, mouth_open_x: 0.3 });
        m.insert("n".into(), Live2DMouthParams { mouth_open_y: 0.2, mouth_form: 0.2, mouth_open_x: 0.3 });
        m.insert("l".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: 0.5, mouth_open_x: 0.3 });
        m.insert("g".into(), Live2DMouthParams { mouth_open_y: 0.2, mouth_form: -0.2, mouth_open_x: 0.3 });
        m.insert("k".into(), Live2DMouthParams { mouth_open_y: 0.2, mouth_form: -0.2, mouth_open_x: 0.3 });
        m.insert("h".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.3, mouth_open_x: 0.4 });
        m.insert("j".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: 0.6, mouth_open_x: 0.2 });
        m.insert("q".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: 0.6, mouth_open_x: 0.2 });
        m.insert("x".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: 0.5, mouth_open_x: 0.2 });
        m.insert("zh".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.4, mouth_open_x: 0.3 });
        m.insert("ch".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.4, mouth_open_x: 0.3 });
        m.insert("sh".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.4, mouth_open_x: 0.3 });
        m.insert("r".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.5, mouth_open_x: 0.3 });
        m.insert("z".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.3, mouth_open_x: 0.3 });
        m.insert("c".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.3, mouth_open_x: 0.3 });
        m.insert("s".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.3, mouth_open_x: 0.4 });
        m.insert("w".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: -0.7, mouth_open_x: 0.1 });
        m.insert("y".into(), Live2DMouthParams { mouth_open_y: 0.3, mouth_form: 0.7, mouth_open_x: 0.2 });
        Self { mappings: m }
    }
}

impl VisemeMapping {
    /// 获取音素对应的口型参数，未知音素回退到中性口型
    pub fn get_params(&self, phoneme: &str) -> Live2DMouthParams {
        self.mappings.get(phoneme).cloned().unwrap_or(Live2DMouthParams {
            mouth_open_y: 0.0,
            mouth_form: 0.0,
            mouth_open_x: 0.0,
        })
    }
}

impl DigitalHumanTaskInfo {
    /// 创建新数字人任务，初始状态为 Processing、进度 0
    pub fn new(task_id: String, params: DigitalHumanParams) -> Self {
        Self::with_status(task_id, params, TaskStatus::Processing)
    }

    /// 创建新数字人任务，指定初始状态
    pub fn with_status(
        task_id: String,
        params: DigitalHumanParams,
        status: TaskStatus,
    ) -> Self {
        let now = Utc::now();
        Self {
            task_id,
            params,
            state: status.as_i32(),
            progress: 0,
            audio_file: None,
            audio_duration: None,
            subtitle_path: None,
            portrait_video_path: None,
            final_video_path: None,
            error_message: None,
            segment_count: None,
            current_segment: None,
            segment_audio_files: None,
            segment_video_files: None,
            merchant_id: None,
            heygem_task_code: None,
            live2d_model_id: None,
            frames_dir: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新任务状态和/或进度，同时刷新 updated_at 时间戳
    pub fn update(&mut self, state: Option<i32>, progress: Option<u32>) {
        if let Some(s) = state {
            self.state = s;
        }
        if let Some(p) = progress {
            self.progress = p;
        }
        self.updated_at = Utc::now();
    }

    /// 判断任务是否已失败
    pub fn is_failed(&self) -> bool {
        self.state == TaskStatus::Failed.as_i32()
    }

    /// 判断任务是否已完成
    pub fn is_completed(&self) -> bool {
        self.state == TaskStatus::Completed.as_i32()
    }
}

/// 人像照片信息，对应 `storage/portraits/` 目录下的一个照片文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortraitInfo {
    /// 照片文件名（UUID 命名，含扩展名）
    pub name: String,
    /// 照片相对路径（相对于存储根目录，如 `portraits/xxx.jpg`）
    pub path: String,
    /// 照片文件大小（字节）
    pub size: u64,
    /// 照片文件类型（扩展名，如 "jpg"、"png"）
    pub file_type: String,
    /// 上传时间（UTC）
    pub uploaded_at: DateTime<Utc>,
}

/// 数字人口播文案最大长度（字）
pub const DH_NARRATION_TEXT_MAX_LEN: usize = 1000;
/// 人像照片最大文件大小（10MB）
pub const DH_PORTRAIT_MAX_SIZE: u64 = 10 * 1024 * 1024;
/// 支持的人像照片扩展名
pub const DH_PORTRAIT_FILE_TYPES: &[&str] = &["jpg", "jpeg", "png"];

/// 图片故事场景，包含一张图片和对应的文字描述
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImageStoryScene {
    /// 场景编号（从 1 开始）
    pub scene_id: u32,
    /// 图片文件路径或URL
    pub image_path: String,
    /// 图片文字描述（用于AI生成视频）
    pub description: String,
    /// 场景时长（秒），可选，默认使用全局设置
    #[serde(default)]
    pub duration: Option<u32>,
    /// 镜头运动方式：push_in / pull_out / pan_left / pan_right / static 等
    #[serde(default)]
    pub camera_movement: Option<String>,
    /// 转场方式：cut / fade / dissolve 等
    #[serde(default)]
    pub transition: Option<String>,
}

/// 图片故事视频生成任务参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImageStoryParams {
    /// 故事主题/标题
    pub story_subject: String,
    /// 故事场景列表（包含图片和文字描述）
    pub scenes: Vec<ImageStoryScene>,
    /// 画面宽高比，如 "16:9"、"9:16"、"1:1"
    #[serde(default)]
    pub video_aspect: Option<String>,
    /// 单个场景时长（秒），默认 4 秒
    #[serde(default)]
    pub scene_duration: Option<u32>,
    /// AI视频生成提供商，如 "cogvideox"、"kling"、"minimax"
    #[serde(default)]
    pub ai_provider: Option<String>,
    /// 是否启用字幕叠加
    #[serde(default)]
    pub subtitle_enabled: Option<bool>,
    /// 字幕位置，"top"/"bottom"/"custom"
    #[serde(default)]
    pub subtitle_position: Option<String>,
    /// 背景音乐类型，如 "random"、"none"
    #[serde(default)]
    pub bgm_type: Option<String>,
    /// 自定义背景音乐文件路径
    #[serde(default)]
    pub bgm_file: Option<String>,
    /// 背景音乐音量（0.0~1.0）
    #[serde(default)]
    pub bgm_volume: Option<f32>,
    /// TTS语音名称（可选，为场景旁白配音）
    #[serde(default)]
    pub voice_name: Option<String>,
    /// 视频编码器名称
    #[serde(default)]
    pub video_encoder: Option<String>,
}

impl ImageStoryParams {
    /// 获取视频宽高比，无法识别时默认竖屏 9:16
    pub fn get_video_aspect(&self) -> VideoAspect {
        self.video_aspect
            .as_deref()
            .and_then(VideoAspect::from_str)
            .unwrap_or(VideoAspect::Portrait)
    }

    /// 获取单个场景时长（秒），默认 4 秒
    pub fn get_scene_duration(&self) -> u32 {
        self.scene_duration.unwrap_or(4)
    }

    /// 获取AI视频生成提供商，默认 "cogvideox"
    pub fn get_ai_provider(&self) -> &str {
        self.ai_provider.as_deref().unwrap_or("cogvideox")
    }

    /// 获取背景音乐音量，默认 0.2
    pub fn get_bgm_volume(&self) -> f32 {
        self.bgm_volume.unwrap_or(0.2)
    }

    /// 获取是否启用字幕叠加，默认 true
    pub fn get_subtitle_enabled(&self) -> bool {
        self.subtitle_enabled.unwrap_or(true)
    }
}

/// 图片故事任务信息，记录单个图片故事任务的完整生命周期数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStoryTaskInfo {
    /// 任务唯一 ID
    pub task_id: String,
    /// 任务生成参数
    pub params: ImageStoryParams,
    /// 当前任务状态码（对应 TaskStatus 的 i32 值）
    pub state: i32,
    /// 任务进度百分比（0~100）
    pub progress: u32,
    /// 生成的视频文件路径
    pub video_path: Option<String>,
    /// 错误信息（任务失败时填充）
    pub error_message: Option<String>,
    /// 任务创建时间（UTC）
    pub created_at: DateTime<Utc>,
    /// 任务最后更新时间（UTC）
    pub updated_at: DateTime<Utc>,
}

impl ImageStoryTaskInfo {
    /// 创建新任务，初始状态为 Processing、进度 0
    pub fn new(task_id: String, params: ImageStoryParams) -> Self {
        Self::with_status(task_id, params, TaskStatus::Processing)
    }

    /// 创建新任务，指定初始状态
    pub fn with_status(task_id: String, params: ImageStoryParams, status: TaskStatus) -> Self {
        let now = Utc::now();
        Self {
            task_id,
            params,
            state: status.as_i32(),
            progress: 0,
            video_path: None,
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新任务状态和/或进度，同时刷新 updated_at 时间戳
    pub fn update(&mut self, state: Option<i32>, progress: Option<u32>) {
        if let Some(s) = state {
            self.state = s;
        }
        if let Some(p) = progress {
            self.progress = p;
        }
        self.updated_at = Utc::now();
    }

    /// 判断任务是否已失败
    pub fn is_failed(&self) -> bool {
        self.state == TaskStatus::Failed.as_i32()
    }

    /// 判断任务是否已完成
    pub fn is_completed(&self) -> bool {
        self.state == TaskStatus::Completed.as_i32()
    }
}
