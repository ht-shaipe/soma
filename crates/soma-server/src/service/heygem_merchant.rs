//! HeyGem 商户模型资产管理
//!
//! 管理各商户的数字人训练资产（静音视频、参考音频、参考文本），
//! 资产以 `{assets_dir}/{merchant_id}/asset.json` 形式持久化，
//! 支持就绪校验、绑定、状态更新、删除等操作。

use std::path::PathBuf;
use soma_core::error::SomaError;
use soma_core::models::{AssetStatus, MerchantAsset, TaskStatus};
use soma_core::utils::validate_merchant_id;
use crate::state;

/// 商户模型资产存储
pub struct MerchantAssetStore {
    assets_dir: PathBuf,
}

impl MerchantAssetStore {
    pub fn new(assets_dir: &str) -> Self {
        Self {
            assets_dir: PathBuf::from(assets_dir),
        }
    }

    /// 计算商户资产目录路径，校验 merchant_id 合法性以防路径穿越
    fn merchant_dir(&self, merchant_id: &str) -> Result<PathBuf, SomaError> {
        validate_merchant_id(merchant_id)?;
        Ok(self.assets_dir.join(merchant_id))
    }

    /// 资产元数据文件路径
    fn asset_json_path(&self, merchant_id: &str) -> Result<PathBuf, SomaError> {
        Ok(self.merchant_dir(merchant_id)?.join("asset.json"))
    }

    /// 校验商户资产是否就绪：asset.json 存在、状态为 Ready、相关文件均存在
    pub fn check_ready(&self, merchant_id: &str) -> Result<bool, SomaError> {
        let asset = match self.get_asset(merchant_id) {
            Ok(a) => a,
            Err(_) => return Ok(false),
        };
        if asset.asset_status != AssetStatus::Ready {
            return Ok(false);
        }
        Ok(
            std::path::Path::new(&asset.silent_video_path).exists()
                && std::path::Path::new(&asset.reference_audio).exists(),
        )
    }

    /// 读取商户资产元数据
    pub fn get_asset(&self, merchant_id: &str) -> Result<MerchantAsset, SomaError> {
        let path = self.asset_json_path(merchant_id)?;
        if !path.exists() {
            return Err(SomaError::Config(format!(
                "商户 {} 不存在",
                merchant_id
            )));
        }
        let content = std::fs::read_to_string(&path).map_err(SomaError::Io)?;
        let asset: MerchantAsset =
            serde_json::from_str(&content).map_err(|e| SomaError::Config(format!(
                "解析 asset.json 失败: {}",
                e
            )))?;
        Ok(asset)
    }

    /// 绑定资产：将静音视频与参考音频拷贝至商户目录，写入 asset.json
    pub fn bind_asset(
        &self,
        merchant_id: &str,
        silent_video: &str,
        reference_audio: &str,
        reference_text: &str,
    ) -> Result<(), SomaError> {
        let dir = self.merchant_dir(merchant_id)?;
        std::fs::create_dir_all(&dir).map_err(SomaError::Io)?;

        let silent_dest = dir.join("silent_video.mp4");
        let audio_dest = dir.join("reference_audio.wav");

        copy_file(silent_video, &silent_dest)?;
        copy_file(reference_audio, &audio_dest)?;

        let asset = MerchantAsset {
            merchant_id: merchant_id.to_string(),
            silent_video_path: silent_dest.to_string_lossy().to_string(),
            reference_audio: audio_dest.to_string_lossy().to_string(),
            reference_text: reference_text.to_string(),
            trained_at: chrono::Utc::now(),
            asset_status: AssetStatus::Ready,
        };

        self.write_asset_atomic(merchant_id, &asset)
    }

    /// 更新资产状态
    pub fn set_status(
        &self,
        merchant_id: &str,
        status: AssetStatus,
    ) -> Result<(), SomaError> {
        let mut asset = self.get_asset(merchant_id)?;
        asset.asset_status = status;
        self.write_asset_atomic(merchant_id, &asset)
    }

    /// 查询该商户是否存在进行中任务
    pub fn has_running_tasks(&self, merchant_id: &str) -> bool {
        let (tasks, _) = state::get_all_dh_tasks(1, 10000);
        let processing = TaskStatus::Processing.as_i32();
        tasks.iter().any(|t| {
            t.merchant_id.as_deref() == Some(merchant_id) && t.state == processing
        })
    }

    /// 删除商户资产，存在进行中任务时拒绝
    pub fn delete_asset(&self, merchant_id: &str) -> Result<(), SomaError> {
        let running = self.has_running_tasks(merchant_id);
        if running {
            return Err(SomaError::VideoGen(
                "该商户有进行中任务，请等待完成后再删除资产".into(),
            ));
        }
        let dir = self.merchant_dir(merchant_id)?;
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(SomaError::Io)?;
        }
        Ok(())
    }

    /// 原子写入 asset.json（先写临时文件再 rename）
    fn write_asset_atomic(
        &self,
        merchant_id: &str,
        asset: &MerchantAsset,
    ) -> Result<(), SomaError> {
        let path = self.asset_json_path(merchant_id)?;
        let tmp = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(asset)
            .map_err(|e| SomaError::Config(format!("序列化 asset.json 失败: {}", e)))?;
        std::fs::write(&tmp, content).map_err(SomaError::Io)?;
        std::fs::rename(&tmp, &path).map_err(SomaError::Io)?;
        Ok(())
    }
}

fn copy_file(src: &str, dest: &PathBuf) -> Result<(), SomaError> {
    let src_path = std::path::Path::new(src);
    if !src_path.exists() {
        return Err(SomaError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("源文件不存在: {}", src),
        )));
    }
    if src_path == dest.as_path() {
        return Ok(());
    }
    std::fs::copy(src_path, dest).map_err(SomaError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merchant_dir_rejects_invalid_id() {
        let store = MerchantAssetStore::new("/tmp/heygem_test");
        assert!(store.merchant_dir("../escape").is_err());
        assert!(store.merchant_dir("a/b").is_err());
        assert!(store.merchant_dir("").is_err());
        assert!(store.merchant_dir("valid_id").is_ok());
    }

    #[test]
    fn test_check_ready_nonexistent() {
        let store = MerchantAssetStore::new("/tmp/heygem_test_nonexistent");
        assert_eq!(store.check_ready("no_such_merchant").unwrap(), false);
    }
}