/// 视频任务 API 处理器
///
/// 处理视频创建（/videos）和任务管理（/tasks）两类 API 请求：
/// - videos 模块：创建视频生成任务
/// - tasks 模块：任务列表查询、单个任务查询、任务删除
use tube::{Result, Value};
use tube_web::RequestParameter;
use soma_core::models::VideoParams;
use crate::state;

fn ai_video_logs_to_value(logs: &Option<Vec<soma_core::models::AiVideoSegmentLog>>) -> Value {
    match logs {
        Some(l) if !l.is_empty() => Value::from_serialize(l).unwrap_or(Value::Null),
        _ => Value::Null,
    }
}

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
        "generatenarration" => generate_narration(param).await,
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
        "stop" => stop_task(param).await,
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
            "narration": t.narration.as_deref().unwrap_or(""),
            "audioFile": t.audio_file.as_deref().unwrap_or(""),
            "audioDuration": t.audio_duration.unwrap_or(0.0),
            "subtitlePath": t.subtitle_path.as_deref().unwrap_or(""),
            "materials": t.materials.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
            "videos": t.videos.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
            "combinedVideos": t.combined_videos.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
            "aiVideoLogs": ai_video_logs_to_value(&t.ai_video_logs),
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
        "narration": task.narration.as_deref().unwrap_or(""),
        "audioFile": task.audio_file.as_deref().unwrap_or(""),
        "audioDuration": task.audio_duration.unwrap_or(0.0),
        "subtitlePath": task.subtitle_path.as_deref().unwrap_or(""),
        "materials": task.materials.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
        "videos": task.videos.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
        "combinedVideos": task.combined_videos.as_deref().unwrap_or(&[]).iter().map(|s| value!(s.clone())).collect::<Vec<Value>>(),
        "aiVideoLogs": ai_video_logs_to_value(&task.ai_video_logs),
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
    // 允许 Draft 和 Failed 状态更新配置：失败后用户返回上一步修改参数再重试是常见操作
    let allow_edit = task.state == soma_core::models::TaskStatus::Draft.as_i32()
        || task.state == soma_core::models::TaskStatus::Failed.as_i32();
    if !allow_edit {
        return Err(error!("仅草稿或失败状态任务可更新配置"));
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

    if let Some(v) = full_json.get("narration").and_then(|v| v.as_str()) {
        if !v.is_empty() {
            data.narration = Some(v.to_string());
            has_extra = true;
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
    crate::state::update_task_data(&task_id, &crate::state::TaskUpdateData {
        error_message: Some(String::new()),
        materials: Some(Vec::new()),
        audio_file: Some(String::new()),
        audio_duration: Some(0.0),
        subtitle_path: Some(String::new()),
        videos: Some(Vec::new()),
        combined_videos: Some(Vec::new()),
        ai_video_logs: Some(Vec::new()),
        ..Default::default()
    });
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
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let conf = crate::Config::get();
            crate::service::pipeline::get_video_materials(&tid, &params, &terms, 0.0, &conf)
        }));
        let _ = tx.send(result);
    });

    let materials = rx.recv().map_err(|_| error!("素材获取线程异常退出"))?
        .map_err(|e| {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                format!("{:?}", e)
            };
            error!("素材获取线程 panic: {}", msg)
        })?
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

async fn generate_narration(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_task(&task_id).ok_or_else(|| error!("任务不存在: {}", task_id))?;

    if task.narration.is_some() && !task.narration.as_deref().unwrap_or("").is_empty() {
        return Ok(value!({
            "taskId": task_id,
            "narration": task.narration.as_deref().unwrap_or(""),
        }));
    }

    let script = task.script.as_deref().unwrap_or("").to_string();
    if script.is_empty() {
        return Err(error!("请先生成或填写脚本"));
    }

    let storyboard_json = task.storyboard.as_ref()
        .map(|sb| serde_json::to_value(sb).unwrap_or(serde_json::Value::Null))
        .unwrap_or(serde_json::Value::Null);

    let _tid = task_id.to_string();
    let params = task.params.clone();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let conf = crate::Config::get();
            let provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");
            let style = params.intent_style.as_deref().unwrap_or("");
            let mood = params.intent_mood.as_deref().unwrap_or("");
            let language = params.video_language.as_deref().unwrap_or("");
            let storyboard_json_val = storyboard_json;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| soma_core::error::SomaError::Llm(e.to_string())).ok();
            if let Some(rt) = rt {
                let local = tokio::task::LocalSet::new();
                let result = local.block_on(&rt, crate::service::llm::generate_narration(provider, &script, &storyboard_json_val, style, mood, language, &conf));
                result.map_err(|e| error!("LLM narration error: {:?}", e))
            } else {
                Err(error!("tokio runtime 创建失败"))
            }
        }));
        let _ = tx.send(result);
    });

    let narration = rx.recv().map_err(|_| error!("旁白生成线程异常退出"))?
        .map_err(|e| {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                format!("{:?}", e)
            };
            error!("旁白生成线程 panic: {}", msg)
        })?
        .map_err(|e| error!("旁白生成失败: {:?}", e))?;

    state::update_task_data(&task_id, &state::TaskUpdateData {
        narration: Some(narration.clone()),
        ..Default::default()
    });

    Ok(value!({
        "taskId": task_id,
        "narration": narration,
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

    let narration_text: String = if let Some(ref n) = task.narration {
        if !n.is_empty() {
            n.clone()
        } else if let Some(ref sb) = task.storyboard {
            if sb.is_empty() {
                task.script.as_deref().unwrap_or("").to_string()
            } else {
                sb.iter().map(|s| s.narration.as_str()).collect::<Vec<&str>>().join(" ")
            }
        } else {
            task.script.as_deref().unwrap_or("").to_string()
        }
    } else if let Some(ref sb) = task.storyboard {
        if sb.is_empty() {
            task.script.as_deref().unwrap_or("").to_string()
        } else {
            sb.iter().map(|s| s.narration.as_str()).collect::<Vec<&str>>().join(" ")
        }
    } else {
        task.script.as_deref().unwrap_or("").to_string()
    };

    let tid = task_id.to_string();
    let params = task.params.clone();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let conf = crate::Config::get();
            crate::service::pipeline::generate_audio(&tid, &params, &narration_text, &conf)
        }));
        let _ = tx.send(result);
    });

    let (audio_file, audio_duration) = rx.recv().map_err(|_| error!("音频生成线程异常退出"))?
        .map_err(|e| {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                format!("{:?}", e)
            };
            error!("音频生成线程 panic: {}", msg)
        })?
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

async fn stop_task(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_task(&task_id).ok_or_else(|| error!("任务不存在: {}", task_id))?;

    if task.state != soma_core::models::TaskStatus::Processing.as_i32() {
        return Err(error!("仅处理中的任务可以停止（当前状态: {}）", task.state));
    }

    state::update_task_data(&task_id, &state::TaskUpdateData {
        state: Some(soma_core::models::TaskStatus::Failed.as_i32()),
        error_message: Some("用户手动停止".to_string()),
        ..Default::default()
    });

    Ok(value!({
        "stopped": true,
        "taskId": task_id,
    }))
}
