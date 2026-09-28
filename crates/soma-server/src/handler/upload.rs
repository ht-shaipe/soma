use crate::Config;
/// 跨平台发布 API 处理器
use tube::{Result, Value};
use tube_web::RequestParameter;

/// 上传模块分发入口：`upload.upload` / `status`（Upload-Post 跨平台发布）
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "upload" => upload(param).await,
        "status" => status(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

async fn upload(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    let video_index: usize = param
        .value
        .get("videoIndex")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let platforms_val = param.value.get("platforms");
    let platforms: Vec<String> = if let Some(arr) = platforms_val.and_then(|v| v.as_array()) {
        arr.iter().filter_map(|v| v.as_str()).collect()
    } else {
        vec!["tiktok".into(), "youtube".into()]
    };

    if task_id.is_empty() {
        return Err(error!("缺少 taskId 参数"));
    }

    let conf = Config::get();
    let ui = &conf.app.ui;

    if !crate::service::upload::is_configured(ui) {
        return Err(error!("Upload-Post 未配置，请在设置中启用并填写 API Key"));
    }

    let task = crate::state::get_task(&task_id);
    let task_info = match task {
        Some(t) => t,
        None => return Err(error!("任务不存在: {}", task_id)),
    };

    let videos = match &task_info.videos {
        Some(v) if !v.is_empty() => v,
        _ => return Err(error!("任务尚未生成视频")),
    };

    let video_path = match videos.get(video_index) {
        Some(p) => p.clone(),
        None => return Err(error!("视频索引超出范围")),
    };

    let title = task_info
        .script
        .as_deref()
        .unwrap_or(&task_id)
        .chars()
        .take(100)
        .collect::<String>();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| tube::error!("runtime error: {}", e))?;
    let local = tokio::task::LocalSet::new();
    let result = local
        .block_on(
            &rt,
            crate::service::upload::upload_video(&video_path, &title, &platforms, ui, None),
        )
        .map_err(|e| tube::error!("发布失败: {:?}", e))?;

    Ok(value!({
        "success": result.success,
        "requestId": result.request_id.unwrap_or_default(),
        "message": result.message.unwrap_or_default(),
    }))
}

async fn status(param: &RequestParameter) -> Result<Value> {
    let request_id = param.value.get_def_string("requestId", "");
    if request_id.is_empty() {
        return Err(error!("缺少 requestId 参数"));
    }

    let conf = Config::get();
    let api_key = conf.app.ui.upload_post_api_key.as_deref().unwrap_or("");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| tube::error!("runtime error: {}", e))?;
    let local = tokio::task::LocalSet::new();
    let result = local
        .block_on(
            &rt,
            crate::service::upload::check_status(&request_id, api_key),
        )
        .map_err(|e| tube::error!("查询失败: {:?}", e))?;

    Ok(value!({
        "status": result,
    }))
}
