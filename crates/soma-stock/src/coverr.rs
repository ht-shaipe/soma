use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::{MaterialInfo, VideoAspect};
use crate::{SomaStockProvider, get_api_key};

pub struct Coverr {
    api_keys: Vec<String>,
}

impl Coverr {
    pub fn new(api_keys: &[String]) -> Self {
        Self { api_keys: api_keys.to_vec() }
    }
}

#[async_trait(?Send)]
impl SomaStockProvider for Coverr {
    async fn search(&self, keyword: &str, _video_aspect: &VideoAspect, min_duration: u32) -> Result<Vec<MaterialInfo>, SomaError> {
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

        let body: serde_json::Value = resp.json().await.map_err(|e| SomaError::Http(e.to_string()))?;
        let hits = body.get("hits")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut items = Vec::new();
        for v in hits {
            let duration = v.get("duration")
                .and_then(|d| d.as_f64())
                .or_else(|| v.get("duration").and_then(|d| d.as_str()).and_then(|s| s.parse::<f64>().ok()))
                .unwrap_or(0.0);
            if (duration as u32) < min_duration {
                continue;
            }
            let mp4_url = v.get("urls")
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
