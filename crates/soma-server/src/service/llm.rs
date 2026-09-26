/// LLM（大语言模型）服务模块（宿主兼容层）
///
/// 0.1.2 起实现已迁至 `soma_feature::llm`（配置参数为 AppConfig），
/// 本模块保留原函数签名（`conf: &Config`）委托转发，
/// handler 层与既有调用方无需改动。
use soma_core::error::SomaError;
use soma_core::models::StoryboardScene;
use crate::Config;

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
    conf: &Config,
) -> Result<String, SomaError> {
    soma_feature::llm::generate_script(
        provider, subject, intent, language, paragraph_number, prompt, system_prompt, &conf.app,
    )
    .await
}

/// 从视频脚本中提取素材搜索关键词
pub async fn generate_terms(
    provider: &str,
    subject: &str,
    script: &str,
    amount: usize,
    conf: &Config,
) -> Result<Vec<String>, SomaError> {
    soma_feature::llm::generate_terms(provider, subject, script, amount, &conf.app).await
}

/// 根据视频脚本生成旁白文案
pub async fn generate_narration(
    provider: &str,
    script: &str,
    storyboard: &serde_json::Value,
    style: &str,
    mood: &str,
    language: &str,
    conf: &Config,
) -> Result<String, SomaError> {
    soma_feature::llm::generate_narration(
        provider, script, storyboard, style, mood, language, &conf.app,
    )
    .await
}

/// 需求理解：将用户简短描述提炼为结构化的视频创作参数
pub async fn generate_intent(
    provider: &str,
    subject: &str,
    language: &str,
    aspect_ratio: &str,
    conf: &Config,
) -> Result<serde_json::Value, SomaError> {
    soma_feature::llm::generate_intent(provider, subject, language, aspect_ratio, &conf.app).await
}

/// 分镜脚本 + 提示词生成：将脚本文案拆分为分镜场景列表
pub async fn generate_storyboard(
    provider: &str,
    subject: &str,
    script: &str,
    clip_duration: u32,
    intent: &serde_json::Value,
    conf: &Config,
) -> Result<Vec<StoryboardScene>, SomaError> {
    soma_feature::llm::generate_storyboard(
        provider, subject, script, clip_duration, intent, &conf.app,
    )
    .await
}

/// 生成社交媒体发布元数据
pub async fn generate_social_metadata(
    provider: &str,
    subject: &str,
    script: &str,
    platform: &str,
    conf: &Config,
) -> Result<serde_json::Value, SomaError> {
    soma_feature::llm::generate_social_metadata(provider, subject, script, platform, &conf.app)
        .await
}
