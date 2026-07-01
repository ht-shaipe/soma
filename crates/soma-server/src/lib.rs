/// soma-server crate 入口模块
///
/// 负责导出各子模块，并引入 tube 框架的宏和 lazy_static 全局静态宏，
/// 为整个服务端提供统一的模块访问入口。

// 引入 tube 框架的宏（如 error!、log!、value! 等）
#[macro_use]
extern crate tube;

use tube::Error;

#[macro_use]
extern crate lazy_static;

/// 配置模块 - 全局配置加载、缓存与读取
pub mod config;
/// 请求处理器模块 - 各类 API 的具体处理逻辑
pub mod handler;
/// 流水线模块 - 6步视频生成流水线编排
pub mod pipeline;
/// 路由模块 - API 请求分发与路由调度
pub mod router;
/// 业务服务模块 - LLM、流水线等核心业务逻辑
pub mod service;
/// 全局状态模块 - 任务存储（TASK_STORE）及 CRUD 操作
pub mod state;
/// 任务存储后端模块 - 内存/Redis 抽象
pub mod store;
/// 任务队列模块 - 并发/排队任务调度与执行
pub mod task;

/// 重新导出 Config 结构体，方便外部通过 soma_server::Config 直接使用
pub use config::Config;
