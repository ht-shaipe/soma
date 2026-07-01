/// 语音列表与预览 API 处理器
///
/// - list: 返回各 TTS 引擎可用的语音列表
/// - preview: 生成语音预览音频并返回

use actix_web::{HttpRequest, HttpResponse};
use tube::{Result, Value};
use tube_web::RequestParameter;
use crate::Config;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_voices(param).await,
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

    let mut all_voices = edge_voices;
    all_voices.extend(sf_voices);
    all_voices.extend(el_voices);
    all_voices.extend(mimo_voices);
    all_voices.extend(gemini_voices);
    all_voices.extend(azure_voices);

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

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(_) => return HttpResponse::InternalServerError().body("runtime error"),
    };

    let result = if soma_tts::voices::is_siliconflow_voice(&voice_name) {
        let sf_key = conf.app.siliconflow.api_key.as_deref().unwrap_or("");
        let tts = soma_tts::siliconflow_tts::SiliconflowTts::new(sf_key);
        rt.block_on(soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_elevenlabs_voice(&voice_name) {
        let el_key = conf.app.elevenlabs.api_key.as_deref().unwrap_or("");
        let el_model = conf.app.elevenlabs.model_id.as_deref().unwrap_or("eleven_multilingual_v2");
        let tts = soma_tts::elevenlabs_tts::ElevenlabsTts::new(el_key, el_model);
        rt.block_on(soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_mimo_voice(&voice_name) {
        let mimo_key = conf.app.app.mimo_api_key.as_deref().unwrap_or("");
        let tts = soma_tts::mimo_tts::MimoTts::new(mimo_key, "", "", "");
        rt.block_on(soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_gemini_voice(&voice_name) {
        let gemini_key = conf.app.app.gemini_api_key.as_deref().unwrap_or("");
        let tts = soma_tts::gemini_tts::GeminiTts::new(gemini_key, "", "");
        rt.block_on(soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else if soma_tts::voices::is_azure_voice(&voice_name) {
        let azure_key = conf.app.azure.speech_key.as_deref().unwrap_or("");
        let azure_region = conf.app.azure.speech_region.as_deref().unwrap_or("eastasia");
        let tts = soma_tts::azure_tts::AzureTts::new(azure_key, azure_region);
        rt.block_on(soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
    } else {
        let tts = soma_tts::edge_tts::EdgeTts::new(conf.app.get_edge_tts_timeout());
        rt.block_on(soma_tts::provider::SomaTtsProvider::synthesize(&tts, &text, &voice_name, 1.0, std::path::Path::new(&tmp_path)))
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
                let mut kv = pair.splitn(2, '=');
                let k = kv.next()?;
                let v = kv.next()?;
                if k == key { Some(v) } else { None }
            })?
    )
}

fn url_decode(s: &str) -> Option<String> {
    let bytes: Vec<u8> = s.as_bytes().iter().copied().collect();
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

    let cues = soma_tts::edge_tts::generate_subtitle_cues_from_text(&text, estimated_duration);
    let json = serde_json::to_string(&cues).unwrap_or_else(|_| "[]".to_string());
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}
