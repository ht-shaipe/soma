use async_trait::async_trait;
use soma_core::error::SomaError;
use soma_core::models::{MaterialInfo, VideoAspect};
use crate::{SomaStockProvider, get_api_key};

pub struct Pixabay {
    api_keys: Vec<String>,
}

impl Pixabay {
    pub fn new(api_keys: &[String]) -> Self {
        Self { api_keys: api_keys.to_vec() }
    }
}

#[async_trait(?Send)]
impl SomaStockProvider for Pixabay {
    async fn search(&self, keyword: &str, video_aspect: &VideoAspect, min_duration: u32) -> Result<Vec<MaterialInfo>, SomaError> {
        let api_key = get_api_key(&self.api_keys)?;
        let (target_w, _) = video_aspect.to_resolution();
        let url = format!(
            "https://pixabay.com/api/videos/?q={}&video_type=all&per_page=50&key={}",
            urlencoding::encode(keyword),
            api_key,
        );

        let resp = reqwest::get(&url).await.map_err(|e| SomaError::Http(e.to_string()))?;
        let body: serde_json::Value = resp.json().await.map_err(|e| SomaError::Http(e.to_string()))?;
        let hits = body.get("hits")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut items = Vec::new();
        for v in hits {
            let duration = v.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0);
            if (duration as u32) < min_duration {
                continue;
            }
            if let Some(files) = v.get("videos").and_then(|f| f.as_object()) {
                for (_quality, vf) in files {
                    let w = vf.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                    if w >= target_w {
                        if let Some(url) = vf.get("url").and_then(|u| u.as_str()) {
                            items.push(MaterialInfo {
                                provider: "pixabay".into(),
                                url: url.to_string(),
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
