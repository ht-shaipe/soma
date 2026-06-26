/// LLM（大语言模型）服务模块
///
/// 封装与 LLM 的交互逻辑，提供三个核心功能：
/// 1. generate_script - 根据主题生成短视频脚本
/// 2. generate_terms - 从脚本中提取视频素材搜索关键词
/// 3. generate_social_metadata - 生成社交媒体发布元数据（标题、描述、标签）
///
/// 同时包含 LLM 提供商配置读取和响应解析的辅助函数。

use soma_core::error::SomaError;
use ai_llm_kit::{LlmFactory, LlmProvider, LlmService};
use crate::Config;

/// 根据主题生成短视频脚本
///
/// 调用 LLM API，根据给定的主题、语言、段落数等参数生成适合短视频的脚本内容。
/// 支持自定义 system_prompt 和 user_prompt，也提供默认的提示词模板。
///
/// 参数：
/// - `provider`: LLM 提供商名称（如 "openai"、"deepseek"、"qwen" 等）
/// - `subject`: 视频主题
/// - `language`: 脚本语言（为空则默认中文）
/// - `paragraph_number`: 段落数量
/// - `prompt`: 自定义用户提示词（为空使用默认模板）
/// - `system_prompt`: 自定义系统提示词（为空使用默认模板）
/// - `conf`: 全局配置引用
///
/// 返回：生成的脚本文本内容，或 LLM 调用错误
pub async fn generate_script(
    provider: &str,
    subject: &str,
    language: &str,
    paragraph_number: u32,
    prompt: &str,
    system_prompt: &str,
    conf: &Config,
) -> Result<String, SomaError> {
    // 获取 LLM 提供商配置（提供商类型、API Key、模型名称）
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = LlmFactory::create(llm_provider, &api_key);

    // 构建 system 消息：若未提供自定义提示词，使用默认的脚本撰写专家提示
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

    // 构建 user 消息：若未提供自定义 prompt，使用默认模板
    let user_msg = if prompt.is_empty() {
        format!("请为以下主题撰写短视频脚本：{}", subject)
    } else {
        format!("{}\n主题：{}", prompt, subject)
    };

    // 构造 LLM 请求体
    let body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": sys_msg},
            {"role": "user", "content": user_msg}
        ],
        "temperature": 0.7,  // 较高温度保证创意性
        "max_tokens": 2048,
    });

    let body_value = serde_json_to_tube_value(&body);
    let result = llm.chat(&body_value).await
        .map_err(|e| SomaError::Llm(format!("LLM chat failed: {:?}", e)))?;

    // 从响应中提取文本内容
    let content = extract_content_from_response(&result);
    Ok(content)
}

/// 从视频脚本中提取素材搜索关键词
///
/// 调用 LLM 分析脚本内容，提取适合在素材网站（Pexels、Pixabay 等）搜索视频素材的英文关键词。
/// 使用较低温度（0.3）以保证关键词的准确性和一致性。
///
/// 参数：
/// - `provider`: LLM 提供商名称
/// - `subject`: 视频主题
/// - `script`: 视频脚本内容
/// - `amount`: 需要提取的关键词数量
/// - `conf`: 全局配置引用
///
/// 返回：关键词列表（英文），或 LLM 调用错误
pub async fn generate_terms(
    provider: &str,
    subject: &str,
    script: &str,
    amount: usize,
    conf: &Config,
) -> Result<Vec<String>, SomaError> {
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = LlmFactory::create(llm_provider, &api_key);

    // 构建关键词提取的 system 提示
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
        "temperature": 0.3,  // 低温度保证关键词提取的准确性
        "max_tokens": 256,
    });

    let body_value = serde_json_to_tube_value(&body);
    let result = llm.chat(&body_value).await
        .map_err(|e| SomaError::Llm(format!("LLM terms failed: {:?}", e)))?;

    let content = extract_content_from_response(&result);
    // 解析 LLM 返回的逗号/换行分隔的关键词
    let terms: Vec<String> = content
        .split(&[',', '，', '\n'][..])
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .take(amount)
        .collect();
    Ok(terms)
}

/// 生成社交媒体发布元数据
///
/// 根据视频主题和脚本，生成适合指定社交媒体平台发布的标题、描述和标签。
/// LLM 以 JSON 格式输出，若解析失败则降级为简单结构。
///
/// 参数：
/// - `provider`: LLM 提供商名称
/// - `subject`: 视频主题
/// - `script`: 视频脚本内容
/// - `platform`: 目标社交平台（如 "tiktok"、"youtube" 等）
/// - `conf`: 全局配置引用
///
/// 返回：包含 title、description、tags 的 JSON Value，或 LLM 调用错误
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
        "temperature": 0.5,  // 中等温度平衡创意与准确性
        "max_tokens": 512,
    });

    let body_value = serde_json_to_tube_value(&body);
    let result = llm.chat(&body_value).await
        .map_err(|e| SomaError::Llm(format!("LLM social metadata failed: {:?}", e)))?;

    let content = extract_content_from_response(&result);

    // 去除 LLM 可能返回的 markdown 代码块标记
    let json_str = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // 尝试解析 JSON，失败则降级为简单结构
    Ok(serde_json::from_str::<serde_json::Value>(json_str).unwrap_or_else(|_| serde_json::json!({
        "title": subject,
        "description": content,
        "tags": []
    })))
}

/// 获取 LLM 提供商配置
///
/// 根据提供商名称，从全局配置中读取对应的 API Key 和模型名称，
/// 返回 LlmProvider 枚举、API Key 和模型名称三元组。
/// 不识别的提供商名称默认使用 OpenAI 配置。
///
/// 参数：
/// - `provider`: 提供商名称字符串
/// - `conf`: 全局配置引用
///
/// 返回：(LlmProvider, api_key, model_name) 三元组，或配置错误
fn get_provider_config(provider: &str, conf: &Config) -> Result<(LlmProvider, String, String), SomaError> {
    match provider {
        // OpenAI 配置，默认模型 gpt-4o-mini
        "openai" => {
            let key = conf.app.app.openai_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.openai_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        // DeepSeek 配置，默认模型 deepseek-chat
        "deepseek" => {
            let key = conf.app.app.deepseek_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.deepseek_model_name.as_deref().unwrap_or("deepseek-chat");
            Ok((LlmProvider::DeepSeek, key.to_string(), model.to_string()))
        }
        // 通义千问配置，默认模型 qwen-plus
        "qwen" => {
            let key = conf.app.app.qwen_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.qwen_model_name.as_deref().unwrap_or("qwen-plus");
            Ok((LlmProvider::QWen, key.to_string(), model.to_string()))
        }
        // Moonshot/Kimi 配置，默认模型 moonshot-v1-8k
        "moonshot" | "kimi" => {
            let key = conf.app.app.moonshot_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.moonshot_model_name.as_deref().unwrap_or("moonshot-v1-8k");
            Ok((LlmProvider::Kimi, key.to_string(), model.to_string()))
        }
        // Ollama 本地模型配置，无需 API Key，默认模型 llama3
        "ollama" => {
            let key = "";
            let model = conf.app.app.ollama_model_name.as_deref().unwrap_or("llama3");
            Ok((LlmProvider::Ollama, key.to_string(), model.to_string()))
        }
        // MiMo 配置
        "mimo" => {
            let key = conf.app.app.mimo_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.mimo_model_name.as_deref().unwrap_or("mimo");
            Ok((LlmProvider::MiMo, key.to_string(), model.to_string()))
        }
        // 未识别的提供商，默认使用 OpenAI 配置
        _ => {
            let key = conf.app.app.openai_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.openai_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
    }
}

/// 从 LLM 响应中提取文本内容
///
/// 解析标准的 OpenAI 格式响应，从 choices[0].message.content 路径提取内容。
/// 若解析失败则直接返回整个响应的字符串形式。
///
/// 参数：
/// - `result`: LLM 响应的 tube::Value
///
/// 返回：提取的文本内容
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
    // 响应格式不符预期时，返回原始内容
    result.to_string()
}

/// 将 serde_json::Value 转换为 tube::Value
///
/// 利用 tube::Value 的 from_serialize 方法进行序列化转换，
/// 转换失败时返回 Null 值
fn serde_json_to_tube_value(json: &serde_json::Value) -> tube::Value {
    tube::Value::from_serialize(json).unwrap_or(tube::Value::Null)
}
