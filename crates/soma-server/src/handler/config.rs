use crate::Config;
/// 配置 API 处理器
///
/// 处理前端设置页的配置读写请求：
/// - "get" → 获取当前配置
/// - "save" → 保存配置到文件
use tube::{Result, Value};
use tube_web::RequestParameter;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "get" => get_config(param).await,
        "save" => save_config(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

async fn get_config(_param: &RequestParameter) -> Result<Value> {
    let conf = Config::get();
    let app = &conf.app.app;

    Ok(value!({
        "app": {
            "name": app.name.as_deref().unwrap_or("Soma"),
            "version": "0.1.0",
            "host": app.host.as_deref().unwrap_or("0.0.0.0"),
            "port": app.port.unwrap_or(8090) as u32,
            "storage_path": app.storage_path.as_deref().unwrap_or("./storage"),
            "concurrent_tasks": app.max_concurrent_tasks.unwrap_or(5),
            "max_queued_tasks": app.max_queued_tasks.unwrap_or(100),
            "video_source": app.video_source.as_deref().unwrap_or("pexels"),
            "video_codec": app.video_codec.as_deref().unwrap_or("libx264"),
            "material_directory": app.material_directory.as_deref().unwrap_or(""),
            "edge_tts_timeout": app.edge_tts_timeout.unwrap_or(0.0),
            "endpoint": app.endpoint.as_deref().unwrap_or(""),
            "ffmpeg_path": app.ffmpeg_path.as_deref().unwrap_or("ffmpeg"),
            "imagemagick_path": app.imagemagick_path.as_deref().unwrap_or(""),
            "subtitle_provider": app.subtitle_provider.as_deref().unwrap_or("edge"),
            "narration_emotion_tags": app.narration_emotion_tags.unwrap_or(false),
            "enable_redis": app.enable_redis.unwrap_or(false),
            "redis_host": app.redis_host.as_deref().unwrap_or("127.0.0.1"),
            "redis_port": app.redis_port.unwrap_or(6379),
            "redis_db": app.redis_db.unwrap_or(0),
            "tls_verify": app.tls_verify.unwrap_or(true),
        },
        "llm": {
            "provider": app.llm_provider.as_deref().unwrap_or("openai"),
            "model": get_current_model(app),
            "api_key": get_current_api_key(app),
            "base_url": get_current_base_url(app),
            "api_version": app.azure_api_version.as_deref().unwrap_or(""),
            "secret_key": app.wenxin_secret_key.as_deref().unwrap_or(""),
            "account_id": app.cloudflare_account_id.as_deref().unwrap_or(""),
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
            "gemini_key": app.gemini_api_key.as_deref().unwrap_or(""),
            "volcengine_app_id": conf.app.volcengine.app_id.as_deref().unwrap_or(""),
            "volcengine_access_token": conf.app.volcengine.access_token.as_deref().unwrap_or(""),
            "volcengine_cluster": conf.app.volcengine.cluster.as_deref().unwrap_or("volcano_tts"),
            "xfyun_app_id": conf.app.xfyun.app_id.as_deref().unwrap_or(""),
            "xfyun_api_key": conf.app.xfyun.api_key.as_deref().unwrap_or(""),
            "xfyun_api_secret": conf.app.xfyun.api_secret.as_deref().unwrap_or(""),
            "fishspeech_base_url": conf.app.fishspeech.get_base_url(),
            "fishspeech_api_key": conf.app.fishspeech.get_api_key(),
            "fishspeech_reference_id": conf.app.fishspeech.get_reference_id(),
        },
        "stock": {
            "pexels_api_key": app.pexels_api_keys.as_ref().and_then(|v| v.first()).unwrap_or(&String::new()).clone(),
            "pixabay_api_key": app.pixabay_api_keys.as_ref().and_then(|v| v.first()).unwrap_or(&String::new()).clone(),
            "coverr_api_key": app.coverr_api_keys.as_ref().and_then(|v| v.first()).unwrap_or(&String::new()).clone(),
        },
        "aivideo": {
            "zhipu_video_api_key": app.zhipu_video_api_key.as_deref().unwrap_or(""),
            "zhipu_video_model": app.zhipu_video_model.as_deref().unwrap_or("cogvideox-flash"),
            "kling_access_key": app.kling_access_key.as_deref().unwrap_or(""),
            "kling_secret_key": app.kling_secret_key.as_deref().unwrap_or(""),
            "kling_video_model": app.kling_video_model.as_deref().unwrap_or("kling-v2-master"),
            "minimax_video_api_key": app.minimax_video_api_key.as_deref().unwrap_or(""),
            "minimax_video_model": app.minimax_video_model.as_deref().unwrap_or("MiniMax-Hailuo-2.3"),
            "video_gen_timeout": app.video_gen_timeout.unwrap_or(300),
        },
        "ffmpeg": {
            "path": app.ffmpeg_path.as_deref().unwrap_or("ffmpeg"),
            "threads": conf.app.app.concurrent_tasks.unwrap_or(4) as u32,
        },
        "whisper": {
            "provider": "local",
            "model": conf.app.whisper.model_size.as_deref().unwrap_or("base"),
            "device": conf.app.whisper.device.as_deref().unwrap_or("cpu"),
            "compute_type": conf.app.whisper.compute_type.as_deref().unwrap_or("int8"),
            "endpoint": "",
        },
        "proxy": {
            "http": conf.app.proxy.http.as_deref().unwrap_or(""),
            "https": conf.app.proxy.https.as_deref().unwrap_or(""),
        },
        "ui": {
            "hide_log": conf.app.ui.hide_log.unwrap_or(false),
            "subtitle_position": conf.app.ui.subtitle_position.as_deref().unwrap_or("bottom"),
            "custom_position": conf.app.ui.custom_position.unwrap_or(70.0),
            "upload_post_enabled": conf.app.ui.upload_post_enabled.unwrap_or(false),
            "upload_post_api_key": conf.app.ui.upload_post_api_key.as_deref().unwrap_or(""),
            "upload_post_username": conf.app.ui.upload_post_username.as_deref().unwrap_or(""),
            "upload_post_platforms": conf.app.ui.upload_post_platforms.as_ref().map(|v| v.iter().map(|s| value!(s.clone())).collect::<Vec<Value>>()).unwrap_or_default(),
            "upload_post_auto_upload": conf.app.ui.upload_post_auto_upload.unwrap_or(false),
        },
    }))
}

async fn save_config(param: &RequestParameter) -> Result<Value> {
    let conf_path = Config::get_conf_path();
    let mut conf = Config::get();

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
        if let Some(mq) = v.get("max_queued_tasks").and_then(|c| c.as_u64()) {
            app.max_queued_tasks = Some(mq as usize);
        }
        if let Some(vs) = v.get("video_source").and_then(|s| s.as_str()) {
            app.video_source = Some(vs.to_string());
        }
        if let Some(vc) = v.get("video_codec").and_then(|s| s.as_str()) {
            app.video_codec = Some(vc.to_string());
        }
        if let Some(md) = v.get("material_directory").and_then(|s| s.as_str()) {
            app.material_directory = Some(md.to_string());
        }
        if let Some(et) = v.get("edge_tts_timeout").and_then(|f| f.as_f64()) {
            app.edge_tts_timeout = Some(et);
        }
        if let Some(ep) = v.get("endpoint").and_then(|s| s.as_str()) {
            app.endpoint = Some(ep.to_string());
        }
        if let Some(fp) = v.get("ffmpeg_path").and_then(|s| s.as_str()) {
            app.ffmpeg_path = Some(fp.to_string());
        }
        if let Some(ip) = v.get("imagemagick_path").and_then(|s| s.as_str()) {
            app.imagemagick_path = Some(ip.to_string());
        }
        if let Some(sp) = v.get("subtitle_provider").and_then(|s| s.as_str()) {
            app.subtitle_provider = Some(sp.to_string());
        }
        if let Some(er) = v.get("enable_redis").and_then(|b| b.as_bool()) {
            app.enable_redis = Some(er);
        }
        if let Some(rh) = v.get("redis_host").and_then(|s| s.as_str()) {
            app.redis_host = Some(rh.to_string());
        }
        if let Some(rp) = v.get("redis_port").and_then(|n| n.as_u64()) {
            app.redis_port = Some(rp as u32);
        }
        if let Some(rd) = v.get("redis_db").and_then(|n| n.as_u64()) {
            app.redis_db = Some(rd as u32);
        }
        if let Some(tv) = v.get("tls_verify").and_then(|b| b.as_bool()) {
            app.tls_verify = Some(tv);
        }
        if let Some(net) = v.get("narration_emotion_tags").and_then(|b| b.as_bool()) {
            app.narration_emotion_tags = Some(net);
        }
    }

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
        if let Some(secret_key) = v.get("secret_key").and_then(|k| k.as_str()) {
            let provider = app.llm_provider.as_deref().unwrap_or("openai");
            if provider == "wenxin" {
                app.wenxin_secret_key = Some(secret_key.to_string());
            }
        }
        if let Some(account_id) = v.get("account_id").and_then(|a| a.as_str()) {
            let provider = app.llm_provider.as_deref().unwrap_or("openai");
            if provider == "cloudflare" {
                app.cloudflare_account_id = Some(account_id.to_string());
                app.openai_base_url = Some(format!(
                    "https://api.cloudflare.com/client/v4/accounts/{}",
                    account_id
                ));
            }
        }
    }

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
        if let Some(key) = v.get("gemini_key").and_then(|k| k.as_str()) {
            app.gemini_api_key = Some(key.to_string());
        }
        if let Some(id) = v.get("volcengine_app_id").and_then(|k| k.as_str()) {
            conf.app.volcengine.app_id = Some(id.to_string());
        }
        if let Some(token) = v.get("volcengine_access_token").and_then(|k| k.as_str()) {
            conf.app.volcengine.access_token = Some(token.to_string());
        }
        if let Some(cluster) = v.get("volcengine_cluster").and_then(|k| k.as_str()) {
            conf.app.volcengine.cluster = Some(cluster.to_string());
        }
        if let Some(id) = v.get("xfyun_app_id").and_then(|k| k.as_str()) {
            conf.app.xfyun.app_id = Some(id.to_string());
        }
        if let Some(key) = v.get("xfyun_api_key").and_then(|k| k.as_str()) {
            conf.app.xfyun.api_key = Some(key.to_string());
        }
        if let Some(secret) = v.get("xfyun_api_secret").and_then(|k| k.as_str()) {
            conf.app.xfyun.api_secret = Some(secret.to_string());
        }
        if let Some(url) = v.get("fishspeech_base_url").and_then(|u| u.as_str()) {
            conf.app.fishspeech.base_url = Some(url.to_string());
        }
        if let Some(key) = v.get("fishspeech_api_key").and_then(|k| k.as_str()) {
            conf.app.fishspeech.api_key = Some(key.to_string());
        }
        if let Some(id) = v.get("fishspeech_reference_id").and_then(|i| i.as_str()) {
            conf.app.fishspeech.reference_id = Some(id.to_string());
        }
    }

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

    if let Some(v) = param.value.get("aivideo") {
        let app = &mut conf.app.app;
        if let Some(key) = v.get("zhipu_video_api_key").and_then(|k| k.as_str()) {
            app.zhipu_video_api_key = Some(key.to_string());
        }
        if let Some(model) = v.get("zhipu_video_model").and_then(|m| m.as_str()) {
            app.zhipu_video_model = Some(model.to_string());
        }
        if let Some(key) = v.get("kling_access_key").and_then(|k| k.as_str()) {
            app.kling_access_key = Some(key.to_string());
        }
        if let Some(key) = v.get("kling_secret_key").and_then(|k| k.as_str()) {
            app.kling_secret_key = Some(key.to_string());
        }
        if let Some(model) = v.get("kling_video_model").and_then(|m| m.as_str()) {
            app.kling_video_model = Some(model.to_string());
        }
        if let Some(key) = v.get("minimax_video_api_key").and_then(|k| k.as_str()) {
            app.minimax_video_api_key = Some(key.to_string());
        }
        if let Some(model) = v.get("minimax_video_model").and_then(|m| m.as_str()) {
            app.minimax_video_model = Some(model.to_string());
        }
        if let Some(timeout) = v.get("video_gen_timeout").and_then(|t| t.as_u64()) {
            app.video_gen_timeout = Some(timeout);
        }
    }

    if let Some(v) = param.value.get("ffmpeg") {
        let app = &mut conf.app.app;
        if let Some(path) = v.get("path").and_then(|p| p.as_str()) {
            app.ffmpeg_path = Some(path.to_string());
        }
        if let Some(threads) = v.get("threads").and_then(|t| t.as_u64()) {
            app.concurrent_tasks = Some(threads as usize);
        }
    }

    if let Some(v) = param.value.get("whisper") {
        let ws = &mut conf.app.whisper;
        if let Some(model) = v.get("model").and_then(|m| m.as_str()) {
            ws.model_size = Some(model.to_string());
        }
        if let Some(device) = v.get("device").and_then(|d| d.as_str()) {
            ws.device = Some(device.to_string());
        }
        if let Some(ct) = v.get("compute_type").and_then(|c| c.as_str()) {
            ws.compute_type = Some(ct.to_string());
        }
    }

    if let Some(v) = param.value.get("proxy") {
        let ps = &mut conf.app.proxy;
        if let Some(http) = v.get("http").and_then(|h| h.as_str()) {
            ps.http = Some(http.to_string());
        }
        if let Some(https) = v.get("https").and_then(|h| h.as_str()) {
            ps.https = Some(https.to_string());
        }
    }

    if let Some(v) = param.value.get("ui") {
        let ui = &mut conf.app.ui;
        if let Some(hl) = v.get("hide_log").and_then(|b| b.as_bool()) {
            ui.hide_log = Some(hl);
        }
        if let Some(sp) = v.get("subtitle_position").and_then(|s| s.as_str()) {
            ui.subtitle_position = Some(sp.to_string());
        }
        if let Some(cp) = v.get("custom_position").and_then(|n| n.as_f64()) {
            ui.custom_position = Some(cp);
        }
        if let Some(ue) = v.get("upload_post_enabled").and_then(|b| b.as_bool()) {
            ui.upload_post_enabled = Some(ue);
        }
        if let Some(ak) = v.get("upload_post_api_key").and_then(|s| s.as_str()) {
            ui.upload_post_api_key = Some(ak.to_string());
        }
        if let Some(un) = v.get("upload_post_username").and_then(|s| s.as_str()) {
            ui.upload_post_username = Some(un.to_string());
        }
        if let Some(platforms) = v.get("upload_post_platforms").and_then(|p| p.as_array()) {
            ui.upload_post_platforms = Some(platforms.iter().filter_map(|v| v.as_str()).collect());
        }
        if let Some(au) = v.get("upload_post_auto_upload").and_then(|b| b.as_bool()) {
            ui.upload_post_auto_upload = Some(au);
        }
    }

    if !conf_path.is_empty() {
        let toml_str = toml::to_string_pretty(&conf.app).unwrap_or_default();
        if let Err(e) = std::fs::write(&conf_path, toml_str) {
            log::error!("配置写入文件失败 {}: {}", conf_path, e);
        }
    }

    Config::set(conf);

    match Config::save() {
        Ok(_) => Ok(value!({
            "saved": true,
        })),
        Err(e) => Err(error!("配置保存失败: {}", e)),
    }
}

fn get_current_model(app: &soma_core::config::AppSection) -> String {
    let provider = app.llm_provider.as_deref().unwrap_or("openai");
    match provider {
        "openai" => app
            .openai_model_name
            .as_deref()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
        "deepseek" => app
            .deepseek_model_name
            .as_deref()
            .unwrap_or("deepseek-chat")
            .to_string(),
        "qwen" => app
            .qwen_model_name
            .as_deref()
            .unwrap_or("qwen-turbo")
            .to_string(),
        "gemini" => app
            .gemini_model_name
            .as_deref()
            .unwrap_or("gemini-pro")
            .to_string(),
        "moonshot" => app
            .moonshot_model_name
            .as_deref()
            .unwrap_or("moonshot-v1-8k")
            .to_string(),
        "azure" => app
            .azure_model_name
            .as_deref()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
        "ollama" => app
            .ollama_model_name
            .as_deref()
            .unwrap_or("llama3")
            .to_string(),
        "groq" => app
            .groq_model_name
            .as_deref()
            .unwrap_or("llama3-8b-8192")
            .to_string(),
        "grok" => app
            .grok_model_name
            .as_deref()
            .unwrap_or("grok-beta")
            .to_string(),
        "minimax" => app
            .minimax_model_name
            .as_deref()
            .unwrap_or("abab6.5s-chat")
            .to_string(),
        "mimo" => app
            .mimo_model_name
            .as_deref()
            .unwrap_or("mimo-chat")
            .to_string(),
        "doubao" => app
            .doubao_model_name
            .as_deref()
            .unwrap_or("doubao-pro-32k")
            .to_string(),
        "zhipu" => app
            .zhipu_model_name
            .as_deref()
            .unwrap_or("glm-4-flash")
            .to_string(),
        "wenxin" => app
            .wenxin_model_name
            .as_deref()
            .unwrap_or("ernie-4.0-8k")
            .to_string(),
        "xunfei" => app
            .xunfei_model_name
            .as_deref()
            .unwrap_or("generalv3.5")
            .to_string(),
        "hunyuan" => app
            .hunyuan_model_name
            .as_deref()
            .unwrap_or("hunyuan-turbo")
            .to_string(),
        "oneapi" => app
            .oneapi_model_name
            .as_deref()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
        "aihubmix" => app
            .aihubmix_model_name
            .as_deref()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
        "evolink" => app
            .evolink_model_name
            .as_deref()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
        "aiml" | "aimlapi" => app
            .aimlapi_model_name
            .as_deref()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
        "modelscope" => app
            .modelscope_model_name
            .as_deref()
            .unwrap_or("qwen-turbo")
            .to_string(),
        "pollinations" => app
            .pollinations_model_name
            .as_deref()
            .unwrap_or("openai")
            .to_string(),
        "g4f" => app
            .g4f_model_name
            .as_deref()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
        "cloudflare" => app
            .openai_model_name
            .as_deref()
            .unwrap_or("@cf/meta/llama-3-8b-instruct")
            .to_string(),
        "litellm" => app
            .litellm_model_name
            .as_deref()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
        _ => app
            .openai_model_name
            .as_deref()
            .unwrap_or("gpt-4o-mini")
            .to_string(),
    }
}

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
        "doubao" => app.doubao_api_key.as_deref().unwrap_or("").to_string(),
        "zhipu" => app.zhipu_api_key.as_deref().unwrap_or("").to_string(),
        "wenxin" => app.wenxin_api_key.as_deref().unwrap_or("").to_string(),
        "xunfei" => app.xunfei_api_key.as_deref().unwrap_or("").to_string(),
        "hunyuan" => app.hunyuan_api_key.as_deref().unwrap_or("").to_string(),
        "oneapi" => app.oneapi_api_key.as_deref().unwrap_or("").to_string(),
        "aihubmix" => app.aihubmix_api_key.as_deref().unwrap_or("").to_string(),
        "evolink" => app.evolink_api_key.as_deref().unwrap_or("").to_string(),
        "aiml" | "aimlapi" => app.aimlapi_api_key.as_deref().unwrap_or("").to_string(),
        "modelscope" => app.modelscope_api_key.as_deref().unwrap_or("").to_string(),
        "pollinations" => app
            .pollinations_api_key
            .as_deref()
            .unwrap_or("")
            .to_string(),
        "g4f" => "".to_string(),
        "cloudflare" => app.openai_api_key.as_deref().unwrap_or("").to_string(),
        "litellm" => app.oneapi_api_key.as_deref().unwrap_or("").to_string(),
        _ => app.openai_api_key.as_deref().unwrap_or("").to_string(),
    }
}

fn get_current_base_url(app: &soma_core::config::AppSection) -> String {
    let provider = app.llm_provider.as_deref().unwrap_or("openai");
    match provider {
        "openai" => app
            .openai_base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
            .to_string(),
        "deepseek" => app
            .deepseek_base_url
            .as_deref()
            .unwrap_or("https://api.deepseek.com/v1")
            .to_string(),
        "qwen" => app
            .qwen_base_url
            .as_deref()
            .unwrap_or("https://dashscope.aliyuncs.com/compatible-mode/v1")
            .to_string(),
        "moonshot" => app
            .moonshot_base_url
            .as_deref()
            .unwrap_or("https://api.moonshot.cn/v1")
            .to_string(),
        "azure" => app.azure_base_url.as_deref().unwrap_or("").to_string(),
        "ollama" => app
            .ollama_base_url
            .as_deref()
            .unwrap_or("http://localhost:11434/v1")
            .to_string(),
        "groq" => app
            .groq_base_url
            .as_deref()
            .unwrap_or("https://api.groq.com/openai/v1")
            .to_string(),
        "grok" => app
            .grok_base_url
            .as_deref()
            .unwrap_or("https://api.x.ai/v1")
            .to_string(),
        "minimax" => app
            .minimax_base_url
            .as_deref()
            .unwrap_or("https://api.minimax.chat/v1")
            .to_string(),
        "mimo" => app
            .mimo_base_url
            .as_deref()
            .unwrap_or("https://api.mimo.com/v1")
            .to_string(),
        "doubao" => app
            .doubao_base_url
            .as_deref()
            .unwrap_or("https://ark.cn-beijing.volces.com/api/v3")
            .to_string(),
        "zhipu" => app
            .zhipu_base_url
            .as_deref()
            .unwrap_or("https://open.bigmodel.cn/api/paas/v4")
            .to_string(),
        "wenxin" => app
            .wenxin_base_url
            .as_deref()
            .unwrap_or("https://aip.baidubce.com")
            .to_string(),
        "xunfei" => app
            .xunfei_base_url
            .as_deref()
            .unwrap_or("https://spark-api.xf-yun.com/v1")
            .to_string(),
        "hunyuan" => app
            .hunyuan_base_url
            .as_deref()
            .unwrap_or("https://hunyuan.tencentcloudapi.com")
            .to_string(),
        "oneapi" => app
            .oneapi_base_url
            .as_deref()
            .unwrap_or("http://localhost:3000/v1")
            .to_string(),
        "aihubmix" => app
            .aihubmix_base_url
            .as_deref()
            .unwrap_or("https://api.aihubmix.com/v1")
            .to_string(),
        "evolink" => app
            .evolink_base_url
            .as_deref()
            .unwrap_or("https://api.evolink.com/v1")
            .to_string(),
        "aiml" | "aimlapi" => app
            .aimlapi_base_url
            .as_deref()
            .unwrap_or("https://api.aimlapi.com/v1")
            .to_string(),
        "modelscope" => app
            .modelscope_base_url
            .as_deref()
            .unwrap_or("https://dashscope.aliyuncs.com/compatible-mode/v1")
            .to_string(),
        "pollinations" => app
            .pollinations_base_url
            .as_deref()
            .unwrap_or("https://text.pollinations.ai")
            .to_string(),
        "g4f" => "".to_string(),
        "cloudflare" => app
            .openai_base_url
            .as_deref()
            .unwrap_or("https://api.cloudflare.com/client/v4/accounts")
            .to_string(),
        "litellm" => "http://localhost:4000".to_string(),
        _ => app
            .openai_base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
            .to_string(),
    }
}

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
        "doubao" => app.doubao_model_name = Some(model.to_string()),
        "zhipu" => app.zhipu_model_name = Some(model.to_string()),
        "wenxin" => app.wenxin_model_name = Some(model.to_string()),
        "xunfei" => app.xunfei_model_name = Some(model.to_string()),
        "hunyuan" => app.hunyuan_model_name = Some(model.to_string()),
        "oneapi" => app.oneapi_model_name = Some(model.to_string()),
        "aihubmix" => app.aihubmix_model_name = Some(model.to_string()),
        "evolink" => app.evolink_model_name = Some(model.to_string()),
        "aiml" | "aimlapi" => app.aimlapi_model_name = Some(model.to_string()),
        "modelscope" => app.modelscope_model_name = Some(model.to_string()),
        "pollinations" => app.pollinations_model_name = Some(model.to_string()),
        "g4f" => app.g4f_model_name = Some(model.to_string()),
        "cloudflare" => app.openai_model_name = Some(model.to_string()),
        "litellm" => app.litellm_model_name = Some(model.to_string()),
        _ => app.openai_model_name = Some(model.to_string()),
    }
}

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
        "doubao" => app.doubao_api_key = Some(api_key.to_string()),
        "zhipu" => app.zhipu_api_key = Some(api_key.to_string()),
        "wenxin" => app.wenxin_api_key = Some(api_key.to_string()),
        "xunfei" => app.xunfei_api_key = Some(api_key.to_string()),
        "hunyuan" => app.hunyuan_api_key = Some(api_key.to_string()),
        "oneapi" => app.oneapi_api_key = Some(api_key.to_string()),
        "aihubmix" => app.aihubmix_api_key = Some(api_key.to_string()),
        "evolink" => app.evolink_api_key = Some(api_key.to_string()),
        "aiml" | "aimlapi" => app.aimlapi_api_key = Some(api_key.to_string()),
        "modelscope" => app.modelscope_api_key = Some(api_key.to_string()),
        "pollinations" => app.pollinations_api_key = Some(api_key.to_string()),
        "cloudflare" => app.openai_api_key = Some(api_key.to_string()),
        "litellm" => app.oneapi_api_key = Some(api_key.to_string()),
        _ => app.openai_api_key = Some(api_key.to_string()),
    }
}

fn set_base_url_for_provider(app: &mut soma_core::config::AppSection, base_url: &str) {
    let provider = app.llm_provider.as_deref().unwrap_or("openai");
    match provider {
        "openai" => app.openai_base_url = Some(base_url.to_string()),
        "deepseek" => app.deepseek_base_url = Some(base_url.to_string()),
        "qwen" => app.qwen_base_url = Some(base_url.to_string()),
        "moonshot" => app.moonshot_base_url = Some(base_url.to_string()),
        "azure" => app.azure_base_url = Some(base_url.to_string()),
        "ollama" => app.ollama_base_url = Some(base_url.to_string()),
        "groq" => app.groq_base_url = Some(base_url.to_string()),
        "grok" => app.grok_base_url = Some(base_url.to_string()),
        "minimax" => app.minimax_base_url = Some(base_url.to_string()),
        "mimo" => app.mimo_base_url = Some(base_url.to_string()),
        "doubao" => app.doubao_base_url = Some(base_url.to_string()),
        "zhipu" => app.zhipu_base_url = Some(base_url.to_string()),
        "wenxin" => app.wenxin_base_url = Some(base_url.to_string()),
        "xunfei" => app.xunfei_base_url = Some(base_url.to_string()),
        "hunyuan" => app.hunyuan_base_url = Some(base_url.to_string()),
        "oneapi" => app.oneapi_base_url = Some(base_url.to_string()),
        "aihubmix" => app.aihubmix_base_url = Some(base_url.to_string()),
        "evolink" => app.evolink_base_url = Some(base_url.to_string()),
        "aiml" | "aimlapi" => app.aimlapi_base_url = Some(base_url.to_string()),
        "modelscope" => app.modelscope_base_url = Some(base_url.to_string()),
        "pollinations" => app.pollinations_base_url = Some(base_url.to_string()),
        "cloudflare" => app.openai_base_url = Some(base_url.to_string()),
        "litellm" => { /* litellm uses oneapi_base_url as proxy */ }
        _ => app.openai_base_url = Some(base_url.to_string()),
    }
}
