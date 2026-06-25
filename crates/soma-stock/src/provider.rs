use async_trait::async_trait;
use soma_core::{error::SomaError, models::MaterialInfo};

#[async_trait(?Send)]
pub trait SomaStockProvider: Send + Sync {
    async fn search(&self, keyword: &str, video_aspect: &str, count: u32) -> Result<Vec<MaterialInfo>, SomaError>;
    async fn download(&self, url: &str, output_path: &str) -> Result<String, SomaError>;
}
