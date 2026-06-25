use std::path::{Path, PathBuf};
use crate::error::SomaError;
use crate::models::PUNCTUATIONS;

pub fn get_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn md5(text: &str) -> String {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn root_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn storage_dir(sub_dir: &str, create: bool) -> PathBuf {
    let d = root_dir().join("storage");
    let d = if sub_dir.is_empty() { d } else { d.join(sub_dir) };
    if create && !d.exists() {
        let _ = std::fs::create_dir_all(&d);
    }
    d
}

pub fn task_dir(task_id: &str) -> PathBuf {
    let d = storage_dir("tasks", true).join(task_id);
    if !d.exists() {
        let _ = std::fs::create_dir_all(&d);
    }
    d
}

pub fn tasks_dir() -> PathBuf {
    storage_dir("tasks", true)
}

pub fn resource_dir(sub_dir: &str) -> PathBuf {
    let d = root_dir().join("resource");
    if sub_dir.is_empty() { d } else { d.join(sub_dir) }
}

pub fn font_dir() -> PathBuf {
    resource_dir("fonts")
}

pub fn song_dir() -> PathBuf {
    let d = resource_dir("songs");
    if !d.exists() {
        let _ = std::fs::create_dir_all(&d);
    }
    d
}

pub fn local_videos_dir() -> PathBuf {
    storage_dir("local_videos", true)
}

pub fn resolve_path_within_directory(base_dir: &str, unsafe_path: &str) -> Result<String, SomaError> {
    if unsafe_path.is_empty() {
        return Err(SomaError::UnsafePath("empty path is not allowed".into()));
    }
    let base_real = std::fs::canonicalize(base_dir).unwrap_or_else(|_| PathBuf::from(base_dir));
    let candidate = if Path::new(unsafe_path).is_absolute() {
        PathBuf::from(unsafe_path)
    } else {
        base_real.join(unsafe_path)
    };
    let resolved = std::fs::canonicalize(&candidate).unwrap_or(candidate);
    let base_str = base_real.to_string_lossy();
    let resolved_str = resolved.to_string_lossy();
    if !resolved_str.starts_with(base_str.as_ref()) {
        return Err(SomaError::UnsafePath("path is outside the allowed directory".into()));
    }
    if !resolved.exists() {
        return Err(SomaError::UnsafePath("file does not exist".into()));
    }
    Ok(resolved.to_string_lossy().to_string())
}

pub fn split_string_by_punctuations(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut txt = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if *ch == '\n' {
            let trimmed = txt.trim().to_string();
            if !trimmed.is_empty() {
                result.push(trimmed);
            }
            txt.clear();
            continue;
        }

        let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
        let next_is_digit = i + 1 < chars.len() && chars[i + 1].is_ascii_digit();

        if (*ch == '.' || *ch == ',') && prev_is_digit && next_is_digit {
            txt.push(*ch);
            continue;
        }

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
    let trimmed = txt.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }
    result
}

pub fn normalize_script_for_subtitle_matching(script: &str) -> String {
    let re = regex::Regex::new(r"[-*_]{3,}").unwrap();
    let lines: Vec<String> = script
        .lines()
        .map(|l| l.trim())
        .filter(|l| !re.is_match(l))
        .map(|l| l.replace('_', ""))
        .collect();
    lines.join("\n").trim().to_string()
}

pub fn time_convert_seconds_to_hmsm(seconds: f64) -> String {
    let hours = (seconds as u64) / 3600;
    let rem = (seconds as u64) % 3600;
    let minutes = rem / 60;
    let secs = rem % 60;
    let millis = ((seconds * 1000.0) as u64) % 1000;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

pub fn text_to_srt(idx: u32, msg: &str, start_time: f64, end_time: f64) -> String {
    format!(
        "{}\n{} --> {}\n{}\n",
        idx,
        time_convert_seconds_to_hmsm(start_time),
        time_convert_seconds_to_hmsm(end_time),
        msg
    )
}

pub fn sanitize_upload_filename(filename: &str) -> Result<String, SomaError> {
    let normalized = filename.replace('\\', "/").split('/').last().unwrap_or("").trim().to_string();
    if normalized.is_empty() || normalized == "." || normalized == ".." {
        return Err(SomaError::Config("invalid filename".into()));
    }
    Ok(normalized)
}

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

pub fn task_file_to_uri(file: &str, endpoint: &str, task_base: &str) -> String {
    if file.starts_with("http://") || file.starts_with("https://") {
        return file.to_string();
    }
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
