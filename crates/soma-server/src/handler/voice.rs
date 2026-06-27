/// 语音列表 API 处理器
///
/// 返回各 TTS 引擎可用的语音列表，供前端下拉框选用。

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

    let mut all_voices = edge_voices;
    all_voices.extend(sf_voices);
    all_voices.extend(el_voices);
    all_voices.extend(mimo_voices);

    Ok(value!({
        "voices": all_voices,
    }))
}
