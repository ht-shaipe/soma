/// 配置处理器 - 配置读写
pub mod config;
/// 数据导出处理器 - CSV / JSON / JSONL 通用导出
pub mod dataexport;
/// 数字人口播视频处理器 - 口播任务创建、列表、查询、删除
pub mod digital_human;
/// 视频下载处理器 - 基于 yt-dlp 的通用视频下载
pub mod download;
/// 功能点处理器 - 功能点列表、裸调用、历史记录
pub mod features;
/// 图片故事处理器 - 图片故事视频生成
pub mod image_story;
/// 剪映草稿处理器 - 生成剪映草稿文件
pub mod jianying;
/// LLM 处理器 - 脚本生成、关键词提取、社交元数据生成
pub mod llm;
/// 素材处理器 - 素材列表和上传
pub mod material;
/// 音乐处理器 - BGM 列表和上传
pub mod music;
/// 通知推送处理器 - Bark / 钉钉 / Telegram
pub mod notify;
/// 平台直连处理器 - 抖音 Web API（a_bogus 签名直连）
pub mod platform;
/// 流媒体处理器 - 视频播放和下载
pub mod stream;
/// 字幕处理处理器 - 翻译/校正/合并/格式转换
pub mod subtitle;
/// 系统环境处理器 - 依赖与配置体检
pub mod system;
/// 发布处理器 - 跨平台视频发布
pub mod upload;
/// 请求处理器模块
///
/// 按业务领域划分为多个子模块，各模块负责对应 API 请求的具体处理逻辑：
/// - video: 视频任务创建、查询、删除
/// - llm: LLM 脚本生成、关键词提取、社交元数据生成
/// - music: 音乐文件列表和上传
/// - material: 素材文件列表和上传
/// - stream: 视频流播放和下载
/// - config: 配置读写
/// - image_story: 图片故事视频生成
///
/// 视频任务处理器 - 任务创建、列表、查询、删除
pub mod video;
/// 语音处理器 - TTS 语音列表
pub mod voice;
