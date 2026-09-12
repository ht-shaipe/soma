//! Live2D 模型管理
//!
//! 管理各 Live2D 卡通模型的资产（.model3.json + 纹理 + 动作 + 表情），
//! 模型以 `{models_dir}/{model_id}/` 目录形式持久化，
//! 支持上传、列表、删除、就绪校验等操作。

use std::path::PathBuf;
use soma_core::error::SomaError;
use soma_core::models::{Live2DModel, Live2DModelStatus, TaskStatus};
use soma_core::utils::validate_live2d_model_id;
use crate::state;

/// Live2D 模型资产存储
pub struct Live2DModelStore {
    models_dir: PathBuf,
}

impl Live2DModelStore {
    pub fn new(models_dir: &str) -> Self {
        let dir = PathBuf::from(models_dir);
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
        Self { models_dir: dir }
    }

    fn model_dir(&self, model_id: &str) -> Result<PathBuf, SomaError> {
        validate_live2d_model_id(model_id)?;
        Ok(self.models_dir.join(model_id))
    }

    fn model_meta_path(&self, model_id: &str) -> Result<PathBuf, SomaError> {
        Ok(self.model_dir(model_id)?.join("model.json"))
    }

    /// 上传模型包：校验 → 剥离可执行文件 → 原子写入
    pub fn upload(
        &self,
        model_id: &str,
        package_dir: &str,
        overwrite_confirm: bool,
    ) -> Result<Live2DModel, SomaError> {
        validate_live2d_model_id(model_id)?;

        let dest_dir = self.model_dir(model_id)?;
        if dest_dir.exists() && !overwrite_confirm {
            return Err(SomaError::Config(format!(
                "模型标识 {} 已存在，请使用新标识或确认覆盖",
                model_id
            )));
        }

        let validation = Self::validate_package(package_dir)?;
        Self::strip_executables(package_dir);

        if dest_dir.exists() {
            std::fs::remove_dir_all(&dest_dir).map_err(SomaError::Io)?;
        }
        std::fs::create_dir_all(&dest_dir).map_err(SomaError::Io)?;

        // 拷贝模型文件
        copy_dir_all(package_dir, &dest_dir)?;

        let model = Live2DModel {
            model_id: model_id.to_string(),
            model3_json: validation.model3_json,
            textures: validation.textures,
            motions: validation.motions,
            expressions: validation.expressions,
            package_size_mb: validation.package_size_mb,
            uploaded_at: chrono::Utc::now(),
            model_status: Live2DModelStatus::Available,
        };

        self.write_model_meta_atomic(model_id, &model)?;
        Ok(model)
    }

    /// 列出所有模型
    pub fn list_models(&self) -> Result<Vec<Live2DModel>, SomaError> {
        let mut result = Vec::new();
        if !self.models_dir.exists() {
            return Ok(result);
        }
        for entry in std::fs::read_dir(&self.models_dir).map_err(SomaError::Io)? {
            let entry = entry.map_err(SomaError::Io)?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(".tmp_") {
                continue;
            }
            let meta_path = entry.path().join("model.json");
            if !meta_path.exists() {
                continue;
            }
            match std::fs::read_to_string(&meta_path) {
                Ok(content) => {
                    if let Ok(model) = serde_json::from_str::<Live2DModel>(&content) {
                        result.push(model);
                    } else {
                        log::warn!("模型元数据损坏: {}", meta_path.display());
                    }
                }
                Err(_) => {
                    log::warn!("读取模型元数据失败: {}", meta_path.display());
                }
            }
        }
        Ok(result)
    }

    /// 获取模型目录路径
    pub fn get_model_path(&self, model_id: &str) -> Result<PathBuf, SomaError> {
        let dir = self.model_dir(model_id)?;
        if !dir.exists() {
            return Err(SomaError::Config(format!(
                "Live2D 模型 {} 不存在",
                model_id
            )));
        }
        Ok(dir)
    }

    /// 校验模型是否就绪
    pub fn check_model_ready(&self, model_id: &str) -> Result<bool, SomaError> {
        let meta_path = self.model_meta_path(model_id)?;
        if !meta_path.exists() {
            return Ok(false);
        }
        let content = std::fs::read_to_string(&meta_path).map_err(SomaError::Io)?;
        let model: Live2DModel =
            serde_json::from_str(&content).map_err(|e| SomaError::Config(format!("解析 model.json 失败: {}", e)))?;
        if model.model_status != Live2DModelStatus::Available {
            return Ok(false);
        }
        let model3_path = self.model_dir(model_id)?.join(&model.model3_json);
        Ok(model3_path.exists())
    }

    /// 查询该模型是否有进行中任务
    pub fn has_running_tasks(&self, model_id: &str) -> bool {
        let (tasks, _) = state::get_all_dh_tasks(1, 10000);
        let processing = TaskStatus::Processing.as_i32();
        tasks.iter().any(|t| {
            t.live2d_model_id.as_deref() == Some(model_id) && t.state == processing
        })
    }

    /// 删除模型
    pub fn delete_model(&self, model_id: &str) -> Result<(), SomaError> {
        if self.has_running_tasks(model_id) {
            return Err(SomaError::VideoGen(
                "该模型有进行中任务，请等待完成后再删除".into(),
            ));
        }
        let dir = self.model_dir(model_id)?;
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(SomaError::Io)?;
        }
        Ok(())
    }

    fn write_model_meta_atomic(
        &self,
        model_id: &str,
        model: &Live2DModel,
    ) -> Result<(), SomaError> {
        let path = self.model_meta_path(model_id)?;
        let tmp = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(model)
            .map_err(|e| SomaError::Config(format!("序列化 model.json 失败: {}", e)))?;
        std::fs::write(&tmp, content).map_err(SomaError::Io)?;
        std::fs::rename(&tmp, &path).map_err(SomaError::Io)?;
        Ok(())
    }

    fn validate_package(package_dir: &str) -> Result<PackageValidation, SomaError> {
        let dir = PathBuf::from(package_dir);
        if !dir.exists() {
            return Err(SomaError::Config("模型包目录不存在".into()));
        }

        // 查找 .model3.json
        let mut model3_json = None;
        for entry in std::fs::read_dir(&dir).map_err(SomaError::Io)? {
            let entry = entry.map_err(SomaError::Io)?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".model3.json") {
                model3_json = Some(name);
                break;
            }
        }
        let model3_json = model3_json.ok_or_else(|| {
            SomaError::Config("模型文件不完整：缺少 .model3.json".into())
        })?;

        // 计算包大小
        let package_size_mb = dir_size_mb(&dir)?;
        if package_size_mb > 100.0 {
            return Err(SomaError::Config(format!(
                "模型包过大（{:.1} MB），上限 100 MB",
                package_size_mb
            )));
        }

        // 解析 model3.json 提取引用文件
        let model3_path = dir.join(&model3_json);
        let content = std::fs::read_to_string(&model3_path).map_err(SomaError::Io)?;
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| SomaError::Config(format!("解析 .model3.json 失败: {}", e)))?;

        let textures = extract_file_list(&json, "Textures");
        let motions = extract_file_list(&json, "Motions");
        let expressions = extract_file_list(&json, "Expressions");

        // 校验纹理文件存在
        for tex in &textures {
            if !dir.join(tex).exists() {
                return Err(SomaError::Config(format!(
                    "模型纹理文件缺失：{}",
                    tex
                )));
            }
        }

        Ok(PackageValidation {
            model3_json,
            textures,
            motions,
            expressions,
            package_size_mb,
        })
    }

    fn strip_executables(package_dir: &str) -> Vec<String> {
        let dir = PathBuf::from(package_dir);
        let dangerous_exts = [".py", ".sh", ".exe", ".bat", ".cmd"];
        let mut stripped = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if dangerous_exts.iter().any(|ext| name.ends_with(ext)) {
                    if std::fs::remove_file(entry.path()).is_ok() {
                        log::warn!("已剥离可执行文件 {}，仅保留模型资产", name);
                        stripped.push(name);
                    }
                }
            }
        }
        stripped
    }
}

struct PackageValidation {
    model3_json: String,
    textures: Vec<String>,
    motions: Vec<String>,
    expressions: Vec<String>,
    package_size_mb: f64,
}

fn extract_file_list(json: &serde_json::Value, key: &str) -> Vec<String> {
    json.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn dir_size_mb(dir: &PathBuf) -> Result<f64, SomaError> {
    let mut total: u64 = 0;
    for entry in std::fs::read_dir(dir).map_err(SomaError::Io)? {
        let entry = entry.map_err(SomaError::Io)?;
        let meta = entry.metadata().map_err(SomaError::Io)?;
        if meta.is_file() {
            total += meta.len();
        }
    }
    Ok(total as f64 / 1024.0 / 1024.0)
}

fn copy_dir_all(src: &str, dest: &PathBuf) -> Result<(), SomaError> {
    for entry in std::fs::read_dir(src).map_err(SomaError::Io)? {
        let entry = entry.map_err(SomaError::Io)?;
        let dest_path = dest.join(entry.file_name());
        let meta = entry.metadata().map_err(SomaError::Io)?;
        if meta.is_file() {
            std::fs::copy(entry.path(), &dest_path).map_err(SomaError::Io)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_dir_rejects_invalid_id() {
        let store = Live2DModelStore::new("/tmp/live2d_test");
        assert!(store.model_dir("../escape").is_err());
        assert!(store.model_dir("a/b").is_err());
        assert!(store.model_dir("").is_err());
        assert!(store.model_dir("valid_id").is_ok());
    }

    #[test]
    fn test_check_model_ready_nonexistent() {
        let store = Live2DModelStore::new("/tmp/live2d_test_nonexistent");
        assert_eq!(store.check_model_ready("no_such_model").unwrap(), false);
    }
}