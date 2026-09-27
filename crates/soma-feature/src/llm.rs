use ai_llm_kit::{LlmFactory, LlmProvider, LlmService, OpenAICompatible};
use soma_core::config::AppConfig;
/// LLM（大语言模型）服务模块（功能点层共享实现）
///
/// 从 soma-server 迁入，供 `llm.*` 系列功能点与宿主兼容层共同复用：
/// 1. generate_intent - 需求理解（提炼结构化创作参数）
/// 2. generate_script - 根据主题生成短视频脚本
/// 3. generate_storyboard - 分镜脚本 + 视觉提示词
/// 4. generate_narration - 旁白文案（口语化改写）
/// 5. generate_terms - 从脚本中提取视频素材搜索关键词
/// 6. generate_social_metadata - 生成社交媒体发布元数据（标题、描述、标签）
///
/// 配置参数为 `AppConfig`（宿主各自的配置管理负责装载）。
use soma_core::error::SomaError;
use soma_core::models::StoryboardScene;

/// 编译正则表达式，失败时 panic 并给出明确错误信息
macro_rules! regex_or_panic {
    ($pat:expr) => {
        regex::Regex::new($pat).expect(concat!("正则编译失败: ", $pat))
    };
}

lazy_static! {
    static ref RE_THINK: regex::Regex = regex_or_panic!(r"(?s)<think>.*?</think>");
    static ref RE_THINKING: regex::Regex =
        regex_or_panic!(r"(?m)^.{0,5}(思考过程|思维过程|Reasoning|Thinking)[:：]\s*");
    static ref RE_HEADING: regex::Regex = regex_or_panic!(r"(?m)^#{1,6}\s*");
    static ref RE_BOLD: regex::Regex = regex_or_panic!(r"\*\*(.+?)\*\*");
    static ref RE_ITALIC: regex::Regex = regex_or_panic!(r"\*(.+?)\*");
    static ref RE_BLANK: regex::Regex = regex_or_panic!(r"\n{3,}");
    static ref RE_CODE_FENCE_OPEN: regex::Regex = regex_or_panic!(r"(?s)^```[a-zA-Z0-9]*\s*");
    static ref RE_CODE_FENCE_CLOSE: regex::Regex = regex_or_panic!(r"\s*```$");
}

/// 根据主题生成短视频脚本，同时提取素材搜索关键词
///
/// 对应设计文档②文案/剧情生成。一次 LLM 调用同时完成脚本撰写和关键词提取，
/// 避免两次串行调用导致超时。关键词以 JSON 数组形式附在脚本之后。
#[allow(clippy::too_many_arguments)]
pub async fn generate_script(
    provider: &str,
    subject: &str,
    intent: &serde_json::Value,
    language: &str,
    paragraph_number: u32,
    prompt: &str,
    system_prompt: &str,
    conf: &AppConfig,
) -> Result<String, SomaError> {
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = create_llm(provider, llm_provider, conf, &api_key);

    let intent_desc = format!(
        "主题：{}\n风格：{}\n情感基调：{}\n目标受众：{}\n时长建议：{}\n目标平台：{}",
        intent
            .get("theme")
            .and_then(|v| v.as_str())
            .unwrap_or(subject),
        intent.get("style").and_then(|v| v.as_str()).unwrap_or(""),
        intent.get("mood").and_then(|v| v.as_str()).unwrap_or(""),
        intent
            .get("audience")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
        intent
            .get("duration")
            .and_then(|v| v.as_str())
            .unwrap_or("30s"),
        intent
            .get("platform")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    );

    let sys_msg = if system_prompt.is_empty() {
        let lang_label = if language.is_empty() {
            "中文"
        } else {
            language
        };
        format!(
            "你是一位顶尖短视频脚本作家，擅长创作引人入胜、信息丰富且情感饱满的短视频旁白脚本。\n\n\
             任务：根据创作参数撰写短视频旁白脚本，并为每个段落提取素材搜索关键词。\n\n\
             输出格式（两部分用 ===KEYWORDS=== 分隔）：\n\n\
             第一部分：旁白脚本\n\
             - 按时间线标注 [开始s-结束s] 旁白文本\n\
             - 每段旁白要言之有物，避免空话套话和重复\n\
             - 用生动的语言描述，有画面感和情感共鸣\n\
             - 语句长短交替，有节奏感，适合语音朗读\n\
             - 段落之间自然衔接，逻辑递进\n\
             - 总时长内的内容密度要足够，不要留大段空白\n\n\
             第二部分：搜索关键词\n\
             - 每个段落对应1-2个英文搜索关键词，用于在Pexels/Pixabay搜索视频素材\n\
             - 关键词要简短精准（1-3个英文单词），如：ocean waves、city night、coffee pouring\n\
             - 不要用长句或描述性短语作为关键词\n\
             - 关键词要能搜索到高质量、有视觉冲击力的视频画面\n\
             - 格式：段落1关键词1,段落1关键词2;段落2关键词1,段落2关键词2;...\n\n\
             语言：{}\n段落数：{}\n\
             不要包含格式标记、标题、序号（时间标注除外）",
            lang_label,
            paragraph_number,
        )
    } else {
        system_prompt.to_string()
    };

    let user_msg = if prompt.is_empty() {
        format!("创作参数：\n{}\n\n请撰写脚本并提取关键词：", intent_desc)
    } else {
        format!(
            "{}\n\n创作参数：\n{}\n主题：{}",
            prompt, intent_desc, subject
        )
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
    let result = llm
        .chat(&body_value)
        .await
        .map_err(|e| SomaError::Llm(format!("LLM chat failed: {:?}", e)))?;

    let content = extract_content_from_response(&result)?;
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
    conf: &AppConfig,
) -> Result<Vec<String>, SomaError> {
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = create_llm(provider, llm_provider, conf, &api_key);

    let sys_msg = format!(
        "你是一个视频素材搜索关键词提取专家。请从给定的视频脚本中提取{}个最适合搜索视频素材的关键词。\
         要求：\n\
         1. 每个关键词必须是简短的英文词组（1-3个单词），适合在Pexels、Pixabay等素材网站搜索\n\
         2. 关键词要精准、有视觉画面感，能搜到高质量视频素材\n\
         3. 好的关键词示例：ocean waves、sunrise mountain、coffee pouring、city lights night、rain window\n\
         4. 不好的关键词示例：a beautiful ocean scene with waves crashing（太长）、emotion（太抽象）、video（太泛）\n\
         5. 按脚本段落顺序排列，每个段落至少1个关键词\n\
         6. 只输出关键词，用逗号分隔\n\
         7. 不要包含任何解释或编号",
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
    let result = llm
        .chat(&body_value)
        .await
        .map_err(|e| SomaError::Llm(format!("LLM terms failed: {:?}", e)))?;

    let content = extract_content_from_response(&result)?;
    let content = clean_llm_output(&content);
    let terms = parse_terms_output(&content, amount);
    Ok(terms)
}

/// 根据视频脚本生成交白文案
///
/// 视频脚本包含场景描述、镜头语言等创作指导，不适合直接用于 TTS 朗读。
/// 此函数将脚本转换为口语化、自然流畅的旁白文案，供 TTS 语音合成使用。
pub async fn generate_narration(
    provider: &str,
    script: &str,
    storyboard: &serde_json::Value,
    style: &str,
    mood: &str,
    language: &str,
    conf: &AppConfig,
) -> Result<String, SomaError> {
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = create_llm(provider, llm_provider, conf, &api_key);

    let storyboard_hint = if let Some(scenes) = storyboard.as_array() {
        if !scenes.is_empty() {
            let scene_narrations: Vec<String> = scenes
                .iter()
                .filter_map(|s| {
                    s.get("narration")
                        .and_then(|n| n.as_str())
                        .map(|n| n.to_string())
                })
                .collect();
            if !scene_narrations.is_empty() {
                format!("\n\n参考分镜中各场景的旁白文本（请在此基础上优化为更自然流畅的口语化旁白）：\n{}", scene_narrations.join("\n"))
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let lang_instruction = if language.is_empty() || language == "zh-CN" {
        "中文"
    } else if language == "en-US" {
        "English"
    } else {
        language
    };

    // Fish-Speech 风格情感标签指令：开启后让 LLM 在旁白中插入 [tag]，
    // 由支持标签的 TTS 引擎（fishspeech）渲染为语气/停顿/情绪；
    // 其他引擎会在合成前自动剥离标签。
    let emotion_tag_instruction = if conf.get_narration_emotion_tags() {
        "\n11. 在合适的语句前插入方括号情感标签来控制语气和情绪，例如 [whisper]（耳语）、\
         [excited]（兴奋）、[pause]（停顿）、[sad]（低落）、[laughing]（轻笑）、\
         [emphasis]（重音）、[sigh]（叹息）、[surprised]（惊讶）\n\
         - 标签必须紧贴它所作用的语句之前\n\
         - 克制使用，全文标签总数不超过段落句子数的一半，每句最多一个\n\
         - 标签之外不要再输出任何其他方括号内容\n"
    } else {
        ""
    };

    let sys_msg = format!(
        "你是一个专业的短视频旁白撰稿人。你的任务是将视频脚本转换为适合语音朗读的旁白文案。\n\n\
         关键要求：\n\
         1. 旁白文案必须是口语化、自然流畅的，像一位专业播音员在娓娓道来\n\
         2. 去掉所有场景描述、镜头指导（如'近景''航拍''推进'等）和视觉术语\n\
         3. 去掉括号内的技术标注、情绪提示等非朗读内容\n\
         4. 保持原文的核心信息和情感表达，用更自然生动的口语方式重新表述\n\
         5. 语句要简短有力，适合TTS语音合成，避免过长复杂句式和书面化表达\n\
         6. 善用短句和停顿制造节奏感，段落之间自然衔接不突兀\n\
         7. 添加适当的语气词和连接词（'然而''不仅如此''想象一下'等），让旁白更有感染力\n\
         8. 每段旁白要有信息增量，避免空洞重复\n\
         9. 风格：{}，情感基调：{}\n\
         10. 语言：{}{}\n\n\
         直接输出旁白文案纯文本，不要加标题、标号或任何解释。",
        style, mood, lang_instruction, emotion_tag_instruction
    );

    let user_msg = format!("视频脚本：\n{}{}", script, storyboard_hint);

    let body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": sys_msg},
            {"role": "user", "content": user_msg}
        ],
        "temperature": 0.5,
        "max_tokens": 4096,
    });

    let body_value = serde_json_to_tube_value(&body);
    let result = llm
        .chat(&body_value)
        .await
        .map_err(|e| SomaError::Llm(format!("LLM narration failed: {:?}", e)))?;

    let content = extract_content_from_response(&result)?;
    let narration = clean_llm_output(&content);
    Ok(narration.trim().to_string())
}

/// 需求理解：将用户简短描述提炼为结构化的视频创作参数
///
/// 对应设计文档①需求理解。LLM 解析用户意图，提取结构化创作参数。
/// 如果用户输入已经很详细（含多行或超过50字），直接构建简单结构返回。
/// 输出格式遵循文档中的 JSON 结构：theme, style, duration, aspect_ratio, audience, mood, language, platform。
pub async fn generate_intent(
    provider: &str,
    subject: &str,
    language: &str,
    aspect_ratio: &str,
    conf: &AppConfig,
) -> Result<serde_json::Value, SomaError> {
    if subject.lines().count() > 2 || subject.chars().count() > 50 {
        return Ok(serde_json::json!({
            "theme": subject,
            "style": "",
            "duration": "",
            "aspect_ratio": aspect_ratio,
            "audience": "",
            "mood": "",
            "language": if language.is_empty() { "中文" } else { language },
            "platform": ""
        }));
    }

    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = create_llm(provider, llm_provider, conf, &api_key);

    let sys_msg = "你是一个短视频创意策划专家。请根据用户的简短描述，提炼出结构化的视频创作参数。\
                   以JSON格式输出，包含以下字段：\n\
                   - theme: 视频主题（精炼关键词）\n\
                   - style: 视觉风格（如：唯美治愈、科技感、纪实、幽默等）\n\
                   - duration: 建议时长（如 30s、60s）\n\
                   - aspect_ratio: 画幅比例（9:16竖屏 或 16:9横屏）\n\
                   - audience: 目标受众\n\
                   - mood: 情感基调（如：温暖、希望、紧张、欢乐等）\n\
                   - language: 语言\n\
                   - platform: 目标平台（抖音、视频号、YouTube等）\n\
                   只输出JSON，不要代码围栏或解释。";

    let user_msg = format!(
        "请提炼以下视频创意：{}\n参考画幅：{}\n参考语言：{}",
        subject,
        aspect_ratio,
        if language.is_empty() {
            "中文"
        } else {
            language
        }
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
    let result = llm
        .chat(&body_value)
        .await
        .map_err(|e| SomaError::Llm(format!("LLM intent failed: {:?}", e)))?;

    let content = extract_content_from_response(&result)?;
    let content = clean_llm_output(&content);
    parse_intent_output(&content, subject, language, aspect_ratio)
}

/// 分镜脚本 + 提示词生成：将脚本文案拆分为分镜场景列表
///
/// 对应设计文档③分镜脚本 + ④提示词生成（合并为一次LLM调用）。
/// 将文案拆解为场景镜头，每个镜头包含完整制作指令。
/// 视觉提示词遵循公式：主体描述 + 环境/场景 + 光线/色彩 + 风格/质量 + 镜头参数。
/// LLM 以 JSON 数组格式输出 StoryboardScene 列表。
/// JSON 解析失败时回退到简单的等分拆分逻辑。
pub async fn generate_storyboard(
    provider: &str,
    subject: &str,
    script: &str,
    clip_duration: u32,
    intent: &serde_json::Value,
    conf: &AppConfig,
) -> Result<Vec<StoryboardScene>, SomaError> {
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = create_llm(provider, llm_provider, conf, &api_key);

    let style = intent.get("style").and_then(|v| v.as_str()).unwrap_or("");
    let mood = intent.get("mood").and_then(|v| v.as_str()).unwrap_or("");
    let user_keywords = intent
        .get("user_visual_keywords")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        })
        .unwrap_or_default();

    let keyword_instruction = if user_keywords.is_empty() {
        String::new()
    } else {
        format!(
            "\n5. 用户已指定以下视觉关键词，请在生成 visual_prompt 时优先参考并融入这些关键词：{}\n\
             请确保每个场景的 visual_prompt 都能体现对应关键词的视觉元素。", user_keywords
        )
    };

    let sys_msg = format!(
        "你是一个专业的视频分镜导演。请根据给定的视频脚本文案，将其拆分为多个分镜场景，并为每个场景生成完整的制作指令和视觉提示词。\n\n\
         输入信息：\n\
         - 主题：{}\n\
         - 视觉风格：{}\n\
         - 情感基调：{}\n\n\
         每个场景包含以下字段：\n\
         - scene_id: 场景编号（从1开始）\n\
         - duration: 该镜头时长（秒），每个场景约{}秒\n\
         - narration: 该场景旁白文本，所有场景narration按顺序合并应还原原始脚本\n\
         - visual_desc: 画面中文描述（如：近景：樱花树上花瓣随风飘落，阳光透过枝头）\n\
         - visual_prompt: 画面英文提示词，遵循公式：主体描述 + 环境/场景 + 光线/色彩 + 风格/质量 + 镜头参数\n\
           示例：cherry blossom petals gently falling from branches, soft golden morning light filtering through, \
           cinematic warm pastel tones 8K, shallow depth of field bokeh\n\
         - search_keyword: 素材搜索英文关键词（1-3个单词，简短精准，适合Pexels/Pixabay搜索）\n\
           示例：cherry blossom、ocean waves、city lights、coffee pouring、golden sunset\n\
           要求：必须是简短词组，不能是长句描述；要能搜到高质量有视觉冲击力的素材\n\
         - camera_movement: 镜头运动，从以下选择：push_in(缓慢推进), pull_out(拉远), pan_left(左摇), pan_right(右摇), \
           tilt_up(上仰), tilt_down(下俯), static(固定), zoom(变焦), tracking(跟随), aerial(航拍), close_up(特写)\n\
         - transition: 转场方式：cut(硬切-节奏感强), fade(淡入淡出-柔和抒情), dissolve(叠化-时间流逝), slide(滑动-空间转换), zoom(缩放-突出重点)\n\
         - text_overlay: 字幕叠加文本（装饰性文字，如\"春 · 起始\"，若与旁白相同则留空）\n\
         - mood: 情绪基调（如：温暖宁静、紧张悬疑、欢乐活泼等）\n\n\
         关键要求：\n\
         1. 场景数量根据脚本长度合理划分，每个场景应是一个完整的视觉画面\n\
         2. visual_prompt 必须是英文，具体、有画面感，适合AI视频生成\n\
         3. search_keyword 必须是简短英文词组（1-3个单词），精准可搜索\n\
         4. camera_movement 要与画面内容和情绪匹配\n\
         5. transition 优先使用 fade 和 dissolve（更柔和专业），避免过多 cut 导致画面跳跃\n\
         6. 镜头运动参考：固定→展示静态场景、推进→增强沉浸感、航拍→宏大场景、跟随→增强代入感、特写→强调细节{}\
         \n\n\
         以JSON数组输出：\n\
         [\n  {{\n    \"scene_id\": 1,\n    \"duration\": {},\n    \"narration\": \"场景旁白\",\n    \"visual_desc\": \"近景：樱花飘落...\",\n    \
         \"visual_prompt\": \"cherry blossom petals falling, soft golden light, cinematic, 8K\",\n    \
         \"search_keyword\": \"cherry blossom\",\n    \
         \"camera_movement\": \"push_in\",\n    \"transition\": \"fade\",\n    \"text_overlay\": \"春 · 起始\",\n    \
         \"mood\": \"温暖宁静\"\n  }}\n]\n\n\
         只输出JSON数组，不要代码围栏或解释。", subject, style, mood, clip_duration, keyword_instruction, clip_duration);

    let user_msg = format!("视频脚本：\n{}", script);

    let body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": sys_msg},
            {"role": "user", "content": user_msg}
        ],
        "temperature": 0.4,
        "max_tokens": 4096,
    });

    let body_value = serde_json_to_tube_value(&body);
    let result = llm
        .chat(&body_value)
        .await
        .map_err(|e| SomaError::Llm(format!("LLM storyboard failed: {:?}", e)))?;

    let content = extract_content_from_response(&result)?;
    let content = clean_llm_output(&content);
    parse_storyboard_output(&content, script, clip_duration)
}

/// 解析分镜脚本 LLM 输出，支持多种格式和容错回退
fn parse_storyboard_output(
    content: &str,
    original_script: &str,
    clip_duration: u32,
) -> Result<Vec<StoryboardScene>, SomaError> {
    let stripped = strip_code_fence(content);

    if let Ok(scenes) = serde_json::from_str::<Vec<StoryboardScene>>(&stripped) {
        if !scenes.is_empty() {
            return Ok(scenes);
        }
    }

    if let Ok(re) = regex::Regex::new(r"(?s)\[.*\]") {
        if let Some(caps) = re.find(&stripped) {
            if let Ok(scenes) = serde_json::from_str::<Vec<StoryboardScene>>(caps.as_str()) {
                if !scenes.is_empty() {
                    return Ok(scenes);
                }
            }
        }
    }

    log!("分镜脚本JSON解析失败，回退到简单等分拆分");
    Ok(fallback_storyboard(original_script, clip_duration))
}

/// 简单等分拆分回退：按标点将脚本拆分为若干场景
fn fallback_storyboard(script: &str, clip_duration: u32) -> Vec<StoryboardScene> {
    use soma_core::models::PUNCTUATIONS;

    let sentences: Vec<&str> = script
        .split(|c: char| PUNCTUATIONS.contains(&c.to_string().as_str()))
        .filter(|s| !s.trim().is_empty())
        .collect();

    if sentences.is_empty() {
        return vec![StoryboardScene {
            scene_id: 1,
            duration: Some(clip_duration),
            narration: script.to_string(),
            visual_desc: Some(script.to_string()),
            visual_prompt: script.to_string(),
            search_keyword: None,
            camera_movement: Some("static".to_string()),
            transition: Some("cut".to_string()),
            text_overlay: None,
            mood: None,
        }];
    }

    sentences
        .iter()
        .enumerate()
        .map(|(i, s)| StoryboardScene {
            scene_id: (i + 1) as u32,
            duration: Some(clip_duration),
            narration: s.trim().to_string(),
            visual_desc: Some(s.trim().to_string()),
            visual_prompt: s.trim().to_string(),
            search_keyword: None,
            camera_movement: Some("static".to_string()),
            transition: if i + 1 < sentences.len() {
                Some("cut".to_string())
            } else {
                None
            },
            text_overlay: None,
            mood: None,
        })
        .collect()
}

/// 解析需求理解 LLM 输出，容错回退
fn parse_intent_output(
    content: &str,
    subject: &str,
    language: &str,
    aspect_ratio: &str,
) -> Result<serde_json::Value, SomaError> {
    let stripped = strip_code_fence(content);

    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&stripped) {
        if v.is_object() {
            return Ok(v);
        }
    }

    if let Ok(re) = regex::Regex::new(r"(?s)\{.*\}") {
        if let Some(caps) = re.find(&stripped) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(caps.as_str()) {
                if v.is_object() {
                    return Ok(v);
                }
            }
        }
    }

    log!("需求理解JSON解析失败，使用默认值");
    Ok(serde_json::json!({
        "theme": subject,
        "style": "",
        "duration": "",
        "aspect_ratio": aspect_ratio,
        "audience": "",
        "mood": "",
        "language": if language.is_empty() { "中文" } else { language },
        "platform": ""
    }))
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
    conf: &AppConfig,
) -> Result<serde_json::Value, SomaError> {
    let (llm_provider, api_key, model_name) = get_provider_config(provider, conf)?;
    let llm = create_llm(provider, llm_provider, conf, &api_key);

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
    let result = llm
        .chat(&body_value)
        .await
        .map_err(|e| SomaError::Llm(format!("LLM social metadata failed: {:?}", e)))?;

    let content = extract_content_from_response(&result)?;
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
        "tiktok" => SocialPlatformSpec {
            title_max: 100,
            caption_max: 2200,
            hashtag_count: 5,
        },
        "youtube_shorts" | "youtube" => SocialPlatformSpec {
            title_max: 100,
            caption_max: 5000,
            hashtag_count: 3,
        },
        "instagram_reels" | "instagram" => SocialPlatformSpec {
            title_max: 125,
            caption_max: 2200,
            hashtag_count: 8,
        },
        "facebook_reels" | "facebook" => SocialPlatformSpec {
            title_max: 125,
            caption_max: 2200,
            hashtag_count: 5,
        },
        "x" | "twitter" => SocialPlatformSpec {
            title_max: 70,
            caption_max: 280,
            hashtag_count: 3,
        },
        _ => SocialPlatformSpec {
            title_max: 100,
            caption_max: 2200,
            hashtag_count: 5,
        },
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
fn fallback_social_metadata(
    subject: &str,
    script: &str,
    spec: &SocialPlatformSpec,
) -> serde_json::Value {
    let default_tags = [
        "#shorts",
        "#viral",
        "#trending",
        "#fyp",
        "#video",
        "#reels",
        "#creator",
        "#content",
    ];
    let tags: Vec<String> = default_tags
        .iter()
        .take(spec.hashtag_count)
        .map(|t| t.to_string())
        .collect();
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
/// 创建 LLM 客户端：openai 提供商支持 `openai_base_url` 自定义端点
/// （本地网关 / 自建 OpenAI 兼容服务），其余走 kit 工厂固定映射
fn create_llm(
    provider: &str,
    llm_provider: LlmProvider,
    conf: &AppConfig,
    api_key: &str,
) -> Box<dyn LlmService> {
    if provider == "openai" {
        if let Some(base) = conf
            .app
            .openai_base_url
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let (host, path) = split_openai_base(base);
            return Box::new(OpenAICompatible::new(&host, &path, api_key));
        }
    }
    LlmFactory::create(llm_provider, api_key)
}

/// 拆分 OpenAI 兼容 base_url 为 (host, api 版本前缀)；
/// 无 `/v1` 时默认补 `/v1`，`/v1/xxx` 形式保留子路径
fn split_openai_base(base: &str) -> (String, String) {
    if let Some(idx) = base.find("/v1") {
        let host = base[..idx].trim_end_matches('/').to_string();
        let rest = base[idx + 3..].trim_end_matches('/').to_string();
        if rest.is_empty() {
            (host, "/v1".into())
        } else {
            (format!("{host}/v1"), format!("/{rest}"))
        }
    } else {
        (base.trim_end_matches('/').to_string(), "/v1".into())
    }
}

fn get_provider_config(
    provider: &str,
    conf: &AppConfig,
) -> Result<(LlmProvider, String, String), SomaError> {
    match provider {
        "openai" => {
            let key = conf.app.openai_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .openai_model_name
                .as_deref()
                .unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "deepseek" => {
            let key = conf.app.deepseek_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .deepseek_model_name
                .as_deref()
                .unwrap_or("deepseek-chat");
            Ok((LlmProvider::DeepSeek, key.to_string(), model.to_string()))
        }
        "qwen" => {
            let key = conf.app.qwen_api_key.as_deref().unwrap_or("");
            let model = conf.app.qwen_model_name.as_deref().unwrap_or("qwen-plus");
            Ok((LlmProvider::QWen, key.to_string(), model.to_string()))
        }
        "moonshot" | "kimi" => {
            let key = conf.app.moonshot_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .moonshot_model_name
                .as_deref()
                .unwrap_or("moonshot-v1-8k");
            Ok((LlmProvider::Kimi, key.to_string(), model.to_string()))
        }
        "ollama" => {
            let key = "";
            let model = conf.app.ollama_model_name.as_deref().unwrap_or("llama3");
            Ok((LlmProvider::Ollama, key.to_string(), model.to_string()))
        }
        "minimax" => {
            let key = conf.app.minimax_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .minimax_model_name
                .as_deref()
                .unwrap_or("abab6.5s-chat");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "mimo" => {
            let key = conf.app.mimo_api_key.as_deref().unwrap_or("");
            let model = conf.app.mimo_model_name.as_deref().unwrap_or("mimo");
            Ok((LlmProvider::MiMo, key.to_string(), model.to_string()))
        }
        "gemini" => {
            // Gemini 通过 OpenAI 兼容接口调用
            let key = conf.app.gemini_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .gemini_model_name
                .as_deref()
                .unwrap_or("gemini-2.0-flash");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "azure" => {
            let key = conf.app.azure_api_key.as_deref().unwrap_or("");
            let model = conf.app.azure_model_name.as_deref().unwrap_or("gpt-4o");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "groq" => {
            let key = conf.app.groq_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .groq_model_name
                .as_deref()
                .unwrap_or("llama-3.1-8b-instant");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "grok" => {
            let key = conf.app.grok_api_key.as_deref().unwrap_or("");
            let model = conf.app.grok_model_name.as_deref().unwrap_or("grok-3");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "doubao" => {
            let key = conf.app.doubao_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .doubao_model_name
                .as_deref()
                .unwrap_or("doubao-pro-32k");
            Ok((LlmProvider::Doubao, key.to_string(), model.to_string()))
        }
        "hunyuan" => {
            let key = conf.app.hunyuan_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .hunyuan_model_name
                .as_deref()
                .unwrap_or("hunyuan-turbo");
            Ok((LlmProvider::Hunyuan, key.to_string(), model.to_string()))
        }
        "zhipu" => {
            let key = conf.app.zhipu_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .zhipu_model_name
                .as_deref()
                .unwrap_or("glm-4-flash");
            Ok((LlmProvider::Zhipu, key.to_string(), model.to_string()))
        }
        "wenxin" => {
            let key = conf.app.wenxin_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .wenxin_model_name
                .as_deref()
                .unwrap_or("ernie-4.0-8k");
            Ok((LlmProvider::Wenxin, key.to_string(), model.to_string()))
        }
        "xunfei" => {
            let key = conf.app.xunfei_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .xunfei_model_name
                .as_deref()
                .unwrap_or("generalv3.5");
            Ok((LlmProvider::Xunfei, key.to_string(), model.to_string()))
        }
        "oneapi" => {
            let key = conf.app.oneapi_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .oneapi_model_name
                .as_deref()
                .unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "aihubmix" => {
            let key = conf.app.aihubmix_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .aihubmix_model_name
                .as_deref()
                .unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "evolink" => {
            let key = conf.app.evolink_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .evolink_model_name
                .as_deref()
                .unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "aiml" | "aimlapi" => {
            let key = conf.app.aimlapi_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .aimlapi_model_name
                .as_deref()
                .unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "modelscope" => {
            let key = conf.app.modelscope_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .modelscope_model_name
                .as_deref()
                .unwrap_or("qwen-turbo");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "pollinations" => {
            let key = conf.app.pollinations_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .pollinations_model_name
                .as_deref()
                .unwrap_or("openai");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "g4f" => {
            let key = "";
            let model = conf.app.g4f_model_name.as_deref().unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "cloudflare" => {
            let key = conf.app.openai_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .openai_model_name
                .as_deref()
                .unwrap_or("@cf/meta/llama-3-8b-instruct");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        "litellm" => {
            let key = conf.app.oneapi_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .litellm_model_name
                .as_deref()
                .unwrap_or("gpt-4o-mini");
            Ok((LlmProvider::ChatGPT, key.to_string(), model.to_string()))
        }
        _ => {
            let key = conf.app.openai_api_key.as_deref().unwrap_or("");
            let model = conf
                .app
                .openai_model_name
                .as_deref()
                .unwrap_or("gpt-4o-mini");
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
fn extract_content_from_response(result: &tube::Value) -> Result<String, SomaError> {
    if let Some(choices) = result.get("choices") {
        if let Some(arr) = choices.as_array() {
            if let Some(first) = arr.first() {
                if let Some(msg) = first.get("message") {
                    if let Some(content) = msg.get("content") {
                        return Ok(content.to_string());
                    }
                }
            }
        }
    }
    // 服务商错误负载（无 choices，形如 {"code": 4xxxx, "msg": ...}）：
    // 此前会被原样当作内容返回，造成"功能成功但产物是错误 JSON"的假阳性
    if let Some(code) = result.get("code").and_then(|v| v.as_i64()) {
        if code != 0 && code != 200 {
            let msg = result
                .get("msg")
                .or_else(|| result.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| "未知错误".to_string());
            return Err(SomaError::Llm(format!(
                "LLM 服务商返回错误（code {code}）: {msg}"
            )));
        }
    }
    let text = result.to_string();
    if text.trim().is_empty() || text == "null" {
        return Err(SomaError::Llm("LLM 响应为空".into()));
    }
    Ok(text)
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
        let terms: Vec<String> = arr
            .iter()
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
                let terms: Vec<String> = arr
                    .iter()
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
