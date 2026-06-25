pub fn parse_voice_name(name: &str) -> String {
    name.replace("-Female", "")
        .replace("-Male", "")
        .trim()
        .to_string()
}

pub fn is_azure_v2_voice(voice_name: &str) -> Option<String> {
    let parsed = parse_voice_name(voice_name);
    if parsed.ends_with("-V2") {
        Some(parsed.replace("-V2", "").trim().to_string())
    } else {
        None
    }
}

pub fn is_siliconflow_voice(voice_name: &str) -> bool {
    voice_name.starts_with("siliconflow:")
}

pub fn is_gemini_voice(voice_name: &str) -> bool {
    voice_name.starts_with("gemini:")
}

pub fn is_mimo_voice(voice_name: &str) -> bool {
    voice_name.starts_with("mimo:")
}

pub fn is_elevenlabs_voice(voice_name: &str) -> bool {
    voice_name.starts_with("elevenlabs:")
}

pub fn is_no_voice(voice_name: &str) -> bool {
    let name = voice_name.trim().to_lowercase();
    name == "no-voice" || name == "none"
}

pub fn convert_rate_to_percent(rate: f32) -> String {
    let percent = ((rate - 1.0) * 100.0).round() as i32;
    if percent >= 0 {
        format!("+{}%", percent)
    } else {
        format!("{}%", percent)
    }
}

fn extract_voice_suffix(voice_name: &str, prefix: &str) -> Option<String> {
    let parts: Vec<&str> = voice_name.splitn(3, ':').collect();
    if parts.len() >= 2 && parts[0] == prefix {
        let voice_with_gender = parts[1];
        let voice = voice_with_gender.split('-').next().unwrap_or(voice_with_gender);
        Some(voice.to_string())
    } else {
        None
    }
}

pub fn extract_siliconflow_voice(voice_name: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = voice_name.splitn(3, ':').collect();
    if parts.len() >= 3 && parts[0] == "siliconflow" {
        let model = parts[1].to_string();
        let voice_with_gender = parts[2];
        let voice = voice_with_gender.split('-').next().unwrap_or(voice_with_gender);
        Some((model.clone(), format!("{}:{}", model, voice)))
    } else {
        None
    }
}

pub fn extract_gemini_voice(voice_name: &str) -> Option<String> {
    extract_voice_suffix(voice_name, "gemini")
}

pub fn extract_mimo_voice(voice_name: &str) -> Option<String> {
    extract_voice_suffix(voice_name, "mimo")
}

pub fn extract_elevenlabs_voice(voice_name: &str) -> Option<String> {
    let parts: Vec<&str> = voice_name.splitn(3, ':').collect();
    if parts.len() >= 2 && parts[0] == "elevenlabs" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

pub fn estimate_no_voice_duration(text: &str) -> f64 {
    if text.trim().is_empty() {
        return 3.0;
    }
    let cjk_count = text.chars().filter(|c| ('\u{4e00}'..='\u{9fff}').contains(c)).count() as f64;
    let word_count = text.split_whitespace().count() as f64;
    let sentence_count = soma_core::utils::split_string_by_punctuations(text).len().max(1) as f64;
    let pause = (sentence_count - 1.0).max(0.0) * 0.35;
    (3.0_f64).max(cjk_count / 4.2 + word_count / 2.7 + pause)
}
