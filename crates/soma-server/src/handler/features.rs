use crate::Config;
use soma_feature::{FeatureRequest, FeatureStatus, NoopProgress};
/// 功能点统一 API 处理器
///
/// - list: 列举全部已注册功能点（含输入输出 JSON Schema，前端据此渲染工作台表单）
/// - run: 裸调用一个功能点（无需任务上下文；产物落 storage/features/{feature}/{run_id}/）
/// - history: 查询某功能点的历史运行记录（读取运行记录文件，按结束时间倒序）
///
/// run 为同步长任务（LLM/TTS/合成可能耗时数分钟），通过 web::block 交给阻塞线程池执行，
/// 避免阻塞 actix worker。功能点执行失败（status=failed）按既有约定返回错误响应。
use tube::{Result, Value};
use tube_web::RequestParameter;

/// 功能点统一分发入口：`features.list`（注册表与 Schema）/ `run`（裸调用）/ `history`（运行历史）
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list(),
        "run" => run(param).await,
        "history" => history(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 列举全部功能点
fn list() -> Result<Value> {
    let descriptors = crate::service::registry::FEATURE_REGISTRY.list();
    Ok(Value::from_serialize(&descriptors).unwrap_or(Value::Null))
}

/// 裸调用功能点
///
/// 请求体字段：featureId（必填）、runId（可选，缺省自动生成）、input（功能点入参对象）。
/// 字段同时兼容 snake_case（feature_id / run_id）。
async fn run(param: &RequestParameter) -> Result<Value> {
    // 解析请求：优先从 text 解析，否则从 value 对象解析（与 videos.create 一致）
    let req: FeatureRequest = if let Some(text) = &param.text {
        serde_json::from_str(text).map_err(|e| error!("参数解析失败: {}", e))?
    } else {
        let json_str = param.value.to_string();
        serde_json::from_str(&json_str).map_err(|e| error!("参数解析失败: {}", e))?
    };

    let conf = Config::get().app;

    // 功能点为同步阻塞执行，放入阻塞线程池
    let output = actix_web::web::block(move || {
        crate::service::registry::FEATURE_REGISTRY.run(&req, conf, &NoopProgress)
    })
    .await
    .map_err(|e| error!("功能点执行线程异常: {}", e))?
    .map_err(|e| error!("功能点执行失败: {}", e))?;

    match output.status {
        FeatureStatus::Success => Ok(Value::from_serialize(&output).unwrap_or(Value::Null)),
        FeatureStatus::Failed => Err(error!(
            "功能点 {} 执行失败: {}",
            output.feature_id,
            output.error.unwrap_or_else(|| "未知错误".into())
        )),
    }
}

/// 查询功能点历史运行记录
///
/// 请求体字段：featureId（或 feature_id）。
async fn history(param: &RequestParameter) -> Result<Value> {
    let feature_id = {
        let v = &param.value;
        let id = v.get_def_string("featureId", "");
        if id.is_empty() {
            v.get_def_string("feature_id", "")
        } else {
            id
        }
    };
    if feature_id.is_empty() {
        return Err(error!("缺少 featureId 参数"));
    }

    let runs = actix_web::web::block(move || {
        crate::service::registry::FEATURE_REGISTRY.list_runs(&feature_id)
    })
    .await
    .map_err(|e| error!("历史查询线程异常: {}", e))?
    .map_err(|e| error!("历史查询失败: {}", e))?;

    Ok(Value::from_serialize(&runs).unwrap_or(Value::Null))
}
