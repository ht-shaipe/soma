/// 全局功能点注册表（服务端宿主）
///
/// 服务端持有唯一的 [`FeatureRegistry`] 实例：
/// - 任务编排器（service/pipeline.rs）通过它逐步执行流水线功能点
/// - 后续 `features/*` HTTP 端点与 Tauri command 共享同一批功能点实现
///
/// 产物根目录默认 `storage/features/`，每次运行会写入
/// `input.json` 与 `run_record.json` 运行记录（按功能点/运行 ID 归档）。

use lazy_static::lazy_static;
use soma_feature::FeatureRegistry;

lazy_static! {
    pub static ref FEATURE_REGISTRY: FeatureRegistry = {
        let registry = FeatureRegistry::new();
        if let Err(e) = soma_feature::features::register_builtin(&registry) {
            log::error!("内置功能点注册失败: {}", e);
        }
        // 数字人依赖服务端任务状态存储，功能点实现留在服务端宿主
        macro_rules! register_dh {
            ($feature:expr) => {
                if let Err(e) = registry.register(std::sync::Arc::new($feature)) {
                    log::error!("数字人功能点注册失败: {}", e);
                }
            };
        }
        register_dh!(crate::service::digital_human::DigitalHumanVideoFeature);
        register_dh!(crate::service::digital_human::DigitalHumanPortraitFeature);
        register_dh!(crate::service::digital_human::DigitalHumanComposeFeature);
        registry
    };
}

/// 执行一个功能点并返回其输出数据
///
/// - 运行 ID 传任务 ID 时，运行记录归档到 `storage/features/{feature_id}/{task_id}/`，
///   便于任务详情查看每一步的输入输出（M3.2 步骤级重跑的数据基础）
/// - 功能点执行失败（status=Failed）转换为 `Err`，与旧流水线的错误传播语义一致
pub fn run_feature(
    feature_id: &str,
    run_id: &str,
    input: serde_json::Value,
    conf: &soma_core::config::AppConfig,
    progress: &dyn soma_feature::ProgressReporter,
) -> Result<serde_json::Value, soma_core::error::SomaError> {
    let output = FEATURE_REGISTRY.run(
        &soma_feature::FeatureRequest {
            feature_id: feature_id.to_string(),
            run_id: Some(run_id.to_string()),
            input,
        },
        conf.clone(),
        progress,
    )?;
    match output.status {
        soma_feature::FeatureStatus::Success => Ok(output.data),
        soma_feature::FeatureStatus::Failed => Err(soma_core::error::SomaError::Feature(
            output
                .error
                .unwrap_or_else(|| format!("功能点 {feature_id} 执行失败")),
        )),
    }
}
