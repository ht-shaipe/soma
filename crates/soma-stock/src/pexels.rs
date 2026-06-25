use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::{MaterialInfo, VideoAspect};
use crate::{SomaStockProvider, get_api_key};

pub struct Pexels {
    api_keys: Vec<String>,
}

impl Pexels {
    pub fn new(api_keys: &[String]) -> Self {
        Self { api_keys: api_keys.to_vec() }
    }
}

#[async_trait(?Send)]
impl SomaStockProvider for Pexels {
    async fn search(&self, keyword: &str, video_aspect: &VideoAspect, min_duration: u32) -> Result<Vec<MaterialInfo>, SomaError> {
        let api_key = get_api_key(&self.api_keys)?;
        let orientation = video_aspect.orientation();
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.pexels.com/videos/search?query={}&per_page=20&orientation={}",
            urlencoding::encode(keyword),
            orientation,
        );

        let resp = client
            .get(&url)
            .header("Authorization", &api_key)
            .header("User-Agent", "Mozilla/5.0")
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| SomaError::Http(e.to_string()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| SomaError::Http(e.to_string()))?;
        let videos = body.get("videos")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let (target_w, target_h) = video_aspect.to_resolution();
        let mut items = Vec::new();

        for v in videos {
            let duration = v.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0);
            if (duration as u32) < min_duration {
                continue;
            }
            if let Some(files) = v.get("video_files").and_then(|f| f.as_array()) {
                for vf in files {
                    let w = vf.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                    let h = vf.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
                    if w == target_w && h == target_h {
                        if let Some(link) = vf.get("link").and_then(|l| l.as_str()) {
                            items.push(MaterialInfo {
                                provider: "pexels".into(),
                                url: link.to_string(),
                                duration,
                            });
                            break;
                        }
                    }
                }
            }
        }
        Ok(items)
    }
}
