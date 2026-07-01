/// LLM（大语言模型）服务模块
///
/// 封装与 LLM 的交互逻辑，提供三个核心功能：
/// 1. generate_script - 根据主题生成短视频脚本
/// 2. generate_terms - 从脚本中提取视频素材搜索关键词
/// 3. generate_social_metadata - 生成社交媒体发布元数据（标题、描述、标签）
///
/// 同时包含 LLM 提供商配置读取和响应解析的辅助函数。

use soma_core::error::SomaError;
use ai_llm_kit::{LlmFactory, LlmProvider};
use crate::Config;

/// 编译正则表达式，失败时 panic 并给出明确错误信息
macro_rules! regex_or_panic {
    ($pat:expr) => {
        regex::Regex::new($pat).expect(concat!("正则编译失败: ", $pat))
    };
}

lazy_static! {
    static ref RE_THINK: regex::Regex = regex_or_panic!(r"(?s)<think>.*?</think>");
    static ref RE_THINKING: regex::Regex = regex_or_panic!(r"(?m)^.{0,5}(思考过程|思维过程|Reasoning|Thinking)[:：]\s*");
    static ref RE_HEADING: regex::Regex = regex_or_panic!(r"(?m)^#{1,6}\s*");
    static ref RE_BOLD: regex::Regex = regex_or_panic!(r"\*\*(.+?)\*\*");
    static ref RE_ITALIC: regex::Regex = regex_or_panic!(r"\*(.+?)\*");
    static ref RE_BLANK: regex::Regex = regex_or_panic!(r"\n{3,}");
    static ref RE_CODE_FENCE_OPEN: regex::Regex = regex_or_panic!(r"(?s)^```[a-zA-Z0-9]*\s*");
    static ref RE_CODE_FENCE_CLOSE: regex::Regex = regex_or_panic!(r"\s*```$");
}

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
    let content = clean_llm_output(&content);
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
    let content = clean_llm_output(&content);
    let terms = parse_terms_output(&content, amount);
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

    let spec = get_social_platform_spec(platform);
    let label = get_social_platform_label(platform);

    let sys_msg = format!(
        "你是一个社交媒体内容优化专家。请根据给定的视频主题和脚本，\
         生成适合发布到{}的标题、描述和标签。以JSON格式输出。\
         \n约束：\n\
         1. title 最多{}字符\n\
         2. description 最多{}字符，末尾加行动号召\n\
         3. tags 为{}个以#开头的标签，无空格\n\
         4. 只输出JSON，不要代码围栏或注释",
        label, spec.title_max, spec.caption_max, spec.hashtag_count,
    );

    let user_msg = format!(
        "视频主题：{}\n视频脚本：\n{}\n目标平台：{}\n\n请输出JSON：{{\"title\":\"标题\",\"description\":\"描述\",\"tags\":[\"#标签1\"]}}",
        subject, script, label
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
    let content = clean_llm_output(&content);

    let json_str = strip_code_fence(&content);

    // 尝试 JSON 解析，正则兜底
    let parsed = if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json_str) {
        Some(v)
    } else {
        // 正则提取 {...} 块
        if let Ok(re) = regex::Regex::new(r"\{.*\}") {
            if let Some(caps) = re.find(&json_str) {
                serde_json::from_str::<serde_json::Value>(caps.as_str()).ok()
            } else {
                None
            }
        } else {
            None
        }
    };

    Ok(parsed.unwrap_or_else(|| fallback_social_metadata(subject, script, &spec)))
}

/// 社交平台规格
struct SocialPlatformSpec {
    title_max: usize,
    caption_max: usize,
    hashtag_count: usize,
}

fn get_social_platform_spec(platform: &str) -> SocialPlatformSpec {
    match platform {
        "tiktok" => SocialPlatformSpec { title_max: 100, caption_max: 2200, hashtag_count: 5 },
        "youtube_shorts" | "youtube" => SocialPlatformSpec { title_max: 100, caption_max: 5000, hashtag_count: 3 },
        "instagram_reels" | "instagram" => SocialPlatformSpec { title_max: 125, caption_max: 2200, hashtag_count: 8 },
        "facebook_reels" | "facebook" => SocialPlatformSpec { title_max: 125, caption_max: 2200, hashtag_count: 5 },
        "x" | "twitter" => SocialPlatformSpec { title_max: 70, caption_max: 280, hashtag_count: 3 },
        _ => SocialPlatformSpec { title_max: 100, caption_max: 2200, hashtag_count: 5 },
    }
}

fn get_social_platform_label(platform: &str) -> &str {
    match platform {
        "tiktok" => "TikTok",
        "youtube_shorts" | "youtube" => "YouTube Shorts",
        "instagram_reels" | "instagram" => "Instagram Reels",
        "facebook_reels" | "facebook" => "Facebook Reels",
        "x" | "twitter" => "X (Twitter)",
        "小红书" | "xiaohongshu" => "小红书",
        _ => platform,
    }
}

/// LLM 失败时的社交元数据兜底生成
fn fallback_social_metadata(subject: &str, script: &str, spec: &SocialPlatformSpec) -> serde_json::Value {
    let default_tags = vec!["#shorts", "#viral", "#trending", "#fyp", "#video", "#reels", "#creator", "#content"];
    let tags: Vec<String> = default_tags.iter().take(spec.hashtag_count).map(|t| t.to_string()).collect();
    serde_json::json!({
        "title": subject.chars().take(spec.title_max).collect::<String>(),
        "description": script.chars().take(spec.caption_max).collect::<String>(),
        "tags": tags,
    })
}

/// 清理 LLM 输出中的思维块和格式标记
///
/// 推理模型（如 DeepSeek-R1、QwQ 等）会在输出中包含思维过程：
/// - `<think>...</think>` 标签
/// - 行首 `思考过程:` / `思维过程:` 等标记
///
/// 同时清除 Markdown 格式标记（# 标题、** 粗体、* 斜体等），
/// 因为视频脚本不应包含这些格式符号。
fn clean_llm_output(text: &str) -> String {
    let mut result = text.to_string();
    result = RE_THINK.replace_all(&result, "").to_string();
    result = RE_THINKING.replace_all(&result, "").to_string();
    result = RE_HEADING.replace_all(&result, "").to_string();
    result = RE_BOLD.replace_all(&result, "$1").to_string();
    result = RE_ITALIC.replace_all(&result, "$1").to_string();
    result = RE_BLANK.replace_all(&result, "\n\n").to_string();
    result.trim().to_string()
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
        "minimax" => {
            let key = conf.app.app.minimax_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.minimax_model_name.as_deref().unwrap_or("abab6.5s-chat");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "mimo" => {
            let key = conf.app.app.mimo_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.mimo_model_name.as_deref().unwrap_or("mimo");
            Ok((LlmProvider::MiMo, key.to_string(), model.to_string()))
        }
        "gemini" => {
            // Gemini 通过 OpenAI 兼容接口调用
            let key = conf.app.app.gemini_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.gemini_model_name.as_deref().unwrap_or("gemini-2.0-flash");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "azure" => {
            let key = conf.app.app.azure_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.azure_model_name.as_deref().unwrap_or("gpt-4o");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "groq" => {
            let key = conf.app.app.groq_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.groq_model_name.as_deref().unwrap_or("llama-3.1-8b-instant");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "grok" => {
            let key = conf.app.app.grok_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.grok_model_name.as_deref().unwrap_or("grok-3");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "doubao" => {
            let key = conf.app.app.doubao_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.doubao_model_name.as_deref().unwrap_or("doubao-pro-32k");
            Ok((LlmProvider::Doubao, key.to_string(), model.to_string()))
        }
        "hunyuan" => {
            let key = conf.app.app.hunyuan_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.hunyuan_model_name.as_deref().unwrap_or("hunyuan-turbo");
            Ok((LlmProvider::Hunyuan, key.to_string(), model.to_string()))
        }
        "zhipu" => {
            let key = conf.app.app.zhipu_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.zhipu_model_name.as_deref().unwrap_or("glm-4-flash");
            Ok((LlmProvider::Zhipu, key.to_string(), model.to_string()))
        }
        "wenxin" => {
            let key = conf.app.app.wenxin_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.wenxin_model_name.as_deref().unwrap_or("ernie-4.0-8k");
            Ok((LlmProvider::Wenxin, key.to_string(), model.to_string()))
        }
        "xunfei" => {
            let key = conf.app.app.xunfei_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.xunfei_model_name.as_deref().unwrap_or("generalv3.5");
            Ok((LlmProvider::Xunfei, key.to_string(), model.to_string()))
        }
        "oneapi" => {
            let key = conf.app.app.oneapi_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.oneapi_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "aihubmix" => {
            let key = conf.app.app.aihubmix_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.aihubmix_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "evolink" => {
            let key = conf.app.app.evolink_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.evolink_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "aiml" | "aimlapi" => {
            let key = conf.app.app.aimlapi_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.aimlapi_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "modelscope" => {
            let key = conf.app.app.modelscope_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.modelscope_model_name.as_deref().unwrap_or("qwen-turbo");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "pollinations" => {
            let key = conf.app.app.pollinations_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.pollinations_model_name.as_deref().unwrap_or("openai");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "g4f" => {
            let key = "";
            let model = conf.app.app.g4f_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "cloudflare" => {
            let key = conf.app.app.openai_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.openai_model_name.as_deref().unwrap_or("@cf/meta/llama-3-8b-instruct");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "litellm" => {
            let key = conf.app.app.oneapi_api_key.as_deref().unwrap_or("");
            let model = conf.app.app.litellm_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
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
    result.to_string()
}

/// 将 serde_json::Value 转换为 tube::Value
///
/// 利用 tube::Value 的 from_serialize 方法进行序列化转换，
/// 转换失败时返回 Null 值
fn serde_json_to_tube_value(json: &serde_json::Value) -> tube::Value {
    tube::Value::from_serialize(json).unwrap_or(tube::Value::Null)
}

/// 解析 LLM 关键词提取输出，支持多种格式
///
/// 三阶段解析策略：
/// 1. 尝试 JSON 数组解析（去除代码围栏后）
/// 2. 正则提取 [...] 块后 JSON 解析
/// 3. 逗号/换行分隔字符串解析（兜底）
fn parse_terms_output(content: &str, amount: usize) -> Vec<String> {
    let stripped = strip_code_fence(content);

    // 阶段1：直接 JSON 解析
    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&stripped) {
        let terms: Vec<String> = arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .filter(|t| !t.is_empty())
            .take(amount)
            .collect();
        if !terms.is_empty() {
            return terms;
        }
    }

    // 阶段2：正则提取 [...] 块
    if let Ok(re) = regex::Regex::new(r"\[.*\]") {
        if let Some(caps) = re.find(&stripped) {
            if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(caps.as_str()) {
                let terms: Vec<String> = arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .filter(|t| !t.is_empty())
                    .take(amount)
                    .collect();
                if !terms.is_empty() {
                    return terms;
                }
            }
        }
    }

    // 阶段3：逗号/换行分隔字符串兜底
    content
        .split(&[',', '\u{FF0C}', '\n'][..])
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .take(amount)
        .collect()
}

/// 去除 LLM 输出中可能包裹的 Markdown 代码围栏
fn strip_code_fence(text: &str) -> String {
    let t = text.trim();
    if t.starts_with("```") {
        let t = RE_CODE_FENCE_OPEN.replace(t, "").to_string();
        RE_CODE_FENCE_CLOSE.replace(&t, "").trim().to_string()
    } else {
        t.to_string()
    }
}
