/// LLM API 处理器
///
/// 处理与大语言模型相关的四类 API 请求：
/// - scripts: 脚本生成（根据主题生成短视频脚本）
/// - terms: 关键词提取（从脚本中提取素材搜索关键词）
/// - social: 社交元数据生成（生成社交媒体标题、描述、标签）
/// - intent: 需求理解（解析用户意图为结构化参数）

use tube::{Result, Value};
use tube_web::RequestParameter;
use crate::Config;

/// 脚本模块请求分发
///
/// 仅支持 "generate" 方法，调用 LLM 生成短视频脚本
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "generate" => generate_script(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 关键词模块请求分发
///
/// 支持 "generate" 和 "extract" 方法，调用 LLM 从脚本中提取搜索关键词
pub async fn distribute_terms(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "generate" | "extract" => generate_terms(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 社交元数据模块请求分发
///
/// 仅支持 "generate" 方法，调用 LLM 生成社交媒体发布内容
pub async fn distribute_social(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "generate" => generate_social(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 需求理解模块请求分发
///
/// 仅支持 "generate" 方法，调用 LLM 解析用户意图为结构化创作参数
pub async fn distribute_intent(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "generate" => generate_intent(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 分镜脚本模块请求分发
///
/// 仅支持 "generate" 方法，调用 LLM 将脚本文案拆分为分镜场景列表
pub async fn distribute_storyboard(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "generate" => generate_storyboard_api(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 生成短视频脚本
///
/// 从请求参数中提取主题、语言、段落数等信息，调用 LLM 服务生成脚本。
/// 支持通过 provider 参数覆盖默认 LLM 提供商。
///
/// 返回：包含 script 字段的 JSON 对象
async fn generate_script(param: &RequestParameter) -> Result<Value> {
    let conf = Config::get();
    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");

    // 从请求参数中提取脚本生成所需字段
    let subject = param.value.get_def_string("videoSubject", "");
    let language = param.value.get_def_string("videoLanguage", "");
    let paragraph_number = param.value.get_i32("paragraphNumber", 1) as u32;
    let prompt = param.value.get_def_string("videoScriptPrompt", "");
    let system_prompt = param.value.get_def_string("customSystemPrompt", "");
    // 允许请求级别覆盖默认提供商
    let req_provider = param.value.get_def_string("provider", provider);

    let intent_style = param.value.get_def_string("intentStyle", "");
    let intent_mood = param.value.get_def_string("intentMood", "");
    let intent_audience = param.value.get_def_string("intentAudience", "");

    let intent = serde_json::json!({
        "theme": subject,
        "style": intent_style,
        "mood": intent_mood,
        "audience": intent_audience,
        "duration": "",
        "aspect_ratio": "9:16",
        "language": if language.is_empty() { "中文" } else { &language },
        "platform": ""
    });

    let script = crate::service::llm::generate_script(
        &req_provider, &subject, &intent, &language, paragraph_number, &prompt, &system_prompt, &conf,
    ).await.map_err(|e| error!("脚本生成失败: {:?}", e))?;

    let (script_text, terms_text) = if let Some(pos) = script.find("===KEYWORDS===") {
        let (s, k) = script.split_at(pos);
        (s.trim().to_string(), k.trim_start_matches('=').trim_start_matches("KEYWORDS").trim().to_string())
    } else if let Some(pos) = script.find("===关键词===") {
        let (s, k) = script.split_at(pos);
        (s.trim().to_string(), k.trim_start_matches('=').trim_start_matches("关键词").trim().to_string())
    } else {
        (script, String::new())
    };

    let terms: Vec<String> = if terms_text.is_empty() {
        vec![]
    } else {
        terms_text.split(&[',', '\u{FF0C}', '\n'][..])
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .take(5)
            .collect()
    };

    Ok(value!({
        "script": script_text,
        "terms": terms.iter().map(|t| value!(t.clone())).collect::<Vec<Value>>(),
    }))
}

/// 提取素材搜索关键词
///
/// 从请求参数中提取主题、脚本内容、关键词数量，调用 LLM 提取英文搜索关键词。
///
/// 返回：包含 terms 数组的 JSON 对象
async fn generate_terms(param: &RequestParameter) -> Result<Value> {
    let conf = Config::get();
    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");

    let subject = param.value.get_def_string("videoSubject", "");
    let script = param.value.get_def_string("videoScript", "");
    let amount = param.value.get_i32("amount", 5) as usize;
    let req_provider = param.value.get_def_string("provider", provider);

    let terms = crate::service::llm::generate_terms(
        &req_provider, &subject, &script, amount, &conf,
    ).await.map_err(|e| error!("关键词提取失败: {:?}", e))?;

    Ok(value!({
        "terms": terms.iter().map(|t| value!(t.clone())).collect::<Vec<Value>>(),
    }))
}

/// 生成社交媒体发布元数据
///
/// 根据视频主题和脚本，调用 LLM 生成适合指定平台的标题、描述和标签。
/// 从 LLM 返回的 JSON 中提取各字段，解析失败时使用主题作为默认标题。
///
/// 返回：包含 title、description、tags 的 JSON 对象
async fn generate_social(param: &RequestParameter) -> Result<Value> {
    let conf = Config::get();
    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");

    let subject = param.value.get_def_string("videoSubject", "");
    let script = param.value.get_def_string("videoScript", "");
    let platform = param.value.get_def_string("platform", "tiktok");
    let req_provider = param.value.get_def_string("provider", provider);

    let metadata = crate::service::llm::generate_social_metadata(
        &req_provider, &subject, &script, &platform, &conf,
    ).await.map_err(|e| error!("社交元数据生成失败: {:?}", e))?;

    // 从 LLM 返回的 JSON 中安全提取各字段
    let title = metadata.get("title").and_then(|v| v.as_str()).unwrap_or(&subject);
    let desc = metadata.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let tags: Vec<serde_json::Value> = metadata.get("tags")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // 将 JSON 标签转为字符串列表
    let tag_strings: Vec<String> = tags.iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    Ok(value!({
        "title": title,
        "description": desc,
        "tags": tag_strings.iter().map(|t| value!(t.clone())).collect::<Vec<Value>>(),
    }))
}

async fn generate_intent(param: &RequestParameter) -> Result<Value> {
    let conf = Config::get();
    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");
    let req_provider = param.value.get_def_string("provider", provider);

    let subject = param.value.get_def_string("videoSubject", "");
    let language = param.value.get_def_string("videoLanguage", "");
    let aspect_ratio = param.value.get_def_string("videoAspect", "9:16");

    if subject.is_empty() {
        return Err(error!("videoSubject 不能为空"));
    }

    let intent = crate::service::llm::generate_intent(
        &req_provider, &subject, &language, &aspect_ratio, &conf,
    ).await.map_err(|e| error!("意图解析失败: {:?}", e))?;

    Ok(tube::Value::from_serialize(&intent).unwrap_or(tube::Value::Null))
}

async fn generate_storyboard_api(param: &RequestParameter) -> Result<Value> {
    let conf = Config::get();
    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");
    let req_provider = param.value.get_def_string("provider", provider);

    let subject = param.value.get_def_string("videoSubject", "");
    let script = param.value.get_def_string("videoScript", "");
    let clip_duration = param.value.get_i32("clipDuration", 3) as u32;

    let intent_style = param.value.get_def_string("intentStyle", "");
    let intent_mood = param.value.get_def_string("intentMood", "");

    let intent = serde_json::json!({
        "theme": subject,
        "style": intent_style,
        "mood": intent_mood,
    });

    if script.is_empty() {
        return Err(error!("videoScript 不能为空"));
    }

    let scenes = crate::service::llm::generate_storyboard(
        &req_provider, &subject, &script, clip_duration, &intent, &conf,
    ).await.map_err(|e| error!("分镜脚本生成失败: {:?}", e))?;

    Ok(tube::Value::from_serialize(&scenes).unwrap_or(tube::Value::Null))
}
