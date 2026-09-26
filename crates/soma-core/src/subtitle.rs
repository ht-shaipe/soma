//! 字幕高级处理模块
//!
//! 提供字幕的翻译、校正、合并和格式转换功能。
//! - 翻译：通过 LLM API 批量翻译字幕文本
//! - 校正：通过 LLM API 纠正字幕文本中的错别字/标点
//! - 合并：将多个 SRT 字幕文件按时间轴合并
//! - 格式转换：SRT / ASS / VTT 互转

use serde::{Deserialize, Serialize};

/// 字幕条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleEntry {
    pub index: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

/// 字幕格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubtitleFormat {
    Srt,
    Ass,
    Vtt,
}

impl SubtitleFormat {
    pub fn from_ext(ext: &str) -> Self {
        match ext.to_lowercase().trim_start_matches('.') {
            "ass" => SubtitleFormat::Ass,
            "vtt" => SubtitleFormat::Vtt,
            _ => SubtitleFormat::Srt,
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  SRT 解析与渲染
// ═══════════════════════════════════════════════════════════

/// 解析 SRT 格式字符串
pub fn parse_srt(content: &str) -> Vec<SubtitleEntry> {
    let mut entries = Vec::new();
    let mut index = 1u32;

    for block in content.split("\n\n") {
        let lines: Vec<&str> = block.lines().collect();
        if lines.len() < 3 {
            continue;
        }
        let time_line = lines.get(1).unwrap_or(&"");
        let text = lines[2..].join("\n").trim().to_string();
        if text.is_empty() {
            continue;
        }
        let parts: Vec<&str> = time_line.split(" --> ").collect();
        if parts.len() != 2 {
            continue;
        }
        entries.push(SubtitleEntry {
            index,
            start_ms: parse_timestamp(parts[0].trim(), ','),
            end_ms: parse_timestamp(parts[1].trim(), ','),
            text,
        });
        index += 1;
    }
    entries
}

/// 渲染为 SRT 格式字符串
pub fn to_srt(entries: &[SubtitleEntry]) -> String {
    let mut output = String::new();
    for entry in entries {
        output.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            entry.index,
            format_timestamp(entry.start_ms, ','),
            format_timestamp(entry.end_ms, ','),
            entry.text
        ));
    }
    output
}

// ═══════════════════════════════════════════════════════════
//  ASS 解析与渲染
// ═══════════════════════════════════════════════════════════

/// 渲染为 ASS 格式字符串
pub fn to_ass(entries: &[SubtitleEntry], style: Option<&str>) -> String {
    let mut output = String::new();
    output.push_str("[Script Info]\n");
    output.push_str("ScriptType: v4.00+\n");
    output.push_str("PlayResX: 1920\n");
    output.push_str("PlayResY: 1080\n\n");

    output.push_str("[V4+ Styles]\n");
    output.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, BackColour, Bold, Italic, BorderStyle, Outline, Alignment, MarginL, MarginR, MarginV\n");
    let style_line = style.unwrap_or("Default,Microsoft YaHei,60,&H00FFFFFF,&H80000000,0,0,1,2,2,80,80,40");
    output.push_str(&format!("Style: {}\n\n", style_line));

    output.push_str("[Events]\n");
    output.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

    for entry in entries {
        let start = format_ass_time(entry.start_ms);
        let end = format_ass_time(entry.end_ms);
        let text = entry.text.replace('\n', "\\N");
        output.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start, end, text
        ));
    }
    output
}

/// ASS 时间格式: H:MM:SS.cc
fn format_ass_time(ms: u64) -> String {
    let h = ms / 3600000;
    let m = (ms % 3600000) / 60000;
    let s = (ms % 60000) / 1000;
    let cs = (ms % 1000) / 10;
    format!("{}:{:02}:{:02}.{:02}", h, m, s, cs)
}

// ═══════════════════════════════════════════════════════════
//  VTT 解析与渲染
// ═══════════════════════════════════════════════════════════

/// 渲染为 WebVTT 格式字符串
pub fn to_vtt(entries: &[SubtitleEntry]) -> String {
    let mut output = String::from("WEBVTT\n\n");
    for entry in entries {
        output.push_str(&format!(
            "{} --> {}\n{}\n\n",
            format_vtt_time(entry.start_ms),
            format_vtt_time(entry.end_ms),
            entry.text
        ));
    }
    output
}

/// VTT 时间格式: HH:MM:SS.mmm
fn format_vtt_time(ms: u64) -> String {
    let h = ms / 3600000;
    let m = (ms % 3600000) / 60000;
    let s = (ms % 60000) / 1000;
    let mms = ms % 1000;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, mms)
}

// ═══════════════════════════════════════════════════════════
//  字幕合并
// ═══════════════════════════════════════════════════════════

/// 合并多个字幕列表
///
/// 按时间轴排序，处理重叠（重叠的条目保留后一个），
/// 重新编号序号。
pub fn merge_subtitles(lists: &[Vec<SubtitleEntry>]) -> Vec<SubtitleEntry> {
    let mut all: Vec<SubtitleEntry> = lists.iter().flat_map(|l| l.iter().cloned()).collect();

    // 按开始时间排序
    all.sort_by_key(|e| e.start_ms);

    // 移除重叠条目（保留后一个）
    let mut merged: Vec<SubtitleEntry> = Vec::new();
    for entry in all {
        if let Some(last) = merged.last() {
            if entry.start_ms < last.end_ms {
                // 重叠：跳过前一个，用后一个替换
                merged.pop();
            }
        }
        merged.push(entry);
    }

    // 重新编号
    for (i, entry) in merged.iter_mut().enumerate() {
        entry.index = (i + 1) as u32;
    }

    merged
}

/// 从多个 SRT 文件合并为一个
pub fn merge_srt_files(paths: &[String]) -> Result<Vec<SubtitleEntry>, String> {
    let mut lists = Vec::new();
    for path in paths {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("读取文件失败 {}: {}", path, e))?;
        lists.push(parse_srt(&content));
    }
    Ok(merge_subtitles(&lists))
}

// ═══════════════════════════════════════════════════════════
//  字幕翻译（LLM API 调用）
// ═══════════════════════════════════════════════════════════

/// 翻译请求参数
#[derive(Debug, Clone)]
pub struct TranslateParams {
    /// LLM API Key
    pub api_key: String,
    /// LLM Base URL（OpenAI 兼容）
    pub base_url: String,
    /// 模型名称
    pub model: String,
    /// 目标语言（如 "English"、"日语"）
    pub target_lang: String,
    /// 每批翻译的条目数（默认 20）
    pub batch_size: usize,
}

/// 批量翻译字幕
///
/// 将字幕文本分批发送给 LLM 翻译，保持时间轴不变。
pub async fn translate_subtitles(
    entries: &[SubtitleEntry],
    params: &TranslateParams,
) -> Result<Vec<SubtitleEntry>, String> {
    let batch_size = if params.batch_size == 0 { 20 } else { params.batch_size };
    let mut result = Vec::new();

    for chunk in entries.chunks(batch_size) {
        let texts: Vec<&str> = chunk.iter().map(|e| e.text.as_str()).collect();
        let translated = translate_batch(&texts, params).await?;

        for (i, entry) in chunk.iter().enumerate() {
            let text = translated.get(i).cloned().unwrap_or_else(|| entry.text.clone());
            result.push(SubtitleEntry {
                index: entry.index,
                start_ms: entry.start_ms,
                end_ms: entry.end_ms,
                text,
            });
        }
    }

    Ok(result)
}

/// 调用 LLM API 翻译一批文本
async fn translate_batch(
    texts: &[&str],
    params: &TranslateParams,
) -> Result<Vec<String>, String> {
    let prompt = format!(
        "将以下字幕文本翻译为{}。保持原意，语气自然。每行一个翻译结果，不要添加序号或额外说明。\n\n{}",
        params.target_lang,
        texts.join("\n")
    );

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", params.base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": params.model,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", params.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("LLM 请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("LLM 返回错误: HTTP {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 LLM 响应失败: {}", e))?;

    let content = json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    Ok(content.lines().map(String::from).collect())
}

// ═══════════════════════════════════════════════════════════
//  字幕校正（LLM API 调用）
// ═══════════════════════════════════════════════════════════

/// 校正请求参数
#[derive(Debug, Clone)]
pub struct CorrectParams {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    /// 校正类型：typo（错别字）/ punctuation（标点）/ all（全部）
    pub correct_type: String,
}

/// 批量校正字幕文本
///
/// 通过 LLM 纠正字幕中的错别字、标点错误等，保持时间轴不变。
pub async fn correct_subtitles(
    entries: &[SubtitleEntry],
    params: &CorrectParams,
) -> Result<Vec<SubtitleEntry>, String> {
    let correct_focus = match params.correct_type.as_str() {
        "typo" => "错别字",
        "punctuation" => "标点符号",
        _ => "错别字和标点符号",
    };

    let texts: Vec<&str> = entries.iter().map(|e| e.text.as_str()).collect();
    let prompt = format!(
        "以下是视频字幕文本，请校正其中的{}错误，保持原意不变。每行一个校正结果，不要添加序号或额外说明。\n\n{}",
        correct_focus,
        texts.join("\n")
    );

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", params.base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": params.model,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.1,
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", params.api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("LLM 请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("LLM 返回错误: HTTP {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 LLM 响应失败: {}", e))?;

    let content = json
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    let corrected: Vec<String> = content.lines().map(String::from).collect();

    Ok(entries
        .iter()
        .enumerate()
        .map(|(i, entry)| SubtitleEntry {
            index: entry.index,
            start_ms: entry.start_ms,
            end_ms: entry.end_ms,
            text: corrected.get(i).cloned().unwrap_or_else(|| entry.text.clone()),
        })
        .collect())
}

// ═══════════════════════════════════════════════════════════
//  工具函数
// ═══════════════════════════════════════════════════════════

/// 解析时间戳为毫秒
///
/// sep 参数: SRT 用 ','，VTT 用 '.'
fn parse_timestamp(ts: &str, sep: char) -> u64 {
    let parts: Vec<&str> = ts.split(sep).collect();
    if parts.len() != 2 {
        return 0;
    }
    let hms: Vec<u64> = parts[0].split(':').filter_map(|s| s.trim().parse().ok()).collect();
    let ms: u64 = parts[1].trim().parse().unwrap_or(0);
    if hms.len() == 3 {
        hms[0] * 3600000 + hms[1] * 60000 + hms[2] * 1000 + ms
    } else if hms.len() == 2 {
        hms[0] * 60000 + hms[1] * 1000 + ms
    } else {
        0
    }
}

/// 格式化毫秒为时间戳字符串
///
/// sep 参数: SRT 用 ','，VTT 用 '.'
fn format_timestamp(ms: u64, sep: char) -> String {
    let h = ms / 3600000;
    let m = (ms % 3600000) / 60000;
    let s = (ms % 60000) / 1000;
    let mms = ms % 1000;
    format!("{:02}:{:02}:{:02}{}{:03}", h, m, s, sep, mms)
}

/// 从文件加载字幕（自动检测格式）
pub fn load_subtitle(path: &str) -> Result<Vec<SubtitleEntry>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("srt");

    match SubtitleFormat::from_ext(ext) {
        SubtitleFormat::Srt => Ok(parse_srt(&content)),
        SubtitleFormat::Vtt => Ok(parse_vtt(&content)),
        SubtitleFormat::Ass => Ok(parse_ass(&content)),
    }
}

/// 解析 VTT 格式
pub fn parse_vtt(content: &str) -> Vec<SubtitleEntry> {
    let mut entries = Vec::new();
    let mut index = 1u32;
    let mut in_header = true;

    for block in content.split("\n\n") {
        if in_header {
            if block.starts_with("WEBVTT") {
                in_header = false;
                continue;
            }
        }
        let lines: Vec<&str> = block.lines().collect();
        if lines.is_empty() {
            continue;
        }

        // 找时间行
        let time_idx = lines.iter().position(|l| l.contains("-->"));
        if let Some(ti) = time_idx {
            let time_line = lines[ti];
            let text = if ti + 1 < lines.len() {
                lines[ti + 1..].join("\n").trim().to_string()
            } else {
                String::new()
            };
            if text.is_empty() {
                continue;
            }
            let parts: Vec<&str> = time_line.split(" --> ").collect();
            if parts.len() == 2 {
                entries.push(SubtitleEntry {
                    index,
                    start_ms: parse_timestamp(parts[0].trim(), '.'),
                    end_ms: parse_timestamp(parts[1].trim(), '.'),
                    text,
                });
                index += 1;
            }
        }
    }
    entries
}

/// 解析 ASS 格式（仅提取 Dialogue 行）
pub fn parse_ass(content: &str) -> Vec<SubtitleEntry> {
    let mut entries = Vec::new();
    let mut index = 1u32;
    let mut in_events = false;
    let mut format_fields: Vec<String> = Vec::new();

    for line in content.lines() {
        if line.starts_with("[Events]") {
            in_events = true;
            continue;
        }
        if line.starts_with('[') {
            in_events = false;
            continue;
        }
        if !in_events {
            continue;
        }

        if line.starts_with("Format:") {
            format_fields = line[7..]
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            continue;
        }

        if line.starts_with("Dialogue:") {
            let fields: Vec<&str> = line[9..].splitn(format_fields.len(), ',').collect();
            let start_idx = format_fields.iter().position(|f| f == "Start");
            let end_idx = format_fields.iter().position(|f| f == "End");
            let text_idx = format_fields.iter().position(|f| f == "Text");

            if let (Some(si), Some(ei), Some(ti)) = (start_idx, end_idx, text_idx) {
                if si < fields.len() && ei < fields.len() && ti < fields.len() {
                    let text = fields[ti].trim().replace("\\N", "\n").replace("\\n", "\n");
                    entries.push(SubtitleEntry {
                        index,
                        start_ms: parse_ass_time(fields[si].trim()),
                        end_ms: parse_ass_time(fields[ei].trim()),
                        text,
                    });
                    index += 1;
                }
            }
        }
    }
    entries
}

/// 解析 ASS 时间格式: H:MM:SS.cc
fn parse_ass_time(ts: &str) -> u64 {
    let parts: Vec<&str> = ts.split('.').collect();
    let hms: Vec<u64> = parts[0].split(':').filter_map(|s| s.parse().ok()).collect();
    let cs: u64 = if parts.len() > 1 { parts[1].parse().unwrap_or(0) } else { 0 };
    if hms.len() == 3 {
        hms[0] * 3600000 + hms[1] * 60000 + hms[2] * 1000 + cs * 10
    } else if hms.len() == 2 {
        hms[0] * 60000 + hms[1] * 1000 + cs * 10
    } else {
        0
    }
}

/// 保存字幕到文件（根据扩展名自动选择格式）
pub fn save_subtitle(entries: &[SubtitleEntry], path: &str) -> Result<(), String> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("srt");

    let content = match SubtitleFormat::from_ext(ext) {
        SubtitleFormat::Srt => to_srt(entries),
        SubtitleFormat::Ass => to_ass(entries, None),
        SubtitleFormat::Vtt => to_vtt(entries),
    };

    std::fs::write(path, content).map_err(|e| format!("写入文件失败: {}", e))
}
