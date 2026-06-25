use soma_core::error::SomaError;
use soma_core::models::SubtitleCue;
use soma_core::utils;

pub fn cues_to_srt(cues: &[SubtitleCue]) -> String {
    let mut srt = String::new();
    for cue in cues {
        let start = utils::time_convert_seconds_to_hmsm(cue.start_ms as f64 / 1000.0);
        let end = utils::time_convert_seconds_to_hmsm(cue.end_ms as f64 / 1000.0);
        srt.push_str(&format!("{}\n{} --> {}\n{}\n\n", cue.index, start, end, cue.text));
    }
    srt
}

pub fn create_subtitle_file(cues: &[SubtitleCue], output_path: &str) -> Result<(), SomaError> {
    let content = cues_to_srt(cues);
    std::fs::write(output_path, content).map_err(SomaError::Io)
}

pub fn file_to_subtitles(srt_path: &str) -> Vec<SubtitleCue> {
    let content = match std::fs::read_to_string(srt_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    parse_srt(&content)
}

pub fn parse_srt(content: &str) -> Vec<SubtitleCue> {
    let mut cues = Vec::new();
    let blocks: Vec<&str> = content.split("\n\n").collect();
    let mut index = 1u32;

    for block in blocks {
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

fn parse_srt_timestamp(ts: &str) -> u64 {
    let parts: Vec<&str> = ts.split(',').collect();
    if parts.len() != 2 {
        return 0;
    }
    let hms: Vec<u64> = parts[0].split(':').filter_map(|s| s.parse().ok()).collect();
    let ms: u64 = parts[1].parse().unwrap_or(0);
    if hms.len() == 3 {
        (hms[0] * 3600000) + (hms[1] * 60000) + (hms[2] * 1000) + ms
    } else {
        0
    }
}

pub fn correct_subtitle(cues: &mut [SubtitleCue], video_script: &str) {
    let script_lines = utils::split_string_by_punctuations(video_script);
    if script_lines.is_empty() || cues.is_empty() {
        return;
    }

    for (i, cue) in cues.iter_mut().enumerate() {
        if i < script_lines.len() {
            cue.text = script_lines[i].clone();
        }
    }
}

pub fn generate_whisper_subtitle(audio_file: &str, subtitle_file: &str) -> Result<(), SomaError> {
    let status = std::process::Command::new("whisper")
        .args(&[
            audio_file,
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
