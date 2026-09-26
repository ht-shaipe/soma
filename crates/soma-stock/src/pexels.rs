use crate::{get_api_key, SomaStockProvider};
/// Pexels 视频素材供应商实现
///
/// 封装 Pexels API（https://www.pexels.com/api/）的视频搜索功能。
/// Pexels 是一个提供免费高质量视频素材的平台，需要 API Key 认证。
/// 搜索结果按指定宽高比筛选，优先匹配精确分辨率。
use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::{MaterialInfo, VideoAspect};

/// Pexels 视频素材供应商
///
/// 通过 Pexels API 搜索免费视频素材，支持关键词搜索和宽高比筛选。
pub struct Pexels {
    /// Pexels API 密钥列表，支持多个密钥轮换
    api_keys: Vec<String>,
}

impl Pexels {
    /// 创建 Pexels 供应商实例
    ///
    /// # 参数
    /// - `api_keys`: Pexels API 密钥列表
    pub fn new(api_keys: &[String]) -> Self {
        Self {
            api_keys: api_keys.to_vec(),
        }
    }
}

#[async_trait(?Send)]
impl SomaStockProvider for Pexels {
    /// 搜索 Pexels 视频素材
    ///
    /// 调用 Pexels Videos Search API，按关键词和宽高比搜索视频，
    /// 并从返回结果中筛选出符合目标分辨率的视频文件链接。
    ///
    /// # 参数
    /// - `keyword`: 搜索关键词
    /// - `video_aspect`: 视频宽高比要求（用于确定目标分辨率和方向）
    /// - `min_duration`: 视频最小时长（秒），时长不足的视频将被过滤
    ///
    /// # 返回
    /// 符合条件的素材信息列表，每个视频包含精确匹配分辨率的下载链接
    async fn search(
        &self,
        keyword: &str,
        video_aspect: &VideoAspect,
        min_duration: u32,
    ) -> Result<Vec<MaterialInfo>, SomaError> {
        let api_key = get_api_key(&self.api_keys)?;
        // 获取视频方向参数（landscape/portrait/square）
        let orientation = video_aspect.orientation();
        let client = reqwest::Client::new();
        // 构造 Pexels 视频搜索 API 请求 URL
        let url = format!(
            "https://api.pexels.com/videos/search?query={}&per_page=20&orientation={}",
            urlencoding::encode(keyword),
            orientation,
        );

        // 发起带认证头的 HTTP 请求
        let resp = client
            .get(&url)
            .header("Authorization", &api_key)
            .header("User-Agent", "Mozilla/5.0")
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        // 解析 JSON 响应体
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;
        // 提取 videos 数组，无数据时返回空列表
        let videos = body
            .get("videos")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // 获取目标分辨率，用于精确匹配视频文件
        let (target_w, target_h) = video_aspect.to_resolution();
        let mut items = Vec::new();

        for v in videos {
            let duration = v.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0);
            if (duration as u32) < min_duration {
                continue;
            }
            if let Some(files) = v.get("video_files").and_then(|f| f.as_array()) {
                let mut best_match: Option<(u32, &serde_json::Value)> = None;
                for vf in files {
                    let w = vf.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                    let h = vf.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
                    if w == 0 || h == 0 {
                        continue;
                    }
                    let exact = w == target_w && h == target_h;
                    let wide_enough = w >= target_w && h >= target_h;
                    let near_match = w >= (target_w as f64 * 0.8) as u32;
                    let quality = if exact {
                        3
                    } else if wide_enough {
                        2
                    } else if near_match {
                        1
                    } else {
                        0
                    };
                    let current_best = best_match.as_ref().map(|(q, _)| *q).unwrap_or(0);
                    if (quality > current_best || (quality == current_best && quality > 0))
                        && quality > 0
                    {
                        best_match = Some((quality, vf));
                    }
                }
                if let Some((_, vf)) = best_match {
                    if let Some(link) = vf.get("link").and_then(|l| l.as_str()) {
                        items.push(MaterialInfo {
                            provider: "pexels".into(),
                            url: link.to_string(),
                            duration,
                        });
                    }
                }
            }
        }
        Ok(items)
    }
}
