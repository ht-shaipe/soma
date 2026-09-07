# Rust 异步调用 Python GPU 推理：block_on_async 模式详解

> Rust 项目要调用 Python GPU 推理，网上教程不多。tokio::spawn 会 panic，block_on 会报"runtime already running"，踩了一整天才找到正确姿势。

> **注意**：本文的 `block_on_async` 是**本项目自封装的函数**，不是 tokio 标准 API。你不会在 tokio 文档里搜到它。它的实现就在下文，20 行代码，复制即用。

## 问题背景

soma 是一个 Rust 项目，数字人推理用 Python（EchoMimicV3-Flash 基于 PyTorch）。调用方式是 SSH 到 GPU 服务器执行 Python 脚本，等它输出结果。

Rust 侧的 TTS Provider trait 是 async 的：

```rust
#[async_trait(?Send)]
pub trait SomaTtsProvider: Send + Sync {
    async fn synthesize(
        &self,
        text: &str,
        voice: &str,
        rate: f32,
        output_path: &Path,
    ) -> Result<TtsResult, SomaError>;
}
```

实现这个 trait 时，需要调用 SSH 执行 Python 脚本并等待结果。SSH 调用用 `tokio::process::Command`，它是 async 的。

看起来很简单，对吧？`async fn` 里 `await` 一下就行了。

**但会 panic。**

> 下面的坑一和坑二看起来矛盾——一个说"没有 runtime"，一个说"已经在 runtime 里"。这是因为它们发生在**不同的调用路径**：有的任务跑在 actix 的 tokio 线程里（有 runtime），有的跑在普通阻塞线程里（无 runtime）。两种情况都要处理。

## 坑一：tokio::spawn panic

第一版代码这样写：

```rust
async fn synthesize(&self, text: &str, ...) -> Result<TtsResult, SomaError> {
    let result = tokio::spawn(async {
        // SSH 调用
        Command::new("ssh").arg(...).output().await
    }).await.unwrap();
    
    Ok(result)
}
```

报错：

```
thread 'main' panicked at 'there is no reactor running, must be called from the context of a Tokio runtime'
```

原因：soma 的任务执行不在 tokio runtime 里。soma 用 actix-web 作为 HTTP 框架，任务通过 `block_on_async` 在独立 runtime 中执行。`tokio::spawn` 需要在 tokio runtime 上下文中调用，但 `block_on_async` 创建的是一个**临时的 current_thread runtime**。

## 坑二：block_on 嵌套报错

第二版尝试直接 `block_on`：

```rust
async fn synthesize(&self, text: &str, ...) -> Result<TtsResult, SomaError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        Command::new("ssh").arg(...).output().await
    });
    
    Ok(result)
}
```

报错：

```
"Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) is supposed to block the current thread while the future is executed, but that thread is already being used for the future."
```

原因：`synthesize` 本身就在 async 上下文里（被 `block_on_async` 调用），在 async 里再 `block_on` 会嵌套 runtime。

## 正确方案：block_on_async + LocalSet

最终的 `block_on_async` 函数：

```rust
fn block_on_async<F>(fut: F) -> Result<F::Output, SomaError>
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
```

关键点：

1. **`new_current_thread()`**：创建单线程 runtime，不用多线程（避免 Send 约束问题）
2. **`enable_all()`**：启用 IO 和 time 驱动（SSH 子进程需要 IO）
3. **`LocalSet`**：允许执行 `!Send` 的 future（reqwest/tokio::process 在 current_thread runtime 里可能不是 Send）
4. **每次创建新 runtime**：不复用全局 runtime，避免嵌套

调用方式：

```rust
let result = retry(max_retries, || {
    let fut = SomaTtsProvider::synthesize(
        &tts, script, voice_name, rate,
        Path::new(&audio_file)
    );
    block_on_async(fut)
})?;
```

## 坑三：retry 闭包不能是 async

`retry` 函数签名：

```rust
fn retry<F, T>(max_retries: usize, f: F) -> Result<T, SomaError>
where
    F: Fn() -> Result<T, SomaError>,
```

`f` 是同步闭包，返回 `Result<T, SomaError>`。但 `synthesize` 是 async 的，返回 `Future`。

**解决方案**：在闭包里 `block_on_async`：

```rust
retry(3, || {
    let fut = SomaTtsProvider::synthesize(&tts, text, voice, rate, output);
    block_on_async(fut)  // 把 Future 转成同步 Result
})
```

`block_on_async` 把 `Future<Output = Result<T, E>>` 转成 `Result<T, E>`，正好匹配 `retry` 的闭包签名。

## 坑四：?Send 约束

Provider trait 用 `#[async_trait(?Send)]`：

```rust
#[async_trait(?Send)]
pub trait SomaTtsProvider: Send + Sync {
    async fn synthesize(...) -> Result<TtsResult, SomaError>;
}
```

为什么 `?Send`？因为 `tokio::process::Command` 在 current_thread runtime 里产生的 future 不是 `Send`。如果用 `#[async_trait]`（要求 Send），编译报错：

```
future is not `Send`
```

`?Send` 放宽了这个约束，允许 `!Send` 的 future。代价是不能用 `tokio::spawn`（spawn 要求 Send），但我们本来就不能用 spawn（坑一），所以没有额外代价。

## 坑五：SSH 超时与进程清理

SSH 调用 GPU 推理可能很久（几分钟），需要超时控制。但 `tokio::process::Command` 的 `output()` 没有超时参数。

**解决方案**：用 `tokio::time::timeout` 包一层：

```rust
let result = tokio::time::timeout(
    Duration::from_secs(timeout),
    Command::new("ssh").arg(...).output()
).await;

match result {
    Ok(Ok(output)) => { /* 成功 */ }
    Ok(Err(e)) => { /* SSH 错误 */ }
    Err(_) => { /* 超时 */ }
}
```

但超时后子进程不会被自动 kill，SSH 连接会泄漏。需要手动管理：

```rust
let mut child = Command::new("ssh").arg(...).spawn()?;
match tokio::time::timeout(dur, child.wait_with_output()).await {
    Ok(result) => result,
    Err(_) => {
        let _ = child.kill().await;  // 超时杀进程
        Err(SomaError::Tts("超时".into()))
    }
}
```

## 可运行的最小示例

把 SSH 换成 `sleep 5`，20 行代码直接跑通，理解整个模式：

```rust
use std::error::Error;

fn block_on_async<F>(fut: F) -> F::Output
where
    F: std::future::Future,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime 创建失败");
    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, fut)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 模拟从非 tokio 上下文调用 async 函数
    let result = block_on_async(async {
        let output = tokio::process::Command::new("sleep")
            .arg("2")
            .output()
            .await?;
        println!("子进程退出码: {:?}", output.status.code());
        Ok::<_, Box<dyn Error>>(())
    });
    result
}
```

`Cargo.toml` 只需 `tokio = { version = "1", features = ["rt", "process", "macros"] }`。跑通后把 `sleep 2` 换成 `ssh user@host python infer.py` 就是项目里的真实用法。

## 完整模式总结

```
┌─ actix-web handler（actix runtime）──────────┐
│                                                │
│  block_on_async(                               │
│    LocalSet::block_on(                         │
│      &current_thread_runtime,                  │
│      async {                                   │
│        SomaTtsProvider::synthesize(...).await  │
│        // 内部用 tokio::process::Command       │
│        // 内部用 reqwest（HTTP 调用）          │
│      }                                         │
│    )                                           │
│  )                                             │
│                                                │
└────────────────────────────────────────────────┘
```

**核心原则**：

1. **不要 `tokio::spawn`**：用 `block_on_async` 创建临时 runtime
2. **用 `current_thread` runtime**：避免 Send 约束
3. **用 `LocalSet`**：允许 `!Send` future
4. **trait 用 `?Send`**：放宽 async trait 约束
5. **超时手动 kill 子进程**：防止 SSH 泄漏

这个模式在 soma 项目里用了十几个地方（TTS、数字人视频生成、LLM 调用），全部稳定运行。希望对同样在 Rust 里调 Python 的朋友有帮助。

---

*本文基于 soma 项目开发实践。下一篇：《声音克隆 TTS 集成：多格式音频处理与路径解析踩坑》*