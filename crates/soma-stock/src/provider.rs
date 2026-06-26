/// 素材供应商特征定义模块
///
/// 定义了 SomaStockProvider 特征（Trait），规定了所有视频素材供应商
/// 必须实现的搜索和下载接口。具体的供应商实现（Pexels、Pixabay、Coverr）
/// 位于各自的模块中。
use async_trait::async_trait;
use soma_core::{error::SomaError, models::MaterialInfo};

/// 素材供应商特征（Trait）
///
/// 所有视频素材供应商的抽象接口，要求实现搜索和下载能力。
/// 使用 `async_trait` 宏支持异步方法，`?Send` 标记表示
/// 异步 Future 不要求 `Send` 约束（兼容单线程运行时）。
#[async_trait(?Send)]
pub trait SomaStockProvider: Send + Sync {
    /// 搜索视频素材
    ///
    /// # 参数
    /// - `keyword`: 搜索关键词
    /// - `video_aspect`: 视频宽高比（字符串形式）
    /// - `count`: 请求返回的结果数量
    ///
    /// # 返回
    /// 搜索到的素材信息列表
    async fn search(&self, keyword: &str, video_aspect: &str, count: u32) -> Result<Vec<MaterialInfo>, SomaError>;

    /// 下载视频到指定路径
    ///
    /// # 参数
    /// - `url`: 视频下载地址
    /// - `output_path`: 本地保存路径
    ///
    /// # 返回
    /// 保存成功后的文件路径
    async fn download(&self, url: &str, output_path: &str) -> Result<String, SomaError>;
}
