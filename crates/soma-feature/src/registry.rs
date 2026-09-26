//! 功能点注册表与统一运行器
//!
//! [`FeatureRegistry`] 是功能点体系的入口：
//! - 注册/查询/列举功能点（`features/list` 的数据源）
//! - 统一运行入口 [`FeatureRegistry::run`]：定位功能点 → 构建运行上下文
//!   → 执行（捕获 panic）→ 组装 [`FeatureOutput`] → 持久化运行记录
//! - 产物根目录可替换（默认 `storage/features`，桌面版迁移到系统数据目录时替换即可）

use crate::context::FeatureContext;
use crate::descriptor::FeatureDescriptor;
use crate::envelope::{FeatureInput, FeatureOutput, FeatureRequest, FeatureStatus};
use crate::feature::Feature;
use crate::progress::ProgressReporter;
use soma_core::config::AppConfig;
use soma_core::error::SomaError;
use soma_core::utils::get_uuid;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// 功能点注册表
///
/// 线程安全；一个宿主进程持有一个全局实例。
pub struct FeatureRegistry {
    features: RwLock<BTreeMap<String, Arc<dyn Feature>>>,
    /// 产物根目录（`{root}/{feature_id}/{run_id}/`）
    storage_root: RwLock<PathBuf>,
}

impl FeatureRegistry {
    /// 创建注册表，产物根目录默认为 `storage/features`
    pub fn new() -> Self {
        Self::with_storage_root(soma_core::utils::storage_dir("features", true))
    }

    /// 创建注册表并指定产物根目录（用于测试与桌面版数据目录迁移）
    pub fn with_storage_root(root: PathBuf) -> Self {
        Self {
            features: RwLock::new(BTreeMap::new()),
            storage_root: RwLock::new(root),
        }
    }

    /// 替换产物根目录（桌面版配置迁移后调用）
    pub fn set_storage_root(&self, root: PathBuf) {
        if let Ok(mut r) = self.storage_root.write() {
            *r = root;
        }
    }

    /// 当前产物根目录
    pub fn storage_root(&self) -> PathBuf {
        self.storage_root
            .read()
            .map(|r| r.clone())
            .unwrap_or_else(|_| soma_core::utils::storage_dir("features", false))
    }

    /// 注册功能点；同 ID 重复注册返回错误
    pub fn register(&self, feature: Arc<dyn Feature>) -> Result<(), SomaError> {
        let id = feature.descriptor().id;
        let mut map = self
            .features
            .write()
            .map_err(|_| SomaError::Feature("功能点注册表锁中毒".into()))?;
        if map.contains_key(&id) {
            return Err(SomaError::Feature(format!("功能点重复注册: {id}")));
        }
        map.insert(id, feature);
        Ok(())
    }

    /// 按 ID 查找功能点
    pub fn get(&self, id: &str) -> Option<Arc<dyn Feature>> {
        self.lock_features().get(id).cloned()
    }

    /// 列举全部功能点描述（按 ID 排序）
    pub fn list(&self) -> Vec<FeatureDescriptor> {
        self.lock_features()
            .values()
            .map(|f| f.descriptor())
            .collect()
    }

    /// 统一运行入口
    ///
    /// 流程：定位功能点 → 解析 run_id（缺省生成 UUID）→ 创建产物目录
    /// → 执行（捕获 panic）→ 组装输出 → 持久化 `input.json` + `run_record.json`。
    ///
    /// 返回值约定：
    /// - `Err`：运行器级错误（功能点不存在、目录创建失败等）
    /// - `Ok(output)`：功能点已执行；失败与否看 `output.status`（执行失败同样落盘运行记录）
    pub fn run(
        &self,
        req: &FeatureRequest,
        config: AppConfig,
        progress: &dyn ProgressReporter,
    ) -> Result<FeatureOutput, SomaError> {
        let feature = self
            .get(&req.feature_id)
            .ok_or_else(|| SomaError::Feature(format!("功能点不存在: {}", req.feature_id)))?;

        let run_id = req
            .run_id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(get_uuid);
        let work_dir = self
            .storage_root()
            .join(&req.feature_id)
            .join(&run_id);
        std::fs::create_dir_all(&work_dir)?;

        let input = FeatureInput {
            payload: req.input.clone(),
            meta: None,
        };
        let started_at = chrono::Utc::now();
        let mut ctx = FeatureContext::new(
            config,
            req.feature_id.clone(),
            run_id.clone(),
            work_dir.clone(),
        );

        // 捕获 panic：与任务队列（task.rs）保持一致的健壮性，
        // ctx/input 以借用方式进入闭包，panic 回卷后仍可读取已登记产物
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            feature.run(&mut ctx, &input, progress)
        }))
        .map_err(|p| SomaError::Feature(format!("功能点执行 panic: {}", panic_msg(p))))
        .and_then(|r| r);

        let finished_at = chrono::Utc::now();
        let duration_ms = (finished_at - started_at).num_milliseconds().max(0) as u64;

        let output = match &result {
            Ok(data) => FeatureOutput {
                feature_id: req.feature_id.clone(),
                run_id: run_id.clone(),
                status: FeatureStatus::Success,
                data: data.clone(),
                artifacts: ctx.artifacts(),
                error: None,
                input: req.input.clone(),
                started_at,
                finished_at,
                duration_ms,
            },
            Err(e) => FeatureOutput {
                feature_id: req.feature_id.clone(),
                run_id: run_id.clone(),
                status: FeatureStatus::Failed,
                data: serde_json::Value::Null,
                artifacts: ctx.artifacts(),
                error: Some(e.to_string()),
                input: req.input.clone(),
                started_at,
                finished_at,
                duration_ms,
            },
        };

        persist_record(&work_dir, &req.input, &output);

        Ok(output)
    }

    /// 列举某功能点的历史运行记录（按结束时间倒序）
    ///
    /// 直接读取产物目录下的 `run_record.json`，无需额外索引存储。
    pub fn list_runs(&self, feature_id: &str) -> Result<Vec<FeatureOutput>, SomaError> {
        let feature_root = self.storage_root().join(feature_id);
        if !feature_root.exists() {
            return Ok(vec![]);
        }
        let mut runs = Vec::new();
        for entry in std::fs::read_dir(&feature_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let record_path = entry.path().join("run_record.json");
            if !record_path.exists() {
                continue;
            }
            match std::fs::read_to_string(&record_path)
                .map_err(SomaError::Io)
                .and_then(|s| serde_json::from_str::<FeatureOutput>(&s).map_err(SomaError::from))
            {
                Ok(record) => runs.push(record),
                Err(e) => log::warn!("解析运行记录失败 {}: {}", record_path.display(), e),
            }
        }
        runs.sort_by(|a, b| {
            b.finished_at
                .cmp(&a.finished_at)
                // 同一毫秒内多次运行时 finished_at 可能并列，
                // 以 run_id 作决胜键保证历史顺序稳定可复现
                .then_with(|| b.run_id.cmp(&a.run_id))
        });
        Ok(runs)
    }

    fn lock_features(&self) -> std::sync::RwLockReadGuard<'_, BTreeMap<String, Arc<dyn Feature>>> {
        self.features
            .read()
            .unwrap_or_else(|e| e.into_inner())
    }
}

impl Default for FeatureRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 持久化运行记录到产物目录：`input.json` + `run_record.json`
///
/// 落盘失败仅告警不阻断（记录属于增强能力，不应影响运行结果返回）。
fn persist_record(work_dir: &std::path::Path, input: &serde_json::Value, output: &FeatureOutput) {
    let write = |filename: &str, value: &serde_json::Value| -> Result<(), SomaError> {
        let path = work_dir.join(filename);
        let content = serde_json::to_string_pretty(value)?;
        // 高负载下偶发瞬时写失败会静默丢失历史记录（list_runs 依赖此文件），
        // 与任务队列的落盘健壮性保持一致：失败重试一次
        match std::fs::write(&path, &content) {
            Ok(()) => Ok(()),
            Err(first) => {
                log::warn!("写入 {} 首次失败（{first}），重试", path.display());
                std::fs::write(&path, content).map_err(SomaError::Io)
            }
        }
    };
    if let Err(e) = write("input.json", input) {
        log::warn!("写入 input.json 失败: {}", e);
    }
    if let Ok(record) = serde_json::to_value(output) {
        if let Err(e) = write("run_record.json", &record) {
            log::warn!("写入 run_record.json 失败: {}", e);
        }
    }
}

/// 从 panic 值提取消息（与 soma-server task.rs 逻辑一致）
fn panic_msg(panic_val: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic_val.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = panic_val.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}
