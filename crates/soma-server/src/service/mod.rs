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
/// 数字人口播视频生成流水线 - 三阶段（音频→口播视频→合成）
pub mod digital_human;
/// 数字人分段口播视频生成 - 长文案分段生成流程
pub mod segment_dh_video;
/// HeyGem 商户模型资产管理
pub mod heygem_merchant;
/// HeyGem 商户模型训练编排
pub mod heygem_trainer;
