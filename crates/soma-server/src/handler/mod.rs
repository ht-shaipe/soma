/// 请求处理器模块
///
/// 按业务领域划分为六个子模块，各模块负责对应 API 请求的具体处理逻辑：
/// - video: 视频任务创建、查询、删除
/// - llm: LLM 脚本生成、关键词提取、社交元数据生成
/// - music: 音乐文件列表和上传
/// - material: 素材文件列表和上传
/// - stream: 视频流播放和下载
/// - config: 配置读写

/// 视频任务处理器 - 任务创建、列表、查询、删除
pub mod video;
/// 数字人口播视频处理器 - 口播任务创建、列表、查询、删除
pub mod digital_human;
/// LLM 处理器 - 脚本生成、关键词提取、社交元数据生成
pub mod llm;
/// 音乐处理器 - BGM 列表和上传
pub mod music;
/// 素材处理器 - 素材列表和上传
pub mod material;
/// 流媒体处理器 - 视频播放和下载
pub mod stream;
/// 配置处理器 - 配置读写
pub mod config;
/// 语音处理器 - TTS 语音列表
pub mod voice;
/// 发布处理器 - 跨平台视频发布
pub mod upload;
