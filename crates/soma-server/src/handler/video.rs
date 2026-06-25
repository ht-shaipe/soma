use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use soma_core::models::VideoParams;
use crate::state;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "create" => create(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

pub async fn distribute_tasks(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list(param).await,
        "get" => get(param).await,
        "delete" => delete(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

async fn create(param: &RequestParameter) -> Result<Value> {
    let params: VideoParams = if let Some(text) = &param.text {
        serde_json::from_str(text).map_err(|e| error!("参数解析失败: {}", e))?
    } else {
        let json_str = param.value.to_string();
        serde_json::from_str(&json_str).map_err(|e| error!("参数解析失败: {}", e))?
    };

    let task_id = soma_core::utils::get_uuid();
    let stop_at = param.value.get_def_string("stopAt", "");

    crate::state::create_task_entry(&task_id, params.clone());

    crate::task::add_task(&task_id, &params, &stop_at)
        .map_err(|e| error!("任务创建失败: {:?}", e))?;

    Ok(value!({
        "taskId": task_id,
    }))
}

async fn list(param: &RequestParameter) -> Result<Value> {
    let page = param.value.get_i32("page", 1) as usize;
    let page_size = param.value.get_i32("pageSize", 10) as usize;

    let (tasks, total) = state::get_all_tasks(page, page_size);

    let task_list: Vec<Value> = tasks.iter().map(|t| {
        value!({
            "taskId": t.task_id.clone(),
            "state": t.state,
            "progress": t.progress,
            "script": t.script.as_deref().unwrap_or(""),
            "terms": t.terms.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
            "audioFile": t.audio_file.as_deref().unwrap_or(""),
            "audioDuration": t.audio_duration.unwrap_or(0.0),
            "subtitlePath": t.subtitle_path.as_deref().unwrap_or(""),
            "materials": t.materials.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
            "videos": t.videos.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
            "combinedVideos": t.combined_videos.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
            "createdAt": t.created_at.to_rfc3339(),
            "updatedAt": t.updated_at.to_rfc3339(),
        })
    }).collect();

    Ok(value!({
        "list": task_list,
        "total": total,
        "page": page,
        "pageSize": page_size,
    }))
}

async fn get(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_task(&task_id).ok_or_else(|| error!("任务不存在: {}", task_id))?;

    Ok(value!({
        "taskId": task.task_id.clone(),
        "state": task.state,
        "progress": task.progress,
        "script": task.script.as_deref().unwrap_or(""),
        "terms": task.terms.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
        "audioFile": task.audio_file.as_deref().unwrap_or(""),
        "audioDuration": task.audio_duration.unwrap_or(0.0),
        "subtitlePath": task.subtitle_path.as_deref().unwrap_or(""),
        "materials": task.materials.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
        "videos": task.videos.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
        "combinedVideos": task.combined_videos.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
        "createdAt": task.created_at.to_rfc3339(),
        "updatedAt": task.updated_at.to_rfc3339(),
    }))
}

async fn delete(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let deleted = state::delete_task(&task_id);
    Ok(value!({
        "deleted": deleted,
    }))
}
