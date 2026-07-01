//! 工具函数模块
//!
//! 提供路径处理、字符串分割、SRT 字幕生成、文件名清理等通用工具函数，
//! 供视频生成流水线的各个环节调用。

use std::path::{Path, PathBuf};
use crate::error::SomaError;
use crate::models::PUNCTUATIONS;

/// 生成 UUID v4 字符串，用于任务 ID 等唯一标识
pub fn get_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 计算文本的 MD5 哈希值，返回十六进制小写字符串
pub fn md5(text: &str) -> String {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 获取应用根目录（当前工作目录），获取失败时回退为 "."
pub fn root_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 获取存储目录路径
///
/// # 参数
/// - `sub_dir`: 存储下的子目录名，空字符串表示 storage 根目录
/// - `create`: 是否在目录不存在时自动创建
pub fn storage_dir(sub_dir: &str, create: bool) -> PathBuf {
    let d = root_dir().join("storage");
    let d = if sub_dir.is_empty() { d } else { d.join(sub_dir) };
    if create && !d.exists() {
        let _ = std::fs::create_dir_all(&d);
    }
    d
}

/// 获取指定任务的工作目录（storage/tasks/{task_id}），不存在则自动创建
pub fn task_dir(task_id: &str) -> PathBuf {
    let d = storage_dir("tasks", true).join(task_id);
    if !d.exists() {
        let _ = std::fs::create_dir_all(&d);
    }
    d
}

/// 获取任务存储根目录（storage/tasks/），不存在则自动创建
pub fn tasks_dir() -> PathBuf {
    storage_dir("tasks", true)
}

/// 获取资源目录路径（resource/{sub_dir}）
///
/// 与 storage_dir 不同，resource 目录不会自动创建
pub fn resource_dir(sub_dir: &str) -> PathBuf {
    let d = root_dir().join("resource");
    if sub_dir.is_empty() { d } else { d.join(sub_dir) }
}

/// 获取字体资源目录（resource/fonts/）
pub fn font_dir() -> PathBuf {
    resource_dir("fonts")
}

/// 获取背景音乐资源目录（resource/songs/），不存在则自动创建
pub fn song_dir() -> PathBuf {
    let d = resource_dir("songs");
    if !d.exists() {
        let _ = std::fs::create_dir_all(&d);
    }
    d
}

/// 获取本地上传视频存储目录（storage/local_videos/），不存在则自动创建
pub fn local_videos_dir() -> PathBuf {
    storage_dir("local_videos", true)
}

/// 在指定基础目录内解析相对路径，防止路径遍历攻击
///
/// 检查目标路径解析后是否仍在基础目录内，防止通过 `../` 等手段
/// 访问允许目录以外的文件。
///
/// # 参数
/// - `base_dir`: 允许访问的基础目录
/// - `unsafe_path`: 待解析的不可信路径（可为相对或绝对路径）
///
/// # 返回
/// 解析成功返回规范化的绝对路径字符串，否则返回 `SomaError::UnsafePath`
pub fn resolve_path_within_directory(base_dir: &str, unsafe_path: &str) -> Result<String, SomaError> {
    if unsafe_path.is_empty() {
        return Err(SomaError::UnsafePath("empty path is not allowed".into()));
    }
    // 获取基础目录的真实路径（解析符号链接）
    let base_real = std::fs::canonicalize(base_dir).unwrap_or_else(|_| PathBuf::from(base_dir));
    // 绝对路径直接使用，相对路径则拼接到基础目录下
    let candidate = if Path::new(unsafe_path).is_absolute() {
        PathBuf::from(unsafe_path)
    } else {
        base_real.join(unsafe_path)
    };
    // 解析候选路径的真实路径
    let resolved = std::fs::canonicalize(&candidate).unwrap_or(candidate);
    let base_str = base_real.to_string_lossy();
    let resolved_str = resolved.to_string_lossy();
    // 安全检查：解析后的路径必须以基础目录为前缀
    if !resolved_str.starts_with(base_str.as_ref()) {
        return Err(SomaError::UnsafePath("path is outside the allowed directory".into()));
    }
    if !resolved.exists() {
        return Err(SomaError::UnsafePath("file does not exist".into()));
    }
    Ok(resolved.to_string_lossy().to_string())
}

/// 按标点符号拆分字符串
///
/// 遇到标点符号时断开，生成多个片段。换行符也会触发断开。
/// 数字之间的小数点和千分位逗号不会被当作分隔符。
///
/// # 示例
/// "你好，世界！测试" → ["你好", "世界", "测试"]
pub fn split_string_by_punctuations(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut txt = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        // 换行符直接触发断句
        if *ch == '\n' {
            let trimmed = txt.trim().to_string();
            if !trimmed.is_empty() {
                result.push(trimmed);
            }
            txt.clear();
            continue;
        }

        // 检查前后字符是否为数字，避免拆散小数（1.5）或千分位（1,000）
        let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
        let next_is_digit = i + 1 < chars.len() && chars[i + 1].is_ascii_digit();

        if (*ch == '.' || *ch == ',') && prev_is_digit && next_is_digit {
            txt.push(*ch);
            continue;
        }

        // 遇到标点符号则断句
        if PUNCTUATIONS.contains(&ch.to_string().as_str()) {
            let trimmed = txt.trim().to_string();
            if !trimmed.is_empty() {
                result.push(trimmed);
            }
            txt.clear();
        } else {
            txt.push(*ch);
        }
    }
    // 处理末尾剩余文本
    let trimmed = txt.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }
    result
}

/// 规范化脚本文本，用于字幕匹配
///
/// 去除 Markdown 分隔线（连续3个以上的 -、*、_）、
/// 过滤空行、移除下划线，使脚本文本更适合与字幕进行比对。
pub fn normalize_script_for_subtitle_matching(script: &str) -> String {
    let re = regex::Regex::new(r"[-*_]{3,}").expect("内建正则编译失败");
    let lines: Vec<String> = script
        .lines()
        .map(|l| l.trim())
        .filter(|l| !re.is_match(l))
        .map(|l| l.replace('_', ""))
        .collect();
    lines.join("\n").trim().to_string()
}

/// 将秒数转换为 SRT 时间格式 `HH:MM:SS,mmm`
///
/// # 示例
/// 3661.5 → "01:01:01,500"
pub fn time_convert_seconds_to_hmsm(seconds: f64) -> String {
    let hours = (seconds as u64) / 3600;
    let rem = (seconds as u64) % 3600;
    let minutes = rem / 60;
    let secs = rem % 60;
    let millis = ((seconds * 1000.0) as u64) % 1000;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

/// 生成单条 SRT 字幕格式文本
///
/// # 参数
/// - `idx`: 字幕序号
/// - `msg`: 字幕文本
/// - `start_time`: 起始时间（秒）
/// - `end_time`: 结束时间（秒）
///
/// # 返回
/// 格式如 "1\n00:00:01,000 --> 00:00:03,500\n你好\n"
pub fn text_to_srt(idx: u32, msg: &str, start_time: f64, end_time: f64) -> String {
    format!(
        "{}\n{} --> {}\n{}\n",
        idx,
        time_convert_seconds_to_hmsm(start_time),
        time_convert_seconds_to_hmsm(end_time),
        msg
    )
}

/// 清理上传文件名，防止路径遍历
///
/// 仅保留文件名的最后一段（去除目录前缀），
/// 拒绝空文件名、`.` 和 `..` 等特殊路径。
pub fn sanitize_upload_filename(filename: &str) -> Result<String, SomaError> {
    let normalized = filename.replace('\\', "/").split('/').last().unwrap_or("").trim().to_string();
    if normalized.is_empty() || normalized == "." || normalized == ".." {
        return Err(SomaError::Config("invalid filename".into()));
    }
    Ok(normalized)
}

/// 构建标准 API 响应 JSON
///
/// # 参数
/// - `status`: 状态码（如 200 表示成功，-1 表示失败）
/// - `data`: 可选的数据载荷
/// - `message`: 可选的消息文本
pub fn get_response(status: i32, data: Option<serde_json::Value>, message: &str) -> serde_json::Value {
    let mut obj = serde_json::json!({ "status": status });
    if let Some(d) = data {
        obj["data"] = d;
    }
    if !message.is_empty() {
        obj["message"] = serde_json::Value::String(message.to_string());
    }
    obj
}

/// 将任务本地文件路径转换为可通过 HTTP 访问的 URI
///
/// 如果文件已经是 HTTP URL 则原样返回；
/// 否则基于 endpoint 和任务基础路径构造 `tasks/...` 格式的 URI。
///
/// # 参数
/// - `file`: 文件路径（本地路径或 URL）
/// - `endpoint`: 服务外部访问端点（如 "http://localhost:8080"）
/// - `task_base`: 任务本地基础路径前缀，用于提取相对路径
pub fn task_file_to_uri(file: &str, endpoint: &str, task_base: &str) -> String {
    // 已经是完整 URL 则直接返回
    if file.starts_with("http://") || file.starts_with("https://") {
        return file.to_string();
    }
    // 去除任务基础路径前缀，获取相对路径
    let relative = if let Some(stripped) = file.strip_prefix(task_base) {
        stripped.trim_start_matches('/').to_string()
    } else {
        file.trim_start_matches('/').to_string()
    };
    let uri = format!("tasks/{}", relative);
    if endpoint.is_empty() {
        format!("/{}", uri)
    } else {
        format!("{}/{}", endpoint.trim_end_matches('/'), uri)
    }
}
