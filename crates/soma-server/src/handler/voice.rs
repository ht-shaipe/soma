/// 语音列表与预览 API 处理器
///
/// - list: 返回各 TTS 引擎可用的语音列表
/// - list_cloned: 返回已克隆的语音列表
/// - delete_cloned: 删除已克隆的语音
/// - preview: 生成语音预览音频并返回
use actix_web::{HttpRequest, HttpResponse};
use actix_multipart::Multipart;
use futures::StreamExt;
use tube::{Result, Value};
use tube_web::RequestParameter;
use crate::Config;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_voices(param).await,
        "list_cloned" => list_cloned_voices(param).await,
        "delete_cloned" => delete_cloned_voice(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

async fn list_voices(_param: &RequestParameter) -> Result<Value> {
    let conf = Config::get();

    // Edge TTS 内置语音列表（常用中文+英文）
    let edge_voices = vec![
        value!({"name": "zh-CN-XiaoxiaoNeural", "label": "晓晓 (女, 中文)"}),
        value!({"name": "zh-CN-XiaoyiNeural", "label": "晓伊 (女, 中文)"}),
        value!({"name": "zh-CN-YunjianNeural", "label": "云健 (男, 中文)"}),
        value!({"name": "zh-CN-YunxiNeural", "label": "云希 (男, 中文)"}),
        value!({"name": "zh-CN-YunxiaNeural", "label": "云夏 (男, 中文)"}),
        value!({"name": "zh-CN-YunyangNeural", "label": "云扬 (男, 中文, 新闻)"}),
        value!({"name": "en-US-JennyNeural", "label": "Jenny (女, 英文)"}),
        value!({"name": "en-US-GuyNeural", "label": "Guy (男, 英文)"}),
        value!({"name": "en-US-AriaNeural", "label": "Aria (女, 英文)"}),
        value!({"name": "en-US-DavisNeural", "label": "Davis (男, 英文)"}),
        value!({"name": "ja-JP-NanamiNeural", "label": "Nanami (女, 日文)"}),
        value!({"name": "ko-KR-SunHiNeural", "label": "SunHi (女, 韩文)"}),
        value!({"name": "no-voice", "label": "无语音 (静音)"}),
    ];

    // SiliconFlow 语音
    let sf_key = conf.app.siliconflow.api_key.as_deref().unwrap_or("");
    let sf_voices = if !sf_key.is_empty() {
        vec![
            value!({"name": "siliconflow:FunAudioLLM/CosyVoice2-0.5B:alice", "label": "Alice (SiliconFlow)"}),
            value!({"name": "siliconflow:FunAudioLLM/CosyVoice2-0.5B:bob", "label": "Bob (SiliconFlow)"}),
            value!({"name": "siliconflow:FunAudioLLM/CosyVoice2-0.5B:cathy", "label": "Cathy (SiliconFlow)"}),
        ]
    } else {
        vec![]
    };

    let el_key = conf.app.elevenlabs.api_key.as_deref().unwrap_or("");
    let el_voices = if !el_key.is_empty() {
        vec![
            value!({"name": "elevenlabs:21m00Tcm4TlvDq8ikWAM:Rachel", "label": "Rachel (ElevenLabs)"}),
            value!({"name": "elevenlabs:AZnzlk1XvdvUeBnXmlld:Domi", "label": "Domi (ElevenLabs)"}),
            value!({"name": "elevenlabs:EXAVITQu4vr4xnSDxMaL:Bella", "label": "Bella (ElevenLabs)"}),
        ]
    } else {
        vec![]
    };

    // MiMo 语音
    let mimo_key = conf.app.app.mimo_api_key.as_deref().unwrap_or("");
    let mimo_voices = if !mimo_key.is_empty() {
        vec![
            value!({"name": "mimo:almara", "label": "Almara (MiMo)"}),
            value!({"name": "mimo:brad", "label": "Brad (MiMo)"}),
        ]
    } else {
        vec![]
    };

    // Gemini 语音
    let gemini_key = conf.app.app.gemini_api_key.as_deref().unwrap_or("");
    let gemini_voices = if !gemini_key.is_empty() {
        vec![
            value!({"name": "gemini:Zephyr", "label": "Zephyr (Gemini)"}),
            value!({"name": "gemini:Puck", "label": "Puck (Gemini)"}),
            value!({"name": "gemini:Charon", "label": "Charon (Gemini)"}),
            value!({"name": "gemini:Kore", "label": "Kore (Gemini)"}),
            value!({"name": "gemini:Fenrir", "label": "Fenrir (Gemini)"}),
            value!({"name": "gemini:Leda", "label": "Leda (Gemini)"}),
            value!({"name": "gemini:Orus", "label": "Orus (Gemini)"}),
            value!({"name": "gemini:Aoede", "label": "Aoede (Gemini)"}),
        ]
    } else {
        vec![]
    };

    // Azure 语音
    let azure_key = conf.app.azure.speech_key.as_deref().unwrap_or("");
    let azure_voices = if !azure_key.is_empty() {
        vec![
            value!({"name": "azure:zh-CN-XiaoxiaoNeural", "label": "晓晓 (Azure, 女, 中文)"}),
            value!({"name": "azure:zh-CN-YunxiNeural", "label": "云希 (Azure, 男, 中文)"}),
            value!({"name": "azure:zh-CN-XiaoyiNeural", "label": "晓伊 (Azure, 女, 中文)"}),
            value!({"name": "azure:zh-CN-YunjianNeural", "label": "云健 (Azure, 男, 中文)"}),
            value!({"name": "azure:en-US-JennyNeural", "label": "Jenny (Azure, 女, 英文)"}),
            value!({"name": "azure:en-US-GuyNeural", "label": "Guy (Azure, 男, 英文)"}),
            value!({"name": "azure:ja-JP-NanamiNeural", "label": "Nanami (Azure, 女, 日文)"}),
            value!({"name": "azure:ko-KR-SunHiNeural", "label": "SunHi (Azure, 女, 韩文)"}),
        ]
    } else {
        vec![]
    };

    // 火山引擎语音
    let volc_appid = conf.app.volcengine.app_id.as_deref().unwrap_or("");
    let volc_token = conf.app.volcengine.access_token.as_deref().unwrap_or("");
    let volc_voices = if !volc_appid.is_empty() && !volc_token.is_empty() {
        vec![
            value!({"name": "volcengine:BV700_streaming", "label": "灿灿 (火山引擎, 女, 中文)"}),
            value!({"name": "volcengine:BV701_streaming", "label": "擎苍 (火山引擎, 男, 中文)"}),
            value!({"name": "volcengine:BV705_streaming", "label": "炀炀 (火山引擎, 女, 中文)"}),
            value!({"name": "volcengine:BV406_streaming", "label": "知性姐姐 (火山引擎, 女, 中文)"}),
            value!({"name": "volcengine:BV407_streaming", "label": "知性小哥 (火山引擎, 男, 中文)"}),
        ]
    } else {
        vec![]
    };

    // 科大讯飞语音
    let xfyun_appid = conf.app.xfyun.app_id.as_deref().unwrap_or("");
    let xfyun_key = conf.app.xfyun.api_key.as_deref().unwrap_or("");
    let xfyun_voices = if !xfyun_appid.is_empty() && !xfyun_key.is_empty() {
        vec![
            value!({"name": "xfyun:xiaoyan", "label": "小燕 (讯飞, 女, 中文)"}),
            value!({"name": "xfyun:aisjiuxu", "label": "许久 (讯飞, 男, 中文)"}),
            value!({"name": "xfyun:aisxinying", "label": "小颖 (讯飞, 女, 中文)"}),
            value!({"name": "xfyun:aisbabyxu", "label": "小X (讯飞, 男, 中文)"}),
        ]
    } else {
        vec![]
    };

    // Fish-Speech S2 语音（自部署 API Server 或 Fish Audio 云端）
    let fs_base = conf.app.fishspeech.get_base_url();
    let mut fishspeech_voices: Vec<Value> = vec![];
    if !fs_base.is_empty() {
        fishspeech_voices.push(value!({
            "name": "fishspeech:",
            "label": "Fish-Speech S2 (默认音色)",
        }));
        let fs_ref_id = conf.app.fishspeech.get_reference_id();
        if !fs_ref_id.is_empty() {
            fishspeech_voices.push(value!({
                "name": format!("fishspeech:{}", fs_ref_id),
                "label": format!("{} (Fish-Speech 参考音色)", fs_ref_id),
            }));
        }
    }

    let mut all_voices = edge_voices;
    all_voices.extend(sf_voices);
    all_voices.extend(el_voices);
    all_voices.extend(mimo_voices);
    all_voices.extend(gemini_voices);
    all_voices.extend(azure_voices);
    all_voices.extend(volc_voices);
    all_voices.extend(xfyun_voices);
    all_voices.extend(fishspeech_voices);

    // 添加已克隆的声音
    if let Ok(cloned) = soma_tts::voice_clone::list_cloned_voices() {
        for v in &cloned {
            all_voices.push(value!({
                "name": soma_tts::voice_clone::cloned_voice_name(v),
                "label": format!("{} ({}克隆)", v.name, v.engine),
            }));
        }
    }

    Ok(value!({
        "voices": all_voices,
    }))
}

/// 语音预览端点（GET /api/v1/voices/preview?voice=xxx&text=yyy）
///
/// 使用指定 TTS 引擎生成短音频并返回 MP3 数据流
pub async fn preview_voice(req: HttpRequest) -> HttpResponse {
    let query = req.query_string();
    let voice_name = extract_query_param(query, "voice").unwrap_or_default();
    let text = extract_query_param(query, "text").unwrap_or_else(|| "这是一段语音预览。".to_string());

    if voice_name.is_empty() || voice_name == "no-voice" {
        return HttpResponse::BadRequest().body("voice parameter required");
    }

    let conf = Config::get();
    let tmp_dir = std::env::temp_dir().join("soma_voice_preview");
    std::fs::create_dir_all(&tmp_dir).ok();
    let tmp_file = tmp_dir.join(format!("preview_{}.mp3", voice_name.replace(':', "_")));
    let tmp_path = tmp_file.to_string_lossy().to_string();

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build() {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().body("runtime error"),
    };
    let local = tokio::task::LocalSet::new();

    let result = if soma_tts::voices::is_siliconflow_voice(&voice_name) {
        let sf_key = conf.app.siliconflow.api_key.as_deref().unwrap_or("");
        let tts = soma_tts::siliconflow_tts::SiliconflowTts::new(sf_key);
        local.block_on(&rt, soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_elevenlabs_voice(&voice_name) {
        let el_key = conf.app.elevenlabs.api_key.as_deref().unwrap_or("");
        let el_model = conf.app.elevenlabs.model_id.as_deref().unwrap_or("eleven_multilingual_v2");
        let tts = soma_tts::elevenlabs_tts::ElevenlabsTts::new(el_key, el_model);
        local.block_on(&rt, soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_mimo_voice(&voice_name) {
        let mimo_key = conf.app.app.mimo_api_key.as_deref().unwrap_or("");
        let tts = soma_tts::mimo_tts::MimoTts::new(mimo_key, "", "", "");
        local.block_on(&rt, soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_gemini_voice(&voice_name) {
        let gemini_key = conf.app.app.gemini_api_key.as_deref().unwrap_or("");
        let tts = soma_tts::gemini_tts::GeminiTts::new(gemini_key, "", "");
        local.block_on(&rt, soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_azure_voice(&voice_name) {
        let azure_key = conf.app.azure.speech_key.as_deref().unwrap_or("");
        let azure_region = conf.app.azure.speech_region.as_deref().unwrap_or("eastasia");
        let tts = soma_tts::azure_tts::AzureTts::new(azure_key, azure_region);
        local.block_on(&rt, soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_volcengine_voice(&voice_name) {
        let volc_appid = conf.app.volcengine.app_id.as_deref().unwrap_or("");
        let volc_token = conf.app.volcengine.access_token.as_deref().unwrap_or("");
        let volc_cluster = conf.app.volcengine.cluster.as_deref().unwrap_or("volcano_tts");
        let tts = soma_tts::volcengine_tts::VolcengineTts::new(volc_appid, volc_token, volc_cluster);
        local.block_on(&rt, soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_xfyun_voice(&voice_name) {
        let xfyun_appid = conf.app.xfyun.app_id.as_deref().unwrap_or("");
        let xfyun_key = conf.app.xfyun.api_key.as_deref().unwrap_or("");
        let xfyun_secret = conf.app.xfyun.api_secret.as_deref().unwrap_or("");
        let tts = soma_tts::xfyun_tts::XfyunTts::new(xfyun_appid, xfyun_key, xfyun_secret);
        local.block_on(&rt, soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_fishspeech_voice(&voice_name) {
        let tts = soma_tts::fishspeech_tts::FishspeechTts::from_config(&conf.app.fishspeech);
        local.block_on(&rt, soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else {
        let tts = soma_tts::edge_tts::EdgeTts::new(conf.app.get_edge_tts_timeout());
        local.block_on(&rt, soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    };

    match result {
        Ok(tts_result) => {
            match std::fs::read(&tts_result.audio_file) {
                Ok(bytes) => {
                    let _ = std::fs::remove_file(&tts_result.audio_file);
                    HttpResponse::Ok()
                        .content_type("audio/mpeg")
                        .body(bytes)
                }
                Err(_) => HttpResponse::InternalServerError().body("read audio failed"),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("TTS failed: {:?}", e)),
    }
}

fn extract_query_param(query: &str, key: &str) -> Option<String> {
    url_decode(
        query.split('&')
            .find_map(|pair| {
                let (k, v) = pair.split_once('=')?;
                
                
                if k == key { Some(v) } else { None }
            })?
    )
}

fn url_decode(s: &str) -> Option<String> {
    let bytes: Vec<u8> = s.as_bytes().to_vec();
    let decoded = percent_encoding::percent_decode(&bytes).decode_utf8().ok()?;
    Some(decoded.to_string())
}

/// 字幕预览端点（GET /api/v1/subtitles/preview?text=xxx&font_size=24&font_color=white）
///
/// 根据文本生成 SRT 格式字幕预览，返回字幕时间轴 JSON
pub async fn preview_subtitle(req: HttpRequest) -> HttpResponse {
    let query = req.query_string();
    let text = extract_query_param(query, "text").unwrap_or_default();
    let estimated_duration = extract_query_param(query, "duration")
        .and_then(|d| d.parse::<f64>().ok())
        .unwrap_or_else(|| soma_tts::voices::estimate_no_voice_duration(&text));

    if text.is_empty() {
        return HttpResponse::BadRequest().body("text parameter required");
    }

    // 字幕预览不显示 Fish-Speech 情感标签
    let plain_text = soma_tts::fishspeech_tts::strip_emotion_tags(&text);
    let cues = soma_tts::edge_tts::generate_subtitle_cues_from_text(&plain_text, estimated_duration);
    let json = serde_json::to_string(&cues).unwrap_or_else(|_| "[]".to_string());
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}

// ==================== 语音克隆相关 ====================

/// 列出所有已克隆的声音
async fn list_cloned_voices(_param: &RequestParameter) -> Result<Value> {
    let voices = soma_tts::voice_clone::list_cloned_voices()
        .map_err(|e| error!("获取克隆声音列表失败: {:?}", e))?;
    let list: Vec<Value> = voices
        .iter()
        .map(|v| {
            value!({
                "id": v.id.clone(),
                "name": v.name.clone(),
                "engine": v.engine.to_string(),
                "remoteId": v.remote_id.clone(),
                "sourceFile": v.source_file.clone(),
                "samplePath": v.sample_path.clone(),
                "createdAt": v.created_at.clone(),
                "voiceName": soma_tts::voice_clone::cloned_voice_name(v),
            })
        })
        .collect();
    Ok(value!({ "voices": list }))
}

/// 删除已克隆的声音
async fn delete_cloned_voice(param: &RequestParameter) -> Result<Value> {
    let id = param.value.get_def_string("id", "");
    if id.is_empty() {
        return Err(error!("缺少参数: id"));
    }

    // 先加载声音信息，用于远端删除
    let voice = soma_tts::voice_clone::get_cloned_voice(&id)
        .map_err(|e| error!("获取克隆声音失败: {:?}", e))?
        .ok_or_else(|| error!("克隆声音不存在: {}", id))?;

    // 获取对应引擎的认证配置
    let conf = Config::get();
    let auth = match voice.engine {
        soma_tts::voice_clone::CloneEngine::Elevenlabs => {
            soma_tts::voice_clone::CloneAuth {
                api_key: conf.app.elevenlabs.api_key.as_deref().unwrap_or("").to_string(),
                ..Default::default()
            }
        }
        soma_tts::voice_clone::CloneEngine::Siliconflow => {
            soma_tts::voice_clone::CloneAuth {
                api_key: conf.app.siliconflow.api_key.as_deref().unwrap_or("").to_string(),
                ..Default::default()
            }
        }
        soma_tts::voice_clone::CloneEngine::Volcengine => {
            soma_tts::voice_clone::CloneAuth {
                app_id: conf.app.volcengine.app_id.as_deref().unwrap_or("").to_string(),
                access_token: conf.app.volcengine.access_token.as_deref().unwrap_or("").to_string(),
                ..Default::default()
            }
        }
        soma_tts::voice_clone::CloneEngine::Xfyun => {
            soma_tts::voice_clone::CloneAuth {
                app_id: conf.app.xfyun.app_id.as_deref().unwrap_or("").to_string(),
                api_key: conf.app.xfyun.api_key.as_deref().unwrap_or("").to_string(),
                api_secret: conf.app.xfyun.api_secret.as_deref().unwrap_or("").to_string(),
                ..Default::default()
            }
        }
    };

    // 尝试远端删除（失败不阻塞本地删除）
    let _ = soma_tts::voice_clone::delete_voice(&voice.engine, &auth, &voice.remote_id).await;

    // 删除本地元数据和音频样本
    soma_tts::voice_clone::delete_cloned_voice_local(&id)
        .map_err(|e| error!("删除克隆声音失败: {:?}", e))?;

    Ok(value!({ "id": id, "deleted": true }))
}

/// 语音克隆端点（POST /api/v1/voices/clone，multipart/form-data）
///
/// 接收上传的音频文件和引擎/名称参数，调用对应引擎的克隆 API 创建自定义声音，
/// 并将元数据持久化到本地 storage/voices/ 目录。
pub async fn clone_voice(mut payload: Multipart) -> HttpResponse {
    let mut engine_str = String::new();
    let mut voice_name = String::new();
    let mut audio_data: Vec<u8> = Vec::new();
    let mut audio_filename = String::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let field_name = match field.content_disposition() {
            Some(cd) => cd.get_name().unwrap_or("").to_string(),
            None => "".to_string(),
        };

        if field_name == "engine" || field_name == "name" {
            let mut buf = Vec::new();
            while let Some(Ok(chunk)) = field.next().await {
                buf.extend_from_slice(&chunk);
            }
            let text = String::from_utf8_lossy(&buf).to_string();
            if field_name == "engine" {
                engine_str = text;
            } else if field_name == "name" {
                voice_name = text;
            }
        } else if field_name == "file" {
            audio_filename = match field.content_disposition() {
                Some(cd) => cd.get_filename().unwrap_or("audio.mp3").to_string(),
                None => "audio.mp3".to_string(),
            };
            while let Some(Ok(chunk)) = field.next().await {
                audio_data.extend_from_slice(&chunk);
            }
        }
    }

    if audio_data.is_empty() {
        let resp = tube_web::response::get_error(error!("未接收到音频文件"));
        return resp.unwrap_or(HttpResponse::BadRequest().finish());
    }
    if engine_str.is_empty() {
        let resp = tube_web::response::get_error(error!("缺少参数: engine"));
        return resp.unwrap_or(HttpResponse::BadRequest().finish());
    }
    if voice_name.is_empty() {
        voice_name = "cloned_voice".to_string();
    }

    let engine: soma_tts::voice_clone::CloneEngine = match engine_str.parse() {
        Ok(e) => e,
        Err(e) => {
            let resp = tube_web::response::get_error(error!("{}", e));
            return resp.unwrap_or(HttpResponse::BadRequest().finish());
        }
    };

    let conf = Config::get();
    let auth = match engine {
        soma_tts::voice_clone::CloneEngine::Elevenlabs => {
            soma_tts::voice_clone::CloneAuth {
                api_key: conf.app.elevenlabs.api_key.as_deref().unwrap_or("").to_string(),
                ..Default::default()
            }
        }
        soma_tts::voice_clone::CloneEngine::Siliconflow => {
            soma_tts::voice_clone::CloneAuth {
                api_key: conf.app.siliconflow.api_key.as_deref().unwrap_or("").to_string(),
                ..Default::default()
            }
        }
        soma_tts::voice_clone::CloneEngine::Volcengine => {
            soma_tts::voice_clone::CloneAuth {
                app_id: conf.app.volcengine.app_id.as_deref().unwrap_or("").to_string(),
                access_token: conf.app.volcengine.access_token.as_deref().unwrap_or("").to_string(),
                ..Default::default()
            }
        }
        soma_tts::voice_clone::CloneEngine::Xfyun => {
            soma_tts::voice_clone::CloneAuth {
                app_id: conf.app.xfyun.app_id.as_deref().unwrap_or("").to_string(),
                api_key: conf.app.xfyun.api_key.as_deref().unwrap_or("").to_string(),
                api_secret: conf.app.xfyun.api_secret.as_deref().unwrap_or("").to_string(),
                ..Default::default()
            }
        }
    };

    // 调用引擎克隆 API
    let remote_id = match soma_tts::voice_clone::clone_voice(
        &engine,
        &auth,
        &voice_name,
        &audio_data,
        &audio_filename,
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            let resp = tube_web::response::get_error(error!("语音克隆失败: {:?}", e));
            return resp.unwrap_or(HttpResponse::InternalServerError().finish());
        }
    };

    // 保存音频样本到本地
    let voices_dir = soma_core::utils::storage_dir("voices", true);
    let voice_id = uuid::Uuid::new_v4().to_string();
    let sample_filename = format!("{}_{}", voice_id, audio_filename);
    let sample_path = voices_dir.join(&sample_filename);
    if let Err(e) = std::fs::write(&sample_path, &audio_data) {
        let resp = tube_web::response::get_error(error!("保存音频样本失败: {}", e));
        return resp.unwrap_or(HttpResponse::InternalServerError().finish());
    }

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let cloned = soma_tts::voice_clone::ClonedVoice {
        id: voice_id.clone(),
        name: voice_name.clone(),
        engine: engine.clone(),
        remote_id: remote_id.clone(),
        source_file: audio_filename.clone(),
        sample_path: sample_path.to_string_lossy().to_string(),
        created_at: now,
    };

    if let Err(e) = soma_tts::voice_clone::save_cloned_voice(&cloned) {
        let resp = tube_web::response::get_error(error!("保存克隆声音失败: {:?}", e));
        return resp.unwrap_or(HttpResponse::InternalServerError().finish());
    }

    let result = value!({
        "id": voice_id,
        "name": voice_name,
        "engine": engine.to_string(),
        "remoteId": remote_id,
        "sourceFile": audio_filename,
        "voiceName": soma_tts::voice_clone::cloned_voice_name(&cloned),
    });
    let resp = tube_web::response::get_success(&result);
    resp.unwrap_or(HttpResponse::Ok().finish())
}
