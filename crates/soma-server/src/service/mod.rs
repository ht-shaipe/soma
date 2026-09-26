/// 业务服务模块
///
/// 导出流水线执行（pipeline）和 LLM 服务（llm）两个子模块，
/// 为 handler 层提供核心业务逻辑调用。
/// 视频生成流水线 - 基于 Feature 编排的完整任务执行逻辑
pub mod pipeline;
/// 功能点注册表 - 服务端宿主的全局 FeatureRegistry 与统一运行入口
pub mod registry;
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
/// Live2D 模型管理
pub mod live2d_model;
/// 图片故事视频生成流水线
pub mod image_story;
