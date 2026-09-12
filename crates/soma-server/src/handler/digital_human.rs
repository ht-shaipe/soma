//! 数字人口播视频任务 API 处理器
//!
//! 处理数字人口播视频相关的 API 请求：
//! - digital_human 模块：create/draft/start
//! - dh_tasks 模块：list/get/delete

use tube::{Result, Value};
use tube_web::RequestParameter;
use soma_core::models::{DigitalHumanParams, TaskStatus};
use crate::state;

/// 数字人模块请求分发
///
/// - "create" → 创建口播任务（立即执行）
/// - "draft" → 创建草稿任务（不执行）
/// - "start" → 启动/重试任务
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "create" => create(param).await,
        "draft" => create_draft(param).await,
        "start" => start_task(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 数字人任务管理请求分发
///
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

/// 解析 DigitalHumanParams
fn parse_params(param: &RequestParameter) -> Result<DigitalHumanParams> {
    if let Some(text) = &param.text {
        serde_json::from_str(text).map_err(|e| error!("参数解析失败: {}", e))
    } else {
        let json_str = param.value.to_string();
        serde_json::from_str(&json_str).map_err(|e| error!("参数解析失败: {}", e))
    }
}

/// 校验人像照片存在性
fn validate_portrait(portrait_image: &str) -> Result<()> {
    if portrait_image.is_empty() {
        return Err(error!("缺少人像照片"));
    }
    let portrait_path = soma_core::utils::storage_dir("portraits", false)
        .join(portrait_image);
    if !portrait_path.exists() {
        return Err(error!("人像照片不存在，请重新上传"));
    }
    Ok(())
}

/// 校验文案
fn validate_narration(text: &str) -> Result<()> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(error!("文案不能为空"));
    }
    if trimmed.chars().count() > soma_core::models::DH_NARRATION_TEXT_MAX_LEN {
        return Err(error!(
            "文案长度不能超过 {} 字",
            soma_core::models::DH_NARRATION_TEXT_MAX_LEN
        ));
    }
    Ok(())
}

/// 敏感词校验
fn check_sensitive_words(text: &str) -> Result<()> {
    let conf = crate::Config::get();
    let sw_path = conf.app.digital_human.get_sensitive_words_path();
    let result = soma_core::filter::check_text(text, sw_path);
    if result.hit {
        log::warn!(
            "文案敏感词命中: words={:?}",
            result.words
        );
        return Err(error!("文案包含敏感内容，请修改后重试"));
    }
    Ok(())
}

/// 创建口播任务（立即执行）
async fn create(param: &RequestParameter) -> Result<Value> {
    let params = parse_params(param)?;

    validate_portrait(&params.portrait_image)?;
    validate_narration(&params.narration_text)?;
    check_sensitive_words(&params.narration_text)?;

    let task_id = soma_core::utils::get_uuid();

    log::info!(
        "创建数字人任务: task_id={}, portrait={}, text_len={}",
        task_id, params.portrait_image, params.narration_text.chars().count()
    );

    state::create_dh_task_entry(&task_id, params.clone());

    crate::task::add_dh_task(&task_id, &params)
        .map_err(|e| error!("任务创建失败: {:?}", e))?;

    Ok(value!({
        "taskId": task_id,
    }))
}

/// 创建草稿任务（不执行）
async fn create_draft(param: &RequestParameter) -> Result<Value> {
    let params = parse_params(param)?;

    validate_portrait(&params.portrait_image)?;
    validate_narration(&params.narration_text)?;
    check_sensitive_words(&params.narration_text)?;

    let task_id = soma_core::utils::get_uuid();

    log::info!("创建数字人草稿: task_id={}", task_id);

    state::create_dh_draft_task_entry(&task_id, params);

    Ok(value!({
        "taskId": task_id,
    }))
}

/// 启动/重试任务
async fn start_task(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("缺少参数: taskId"));
    }

    let task = state::get_dh_task(&task_id)
        .ok_or_else(|| error!("任务不存在"))?;

    match TaskStatus::from_i32(task.state) {
        TaskStatus::Processing => {
            return Err(error!("任务正在执行中，请等待完成"));
        }
        TaskStatus::Completed => {
            return Err(error!("任务已完成，无需重试"));
        }
        TaskStatus::Draft | TaskStatus::Failed => {}
        _ => {}
    }

    log::info!("启动数字人任务: task_id={}", task_id);

    state::update_dh_task_data(&task_id, &state::DhTaskUpdateData {
        state: Some(TaskStatus::Processing.as_i32()),
        progress: Some(0),
        audio_file: None,
        audio_duration: None,
        subtitle_path: None,
        portrait_video_path: None,
        final_video_path: None,
        error_message: None,
        ..Default::default()
    });

    crate::task::add_dh_task(&task_id, &task.params)
        .map_err(|e| error!("任务启动失败: {:?}", e))?;

    Ok(value!({
        "taskId": task_id,
    }))
}

/// 查询任务列表（分页）
async fn list(param: &RequestParameter) -> Result<Value> {
    let page = param.value.get_i32("page", 1) as usize;
    let page_size = param.value.get_i32("pageSize", 10) as usize;

    let (tasks, total) = state::get_all_dh_tasks(page, page_size);

    let task_list: Vec<Value> = tasks.iter().map(|t| {
        let params_json = Value::from_serialize(&t.params).unwrap_or(Value::Null);
        value!({
            "taskId": t.task_id.clone(),
            "state": t.state,
            "progress": t.progress,
            "params": params_json,
            "audioFile": t.audio_file.as_deref().unwrap_or(""),
            "audioDuration": t.audio_duration.unwrap_or(0.0),
            "subtitlePath": t.subtitle_path.as_deref().unwrap_or(""),
            "portraitVideoPath": t.portrait_video_path.as_deref().unwrap_or(""),
            "finalVideoPath": t.final_video_path.as_deref().unwrap_or(""),
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

/// 查询单个任务
async fn get(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("缺少参数: taskId"));
    }

    let task = state::get_dh_task(&task_id)
        .ok_or_else(|| error!("任务不存在"))?;

    let params_json = Value::from_serialize(&task.params).unwrap_or(Value::Null);

    Ok(value!({
        "taskId": task.task_id.clone(),
        "state": task.state,
        "progress": task.progress,
        "params": params_json,
        "audioFile": task.audio_file.as_deref().unwrap_or(""),
        "audioDuration": task.audio_duration.unwrap_or(0.0),
        "subtitlePath": task.subtitle_path.as_deref().unwrap_or(""),
        "portraitVideoPath": task.portrait_video_path.as_deref().unwrap_or(""),
        "finalVideoPath": task.final_video_path.as_deref().unwrap_or(""),
        "errorMessage": task.error_message.as_deref().unwrap_or(""),
        "createdAt": task.created_at.to_rfc3339(),
        "updatedAt": task.updated_at.to_rfc3339(),
    }))
}

/// 删除任务
async fn delete(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("缺少参数: taskId"));
    }

    state::delete_dh_task(&task_id);

    let task_dir = soma_core::utils::task_dir(&task_id);
    if task_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&task_dir) {
            log::warn!("删除任务文件目录失败: task_id={}, err={}", task_id, e);
        }
    }

    log::info!("删除数字人任务: task_id={}", task_id);

    Ok(value!({
        "deleted": true,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use soma_core::models::{DigitalHumanParams, DH_NARRATION_TEXT_MAX_LEN};
    use std::sync::Once;

    static SETUP: Once = Once::new();

    /// 初始化测试环境：设置敏感词库绝对路径并预热全局过滤器缓存
    ///
    /// 由于 `soma_core::filter::GLOBAL_FILTER` 是 `OnceLock` 全局缓存，
    /// 必须在首次调用 `check_text` 之前设置正确的敏感词库路径，
    /// 否则缓存将永久持有空过滤器。本函数通过 `Once` 保证只执行一次。
    fn ensure_setup() {
        SETUP.call_once(|| {
            let mut conf = crate::Config::get();
            let sw_path = std::env::current_dir()
                .unwrap()
                .join("../../resource/sensitive_words.txt")
                .to_string_lossy()
                .to_string();
            conf.app.digital_human.sensitive_words_path = Some(sw_path);
            crate::Config::set(conf);

            let conf = crate::Config::get();
            let path = conf.app.digital_human.get_sensitive_words_path();
            soma_core::filter::check_text("init", path);
        });
    }

    /// 通过 JSON 字符串构造 RequestParameter（text 模式）
    fn make_param_with_text(text: &str) -> RequestParameter {
        let mut param = RequestParameter::default();
        param.text = Some(text.to_string());
        param
    }

    /// 通过 JSON 字符串构造 RequestParameter（value 模式）
    fn make_param_with_value(json: &str) -> RequestParameter {
        let mut param = RequestParameter::default();
        param.value = Value::from_str(json).unwrap_or(Value::Null);
        param
    }

    /// 创建临时人像照片文件，返回文件名
    fn create_temp_portrait() -> String {
        let portrait_dir = soma_core::utils::storage_dir("portraits", true);
        let portrait_name = format!("test-portrait-{}.jpg", soma_core::utils::get_uuid());
        let portrait_path = portrait_dir.join(&portrait_name);
        std::fs::write(&portrait_path, b"test image content").unwrap();
        portrait_name
    }

    /// 清理临时人像照片文件
    fn cleanup_portrait(name: &str) {
        let portrait_dir = soma_core::utils::storage_dir("portraits", false);
        let portrait_path = portrait_dir.join(name);
        std::fs::remove_file(&portrait_path).ok();
    }

    /// 构造有效的 DigitalHumanParams JSON
    fn valid_params_json(portrait: &str) -> String {
        format!(
            r#"{{"portrait_image":"{}","narration_text":"这是一段测试文案"}}"#,
            portrait
        )
    }

    // ===== parse_params 测试 =====

    #[test]
    fn test_parse_params_from_text_valid() {
        let param = make_param_with_text(r#"{"portrait_image":"test.jpg","narration_text":"文案"}"#);
        let result = parse_params(&param);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.portrait_image, "test.jpg");
        assert_eq!(params.narration_text, "文案");
    }

    #[test]
    fn test_parse_params_from_text_invalid_json() {
        let param = make_param_with_text("invalid json");
        let result = parse_params(&param);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_params_missing_required_field() {
        let param = make_param_with_text(r#"{"narration_text":"文案"}"#);
        let result = parse_params(&param);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_params_from_value_valid() {
        let param = make_param_with_value(r#"{"portrait_image":"v.jpg","narration_text":"值模式"}"#);
        let result = parse_params(&param);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.portrait_image, "v.jpg");
        assert_eq!(params.narration_text, "值模式");
    }

    #[test]
    fn test_parse_params_with_optional_fields() {
        let param = make_param_with_text(r#"{
            "portrait_image": "p.jpg",
            "narration_text": "文案",
            "voice_name": "zh-CN-XiaoxiaoNeural",
            "video_aspect": "16:9",
            "subtitle_enabled": true
        }"#);
        let result = parse_params(&param).unwrap();
        assert_eq!(result.voice_name, Some("zh-CN-XiaoxiaoNeural".to_string()));
        assert_eq!(result.video_aspect, Some("16:9".to_string()));
        assert_eq!(result.subtitle_enabled, Some(true));
    }

    // ===== validate_portrait 测试 =====

    #[test]
    fn test_validate_portrait_empty() {
        let result = validate_portrait("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_portrait_nonexistent() {
        let result = validate_portrait("nonexistent-file-12345.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_portrait_exists() {
        let name = create_temp_portrait();
        let result = validate_portrait(&name);
        assert!(result.is_ok());
        cleanup_portrait(&name);
    }

    // ===== validate_narration 测试 =====

    #[test]
    fn test_validate_narration_empty() {
        assert!(validate_narration("").is_err());
    }

    #[test]
    fn test_validate_narration_whitespace_only() {
        assert!(validate_narration("   \n\t  ").is_err());
    }

    #[test]
    fn test_validate_narration_valid() {
        assert!(validate_narration("这是一段测试文案").is_ok());
    }

    #[test]
    fn test_validate_narration_max_length() {
        let text = "a".repeat(DH_NARRATION_TEXT_MAX_LEN);
        assert!(validate_narration(&text).is_ok());
    }

    #[test]
    fn test_validate_narration_exceeds_max_length() {
        let text = "a".repeat(DH_NARRATION_TEXT_MAX_LEN + 1);
        assert!(validate_narration(&text).is_err());
    }


    // ===== check_sensitive_words 测试 =====

    #[test]
    fn test_check_sensitive_words_clean_text() {
        ensure_setup();
        let result = check_sensitive_words("这是一段干净的测试文案");
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_sensitive_words_with_sensitive_word() {
        ensure_setup();
        let result = check_sensitive_words("这段文案包含示例敏感词");
        assert!(result.is_err());
    }

    // ===== distribute / distribute_tasks 测试 =====

    #[tokio::test]
    async fn test_distribute_unknown_method() {
        let mut param = RequestParameter::default();
        param.method = "unknown_method".to_string();
        let result = distribute(&param).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_distribute_tasks_unknown_method() {
        let mut param = RequestParameter::default();
        param.method = "unknown_method".to_string();
        let result = distribute_tasks(&param).await;
        assert!(result.is_err());
    }

    // ===== create_draft 测试 =====

    #[tokio::test]
    async fn test_create_draft_success() {
        ensure_setup();
        let portrait = create_temp_portrait();
        let param = make_param_with_text(&valid_params_json(&portrait));

        let result = create_draft(&param).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        let task_id = value.get_def_string("taskId", "");
        assert!(!task_id.is_empty());

        state::delete_dh_task(&task_id);
        cleanup_portrait(&portrait);
    }

    #[tokio::test]
    async fn test_create_draft_invalid_params() {
        let param = make_param_with_text("invalid json");
        let result = create_draft(&param).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_draft_portrait_not_found() {
        let param = make_param_with_text(r#"{"portrait_image":"nonexistent-12345.jpg","narration_text":"文案"}"#);
        let result = create_draft(&param).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_draft_empty_narration() {
        let portrait = create_temp_portrait();
        let param = make_param_with_text(&format!(
            r#"{{"portrait_image":"{}","narration_text":""}}"#,
            portrait
        ));
        let result = create_draft(&param).await;
        assert!(result.is_err());
        cleanup_portrait(&portrait);
    }

    #[tokio::test]
    async fn test_create_draft_sensitive_word() {
        ensure_setup();
        let portrait = create_temp_portrait();
        let param = make_param_with_text(&format!(
            r#"{{"portrait_image":"{}","narration_text":"包含示例敏感词的文案"}}"#,
            portrait
        ));
        let result = create_draft(&param).await;
        assert!(result.is_err());
        cleanup_portrait(&portrait);
    }


    // ===== start_task 测试（错误路径） =====

    #[tokio::test]
    async fn test_start_task_missing_id() {
        let param = make_param_with_value(r#"{"other":"value"}"#);
        let result = start_task(&param).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_start_task_not_found() {
        let param = make_param_with_value(r#"{"taskId":"nonexistent-task-id-12345"}"#);
        let result = start_task(&param).await;
        assert!(result.is_err());
    }

    // ===== list 测试 =====

    #[tokio::test]
    async fn test_list_default_pagination() {
        let param = make_param_with_value(r#"{}"#);
        let result = list(&param).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.contains_key("list"));
        assert!(value.contains_key("total"));
        assert!(value.contains_key("page"));
        assert!(value.contains_key("pageSize"));
    }

    #[tokio::test]
    async fn test_list_custom_pagination() {
        let param = make_param_with_value(r#"{"page":2,"pageSize":5}"#);
        let result = list(&param).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        let page = value.get_def_string("page", "1");
        let page_size = value.get_def_string("pageSize", "10");
        assert_eq!(page, "2");
        assert_eq!(page_size, "5");
    }

    #[tokio::test]
    async fn test_list_with_existing_tasks() {
        let portrait = create_temp_portrait();
        let params: DigitalHumanParams = serde_json::from_str(&valid_params_json(&portrait)).unwrap();
        let task_id1 = soma_core::utils::get_uuid();
        let task_id2 = soma_core::utils::get_uuid();
        state::create_dh_draft_task_entry(&task_id1, params.clone());
        state::create_dh_draft_task_entry(&task_id2, params);

        let param = make_param_with_value(r#"{"page":1,"pageSize":10}"#);
        let result = list(&param).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        let total_str = value.get_def_string("total", "0");
        let total: usize = total_str.parse().unwrap_or(0);
        assert!(total >= 2);

        state::delete_dh_task(&task_id1);
        state::delete_dh_task(&task_id2);
        cleanup_portrait(&portrait);
    }

    // ===== get 测试 =====

    #[tokio::test]
    async fn test_get_missing_id() {
        let param = make_param_with_value(r#"{"other":"value"}"#);
        let result = get(&param).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let param = make_param_with_value(r#"{"taskId":"nonexistent-task-get-12345"}"#);
        let result = get(&param).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_existing_task() {
        let portrait = create_temp_portrait();
        let params: DigitalHumanParams = serde_json::from_str(&valid_params_json(&portrait)).unwrap();
        let task_id = soma_core::utils::get_uuid();
        state::create_dh_draft_task_entry(&task_id, params);

        let param = make_param_with_value(&format!(r#"{{"taskId":"{}"}}"#, task_id));
        let result = get(&param).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        let returned_id = value.get_def_string("taskId", "");
        assert_eq!(returned_id, task_id);

        state::delete_dh_task(&task_id);
        cleanup_portrait(&portrait);
    }

    // ===== delete 测试 =====

    #[tokio::test]
    async fn test_delete_missing_id() {
        let param = make_param_with_value(r#"{"other":"value"}"#);
        let result = delete(&param).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_success() {
        let portrait = create_temp_portrait();
        let params: DigitalHumanParams = serde_json::from_str(&valid_params_json(&portrait)).unwrap();
        let task_id = soma_core::utils::get_uuid();
        state::create_dh_draft_task_entry(&task_id, params);

        let param = make_param_with_value(&format!(r#"{{"taskId":"{}"}}"#, task_id));
        let result = delete(&param).await;
        assert!(result.is_ok());
        let value = result.unwrap();
        let deleted_str = value.get_def_string("deleted", "false");
        assert_eq!(deleted_str, "true");

        assert!(state::get_dh_task(&task_id).is_none());
        cleanup_portrait(&portrait);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_task() {
        let param = make_param_with_value(r#"{"taskId":"nonexistent-delete-12345"}"#);
        let result = delete(&param).await;
        assert!(result.is_ok());
    }
}