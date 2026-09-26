//! yt-dlp 视频下载模块
//!
//! 通过命令行调用 yt-dlp 工具，支持从 1000+ 站点下载视频/音频。
//! 包括 YouTube、TikTok、Instagram、Twitter/X、B站、抖音等。
//!
//! 使用方式：
//! - 确保 yt-dlp 已安装（`pip install yt-dlp` 或 `brew install yt-dlp`）
//! - 调用 `download` 函数下载视频
//! - 调用 `get_info` 函数获取视频信息（不下载）

use soma_core::error::SomaError;
use std::process::Command;

/// yt-dlp 下载参数
#[derive(Debug, Clone)]
pub struct YtdlpParams {
    /// 视频 URL
    pub url: String,
    /// 保存目录
    pub save_dir: String,
    /// 输出文件名模板（如 "%(title)s.%(ext)s"），为空则使用视频 ID
    pub output_template: Option<String>,
    /// 视频格式选择（如 "best[ext=mp4]"、"bestvideo+bestaudio"），为空则 "best"
    pub format: Option<String>,
    /// 是否仅提取音频（MP3）
    pub audio_only: bool,
    /// 音频质量（仅 audio_only 时生效，如 "192K"）
    pub audio_quality: Option<String>,
    /// 代理 URL（如 "http://127.0.0.1:7890"）
    pub proxy: Option<String>,
    /// 限制速率（如 "1M"）
    pub rate_limit: Option<String>,
    /// 是否跳过已下载的文件
    pub skip_existing: bool,
}

impl Default for YtdlpParams {
    fn default() -> Self {
        Self {
            url: String::new(),
            save_dir: String::new(),
            output_template: None,
            format: None,
            audio_only: false,
            audio_quality: None,
            proxy: None,
            rate_limit: None,
            skip_existing: true,
        }
    }
}

/// yt-dlp 视频信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoInfo {
    /// 视频 ID
    pub id: String,
    /// 视频标题
    pub title: String,
    /// 时长（秒）
    pub duration: Option<f64>,
    /// 上传者
    pub uploader: Option<String>,
    /// 上传日期
    pub upload_date: Option<String>,
    /// 视频描述
    pub description: Option<String>,
    /// 缩略图 URL
    pub thumbnail: Option<String>,
    /// 视频格式列表
    pub formats: Vec<FormatInfo>,
}

/// 视频格式信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormatInfo {
    /// 格式 ID
    pub format_id: String,
    /// 扩展名
    pub ext: String,
    /// 分辨率
    pub resolution: Option<String>,
    /// 文件大小（字节）
    pub filesize: Option<u64>,
    /// 是否包含视频
    pub has_video: bool,
    /// 是否包含音频
    pub has_audio: bool,
}

/// 检查 yt-dlp 是否已安装
pub fn check_ytdlp() -> Result<String, SomaError> {
    let output = Command::new("yt-dlp")
        .arg("--version")
        .output()
        .map_err(|e| SomaError::Stock(format!("yt-dlp 未安装或不可用: {}", e)))?;

    if !output.status.success() {
        return Err(SomaError::Stock("yt-dlp 命令执行失败".into()));
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}

/// 获取视频信息（不下载）
///
/// 调用 `yt-dlp --dump-json` 获取视频元数据
pub async fn get_info(url: &str, proxy: Option<&str>) -> Result<VideoInfo, SomaError> {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("--dump-json").arg("--no-playlist");

    if let Some(p) = proxy {
        cmd.arg("--proxy").arg(p);
    }

    cmd.arg(url);

    let output = cmd
        .output()
        .map_err(|e| SomaError::Stock(format!("yt-dlp 执行失败: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SomaError::Stock(format!("yt-dlp 获取信息失败: {}", stderr)));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| SomaError::Stock(format!("解析 yt-dlp 输出失败: {}", e)))?;

    let formats = v.get("formats")
        .and_then(|f| f.as_array())
        .map(|arr| {
            arr.iter().filter_map(|f| {
                let format_id = f.get("format_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if format_id.is_empty() {
                    return None;
                }
                Some(FormatInfo {
                    format_id,
                    ext: f.get("ext").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    resolution: f.get("resolution").and_then(|v| v.as_str()).map(String::from),
                    filesize: f.get("filesize").and_then(|v| v.as_u64()),
                    has_video: f.get("vcodec").and_then(|v| v.as_str()).map_or(false, |s| s != "none"),
                    has_audio: f.get("acodec").and_then(|v| v.as_str()).map_or(false, |s| s != "none"),
                })
            }).collect()
        })
        .unwrap_or_default();

    Ok(VideoInfo {
        id: v.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        title: v.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        duration: v.get("duration").and_then(|v| v.as_f64()),
        uploader: v.get("uploader").and_then(|v| v.as_str()).map(String::from),
        upload_date: v.get("upload_date").and_then(|v| v.as_str()).map(String::from),
        description: v.get("description").and_then(|v| v.as_str()).map(String::from),
        thumbnail: v.get("thumbnail").and_then(|v| v.as_str()).map(String::from),
        formats,
    })
}

/// 下载视频
///
/// 调用 yt-dlp 命令行工具下载视频到指定目录
pub async fn download(params: &YtdlpParams) -> Result<String, SomaError> {
    if params.url.is_empty() {
        return Err(SomaError::Stock("URL 不能为空".into()));
    }

    let save_dir = std::path::Path::new(&params.save_dir);
    if !save_dir.exists() {
        std::fs::create_dir_all(save_dir).map_err(SomaError::Io)?;
    }

    let template = params.output_template.clone()
        .unwrap_or_else(|| "%(id)s.%(ext)s".to_string());
    let output_path = save_dir.join(&template);

    let mut cmd = Command::new("yt-dlp");
    cmd.arg("--no-playlist")
        .arg("-o").arg(output_path.to_string_lossy().to_string());

    if params.skip_existing {
        cmd.arg("--no-overwrites");
    }

    if let Some(p) = &params.proxy {
        cmd.arg("--proxy").arg(p);
    }

    if let Some(r) = &params.rate_limit {
        cmd.arg("--limit-rate").arg(r);
    }

    if params.audio_only {
        cmd.arg("-x").arg("--audio-format").arg("mp3");
        if let Some(q) = &params.audio_quality {
            cmd.arg("--audio-quality").arg(q);
        }
    } else {
        let fmt = params.format.clone().unwrap_or_else(|| "best[ext=mp4]/best".to_string());
        cmd.arg("-f").arg(fmt);
    }

    cmd.arg(&params.url);

    let output = cmd
        .output()
        .map_err(|e| SomaError::Stock(format!("yt-dlp 执行失败: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SomaError::Stock(format!("yt-dlp 下载失败: {}", stderr)));
    }

    // 从 stdout 解析实际下载的文件路径
    let stdout = String::from_utf8_lossy(&output.stdout);
    let downloaded_path = parse_downloaded_path(&stdout)
        .unwrap_or_else(|| {
            // 回退：根据模板猜测文件路径
            let ext = if params.audio_only { "mp3" } else { "mp4" };
            save_dir.join(format!("*.{}", ext)).to_string_lossy().to_string()
        });

    Ok(downloaded_path)
}

/// 批量下载视频
///
/// 依次下载多个 URL，返回成功下载的文件路径列表
pub async fn download_batch(
    urls: &[String],
    save_dir: &str,
    proxy: Option<&str>,
    audio_only: bool,
) -> Result<Vec<String>, SomaError> {
    let mut results = Vec::new();
    for url in urls {
        let params = YtdlpParams {
            url: url.clone(),
            save_dir: save_dir.to_string(),
            proxy: proxy.map(String::from),
            audio_only,
            ..Default::default()
        };
        match download(&params).await {
            Ok(path) => results.push(path),
            Err(e) => {
                log!("yt-dlp 下载失败: {} -> {:?}", url, e);
            }
        }
    }
    Ok(results)
}

/// 从 yt-dlp stdout 解析已下载的文件路径
///
/// yt-dlp 输出格式: "[download] Destination: /path/to/file.mp4"
/// 或: "[download] /path/to/file.mp4 has already been downloaded"
fn parse_downloaded_path(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        if line.contains("Destination:") {
            return line.split("Destination:").nth(1).map(|s| s.trim().to_string());
        }
        if line.contains("has already been downloaded") {
            return line.split_whitespace()
                .find(|w| w.contains('.') && !w.starts_with('['))
                .map(String::from);
        }
        if line.contains("Merging") || line.contains("Deleting") {
            continue;
        }
    }
    None
}

/// 从分享文本中提取 URL
///
/// 支持从抖音、小红书、快手等平台的分享文本中提取链接
pub fn extract_urls(text: &str) -> Vec<String> {
    let url_regex = regex::Regex::new(
        r#"https?://[^\s<>"]+"#
    ).unwrap();
    url_regex
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// 检测 URL 对应的平台
pub fn detect_platform(url: &str) -> &'static str {
    let lower = url.to_lowercase();
    if lower.contains("youtube.com") || lower.contains("youtu.be") {
        "youtube"
    } else if lower.contains("tiktok.com") {
        "tiktok"
    } else if lower.contains("instagram.com") {
        "instagram"
    } else if lower.contains("twitter.com") || lower.contains("x.com") {
        "twitter"
    } else if lower.contains("bilibili.com") || lower.contains("b23.tv") {
        "bilibili"
    } else if lower.contains("douyin.com") {
        "douyin"
    } else if lower.contains("kuaishou.com") {
        "kuaishou"
    } else if lower.contains("xhslink.com") || lower.contains("xiaohongshu.com") {
        "xhs"
    } else if lower.contains("weibo.com") {
        "weibo"
    } else {
        "unknown"
    }
}
