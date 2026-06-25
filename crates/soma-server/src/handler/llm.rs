use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use crate::Config;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "generate" => generate_script(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

pub async fn distribute_terms(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "generate" | "extract" => generate_terms(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

pub async fn distribute_social(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "generate" => generate_social(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

async fn generate_script(param: &RequestParameter) -> Result<Value> {
    let conf = Config::get();
    let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");

    let subject = param.value.get_def_string("videoSubject", "");
    let language = param.value.get_def_string("videoLanguage", "");
    let paragraph_number = param.value.get_i32("paragraphNumber", 1) as u32;
    let prompt = param.value.get_def_string("videoScriptPrompt", "");
    let system_prompt = param.value.get_def_string("customSystemPrompt", "");
    let req_provider = param.value.get_def_string("provider", provider);

    let script = crate::service::llm::generate_script(
        &req_provider, &subject, &language, paragraph_number, &prompt, &system_prompt, &conf,
    ).await.map_err(|e| error!("脚本生成失败: {:?}", e))?;

    Ok(value!({
        "script": script,
    }))
}

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

    let title = metadata.get("title").and_then(|v| v.as_str()).unwrap_or(&subject);
    let desc = metadata.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let tags = metadata.get("tags").and_then(|v| v.as_array())
        .unwrap_or(&[])
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect::<Vec<String>>();

    Ok(value!({
        "title": title,
        "description": desc,
        "tags": tags.iter().map(|t| value!(t.clone())).collect::<Vec<Value>>(),
    }))
}
