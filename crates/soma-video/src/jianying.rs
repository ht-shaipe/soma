//! 剪映（JianYing/CapCut）草稿文件生成模块
//!
//! 生成剪映 draft_content.json 等草稿文件结构，
//! 包括视频轨道、音频轨道、字幕轨道、素材管理、片段裁剪。
//! 生成的草稿可直接在剪映客户端中打开编辑。

use serde::{Deserialize, Serialize};
use std::path::Path;

/// 剪映草稿项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JianYingDraft {
    pub id: String,
    pub name: String,
    pub canvas: Canvas,
    pub duration: u64,
    pub materials: Materials,
    pub tracks: Vec<Track>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub width: u64,
    pub height: u64,
    pub ratio: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Materials {
    pub videos: Vec<VideoMaterial>,
    pub audios: Vec<AudioMaterial>,
    pub texts: Vec<TextMaterial>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMaterial {
    pub id: String,
    pub path: String,
    pub duration: u64,
    pub width: u64,
    pub height: u64,
    pub fps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMaterial {
    pub id: String,
    pub path: String,
    pub duration: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMaterial {
    pub id: String,
    pub content: String,
    pub font_path: String,
    pub font_size: f64,
    pub font_color: String,
}

/// 轨道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub track_type: String,
    pub segments: Vec<Segment>,
}

/// 片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub material_id: String,
    pub target_timerange: TimeRange,
    pub source_timerange: TimeRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: u64,
    pub duration: u64,
}

/// 剪映草稿构建器
pub struct JianYingBuilder {
    draft: JianYingDraft,
}

impl JianYingBuilder {
    /// 创建新草稿
    pub fn new(name: &str, width: u64, height: u64) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let ratio = match (width, height) {
            (1920, 1080) => "16:9".to_string(),
            (1080, 1920) => "9:16".to_string(),
            (1080, 1080) => "1:1".to_string(),
            _ => format!("{}:{}", width, height),
        };

        Self {
            draft: JianYingDraft {
                id,
                name: name.to_string(),
                canvas: Canvas {
                    width,
                    height,
                    ratio,
                },
                duration: 0,
                materials: Materials {
                    videos: Vec::new(),
                    audios: Vec::new(),
                    texts: Vec::new(),
                },
                tracks: Vec::new(),
                version: "4.5.0".to_string(),
            },
        }
    }

    /// 添加视频素材
    pub fn add_video(
        &mut self,
        path: &str,
        duration: u64,
        width: u64,
        height: u64,
        fps: f64,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.draft.materials.videos.push(VideoMaterial {
            id: id.clone(),
            path: path.to_string(),
            duration,
            width,
            height,
            fps,
        });
        id
    }

    /// 添加音频素材
    pub fn add_audio(&mut self, path: &str, duration: u64, name: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.draft.materials.audios.push(AudioMaterial {
            id: id.clone(),
            path: path.to_string(),
            duration,
            name: name.to_string(),
        });
        id
    }

    /// 添加文本素材
    pub fn add_text(&mut self, content: &str, font_size: f64, font_color: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.draft.materials.texts.push(TextMaterial {
            id: id.clone(),
            content: content.to_string(),
            font_path: String::new(),
            font_size,
            font_color: font_color.to_string(),
        });
        id
    }

    /// 添加视频轨道
    pub fn add_video_track(&mut self) -> &mut Track {
        let track = Track {
            id: uuid::Uuid::new_v4().to_string(),
            track_type: "video".to_string(),
            segments: Vec::new(),
        };
        self.draft.tracks.push(track);
        self.draft.tracks.last_mut().unwrap()
    }

    /// 添加音频轨道
    pub fn add_audio_track(&mut self) -> &mut Track {
        let track = Track {
            id: uuid::Uuid::new_v4().to_string(),
            track_type: "audio".to_string(),
            segments: Vec::new(),
        };
        self.draft.tracks.push(track);
        self.draft.tracks.last_mut().unwrap()
    }

    /// 添加字幕轨道
    pub fn add_text_track(&mut self) -> &mut Track {
        let track = Track {
            id: uuid::Uuid::new_v4().to_string(),
            track_type: "text".to_string(),
            segments: Vec::new(),
        };
        self.draft.tracks.push(track);
        self.draft.tracks.last_mut().unwrap()
    }

    /// 向轨道添加片段
    pub fn add_segment(
        track: &mut Track,
        material_id: &str,
        target_start: u64,
        target_duration: u64,
        source_start: u64,
        source_duration: u64,
    ) {
        track.segments.push(Segment {
            id: uuid::Uuid::new_v4().to_string(),
            material_id: material_id.to_string(),
            target_timerange: TimeRange {
                start: target_start,
                duration: target_duration,
            },
            source_timerange: TimeRange {
                start: source_start,
                duration: source_duration,
            },
        });
    }

    /// 设置总时长
    pub fn set_duration(&mut self, duration: u64) {
        self.draft.duration = duration;
    }

    /// 构建
    pub fn build(self) -> JianYingDraft {
        self.draft
    }

    /// 保存草稿到目录
    ///
    /// 生成 draft_content.json 文件
    pub fn save_to_dir(&self, dir: &str) -> Result<String, String> {
        let dir = Path::new(dir);
        if !dir.exists() {
            std::fs::create_dir_all(dir).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let json =
            serde_json::to_string_pretty(&self.draft).map_err(|e| format!("序列化失败: {}", e))?;

        let file_path = dir.join("draft_content.json");
        std::fs::write(&file_path, json).map_err(|e| format!("写入文件失败: {}", e))?;

        Ok(file_path.to_string_lossy().to_string())
    }
}

/// 获取视频时长（微秒），通过 ffprobe
pub fn get_video_duration_us(path: &str) -> Result<u64, String> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path,
        ])
        .output()
        .map_err(|e| format!("ffprobe 执行失败: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let seconds: f64 = duration_str
        .parse()
        .map_err(|e| format!("解析时长失败: {}", e))?;
    Ok((seconds * 1_000_000.0) as u64)
}

/// 获取视频分辨率和帧率
pub fn get_video_info(path: &str) -> Result<(u64, u64, f64), String> {
    let output = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,r_frame_rate",
            "-of",
            "csv=p=0",
            path,
        ])
        .output()
        .map_err(|e| format!("ffprobe 执行失败: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let info = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parts: Vec<&str> = info.split(',').collect();
    if parts.len() < 3 {
        return Err("无法解析视频信息".into());
    }

    let width: u64 = parts[0].parse().unwrap_or(1920);
    let height: u64 = parts[1].parse().unwrap_or(1080);
    let fps = if parts[2].contains('/') {
        let fr: Vec<&str> = parts[2].split('/').collect();
        if fr.len() == 2 {
            let num: f64 = fr[0].parse().unwrap_or(30.0);
            let den: f64 = fr[1].parse().unwrap_or(1.0);
            num / den
        } else {
            30.0
        }
    } else {
        parts[2].parse().unwrap_or(30.0)
    };

    Ok((width, height, fps))
}

/// 从视频文件列表生成剪映草稿
///
/// 自动获取每个视频的时长、分辨率，按顺序排列在视频轨道上。
pub fn create_draft_from_videos(
    name: &str,
    video_paths: &[String],
    audio_path: Option<&str>,
    width: u64,
    height: u64,
    output_dir: &str,
) -> Result<String, String> {
    let mut builder = JianYingBuilder::new(name, width, height);
    let video_track = builder.add_video_track();
    let video_track_id = video_track.id.clone();

    let mut total_duration_us: u64 = 0;

    for path in video_paths {
        let duration_us = get_video_duration_us(path)?;
        let (vw, vh, fps) = get_video_info(path)?;
        let material_id = builder.add_video(path, duration_us, vw, vh, fps);

        // 在 track 中找到对应 track 并添加 segment
        let track = builder
            .draft
            .tracks
            .iter_mut()
            .find(|t| t.id == video_track_id);
        if let Some(track) = track {
            JianYingBuilder::add_segment(
                track,
                &material_id,
                total_duration_us,
                duration_us,
                0,
                duration_us,
            );
        }
        total_duration_us += duration_us;
    }

    // 添加音频
    if let Some(audio) = audio_path {
        let audio_duration = get_video_duration_us(audio).unwrap_or(total_duration_us);
        let audio_id = builder.add_audio(audio, audio_duration, "background_audio");
        let audio_track = builder.add_audio_track();
        JianYingBuilder::add_segment(
            audio_track,
            &audio_id,
            0,
            total_duration_us.min(audio_duration),
            0,
            total_duration_us.min(audio_duration),
        );
    }

    builder.set_duration(total_duration_us);
    builder.save_to_dir(output_dir)
}
