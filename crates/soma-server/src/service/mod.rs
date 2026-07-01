/// 业务服务模块
///
/// 导出流水线执行（pipeline）和 LLM 服务（llm）两个子模块，
/// 为 handler 层提供核心业务逻辑调用。

/// 视频生成流水线 - 完整的6步任务执行逻辑
pub mod pipeline;
/// LLM 服务 - 脚本生成、关键词提取、社交元数据生成
pub mod llm;
/// 跨平台发布服务 - Upload-Post API
pub mod upload;
