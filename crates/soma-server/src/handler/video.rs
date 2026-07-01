/// 视频任务 API 处理器
///
/// 处理视频创建（/videos）和任务管理（/tasks）两类 API 请求：
/// - videos 模块：创建视频生成任务
/// - tasks 模块：任务列表查询、单个任务查询、任务删除

use tube::{Result, Value};
use tube_web::RequestParameter;
use soma_core::models::VideoParams;
use crate::state;

/// 视频模块请求分发
///
/// 根据 method 字段分发到对应的处理函数，当前仅支持 "create" 方法
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "create" => create(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 任务模块请求分发
///
/// 根据 method 字段分发到对应的处理函数：
/// - "list" → 查询任务列表
/// - "get" → 查询单个任务
/// - "delete" → 删除任务
pub async fn distribute_tasks(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list(param).await,
        "get" => get(param).await,
        "delete" => delete(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 创建视频生成任务
///
/// 解析请求中的 VideoParams 参数，生成唯一 task_id，
/// 将任务写入存储并加入任务队列开始执行。
///
/// 返回：包含 taskId 的 JSON 对象
async fn create(param: &RequestParameter) -> Result<Value> {
    // 解析视频参数：优先从 text 字段解析，否则从 value 对象解析
    let params: VideoParams = if let Some(text) = &param.text {
        serde_json::from_str(text).map_err(|e| error!("参数解析失败: {}", e))?
    } else {
        let json_str = param.value.to_string();
        serde_json::from_str(&json_str).map_err(|e| error!("参数解析失败: {}", e))?
    };

    // 生成唯一任务 ID
    let task_id = soma_core::utils::get_uuid();
    // 获取流水线停止步骤（用于调试/预览）
    let stop_at = param.value.get_def_string("stopAt", "");

    // 在全局存储中创建任务记录
    crate::state::create_task_entry(&task_id, params.clone());

    // 将任务加入队列开始执行
    crate::task::add_task(&task_id, &params, &stop_at)
        .map_err(|e| error!("任务创建失败: {:?}", e))?;

    Ok(value!({
        "taskId": task_id,
    }))
}

/// 查询任务列表（分页）
///
/// 支持 page 和 pageSize 参数，返回按更新时间降序排列的任务列表及总数。
///
/// 返回：包含 list、total、page、pageSize 的 JSON 对象
async fn list(param: &RequestParameter) -> Result<Value> {
    let page = param.value.get_i32("page", 1) as usize;
    let page_size = param.value.get_i32("pageSize", 10) as usize;

    let (tasks, total) = state::get_all_tasks(page, page_size);

    // 将 TaskInfo 转换为前端所需的 JSON 格式
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
            "errorMessage": t.error_message.as_deref().unwrap_or(""),
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

/// 查询单个任务详情
///
/// 根据 taskId 查询任务的完整信息。
///
/// 返回：包含任务所有字段的 JSON 对象
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
        "errorMessage": task.error_message.as_deref().unwrap_or(""),
        "createdAt": task.created_at.to_rfc3339(),
        "updatedAt": task.updated_at.to_rfc3339(),
    }))
}

/// 删除任务
///
/// 根据 taskId 从全局存储中删除任务记录。
///
/// 返回：包含 deleted 布尔值的 JSON 对象（true 表示删除成功）
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
