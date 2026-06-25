use thiserror::Error;

#[derive(Error, Debug)]
pub enum SomaError {
    #[error("配置错误: {0}")]
    Config(String),
    #[error("LLM 调用失败: {0}")]
    Llm(String),
    #[error("TTS 合成失败: {0}")]
    Tts(String),
    #[error("素材获取失败: {0}")]
    Stock(String),
    #[error("视频合成失败: {0}")]
    Video(String),
    #[error("FFmpeg 执行失败: {0}")]
    Ffmpeg(String),
    #[error("任务不存在: {0}")]
    TaskNotFound(String),
    #[error("任务队列已满")]
    TaskQueueFull,
    #[error("文件路径不安全: {0}")]
    UnsafePath(String),
    #[error("HTTP 请求失败: {0}")]
    Http(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),
}
