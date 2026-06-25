use soma_core::error::SomaError;
use ai_llm_kit::{LlmFactory, LlmProvider, LlmService};use crate::Config;

pub async fn generate_script(
    provider: &str,
    subject: &str,
    language: &str,
    paragraph_number: u32,
    prompt: &str,
    system_prompt: &str,
    conf: &Config,
) -> Result<String, SomaError> {
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = LlmFactory::create(llm_provider, &api_key);

    let sys_msg = if system_prompt.is_empty() {
        format!(
            "你是一个短视频脚本撰写专家。请根据给定的主题，撰写一段适合短视频的脚本。\
             要求：\n\
             1. 语言为{}\n\
             2. 段落数为{}段\n\
             3. 内容要吸引人，有节奏感\n\
             4. 不要包含任何格式标记、标题、序号等\n\
             5. 只输出脚本内容本身",
            if language.is_empty() { "中文" } else { language },
            paragraph_number,
        )
    } else {
        system_prompt.to_string()
    };

    let user_msg = if prompt.is_empty() {
        format!("请为以下主题撰写短视频脚本：{}", subject)
    } else {
        format!("{}\n主题：{}", prompt, subject)
    };

    let body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": sys_msg},
            {"role": "user", "content": user_msg}
        ],
        "temperature": 0.7,
        "max_tokens": 2048,
    });

    let body_value = serde_json_to_tube_value(&body);
    let result = llm.chat(&body_value).await
        .map_err(|e| SomaError::Llm(format!("LLM chat failed: {:?}", e)))?;

    let content = extract_content_from_response(&result);
    Ok(content)
}

pub async fn generate_terms(
    provider: &str,
    subject: &str,
    script: &str,
    amount: usize,
    conf: &Config,
) -> Result<Vec<String>, SomaError> {
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = LlmFactory::create(llm_provider, &api_key);

    let sys_msg = format!(
        "你是一个视频素材搜索关键词提取专家。请从给定的视频脚本中提取{}个最适合搜索视频素材的关键词。\
         要求：\n\
         1. 每个关键词应该是英文的，适合在Pexels、Pixabay等素材网站搜索\n\
         2. 关键词要具体、有视觉画面感\n\
         3. 只输出关键词，用逗号分隔\n\
         4. 不要包含任何解释或编号",
        amount,
    );

    let user_msg = format!("视频主题：{}\n\n视频脚本：\n{}", subject, script);

    let body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": sys_msg},
            {"role": "user", "content": user_msg}
        ],
        "temperature": 0.3,
        "max_tokens": 256,
    });

    let body_value = serde_json_to_tube_value(&body);
    let result = llm.chat(&body_value).await
        .map_err(|e| SomaError::Llm(format!("LLM terms failed: {:?}", e)))?;

    let content = extract_content_from_response(&result);
    let terms: Vec<String> = content
        .split(&[',', '，', '\n'][..])
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .take(amount)
        .collect();
    Ok(terms)
}

pub async fn generate_social_metadata(
    provider: &str,
    subject: &str,
    script: &str,
    platform: &str,
    conf: &Config,
) -> Result<serde_json::Value, SomaError> {
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = LlmFactory::create(llm_provider, &api_key);

    let sys_msg = "你是一个社交媒体内容优化专家。请根据给定的视频主题和脚本，生成适合发布到社交媒体的标题、描述和标签。以JSON格式输出。";

    let user_msg = format!(
        "视频主题：{}\n视频脚本：\n{}\n目标平台：{}\n\n请输出JSON格式：{{\"title\": \"标题\", \"description\": \"描述\", \"tags\": [\"标签1\", \"标签2\"]}}",
        subject, script, platform
    );

    let body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": sys_msg},
            {"role": "user", "content": user_msg}
        ],
        "temperature": 0.5,
        "max_tokens": 512,
    });

    let body_value = serde_json_to_tube_value(&body);
    let result = llm.chat(&body_value).await
        .map_err(|e| SomaError::Llm(format!("LLM social metadata failed: {:?}", e)))?;

    let content = extract_content_from_response(&result);

    let json_str = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str(json_str).unwrap_or(serde_json::json!({
        "title": subject,
        "description": content,
        "tags": []
    }))
}

fn get_provider_config(provider: &str, conf: &Config) -> Result<(LlmProvider, String, String), SomaError> {
    match provider {
        "openai" => {
            let key = conf.app.app.openai_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.openai_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "deepseek" => {
            let key = conf.app.app.deepseek_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.deepseek_model_name.as_deref().unwrap_or("deepseek-chat");
            Ok((LlmProvider::DeepSeek, key.to_string(), model.to_string()))
        }
        "qwen" => {
            let key = conf.app.app.qwen_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.qwen_model_name.as_deref().unwrap_or("qwen-plus");
            Ok((LlmProvider::QWen, key.to_string(), model.to_string()))
        }
        "moonshot" | "kimi" => {
            let key = conf.app.app.moonshot_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.moonshot_model_name.as_deref().unwrap_or("moonshot-v1-8k");
            Ok((LlmProvider::Kimi, key.to_string(), model.to_string()))
        }
        "ollama" => {
            let key = "";
            let model = conf.app.app.ollama_model_name.as_deref().unwrap_or("llama3");
            Ok((LlmProvider::Ollama, key.to_string(), model.to_string()))
        }
        "mimo" => {
            let key = conf.app.app.mimo_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.mimo_model_name.as_deref().unwrap_or("mimo");
            Ok((LlmProvider::MiMo, key.to_string(), model.to_string()))
        }
        _ => {
            let key = conf.app.app.openai_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.openai_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
    }
}

fn extract_content_from_response(result: &tube::Value) -> String {
    if let Some(choices) = result.get("choices") {
        if let Some(arr) = choices.as_array() {
            if let Some(first) = arr.first() {
                if let Some(msg) = first.get("message") {
                    if let Some(content) = msg.get("content") {
                        return content.to_string();
                    }
                }
            }
        }
    }
    result.to_string()
}

fn serde_json_to_tube_value(json: &serde_json::Value) -> tube::Value {
    tube::Value::from_serialize(json).unwrap_or(tube::Value::Null)
}
