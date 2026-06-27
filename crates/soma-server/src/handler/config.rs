/// 配置 API 处理器
///
/// 处理前端设置页的配置读写请求：
/// - "get" → 获取当前配置
/// - "save" → 保存配置到文件

use tube::{Result, Value};
use tube_web::RequestParameter;
use crate::Config;

/// 配置模块请求分发
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "get" => get_config(param).await,
        "save" => save_config(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 获取当前配置
///
/// 将后端 AppConfig 结构转换为前端期望的简化格式返回
async fn get_config(_param: &RequestParameter) -> Result<Value> {
    let conf = Config::get();
    let app = &conf.app.app;

    Ok(value!({
        "app": {
            "name": app.name.as_deref().unwrap_or("Soma"),
            "version": "1.0.0",
            "host": app.host.as_deref().unwrap_or("0.0.0.0"),
            "port": app.port.unwrap_or(8080) as u32,
            "storage_path": app.storage_path.as_deref().unwrap_or("./storage"),
            "concurrent_tasks": app.max_concurrent_tasks.unwrap_or(5),
        },
        "llm": {
            "provider": app.llm_provider.as_deref().unwrap_or("openai"),
            "model": get_current_model(app),
            "api_key": get_current_api_key(app),
            "base_url": get_current_base_url(app),
            "api_version": app.azure_api_version.as_deref().unwrap_or(""),
        },
        "tts": {
            "provider": app.subtitle_provider.as_deref().unwrap_or("edge-tts"),
            "voice_name": "",
            "azure_speech_key": conf.app.azure.speech_key.as_deref().unwrap_or(""),
            "azure_speech_region": conf.app.azure.speech_region.as_deref().unwrap_or(""),
            "siliconflow_key": conf.app.siliconflow.api_key.as_deref().unwrap_or(""),
            "elevenlabs_key": conf.app.elevenlabs.api_key.as_deref().unwrap_or(""),
            "elevenlabs_model": conf.app.elevenlabs.model_id.as_deref().unwrap_or("eleven_multilingual_v2"),
            "mimo_key": app.mimo_api_key.as_deref().unwrap_or(""),
        },
        "stock": {
            "pexels_api_key": app.pexels_api_keys.as_ref().and_then(|v| v.first()).unwrap_or(&String::new()).clone(),
            "pixabay_api_key": app.pixabay_api_keys.as_ref().and_then(|v| v.first()).unwrap_or(&String::new()).clone(),
            "coverr_api_key": app.coverr_api_keys.as_ref().and_then(|v| v.first()).unwrap_or(&String::new()).clone(),
        },
        "ffmpeg": {
            "path": app.ffmpeg_path.as_deref().unwrap_or("ffmpeg"),
            "threads": app.max_concurrent_tasks.unwrap_or(2) as u32,
        },
        "whisper": {
            "provider": "local",
            "model": conf.app.whisper.model_size.as_deref().unwrap_or("base"),
            "endpoint": "",
        },
    }))
}

/// 保存配置
///
/// 从请求参数中提取配置项，更新内存中的配置并尝试写回 TOML 文件
async fn save_config(param: &RequestParameter) -> Result<Value> {
    let conf_path = Config::get_conf_path();

    let mut conf = Config::get();

    // 更新应用配置
    if let Some(v) = param.value.get("app") {
        let app = &mut conf.app.app;
        if let Some(name) = v.get("name").and_then(|n| n.as_str()) {
            app.name = Some(name.to_string());
        }
        if let Some(host) = v.get("host").and_then(|h| h.as_str()) {
            app.host = Some(host.to_string());
        }
        if let Some(port) = v.get("port").and_then(|p| p.as_u64()) {
            app.port = Some(port as u16);
        }
        if let Some(sp) = v.get("storage_path").and_then(|s| s.as_str()) {
            app.storage_path = Some(sp.to_string());
        }
        if let Some(ct) = v.get("concurrent_tasks").and_then(|c| c.as_u64()) {
            app.concurrent_tasks = Some(ct as usize);
            app.max_concurrent_tasks = Some(ct as usize);
        }
    }

    // 更新 LLM 配置
    if let Some(v) = param.value.get("llm") {
        let app = &mut conf.app.app;
        if let Some(provider) = v.get("provider").and_then(|p| p.as_str()) {
            app.llm_provider = Some(provider.to_string());
        }
        if let Some(model) = v.get("model").and_then(|m| m.as_str()) {
            set_model_for_provider(app, &model);
        }
        if let Some(api_key) = v.get("api_key").and_then(|k| k.as_str()) {
            set_api_key_for_provider(app, &api_key);
        }
        if let Some(base_url) = v.get("base_url").and_then(|u| u.as_str()) {
            set_base_url_for_provider(app, &base_url);
        }
    }

    // 更新 TTS 配置
    if let Some(v) = param.value.get("tts") {
        let app = &mut conf.app.app;
        if let Some(provider) = v.get("provider").and_then(|p| p.as_str()) {
            app.subtitle_provider = Some(provider.to_string());
        }
        if let Some(key) = v.get("mimo_key").and_then(|k| k.as_str()) {
            app.mimo_api_key = Some(key.to_string());
        }
        if let Some(key) = v.get("azure_speech_key").and_then(|k| k.as_str()) {
            conf.app.azure.speech_key = Some(key.to_string());
        }
        if let Some(region) = v.get("azure_speech_region").and_then(|r| r.as_str()) {
            conf.app.azure.speech_region = Some(region.to_string());
        }
        if let Some(key) = v.get("siliconflow_key").and_then(|k| k.as_str()) {
            conf.app.siliconflow.api_key = Some(key.to_string());
        }
        if let Some(key) = v.get("elevenlabs_key").and_then(|k| k.as_str()) {
            conf.app.elevenlabs.api_key = Some(key.to_string());
        }
        if let Some(model) = v.get("elevenlabs_model").and_then(|m| m.as_str()) {
            conf.app.elevenlabs.model_id = Some(model.to_string());
        }
    }

    // 更新素材源配置
    if let Some(v) = param.value.get("stock") {
        let app = &mut conf.app.app;
        if let Some(key) = v.get("pexels_api_key").and_then(|k| k.as_str()) {
            app.pexels_api_keys = Some(vec![key.to_string()]);
        }
        if let Some(key) = v.get("pixabay_api_key").and_then(|k| k.as_str()) {
            app.pixabay_api_keys = Some(vec![key.to_string()]);
        }
        if let Some(key) = v.get("coverr_api_key").and_then(|k| k.as_str()) {
            app.coverr_api_keys = Some(vec![key.to_string()]);
        }
    }

    // 更新 FFmpeg 配置
    if let Some(v) = param.value.get("ffmpeg") {
        let app = &mut conf.app.app;
        if let Some(path) = v.get("path").and_then(|p| p.as_str()) {
            app.ffmpeg_path = Some(path.to_string());
        }
    }

    // 尝试写回 TOML 文件
    if !conf_path.is_empty() {
        let toml_str = toml::to_string_pretty(&conf.app).unwrap_or_default();
        if std::fs::write(&conf_path, toml_str).is_err() {
            // 写文件失败时仅更新内存，不返回错误
        }
    }

    // 更新内存中的全局配置
    Config::set(conf);

    Ok(value!({
        "saved": true,
    }))
}

/// 根据当前 LLM provider 获取模型名称
fn get_current_model(app: &soma_core::config::AppSection) -> String {
    let provider = app.llm_provider.as_deref().unwrap_or("openai");
    match provider {
        "openai" => app.openai_model_name.as_deref().unwrap_or("gpt-4o-mini").to_string(),
        "deepseek" => app.deepseek_model_name.as_deref().unwrap_or("deepseek-chat").to_string(),
        "qwen" => app.qwen_model_name.as_deref().unwrap_or("qwen-turbo").to_string(),
        "gemini" => app.gemini_model_name.as_deref().unwrap_or("gemini-pro").to_string(),
        "moonshot" => app.moonshot_model_name.as_deref().unwrap_or("moonshot-v1-8k").to_string(),
        "azure" => app.azure_model_name.as_deref().unwrap_or("gpt-4o-mini").to_string(),
        "ollama" => app.ollama_model_name.as_deref().unwrap_or("llama3").to_string(),
        "groq" => app.groq_model_name.as_deref().unwrap_or("llama3-8b-8192").to_string(),
        "grok" => app.grok_model_name.as_deref().unwrap_or("grok-beta").to_string(),
        "minimax" => app.minimax_model_name.as_deref().unwrap_or("abab6.5s-chat").to_string(),
        "mimo" => app.mimo_model_name.as_deref().unwrap_or("mimo-chat").to_string(),
        _ => app.openai_model_name.as_deref().unwrap_or("gpt-4o-mini").to_string(),
    }
}

/// 根据当前 LLM provider 获取 API 密钥
fn get_current_api_key(app: &soma_core::config::AppSection) -> String {
    let provider = app.llm_provider.as_deref().unwrap_or("openai");
    match provider {
        "openai" => app.openai_api_key.as_deref().unwrap_or("").to_string(),
        "deepseek" => app.deepseek_api_key.as_deref().unwrap_or("").to_string(),
        "qwen" => app.qwen_api_key.as_deref().unwrap_or("").to_string(),
        "gemini" => app.gemini_api_key.as_deref().unwrap_or("").to_string(),
        "moonshot" => app.moonshot_api_key.as_deref().unwrap_or("").to_string(),
        "azure" => app.azure_api_key.as_deref().unwrap_or("").to_string(),
        "groq" => app.groq_api_key.as_deref().unwrap_or("").to_string(),
        "grok" => app.grok_api_key.as_deref().unwrap_or("").to_string(),
        "minimax" => app.minimax_api_key.as_deref().unwrap_or("").to_string(),
        "mimo" => app.mimo_api_key.as_deref().unwrap_or("").to_string(),
        _ => app.openai_api_key.as_deref().unwrap_or("").to_string(),
    }
}

/// 根据当前 LLM provider 获取基础 URL
fn get_current_base_url(app: &soma_core::config::AppSection) -> String {
    let provider = app.llm_provider.as_deref().unwrap_or("openai");
    match provider {
        "openai" => app.openai_base_url.as_deref().unwrap_or("https://api.openai.com/v1").to_string(),
        "deepseek" => app.deepseek_base_url.as_deref().unwrap_or("https://api.deepseek.com/v1").to_string(),
        "moonshot" => app.moonshot_base_url.as_deref().unwrap_or("https://api.moonshot.cn/v1").to_string(),
        "azure" => app.azure_base_url.as_deref().unwrap_or("").to_string(),
        "ollama" => app.ollama_base_url.as_deref().unwrap_or("http://localhost:11434/v1").to_string(),
        "groq" => app.groq_base_url.as_deref().unwrap_or("https://api.groq.com/openai/v1").to_string(),
        "grok" => app.grok_base_url.as_deref().unwrap_or("https://api.x.ai/v1").to_string(),
        "minimax" => app.minimax_base_url.as_deref().unwrap_or("https://api.minimax.chat/v1").to_string(),
        "mimo" => app.mimo_base_url.as_deref().unwrap_or("https://api.mimo.com/v1").to_string(),
        _ => app.openai_base_url.as_deref().unwrap_or("https://api.openai.com/v1").to_string(),
    }
}

/// 根据 provider 更新对应的模型名称
fn set_model_for_provider(app: &mut soma_core::config::AppSection, model: &str) {
    let provider = app.llm_provider.as_deref().unwrap_or("openai");
    match provider {
        "openai" => app.openai_model_name = Some(model.to_string()),
        "deepseek" => app.deepseek_model_name = Some(model.to_string()),
        "qwen" => app.qwen_model_name = Some(model.to_string()),
        "gemini" => app.gemini_model_name = Some(model.to_string()),
        "moonshot" => app.moonshot_model_name = Some(model.to_string()),
        "azure" => app.azure_model_name = Some(model.to_string()),
        "ollama" => app.ollama_model_name = Some(model.to_string()),
        "groq" => app.groq_model_name = Some(model.to_string()),
        "grok" => app.grok_model_name = Some(model.to_string()),
        "minimax" => app.minimax_model_name = Some(model.to_string()),
        "mimo" => app.mimo_model_name = Some(model.to_string()),
        _ => app.openai_model_name = Some(model.to_string()),
    }
}

/// 根据 provider 更新对应的 API 密钥
fn set_api_key_for_provider(app: &mut soma_core::config::AppSection, api_key: &str) {
    let provider = app.llm_provider.as_deref().unwrap_or("openai");
    match provider {
        "openai" => app.openai_api_key = Some(api_key.to_string()),
        "deepseek" => app.deepseek_api_key = Some(api_key.to_string()),
        "qwen" => app.qwen_api_key = Some(api_key.to_string()),
        "gemini" => app.gemini_api_key = Some(api_key.to_string()),
        "moonshot" => app.moonshot_api_key = Some(api_key.to_string()),
        "azure" => app.azure_api_key = Some(api_key.to_string()),
        "groq" => app.groq_api_key = Some(api_key.to_string()),
        "grok" => app.grok_api_key = Some(api_key.to_string()),
        "minimax" => app.minimax_api_key = Some(api_key.to_string()),
        "mimo" => app.mimo_api_key = Some(api_key.to_string()),
        _ => app.openai_api_key = Some(api_key.to_string()),
    }
}

/// 根据 provider 更新对应的基础 URL
fn set_base_url_for_provider(app: &mut soma_core::config::AppSection, base_url: &str) {
    let provider = app.llm_provider.as_deref().unwrap_or("openai");
    match provider {
        "openai" => app.openai_base_url = Some(base_url.to_string()),
        "deepseek" => app.deepseek_base_url = Some(base_url.to_string()),
        "moonshot" => app.moonshot_base_url = Some(base_url.to_string()),
        "azure" => app.azure_base_url = Some(base_url.to_string()),
        "ollama" => app.ollama_base_url = Some(base_url.to_string()),
        "groq" => app.groq_base_url = Some(base_url.to_string()),
        "grok" => app.grok_base_url = Some(base_url.to_string()),
        "minimax" => app.minimax_base_url = Some(base_url.to_string()),
        "mimo" => app.mimo_base_url = Some(base_url.to_string()),
        _ => app.openai_base_url = Some(base_url.to_string()),
    }
}
