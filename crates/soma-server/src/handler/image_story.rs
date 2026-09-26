/// 图片故事 API 处理器
///
/// 处理图片故事视频生成相关的 API 请求：
/// - "create" → 创建图片故事任务
/// - "list" → 查询任务列表
/// - "get" → 查询单个任务
/// - "delete" → 删除任务
/// - "upload_images" → 批量上传图片
use tube::{Result, Value};
use tube_web::RequestParameter;
use soma_core::models::ImageStoryParams;
use crate::state;

/// 图片故事模块请求分发
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "create" => create(param).await,
        "list" => list(param).await,
        "get" => get(param).await,
        "delete" => delete(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 创建图片故事任务
async fn create(param: &RequestParameter) -> Result<Value> {
    let params: ImageStoryParams = if let Some(text) = &param.text {
        serde_json::from_str(text).map_err(|e| error!("参数解析失败: {}", e))?
    } else {
        let json_str = param.value.to_string();
        serde_json::from_str(&json_str).map_err(|e| error!("参数解析失败: {}", e))?
    };

    // 验证参数
    if params.scenes.is_empty() {
        return Err(error!("至少需要一个场景"));
    }

    for scene in &params.scenes {
        if scene.image_path.is_empty() {
            return Err(error!("场景 {} 的图片路径不能为空", scene.scene_id));
        }
        if scene.description.is_empty() {
            return Err(error!("场景 {} 的描述不能为空", scene.scene_id));
        }
    }

    // 生成唯一任务 ID
    let task_id = soma_core::utils::get_uuid();

    // 在全局存储中创建任务记录
    crate::state::create_image_story_task_entry(&task_id, params.clone());

    // 将任务加入队列开始执行
    crate::task::add_image_story_task(&task_id, &params)
        .map_err(|e| error!("任务创建失败: {:?}", e))?;

    Ok(value!({
        "taskId": task_id,
    }))
}

/// 查询任务列表
async fn list(param: &RequestParameter) -> Result<Value> {
    let page = param.value.get_i32("page", 1) as usize;
    let page_size = param.value.get_i32("pageSize", 10) as usize;

    let (tasks, total) = state::get_all_image_story_tasks(page, page_size);

    let task_list: Vec<Value> = tasks.iter().map(|t| {
        let params_json = Value::from_serialize(&t.params).unwrap_or(Value::Null);
        value!({
            "taskId": t.task_id.clone(),
            "state": t.state,
            "progress": t.progress,
            "params": params_json,
            "videoPath": t.video_path.as_deref().unwrap_or(""),
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
async fn get(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_image_story_task(&task_id)
        .ok_or_else(|| error!("任务不存在: {}", task_id))?;

    let params_json = Value::from_serialize(&task.params).unwrap_or(Value::Null);

    Ok(value!({
        "taskId": task.task_id.clone(),
        "state": task.state,
        "progress": task.progress,
        "params": params_json,
        "videoPath": task.video_path.as_deref().unwrap_or(""),
        "errorMessage": task.error_message.as_deref().unwrap_or(""),
        "createdAt": task.created_at.to_rfc3339(),
        "updatedAt": task.updated_at.to_rfc3339(),
    }))
}

/// 删除任务
async fn delete(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let deleted = state::delete_image_story_task(&task_id);
    Ok(value!({
        "deleted": deleted,
    }))
}