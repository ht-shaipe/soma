use crate::{get_api_key, SomaStockProvider};
/// Coverr 视频素材供应商实现
///
/// 封装 Coverr API（https://api.coverr.co）的视频搜索功能。
/// Coverr 是一个提供免费视频素材的平台，需要 API Key（Bearer Token）认证。
/// 注意：Coverr API 不支持按宽高比筛选，video_aspect 参数当前未使用。
use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::{MaterialInfo, VideoAspect};

/// Coverr 视频素材供应商
///
/// 通过 Coverr API 搜索免费视频素材，支持关键词搜索。
/// 当前不支持按宽高比筛选分辨率。
pub struct Coverr {
    /// Coverr API 密钥列表，支持多个密钥轮换
    api_keys: Vec<String>,
}

impl Coverr {
    /// 创建 Coverr 供应商实例
    ///
    /// # 参数
    /// - `api_keys`: Coverr API 密钥列表
    pub fn new(api_keys: &[String]) -> Self {
        Self {
            api_keys: api_keys.to_vec(),
        }
    }
}

#[async_trait(?Send)]
impl SomaStockProvider for Coverr {
    /// 搜索 Coverr 视频素材
    ///
    /// 调用 Coverr Videos API，按关键词搜索视频，
    /// 并从返回结果中提取 MP4 下载链接。
    /// 搜索结果按热门程度排序（sort=popular）。
    ///
    /// # 参数
    /// - `keyword`: 搜索关键词
    /// - `_video_aspect`: 视频宽高比要求（当前未使用，Coverr API 不支持分辨率筛选）
    /// - `min_duration`: 视频最小时长（秒），时长不足的视频将被过滤
    ///
    /// # 返回
    /// 符合条件的素材信息列表，每个视频包含 MP4 下载链接
    async fn search(
        &self,
        keyword: &str,
        video_aspect: &VideoAspect,
        min_duration: u32,
    ) -> Result<Vec<MaterialInfo>, SomaError> {
        let api_key = get_api_key(&self.api_keys)?;
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.coverr.co/videos?query={}&page_size=20&urls=true&sort=popular",
            urlencoding::encode(keyword),
        );

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;
        let hits = body
            .get("hits")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let (target_w, target_h) = video_aspect.to_resolution();
        let mut items = Vec::new();
        for v in hits {
            let duration = v
                .get("duration")
                .and_then(|d| d.as_f64())
                .or_else(|| {
                    v.get("duration")
                        .and_then(|d| d.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                })
                .unwrap_or(0.0);
            if (duration as u32) < min_duration {
                continue;
            }

            // 尝试从响应中获取分辨率信息进行后置过滤
            let video_w = v.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
            let video_h = v.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
            if video_w > 0 && video_h > 0 {
                let video_aspect_ratio = (video_w as f64) / (video_h as f64);
                let target_ratio = (target_w as f64) / (target_h as f64);
                // 允许 10% 的宽高比偏差
                if (video_aspect_ratio - target_ratio).abs() / target_ratio > 0.1 {
                    continue;
                }
            }

            let mp4_url = v
                .get("urls")
                .and_then(|u| u.get("mp4_download"))
                .and_then(|u| u.as_str());
            if let Some(url) = mp4_url {
                items.push(MaterialInfo {
                    provider: "coverr".into(),
                    url: url.to_string(),
                    duration,
                });
            }
        }
        Ok(items)
    }
}
