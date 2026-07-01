//! 错误类型模块
//!
//! 定义 soma-core 中统一的错误枚举 SomaError，
//! 使用 thiserror 派生宏实现 std::error::Error 和 Display trait。

use thiserror::Error;

/// 核心错误枚举，涵盖配置、LLM、TTS、素材、视频合成等各环节的错误类型
#[derive(Error, Debug)]
pub enum SomaError {
    /// 配置文件解析或参数错误
    #[error("配置错误: {0}")]
    Config(String),
    /// LLM 大语言模型调用失败（API 错误、超时、响应格式异常等）
    #[error("LLM 调用失败: {0}")]
    Llm(String),
    /// TTS 语音合成失败（Edge TTS / Azure / ElevenLabs 等服务错误）
    #[error("TTS 合成失败: {0}")]
    Tts(String),
    /// 素材获取失败（Pexels / Pixabay / Coverr API 请求或下载错误）
    #[error("素材获取失败: {0}")]
    Stock(String),
    /// 视频合成失败（FFmpeg 合并/转码/叠加字幕等处理错误）
    #[error("视频合成失败: {0}")]
    Video(String),
    /// FFmpeg 命令执行失败（进程启动失败或返回非零退出码）
    #[error("FFmpeg 执行失败: {0}")]
    Ffmpeg(String),
    /// 指定任务 ID 不存在
    #[error("任务不存在: {0}")]
    TaskNotFound(String),
    /// 任务队列已满，无法接受新任务
    #[error("任务队列已满")]
    TaskQueueFull,
    /// 文件路径不安全（路径遍历攻击检测，路径超出允许目录）
    #[error("文件路径不安全: {0}")]
    UnsafePath(String),
    /// HTTP 请求失败（网络连接、状态码异常等）
    #[error("HTTP 请求失败: {0}")]
    Http(String),
    /// 跨平台发布失败
    #[error("发布失败: {0}")]
    Upload(String),
    /// IO 错误（文件读写、目录操作等），自动从 std::io::Error 转换
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    /// JSON 序列化/反序列化错误，自动从 serde_json::Error 转换
    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),
}
