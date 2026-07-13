//! 数据模型模块
//!
//! 定义视频生成任务的核心数据结构，包括任务状态枚举、视频参数、
//! 字幕时间轴、素材信息等，以及相关常量。

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl VideoParams {
    /// 获取视频宽高比，无法识别时默认竖屏 9:16
    pub fn get_video_aspect(&self) -> VideoAspect {
        self.video_aspect
            .as_deref()
            .and_then(VideoAspect::from_str)
            .unwrap_or(VideoAspect::Portrait)
    }

    /// 获取素材拼接模式，默认随机
    pub fn get_concat_mode(&self) -> VideoConcatMode {
        match self.video_concat_mode.as_deref().unwrap_or("random") {
            "sequential" => VideoConcatMode::Sequential,
            _ => VideoConcatMode::Random,
        }
    }

    /// 获取单个素材片段时长（秒），默认 5 秒
    pub fn get_clip_duration(&self) -> u32 {
        self.video_clip_duration.unwrap_or(5)
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
        self.paragraph_number.unwrap_or(1).min(10).max(1)
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
