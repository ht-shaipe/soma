//! 语音名称解析与类型检测模块
//!
//! 提供语音名称的解析、引擎类型判断、参数提取和语速转换等功能。
//! 语音名称格式通常为 `引擎前缀:参数`，例如 `siliconflow:model:voice`、`mimo:voice` 等。

/// 解析语音名称，去除性别后缀
///
/// EdgeTTS 语音名称可能包含 `-Female` 或 `-Male` 后缀表示性别，
/// 此函数去除这些后缀，返回纯语音名称。
///
/// - `name` - 原始语音名称（如 `zh-CN-XiaoxiaoNeural-Female`）
///
/// 返回去除性别后缀的语音名称（如 `zh-CN-XiaoxiaoNeural`）。
pub fn parse_voice_name(name: &str) -> String {
    name.replace("-Female", "")
        .replace("-Male", "")
        .trim()
        .to_string()
}

/// 检测是否为 Azure V2 版本语音，并返回去除 V2 后缀的名称
///
/// Azure 语音名称中 `-V2` 后缀表示改进版语音。
///
/// - `voice_name` - 语音名称
///
/// 如果是 V2 语音，返回 `Some(去除V2后的名称)`；否则返回 `None`。
pub fn is_azure_v2_voice(voice_name: &str) -> Option<String> {
    let parsed = parse_voice_name(voice_name);
    if parsed.ends_with("-V2") {
        Some(parsed.replace("-V2", "").trim().to_string())
    } else {
        None
    }
}

/// 判断是否为 SiliconFlow 语音（以 `siliconflow:` 开头）
pub fn is_siliconflow_voice(voice_name: &str) -> bool {
    voice_name.starts_with("siliconflow:")
}

/// 判断是否为 Gemini 语音（以 `gemini:` 开头）
pub fn is_gemini_voice(voice_name: &str) -> bool {
    voice_name.starts_with("gemini:")
}

/// 判断是否为 MiMo 语音（以 `mimo:` 开头）
pub fn is_mimo_voice(voice_name: &str) -> bool {
    voice_name.starts_with("mimo:")
}

/// 判断是否为 ElevenLabs 语音（以 `elevenlabs:` 开头）
pub fn is_elevenlabs_voice(voice_name: &str) -> bool {
    voice_name.starts_with("elevenlabs:")
}

/// 判断是否为 Azure 语音（以 `azure:` 开头或包含 Neural 后缀且无其他引擎前缀）
pub fn is_azure_voice(voice_name: &str) -> bool {
    voice_name.starts_with("azure:")
}

/// 判断是否为"无语音"模式
///
/// 当语音名称为 `no-voice` 或 `none`（不区分大小写）时，表示不需要实际语音合成，
/// 而是生成对应时长的静音音频。
pub fn is_no_voice(voice_name: &str) -> bool {
    let name = voice_name.trim().to_lowercase();
    name == "no-voice" || name == "none"
}

/// 将语速倍率转换为百分比字符串
///
/// EdgeTTS 命令行工具使用百分比格式表示语速偏移。
/// 例如：rate=1.5 → "+50%"，rate=0.8 → "-20%"。
///
/// - `rate` - 语速倍率（1.0 为正常语速）
///
/// 返回百分比格式字符串（如 `+50%` 或 `-20%`）。
pub fn convert_rate_to_percent(rate: f32) -> String {
    let percent = ((rate - 1.0) * 100.0).round() as i32;
    if percent >= 0 {
        format!("+{}%", percent)
    } else {
        format!("{}%", percent)
    }
}

/// 从带前缀的语音名称中提取语音标识（通用辅助函数）
///
/// 语音名称格式为 `prefix:voice-with-gender`，提取 `:` 后的语音标识，
/// 并去除性别部分（`-` 之前的部分）。
///
/// - `voice_name` - 带前缀的语音名称
/// - `prefix` - 引擎前缀（如 `gemini`、`mimo`）
///
/// 如果前缀匹配，返回提取的语音标识；否则返回 `None`。
fn extract_voice_suffix(voice_name: &str, prefix: &str) -> Option<String> {
    let parts: Vec<&str> = voice_name.splitn(3, ':').collect();
    if parts.len() >= 2 && parts[0] == prefix {
        let voice_with_gender = parts[1];
        // 取 '-' 前的部分作为纯语音标识
        let voice = voice_with_gender.split('-').next().unwrap_or(voice_with_gender);
        Some(voice.to_string())
    } else {
        None
    }
}

/// 从 SiliconFlow 语音名称中提取模型和语音标识
///
/// SiliconFlow 语音名称格式为 `siliconflow:model:voice-with-gender`，
/// 例如 `siliconflow:FunAudioLLM/CosyVoice2-0.5B:alice-Female`。
/// 对于语音克隆的 URI，不会包含性别后缀，因此不做截断。
///
/// - `voice_name` - SiliconFlow 格式的语音名称
///
/// 返回 `(模型名称, "模型:语音标识")` 的元组，用于 API 调用。
pub fn extract_siliconflow_voice(voice_name: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = voice_name.splitn(3, ':').collect();
    if parts.len() >= 3 && parts[0] == "siliconflow" {
        let model = parts[1].to_string();
        let voice_with_gender = parts[2];
        // 只去除 -Female / -Male 性别后缀，避免截断克隆声音的 URI
        let voice = voice_with_gender
            .strip_suffix("-Female")
            .or_else(|| voice_with_gender.strip_suffix("-Male"))
            .unwrap_or(voice_with_gender);
        Some((model.clone(), format!("{}:{}", model, voice)))
    } else {
        None
    }
}

/// 从 Gemini 语音名称中提取语音标识
///
/// 格式：`gemini:voice-name`
pub fn extract_gemini_voice(voice_name: &str) -> Option<String> {
    extract_voice_suffix(voice_name, "gemini")
}

/// 从 MiMo 语音名称中提取语音标识
///
/// 格式：`mimo:voice-name`
pub fn extract_mimo_voice(voice_name: &str) -> Option<String> {
    extract_voice_suffix(voice_name, "mimo")
}

/// 从 ElevenLabs 语音名称中提取语音 ID
///
/// 格式：`elevenlabs:voice-id`
/// 与其他引擎不同，ElevenLabs 不需要去除性别后缀。
pub fn extract_elevenlabs_voice(voice_name: &str) -> Option<String> {
    let parts: Vec<&str> = voice_name.splitn(3, ':').collect();
    if parts.len() >= 2 && parts[0] == "elevenlabs" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// 从 Azure 语音名称中提取语音标识
///
/// 格式：`azure:voice-name`（如 `azure:zh-CN-XiaoxiaoNeural`）
pub fn extract_azure_voice(voice_name: &str) -> Option<String> {
    let parts: Vec<&str> = voice_name.splitn(2, ':').collect();
    if parts.len() >= 2 && parts[0] == "azure" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// 判断是否为火山引擎语音（以 `volcengine:` 开头）
pub fn is_volcengine_voice(voice_name: &str) -> bool {
    voice_name.starts_with("volcengine:")
}

/// 判断是否为科大讯飞语音（以 `xfyun:` 开头）
pub fn is_xfyun_voice(voice_name: &str) -> bool {
    voice_name.starts_with("xfyun:")
}

/// 从火山引擎语音名称中提取 voice_id
///
/// 格式：`volcengine:voice-id`（如 `volcengine:BV700_streaming`）
pub fn extract_volcengine_voice(voice_name: &str) -> Option<String> {
    let parts: Vec<&str> = voice_name.splitn(2, ':').collect();
    if parts.len() >= 2 && parts[0] == "volcengine" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// 从科大讯飞语音名称中提取 vcn（声音名称）
///
/// 格式：`xfyun:vcn`（如 `xfyun:xiaoyan`）
pub fn extract_xfyun_voice(voice_name: &str) -> Option<String> {
    let parts: Vec<&str> = voice_name.splitn(2, ':').collect();
    if parts.len() >= 2 && parts[0] == "xfyun" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

/// 估算无语音模式下的音频时长
///
/// 根据文本中的中文字符数、英文单词数和句子数估算朗读时长。
/// 估算公式：max(3.0, 中文字数/4.2 + 英文词数/2.7 + 句间停顿时长)
///
/// - 中文字符按约 4.2 字/秒计算
/// - 英文单词按约 2.7 词/秒计算
/// - 句间停顿约 0.35 秒
/// - 最短时长为 3 秒
///
/// - `text` - 待估算的文本
///
/// 返回估算的音频时长（秒）。
pub fn estimate_no_voice_duration(text: &str) -> f64 {
    // 空文本默认 3 秒
    if text.trim().is_empty() {
        return 3.0;
    }
    // 统计中文字符数（Unicode 范围 U+4E00 ~ U+9FFF）
    let cjk_count = text.chars().filter(|c| ('\u{4e00}'..='\u{9fff}').contains(c)).count() as f64;
    // 统计英文单词数（按空格分割）
    let word_count = text.split_whitespace().count() as f64;
    // 句间停顿时长：句子数 - 1 个停顿，每个 0.35 秒
    let sentence_count = soma_core::utils::split_string_by_punctuations(text).len().max(1) as f64;
    let pause = (sentence_count - 1.0).max(0.0) * 0.35;
    // 取最短 3 秒
    (3.0_f64).max(cjk_count / 4.2 + word_count / 2.7 + pause)
}
