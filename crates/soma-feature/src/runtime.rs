//! 运行时辅助：阻塞桥接 async future 与带退避的重试
//!
//! 功能点在宿主的工作线程上同步运行（任务队列 std::thread / Tauri spawn_blocking），
//! 内部调用异步能力（LLM / TTS / 素材下载）时通过 [`block_on_async`] 桥接。

use soma_core::error::SomaError;

/// 在当前线程上阻塞执行一个 async future（独立 current-thread runtime + LocalSet）
///
/// 支持非 Send 的 future，任务队列工作线程可直接调用。
pub fn block_on_async<F>(fut: F) -> Result<F::Output, SomaError>
where
    F: std::future::Future,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| SomaError::Llm(format!("tokio runtime 创建失败: {}", e)))?;
    let local = tokio::task::LocalSet::new();
    Ok(local.block_on(&rt, fut))
}

/// 带指数退避的重试：首次执行 + 最多 max_retries 次重试
///
/// 退避间隔 500ms × 2^attempt。
pub fn retry<F, T>(max_retries: usize, mut f: F) -> Result<T, SomaError>
where
    F: FnMut() -> Result<T, SomaError>,
{
    let mut last_err = None;
    for attempt in 0..=max_retries {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                log::warn!(
                    "retry attempt {}/{} failed: {:?}",
                    attempt + 1,
                    max_retries + 1,
                    e
                );
                if attempt < max_retries {
                    let delay = std::time::Duration::from_millis(500 * (1 << attempt) as u64);
                    std::thread::sleep(delay);
                }
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| SomaError::Llm("重试全部失败且无错误记录".into())))
}
