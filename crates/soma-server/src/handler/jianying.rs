//! 剪映草稿导出 API 处理器

use tube::{Result, Value};
use tube_web::RequestParameter;
use soma_video::jianying;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "create" => create(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 从视频文件列表生成剪映草稿
async fn create(param: &RequestParameter) -> Result<Value> {
    let name = param.value.get_def_string("name", "soma_draft");
    let output_dir = param.value.get_def_string("outputDir", "");
    if output_dir.is_empty() {
        return Err(error!("缺少 outputDir 参数"));
    }

    let width = param.value.get("width").and_then(|v| v.as_u64()).unwrap_or(1920);
    let height = param.value.get("height").and_then(|v| v.as_u64()).unwrap_or(1080);

    let videos_val = param.value.get("videos").and_then(|v| v.as_array());
    let videos: Vec<String> = if let Some(arr) = videos_val {
        arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
    } else {
        return Err(error!("缺少 videos 参数"));
    };

    if videos.is_empty() {
        return Err(error!("视频列表为空"));
    }

    let audio = param.value.get_def_string("audio", "");
    let audio = if audio.is_empty() { None } else { Some(audio.as_str()) };

    let draft_path = jianying::create_draft_from_videos(&name, &videos, audio, width, height, &output_dir)
        .map_err(|e| error!("{}", e))?;

    Ok(value!({
        "success": true,
        "draftPath": draft_path,
        "videoCount": videos.len(),
    }))
}
