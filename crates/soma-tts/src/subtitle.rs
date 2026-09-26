//! 字幕生成与处理模块
//!
//! 提供 SRT 字幕格式转换、文件读写、解析、文本修正以及
//! 基于 Whisper 的语音识别字幕生成功能。

use soma_core::error::SomaError;
use soma_core::models::SubtitleCue;
use soma_core::utils;

/// 将字幕时间轴列表转换为 SRT 格式字符串
///
/// SRT 格式示例：
/// ```text
/// 1
/// 00:00:01,000 --> 00:00:03,000
/// 第一句字幕
/// ```
///
/// 自动对超长字幕文本进行换行处理：
/// - 中文/全角字符：每行最多 20 字
/// - 英文/半角字符：每行最多 40 字
///
/// - `cues` - 字幕时间轴列表
///
/// 返回 SRT 格式的字符串内容。
pub fn cues_to_srt(cues: &[SubtitleCue]) -> String {
    let mut srt = String::new();
    for cue in cues {
        let start = utils::time_convert_seconds_to_hmsm(cue.start_ms as f64 / 1000.0);
        let end = utils::time_convert_seconds_to_hmsm(cue.end_ms as f64 / 1000.0);
        let wrapped = wrap_subtitle_text(&cue.text, 14, 28);
        srt.push_str(&format!("{}\n{} --> {}\n{}\n\n", cue.index, start, end, wrapped));
    }
    srt
}

/// 对字幕文本进行自动换行
///
/// 按字符宽度累计：中文/全角字符计为 2 单位，英文/半角字符计为 1 单位。
/// 当累计宽度超过 max_units 时换行。max_units 对应 max_cjk 个中文字符或 max_ascii 个英文字符。
///
/// - `text` - 原始字幕文本
/// - `max_cjk` - 中文每行最大字符数
/// - `max_ascii` - 英文每行最大字符数（= max_cjk * 2）
fn wrap_subtitle_text(text: &str, max_cjk: usize, max_ascii: usize) -> String {
    let max_units = max_cjk * 2;
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_units = 0;

    for ch in text.chars() {
        let char_units = if ch.is_ascii() { 1 } else { 2 };
        if current_units + char_units > max_units && !current.is_empty() {
            lines.push(current.clone());
            current.clear();
            current_units = 0;
        }
        current.push(ch);
        current_units += char_units;
    }
    if !current.is_empty() {
        lines.push(current);
    }

    let _ = (max_ascii, max_cjk);
    lines.join("\n")
}

/// 将字幕时间轴列表写入 SRT 文件
///
/// - `cues` - 字幕时间轴列表
/// - `output_path` - SRT 文件输出路径
pub fn create_subtitle_file(cues: &[SubtitleCue], output_path: &str) -> Result<(), SomaError> {
    let content = cues_to_srt(cues);
    std::fs::write(output_path, content).map_err(SomaError::Io)
}

/// 从 SRT 文件读取并解析字幕
///
/// 如果文件读取失败，返回空列表。
///
/// - `srt_path` - SRT 文件路径
pub fn file_to_subtitles(srt_path: &str) -> Vec<SubtitleCue> {
    let content = match std::fs::read_to_string(srt_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    parse_srt(&content)
}

/// 解析 SRT 格式字符串为字幕时间轴列表
///
/// SRT 格式以空行分隔每个字幕块，每个块包含：
/// - 序号
/// - 时间轴（`HH:MM:SS,mmm --> HH:MM:SS,mmm`）
/// - 字幕文本（可多行）
///
/// - `content` - SRT 格式字符串
pub fn parse_srt(content: &str) -> Vec<SubtitleCue> {
    let mut cues = Vec::new();
    // 按空行分割为字幕块
    let blocks: Vec<&str> = content.split("\n\n").collect();
    let mut index = 1u32;

    for block in blocks {
        let lines: Vec<&str> = block.lines().collect();
        // 至少需要 3 行：序号、时间轴、文本
        if lines.len() < 3 {
            continue;
        }
        // 第二行为时间轴
        let time_line = lines.get(1).unwrap_or(&"");
        // 第三行起为字幕文本（支持多行）
        let text = lines[2..].join("\n").trim().to_string();
        if text.is_empty() {
            continue;
        }
        // 解析时间轴中的起止时间
        let parts: Vec<&str> = time_line.split(" --> ").collect();
        if parts.len() != 2 {
            continue;
        }
        let start_ms = parse_srt_timestamp(parts[0].trim());
        let end_ms = parse_srt_timestamp(parts[1].trim());

        cues.push(SubtitleCue {
            index,
            start_ms,
            end_ms,
            text,
        });
        index += 1;
    }
    cues
}

/// 解析 SRT 时间戳为毫秒
///
/// SRT 时间戳格式为 `HH:MM:SS,mmm`，例如 `00:01:23,456` 表示 1 分 23 秒 456 毫秒。
///
/// - `ts` - SRT 时间戳字符串
///
/// 返回对应的毫秒数，解析失败返回 0。
fn parse_srt_timestamp(ts: &str) -> u64 {
    // 以逗号分隔 "HH:MM:SS" 和 "mmm"
    let parts: Vec<&str> = ts.split(',').collect();
    if parts.len() != 2 {
        return 0;
    }
    let hms: Vec<u64> = parts[0].split(':').filter_map(|s| s.parse().ok()).collect();
    let ms: u64 = parts[1].parse().unwrap_or(0);
    if hms.len() == 3 {
        // 时 * 3600000 + 分 * 60000 + 秒 * 1000 + 毫秒
        (hms[0] * 3600000) + (hms[1] * 60000) + (hms[2] * 1000) + ms
    } else {
        0
    }
}

/// 使用视频脚本文本修正字幕内容
///
/// 将字幕的时间轴保持不变，仅用脚本按标点分句后的文本替换原始字幕文本。
/// 适用于 Whisper 生成的字幕文本不准确但时间轴正确的情况。
///
/// - `cues` - 待修正的字幕列表（就地修改）
/// - `video_script` - 视频脚本文本
pub fn correct_subtitle(cues: &mut [SubtitleCue], video_script: &str) {
    // 将脚本文本按标点分句
    let script_lines = utils::split_string_by_punctuations(video_script);
    if script_lines.is_empty() || cues.is_empty() {
        return;
    }

    // 逐条替换字幕文本，保留原有时间轴
    for (i, cue) in cues.iter_mut().enumerate() {
        if i < script_lines.len() {
            cue.text = script_lines[i].clone();
        }
    }
}

/// 使用 Whisper 语音识别模型生成字幕文件
///
/// 调用系统安装的 `whisper` 命令行工具对音频文件进行语音识别，
/// 输出 SRT 格式字幕到指定目录。
///
/// - `audio_file` - 输入音频文件路径
/// - `subtitle_file` - 字幕文件输出路径（用于确定输出目录）
pub fn generate_whisper_subtitle(audio_file: &str, subtitle_file: &str, model_size: &str, device: &str, compute_type: &str) -> Result<(), SomaError> {
    let model = if model_size.is_empty() { "base" } else { model_size };
    let dev = if device.is_empty() { "cpu" } else { device };
    let ct = if compute_type.is_empty() { "int8" } else { compute_type };
    let status = std::process::Command::new("whisper")
        .args([
            audio_file,
            "--model", model,
            "--device", dev,
            "--compute_type", ct,
            "--output_format", "srt",
            "--output_dir", std::path::Path::new(subtitle_file).parent().unwrap_or(std::path::Path::new(".")).to_string_lossy().as_ref(),
        ])
        .status()
        .map_err(|e| SomaError::Ffmpeg(format!("whisper command failed: {}", e)))?;

    if !status.success() {
        return Err(SomaError::Tts("whisper subtitle generation failed".into()));
    }
    Ok(())
}
