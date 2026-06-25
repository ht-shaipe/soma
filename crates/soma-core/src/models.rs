use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Failed = -1,
    Completed = 1,
    Processing = 4,
}

impl TaskStatus {
    pub fn from_i32(v: i32) -> Self {
        match v {
            -1 => TaskStatus::Failed,
            1 => TaskStatus::Completed,
            _ => TaskStatus::Processing,
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            TaskStatus::Failed => -1,
            TaskStatus::Completed => 1,
            TaskStatus::Processing => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VideoConcatMode {
    Random,
    Sequential,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoTransitionMode {
    None,
    Shuffle,
    FadeIn,
    FadeOut,
    SlideIn,
    SlideOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VideoAspect {
    #[serde(rename = "16:9")]
    Landscape,
    #[serde(rename = "9:16")]
    Portrait,
    #[serde(rename = "1:1")]
    Square,
}

impl VideoAspect {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "16:9" => Some(VideoAspect::Landscape),
            "9:16" => Some(VideoAspect::Portrait),
            "1:1" => Some(VideoAspect::Square),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            VideoAspect::Landscape => "16:9",
            VideoAspect::Portrait => "9:16",
            VideoAspect::Square => "1:1",
        }
    }

    pub fn to_resolution(&self) -> (u32, u32) {
        match self {
            VideoAspect::Landscape => (1920, 1080),
            VideoAspect::Portrait => (1080, 1920),
            VideoAspect::Square => (1080, 1080),
        }
    }

    pub fn orientation(&self) -> &'static str {
        match self {
            VideoAspect::Landscape => "landscape",
            VideoAspect::Portrait => "portrait",
            VideoAspect::Square => "square",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialInfo {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub duration: f64,
}

fn default_provider() -> String {
    "pexels".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoParams {
    pub video_subject: String,
    #[serde(default)]
    pub video_script: String,
    pub video_terms: Option<serde_json::Value>,
    pub video_aspect: Option<String>,
    pub video_concat_mode: Option<String>,
    pub video_transition_mode: Option<String>,
    pub video_clip_duration: Option<u32>,
    pub match_materials_to_script: Option<bool>,
    pub video_count: Option<u32>,
    pub video_source: Option<String>,
    pub video_materials: Option<Vec<MaterialInfo>>,
    pub custom_audio_file: Option<String>,
    pub video_language: Option<String>,
    pub voice_name: Option<String>,
    pub voice_volume: Option<f32>,
    pub voice_rate: Option<f32>,
    pub bgm_type: Option<String>,
    pub bgm_file: Option<String>,
    pub bgm_volume: Option<f32>,
    pub subtitle_enabled: Option<bool>,
    pub subtitle_position: Option<String>,
    pub custom_position: Option<f64>,
    pub font_name: Option<String>,
    pub text_fore_color: Option<String>,
    pub text_background_color: Option<serde_json::Value>,
    pub rounded_subtitle_background: Option<bool>,
    pub font_size: Option<u32>,
    pub stroke_color: Option<String>,
    pub stroke_width: Option<f32>,
    pub paragraph_number: Option<u32>,
    pub n_threads: Option<u32>,
    pub video_script_prompt: Option<String>,
    pub custom_system_prompt: Option<String>,
}

impl VideoParams {
    pub fn get_video_aspect(&self) -> VideoAspect {
        self.video_aspect
            .as_deref()
            .and_then(VideoAspect::from_str)
            .unwrap_or(VideoAspect::Portrait)
    }

    pub fn get_concat_mode(&self) -> VideoConcatMode {
        match self.video_concat_mode.as_deref().unwrap_or("random") {
            "sequential" => VideoConcatMode::Sequential,
            _ => VideoConcatMode::Random,
        }
    }

    pub fn get_clip_duration(&self) -> u32 {
        self.video_clip_duration.unwrap_or(5)
    }

    pub fn get_video_count(&self) -> u32 {
        self.video_count.unwrap_or(1)
    }

    pub fn get_voice_name(&self) -> &str {
        self.voice_name.as_deref().unwrap_or("")
    }

    pub fn get_voice_rate(&self) -> f32 {
        self.voice_rate.unwrap_or(1.0)
    }

    pub fn get_voice_volume(&self) -> f32 {
        self.voice_volume.unwrap_or(1.0)
    }

    pub fn get_bgm_volume(&self) -> f32 {
        self.bgm_volume.unwrap_or(0.2)
    }

    pub fn get_subtitle_enabled(&self) -> bool {
        self.subtitle_enabled.unwrap_or(true)
    }

    pub fn get_n_threads(&self) -> u32 {
        self.n_threads.unwrap_or(2)
    }

    pub fn get_paragraph_number(&self) -> u32 {
        self.paragraph_number.unwrap_or(1).min(10).max(1)
    }

    pub fn is_no_voice(&self) -> bool {
        let name = self.get_voice_name().to_lowercase();
        name == "no-voice" || name == "none"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_id: String,
    pub params: VideoParams,
    pub state: i32,
    pub progress: u32,
    pub script: Option<String>,
    pub terms: Option<Vec<String>>,
    pub audio_file: Option<String>,
    pub audio_duration: Option<f64>,
    pub subtitle_path: Option<String>,
    pub materials: Option<Vec<String>>,
    pub videos: Option<Vec<String>>,
    pub combined_videos: Option<Vec<String>>,
    pub cross_post_results: Option<Vec<serde_json::Value>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskInfo {
    pub fn new(task_id: String, params: VideoParams) -> Self {
        let now = Utc::now();
        Self {
            task_id,
            params,
            state: TaskStatus::Processing.as_i32(),
            progress: 0,
            script: None,
            terms: None,
            audio_file: None,
            audio_duration: None,
            subtitle_path: None,
            materials: None,
            videos: None,
            combined_videos: None,
            cross_post_results: None,
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, state: Option<i32>, progress: Option<u32>) {
        if let Some(s) = state {
            self.state = s;
        }
        if let Some(p) = progress {
            self.progress = p;
        }
        self.updated_at = Utc::now();
    }

    pub fn is_failed(&self) -> bool {
        self.state == TaskStatus::Failed.as_i32()
    }

    pub fn is_completed(&self) -> bool {
        self.state == TaskStatus::Completed.as_i32()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleCue {
    pub index: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub const PUNCTUATIONS: &[&str] = &[
    "?", ",", ".", "、", ";", ":", "!", "…", "？", "，", "。", "；", "：", "！", "...",
    "،", "؛", "؟",
];

pub const FILE_TYPE_VIDEOS: &[&str] = &["mp4", "mov", "mkv", "webm"];
pub const FILE_TYPE_IMAGES: &[&str] = &["jpg", "jpeg", "png", "bmp"];
