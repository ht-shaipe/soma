use crate::{get_api_key, SomaStockProvider};
/// Pixabay 视频素材供应商实现
///
/// 封装 Pixabay API（https://pixabay.com/api/docs/）的视频搜索功能。
/// Pixabay 是一个提供免费视频素材的平台，需要 API Key 认证。
/// 搜索结果按宽度筛选，选择宽度不低于目标值的视频文件。
use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::{MaterialInfo, VideoAspect};

/// Pixabay 视频素材供应商
///
/// 通过 Pixabay API 搜索免费视频素材，支持关键词搜索和宽高比筛选。
pub struct Pixabay {
    /// Pixabay API 密钥列表，支持多个密钥轮换
    api_keys: Vec<String>,
}

impl Pixabay {
    /// 创建 Pixabay 供应商实例
    ///
    /// # 参数
    /// - `api_keys`: Pixabay API 密钥列表
    pub fn new(api_keys: &[String]) -> Self {
        Self {
            api_keys: api_keys.to_vec(),
        }
    }
}

#[async_trait(?Send)]
impl SomaStockProvider for Pixabay {
    /// 搜索 Pixabay 视频素材
    ///
    /// 调用 Pixabay Videos API，按关键词搜索视频，
    /// 并从返回结果中筛选出宽度不低于目标值的视频文件链接。
    /// Pixabay 的视频按画质分组（large/medium/small 等），选择满足宽度要求的版本。
    ///
    /// # 参数
    /// - `keyword`: 搜索关键词
    /// - `video_aspect`: 视频宽高比要求（用于确定目标最小宽度）
    /// - `min_duration`: 视频最小时长（秒），时长不足的视频将被过滤
    ///
    /// # 返回
    /// 符合条件的素材信息列表，每个视频包含满足宽度要求的下载链接
    async fn search(
        &self,
        keyword: &str,
        video_aspect: &VideoAspect,
        min_duration: u32,
    ) -> Result<Vec<MaterialInfo>, SomaError> {
        let api_key = get_api_key(&self.api_keys)?;
        // 获取目标宽度，用于筛选满足分辨率要求的视频
        let (target_w, _) = video_aspect.to_resolution();
        // 构造 Pixabay 视频搜索 API 请求 URL，key 作为查询参数
        let url = format!(
            "https://pixabay.com/api/videos/?q={}&video_type=all&per_page=50&key={}",
            urlencoding::encode(keyword),
            api_key,
        );

        // Pixabay API key 在 URL 参数中传递，无需额外的认证头
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;
        // 解析 JSON 响应体
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;
        // 提取 hits 数组，无数据时返回空列表
        let hits = body
            .get("hits")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut items = Vec::new();
        for v in hits {
            let duration = v.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0);
            // 过滤掉时长不足的视频
            if (duration as u32) < min_duration {
                continue;
            }
            // 遍历视频的不同画质版本（按 quality 键分组，如 large/medium/small）
            if let Some(files) = v.get("videos").and_then(|f| f.as_object()) {
                for (_quality, vf) in files {
                    let w = vf.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                    // 选择宽度不低于目标值的视频版本（宽于目标即可接受）
                    if w >= target_w {
                        if let Some(url) = vf.get("url").and_then(|u| u.as_str()) {
                            items.push(MaterialInfo {
                                provider: "pixabay".into(),
                                url: url.to_string(),
                                duration,
                            });
                            // 找到满足条件的高画质版本后跳出，避免添加同一视频的多个格式
                            break;
                        }
                    }
                }
            }
        }
        Ok(items)
    }
}
