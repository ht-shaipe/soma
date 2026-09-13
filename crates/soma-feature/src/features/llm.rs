//! LLM 文本生成类功能点：需求理解、脚本、分镜、旁白
//!
//! 输入输出为显式结构（替代原流水线经 TaskInfo 字段的隐式传递），
//! 具体逻辑复用 [`crate::llm`] 的 LLM 服务。

use crate::context::FeatureContext;
use crate::descriptor::{FeatureKind, FeatureMeta};
use crate::feature::TypedFeature;
use crate::llm;
use crate::progress::ProgressReporter;
use crate::runtime::{block_on_async, retry};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soma_core::error::SomaError;
use soma_core::models::StoryboardScene;

/// 从分镜场景列表提取素材搜索关键词（search_keyword 优先，回退 visual_prompt）
///
/// 供编排器处理已有分镜数据时复用同一推导规则，与功能点内部行为一致。
pub fn derive_terms(storyboard: &[StoryboardScene]) -> Vec<String> {
    storyboard
        .iter()
        .map(|s| {
            s.search_keyword
                .as_deref()
                .filter(|k| !k.is_empty())
                .unwrap_or(&s.visual_prompt)
                .to_string()
        })
        .collect()
}

// ============ llm.intent：需求理解 ============

/// llm.intent 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LlmIntentInput {
    /// 视频主题/用户描述
    pub video_subject: String,
    /// 语言代码（如 "zh-CN"），空表示中文
    #[serde(default)]
    pub language: Option<String>,
    /// 画幅比例（如 "9:16"），默认 9:16
    #[serde(default)]
    pub aspect_ratio: Option<String>,
    /// 风格偏好
    #[serde(default)]
    pub style: Option<String>,
    /// 情绪基调偏好
    #[serde(default)]
    pub mood: Option<String>,
}

/// llm.intent 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LlmIntentOutput {
    /// 结构化创作意图（theme/style/duration/aspect_ratio/audience/mood/language/platform）
    pub intent: serde_json::Value,
}

/// 需求理解：LLM 将用户简短描述提炼为结构化创作参数
pub struct LlmIntentFeature;

impl TypedFeature for LlmIntentFeature {
    type Input = LlmIntentInput;
    type Output = LlmIntentOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "llm.intent".into(),
            name: "需求理解".into(),
            description: "LLM 解析用户简短描述，提炼结构化创作参数（theme/style/mood 等）".into(),
            kind: FeatureKind::Llm,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: LlmIntentInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<LlmIntentOutput, SomaError> {
        let conf = ctx.config();
        let provider = conf.app.llm_provider.as_deref().unwrap_or("openai");
        let language = input.language.clone().unwrap_or_default();
        let aspect_ratio = input.aspect_ratio.clone().unwrap_or_else(|| "9:16".into());
        let intent = retry(3, || {
            let fut = llm::generate_intent(
                provider,
                &input.video_subject,
                &language,
                &aspect_ratio,
                conf,
            );
            block_on_async(fut)?
        })?;
        Ok(LlmIntentOutput { intent })
    }
}

// ============ llm.script：文案/剧情生成 ============

/// llm.script 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LlmScriptInput {
    /// 视频主题
    pub video_subject: String,
    /// 结构化创作意图（来自 llm.intent 输出）
    #[serde(default)]
    pub intent: serde_json::Value,
    /// 语言代码，空表示中文
    #[serde(default)]
    pub language: Option<String>,
    /// 脚本分段数（1~10）
    #[serde(default)]
    pub paragraph_number: Option<u32>,
    /// 自定义生成提示词
    #[serde(default)]
    pub script_prompt: Option<String>,
    /// 自定义系统提示词
    #[serde(default)]
    pub system_prompt: Option<String>,
    /// 是否使用自定义系统提示词
    #[serde(default)]
    pub use_custom_system_prompt: bool,
}

/// llm.script 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LlmScriptOutput {
    /// 生成的视频脚本文案
    pub script: String,
}

/// 文案/剧情生成：LLM 根据主题与意图撰写短视频脚本
pub struct LlmScriptFeature;

impl TypedFeature for LlmScriptFeature {
    type Input = LlmScriptInput;
    type Output = LlmScriptOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "llm.script".into(),
            name: "脚本生成".into(),
            description: "LLM 根据主题与创作意图撰写短视频脚本".into(),
            kind: FeatureKind::Llm,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: LlmScriptInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<LlmScriptOutput, SomaError> {
        let conf = ctx.config();
        let provider = conf.app.llm_provider.as_deref().unwrap_or("openai");
        let language = input.language.clone().unwrap_or_default();
        let paragraph_number = input.paragraph_number.unwrap_or(1).clamp(1, 10);
        let prompt = input.script_prompt.clone().unwrap_or_default();
        let system_prompt = if input.use_custom_system_prompt {
            input.system_prompt.clone().unwrap_or_default()
        } else {
            String::new()
        };

        let script = retry(5, || {
            let fut = llm::generate_script(
                provider,
                &input.video_subject,
                &input.intent,
                &language,
                paragraph_number,
                &prompt,
                &system_prompt,
                conf,
            );
            block_on_async(fut)?
        })?;
        Ok(LlmScriptOutput { script })
    }
}

// ============ llm.storyboard：分镜脚本 + 提示词 ============

/// llm.storyboard 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LlmStoryboardInput {
    /// 视频主题
    pub video_subject: String,
    /// 视频脚本文案
    pub script: String,
    /// 单镜头时长（秒），默认 4
    #[serde(default)]
    pub clip_duration: Option<u32>,
    /// 结构化创作意图
    #[serde(default)]
    pub intent: serde_json::Value,
    /// 用户指定的视觉关键词（作为提示词约束）
    #[serde(default)]
    pub user_terms: Vec<String>,
}

/// llm.storyboard 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LlmStoryboardOutput {
    /// 分镜场景列表
    pub storyboard: Vec<StoryboardScene>,
    /// 从分镜推导的素材搜索关键词
    pub terms: Vec<String>,
}

/// 分镜脚本生成：LLM 将脚本拆分为分镜场景并生成视觉提示词与搜索关键词
pub struct LlmStoryboardFeature;

impl TypedFeature for LlmStoryboardFeature {
    type Input = LlmStoryboardInput;
    type Output = LlmStoryboardOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "llm.storyboard".into(),
            name: "分镜脚本".into(),
            description: "LLM 将脚本拆分为分镜场景，生成画面提示词与素材搜索关键词".into(),
            kind: FeatureKind::Llm,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: LlmStoryboardInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<LlmStoryboardOutput, SomaError> {
        let conf = ctx.config();
        let provider = conf.app.llm_provider.as_deref().unwrap_or("openai");
        let clip_duration = input.clip_duration.unwrap_or(4);

        // 用户指定的视觉关键词作为提示词约束注入意图
        let mut enriched_intent = input.intent.clone();
        if !input.user_terms.is_empty() {
            enriched_intent["user_visual_keywords"] = serde_json::Value::Array(
                input
                    .user_terms
                    .iter()
                    .map(|t| serde_json::Value::String(t.clone()))
                    .collect(),
            );
        }

        let storyboard = retry(3, || {
            let fut = llm::generate_storyboard(
                provider,
                &input.video_subject,
                &input.script,
                clip_duration,
                &enriched_intent,
                conf,
            );
            block_on_async(fut)?
        })?;
        let terms = derive_terms(&storyboard);
        Ok(LlmStoryboardOutput { storyboard, terms })
    }
}

// ============ llm.narration：旁白文案 ============

/// llm.narration 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LlmNarrationInput {
    /// 视频脚本文案
    pub script: String,
    /// 分镜场景列表（可选参考）
    #[serde(default)]
    pub storyboard: Vec<StoryboardScene>,
    /// 风格偏好
    #[serde(default)]
    pub style: Option<String>,
    /// 情绪基调偏好
    #[serde(default)]
    pub mood: Option<String>,
    /// 语言代码，空表示中文
    #[serde(default)]
    pub language: Option<String>,
}

/// llm.narration 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LlmNarrationOutput {
    /// 口语化旁白文案（用于 TTS 朗读）
    pub narration: String,
}

/// 旁白文案生成：LLM 将脚本转换为口语化、适合朗读的旁白
pub struct LlmNarrationFeature;

impl TypedFeature for LlmNarrationFeature {
    type Input = LlmNarrationInput;
    type Output = LlmNarrationOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "llm.narration".into(),
            name: "旁白文案".into(),
            description: "LLM 将视频脚本转换为口语化、自然流畅的旁白文案".into(),
            kind: FeatureKind::Llm,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: LlmNarrationInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<LlmNarrationOutput, SomaError> {
        let conf = ctx.config();
        let provider = conf.app.llm_provider.as_deref().unwrap_or("openai");
        let style = input.style.clone().unwrap_or_default();
        let mood = input.mood.clone().unwrap_or_default();
        let language = input.language.clone().unwrap_or_default();
        let storyboard_json =
            serde_json::to_value(&input.storyboard).unwrap_or(serde_json::Value::Null);

        let narration = retry(3, || {
            let fut = llm::generate_narration(
                provider,
                &input.script,
                &storyboard_json,
                &style,
                &mood,
                &language,
                conf,
            );
            block_on_async(fut)?
        })?;
        Ok(LlmNarrationOutput { narration })
    }
}

// ============ llm.terms：素材关键词提取 ============

/// llm.terms 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LlmTermsInput {
    /// 视频主题
    pub video_subject: String,
    /// 视频脚本文案
    pub script: String,
    /// 需要提取的关键词数量
    #[serde(default)]
    pub amount: Option<u32>,
}

/// llm.terms 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LlmTermsOutput {
    /// 素材搜索关键词列表（英文）
    pub terms: Vec<String>,
}

/// 素材关键词提取：LLM 从脚本中提取适合素材网站搜索的英文关键词
pub struct LlmTermsFeature;

impl TypedFeature for LlmTermsFeature {
    type Input = LlmTermsInput;
    type Output = LlmTermsOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "llm.terms".into(),
            name: "关键词提取".into(),
            description: "LLM 从视频脚本中提取适合 Pexels/Pixabay 搜索的英文关键词".into(),
            kind: FeatureKind::Llm,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: LlmTermsInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<LlmTermsOutput, SomaError> {
        let conf = ctx.config();
        let provider = conf.app.llm_provider.as_deref().unwrap_or("openai");
        let amount = input.amount.unwrap_or(5) as usize;
        let terms = crate::runtime::block_on_async(llm::generate_terms(
            provider,
            &input.video_subject,
            &input.script,
            amount,
            conf,
        ))??;
        Ok(LlmTermsOutput { terms })
    }
}

// ============ llm.social：社交发布元数据 ============

/// llm.social 入参
#[derive(Debug, Deserialize, JsonSchema)]
pub struct LlmSocialInput {
    /// 视频主题
    pub video_subject: String,
    /// 视频脚本文案
    pub script: String,
    /// 目标平台（tiktok/youtube/instagram/x/小红书 等）
    #[serde(default)]
    pub platform: Option<String>,
}

/// llm.social 出参
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LlmSocialOutput {
    /// 发布元数据（title/description/tags）
    pub metadata: serde_json::Value,
}

/// 社交元数据生成：LLM 为指定平台生成标题、描述与标签
pub struct LlmSocialFeature;

impl TypedFeature for LlmSocialFeature {
    type Input = LlmSocialInput;
    type Output = LlmSocialOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "llm.social".into(),
            name: "发布元数据".into(),
            description: "LLM 根据主题与脚本生成指定社交平台的标题、描述和标签".into(),
            kind: FeatureKind::Llm,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: LlmSocialInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<LlmSocialOutput, SomaError> {
        let conf = ctx.config();
        let provider = conf.app.llm_provider.as_deref().unwrap_or("openai");
        let platform = input.platform.as_deref().unwrap_or("");
        let metadata = crate::runtime::block_on_async(llm::generate_social_metadata(
            provider,
            &input.video_subject,
            &input.script,
            platform,
            conf,
        ))??;
        Ok(LlmSocialOutput { metadata })
    }
}
