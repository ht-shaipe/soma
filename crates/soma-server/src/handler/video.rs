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
/// 根据 method 字段分发到对应的处理函数：
/// - "create" → 创建视频任务（立即执行）
/// - "draft" → 创建草稿任务（不执行）
/// - "updateConfig" → 更新草稿任务配置
/// - "start" → 启动草稿任务执行
/// - "fetchMaterials" → 为草稿任务预获取素材
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "create" => create(param).await,
        "draft" => create_draft(param).await,
        "updateconfig" => update_config(param).await,
        "start" => start_task(param).await,
        "fetchmaterials" => fetch_materials(param).await,
        "generateaudio" => generate_audio(param).await,
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
        let params_json = Value::from_serialize(&t.params).unwrap_or(Value::Null);
        let storyboard_json = Value::from_serialize(&t.storyboard).unwrap_or(Value::Null);
        value!({
            "taskId": t.task_id.clone(),
            "state": t.state,
            "progress": t.progress,
            "script": t.script.as_deref().unwrap_or(""),
            "terms": t.terms.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
            "params": params_json,
            "storyboard": storyboard_json,
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

    let params_json = Value::from_serialize(&task.params).unwrap_or(Value::Null);
    let storyboard_json = Value::from_serialize(&task.storyboard).unwrap_or(Value::Null);

    Ok(value!({
        "taskId": task.task_id.clone(),
        "state": task.state,
        "progress": task.progress,
        "script": task.script.as_deref().unwrap_or(""),
        "terms": task.terms.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
        "params": params_json,
        "storyboard": storyboard_json,
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

async fn create_draft(param: &RequestParameter) -> Result<Value> {
    let params: VideoParams = if let Some(text) = &param.text {
        serde_json::from_str(text).map_err(|e| error!("参数解析失败: {}", e))?
    } else {
        let json_str = param.value.to_string();
        serde_json::from_str(&json_str).map_err(|e| error!("参数解析失败: {}", e))?
    };

    let task_id = soma_core::utils::get_uuid();
    crate::state::create_draft_task_entry(&task_id, params);

    Ok(value!({
        "taskId": task_id,
    }))
}

async fn update_config(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_task(&task_id).ok_or_else(|| error!("任务不存在: {}", task_id))?;
    if task.state != soma_core::models::TaskStatus::Draft.as_i32() {
        return Err(error!("仅草稿状态任务可更新配置"));
    }

    let full_json: serde_json::Value = if let Some(text) = &param.text {
        serde_json::from_str(text).unwrap_or_default()
    } else {
        serde_json::from_str(&param.value.to_string()).unwrap_or_default()
    };

    if !full_json.is_null() {
        let json = full_json.clone();
        crate::state::update_task_params_from_json(&task_id, &json);
    }

    let mut data = state::TaskUpdateData::default();
    let mut has_extra = false;

    if let Some(v) = full_json.get("script").and_then(|v| v.as_str()) {
        if !v.is_empty() {
            data.script = Some(v.to_string());
            has_extra = true;
        }
    }

    if let Some(v) = full_json.get("terms") {
        if let Ok(terms) = serde_json::from_value::<Vec<String>>(v.clone()) {
            if !terms.is_empty() {
                data.terms = Some(terms);
                has_extra = true;
            }
        }
    }

    if let Some(v) = full_json.get("storyboard") {
        if let Ok(storyboard) = serde_json::from_value::<Vec<soma_core::models::StoryboardScene>>(v.clone()) {
            if !storyboard.is_empty() {
                data.storyboard = Some(storyboard);
                has_extra = true;
            }
        }
    }

    if has_extra {
        crate::state::update_task_data(&task_id, &data);
    }

    Ok(value!({
        "updated": true,
    }))
}

async fn start_task(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_task(&task_id).ok_or_else(|| error!("任务不存在: {}", task_id))?;
    let task_state = soma_core::models::TaskStatus::from_i32(task.state);
    match task_state {
        soma_core::models::TaskStatus::Draft | soma_core::models::TaskStatus::Failed => {}
        soma_core::models::TaskStatus::Processing => {
            return Err(error!("任务正在执行中，请等待完成"));
        }
        _ => {
            return Err(error!("任务状态不允许重新启动（当前状态: {}）", task.state));
        }
    }

    let stop_at = param.value.get_def_string("stopAt", "");

    crate::state::set_task_state(&task_id, soma_core::models::TaskStatus::Processing);
    crate::task::add_task(&task_id, &task.params, &stop_at)
        .map_err(|e| error!("任务启动失败: {:?}", e))?;

    Ok(value!({
        "started": true,
        "taskId": task_id,
    }))
}

async fn fetch_materials(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_task(&task_id).ok_or_else(|| error!("任务不存在: {}", task_id))?;

    let params = task.params.clone();
    let terms: Vec<String> = if let Some(ref t) = params.video_terms {
        if let Some(arr) = t.as_array() {
            arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
        } else if let Some(s) = t.as_str() {
            s.split(&[',', '，'][..]).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
        } else {
            vec![]
        }
    } else if let Some(ref sb) = task.storyboard {
        sb.iter().map(|s| s.visual_prompt.clone()).collect()
    } else {
        vec![]
    };

    if terms.is_empty() {
        return Err(error!("没有可用的搜索关键词，请先填写提示词或生成分镜"));
    }

    let tid = task_id.to_string();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let conf = crate::Config::get();
        let result = crate::service::pipeline::get_video_materials(&tid, &params, &terms, 0.0, &conf);
        let _ = tx.send(result);
    });

    let materials = rx.recv().map_err(|e| error!("素材获取线程异常: {}", e))?
        .map_err(|e| error!("素材获取失败: {:?}", e))?;

    state::update_task_data(&task_id, &state::TaskUpdateData {
        materials: Some(materials.clone()),
        ..Default::default()
    });

    let materials_val: Vec<Value> = materials.iter().map(|s| value!(s.clone())).collect();
    let total = materials_val.len();
    Ok(value!({
        "taskId": task_id,
        "materials": materials_val,
        "total": total,
    }))
}

async fn generate_audio(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_task(&task_id).ok_or_else(|| error!("任务不存在: {}", task_id))?;

    if task.audio_file.is_some() && !task.audio_file.as_deref().unwrap_or("").is_empty() {
        return Ok(value!({
            "taskId": task_id,
            "audioFile": task.audio_file.as_deref().unwrap_or(""),
            "audioDuration": task.audio_duration.unwrap_or(0.0),
        }));
    }

    let script = task.script.as_deref().unwrap_or("").to_string();
    if script.is_empty() {
        return Err(error!("请先生成或填写文案"));
    }

    let narration_text: String = if let Some(ref sb) = task.storyboard {
        if sb.is_empty() {
            script.clone()
        } else {
            sb.iter().map(|s| s.narration.as_str()).collect::<Vec<&str>>().join(" ")
        }
    } else {
        script.clone()
    };

    let tid = task_id.to_string();
    let params = task.params.clone();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let conf = crate::Config::get();
        let result = crate::service::pipeline::generate_audio(&tid, &params, &narration_text, &conf);
        let _ = tx.send(result);
    });

    let (audio_file, audio_duration) = rx.recv().map_err(|e| error!("音频生成线程异常: {}", e))?
        .map_err(|e| error!("音频生成失败: {:?}", e))?;

    state::update_task_data(&task_id, &state::TaskUpdateData {
        audio_file: Some(audio_file.clone()),
        audio_duration: Some(audio_duration),
        ..Default::default()
    });

    Ok(value!({
        "taskId": task_id,
        "audioFile": audio_file,
        "audioDuration": audio_duration,
    }))
}
