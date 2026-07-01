/// 任务存储抽象模块
///
/// 提供 TaskStore trait，支持内存和 Redis 两种后端实现。
/// 内存后端为默认实现，Redis 后端需启用 `redis` feature。

use soma_core::models::TaskInfo;
use std::collections::HashMap;
use std::sync::Mutex;

/// 任务存储 trait
pub trait TaskStore: Send + Sync {
    fn create(&self, task: TaskInfo);
    fn get(&self, task_id: &str) -> Option<TaskInfo>;
    fn get_all(&self) -> Vec<TaskInfo>;
    fn update(&self, task_id: &str, f: Box<dyn FnOnce(&mut TaskInfo) + Send + 'static>);
    fn delete(&self, task_id: &str) -> bool;
}

/// 内存任务存储（默认实现）
pub struct InMemoryTaskStore {
    store: Mutex<HashMap<String, TaskInfo>>,
}

impl InMemoryTaskStore {
    pub fn new() -> Self {
        Self { store: Mutex::new(HashMap::new()) }
    }
}

impl TaskStore for InMemoryTaskStore {
    fn create(&self, task: TaskInfo) {
        let mut store = self.store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        store.insert(task.task_id.clone(), task);
    }

    fn get(&self, task_id: &str) -> Option<TaskInfo> {
        let store = self.store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        store.get(task_id).cloned()
    }

    fn get_all(&self) -> Vec<TaskInfo> {
        let store = self.store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        store.values().cloned().collect()
    }

    fn update(&self, task_id: &str, f: Box<dyn FnOnce(&mut TaskInfo) + Send>) {
        let mut store = self.store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        if let Some(task) = store.get_mut(task_id) {
            f(task);
        }
    }

    fn delete(&self, task_id: &str) -> bool {
        let mut store = self.store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        store.remove(task_id).is_some()
    }
}

#[cfg(feature = "redis")]
mod redis_store {
    use super::TaskStore;
    use soma_core::models::TaskInfo;
    use redis::AsyncCommands;
    use std::sync::Arc;

    const KEY_PREFIX: &str = "soma:task:";

    pub struct RedisTaskStore {
        client: redis::Client,
    }

    impl RedisTaskStore {
        pub fn new(url: &str) -> Result<Self, String> {
            let client = redis::Client::open(url)
                .map_err(|e| format!("Redis 连接失败: {}", e))?;
            Ok(Self { client })
        }

        fn task_key(task_id: &str) -> String {
            format!("{}{}", KEY_PREFIX, task_id)
        }

        fn index_key() -> String {
            format!("{}__index__", KEY_PREFIX)
        }
    }

    impl TaskStore for RedisTaskStore {
        fn create(&self, task: TaskInfo) {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => { log::error!("Redis create runtime failed: {}", e); return; }
            };
            let _ = rt.block_on(async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let json = serde_json::to_string(&task).ok()?;
                let key = Self::task_key(&task.task_id);
                redis::cmd("SET")
                    .arg(&key)
                    .arg(&json)
                    .exec_async(&mut conn)
                    .await.ok()?;
                let idx = Self::index_key();
                conn.sadd(&idx, &task.task_id).await.ok()
            });
        }

        fn get(&self, task_id: &str) -> Option<TaskInfo> {
            let rt = tokio::runtime::Runtime::new().ok()?;
            rt.block_on(async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let key = Self::task_key(task_id);
                let json: Option<String> = conn.get(&key).await.ok()?;
                json.and_then(|j| serde_json::from_str(&j).ok())
            })
        }

        fn get_all(&self) -> Vec<TaskInfo> {
            let rt = tokio::runtime::Runtime::new().ok()?;
            rt.block_on(async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let idx = Self::index_key();
                let ids: Vec<String> = conn.smembers(&idx).await.ok()?;
                let mut tasks = Vec::new();
                for id in ids {
                    let key = Self::task_key(&id);
                    let json: Option<String> = conn.get(&key).await.ok()?;
                    if let Some(j) = json {
                        if let Ok(task) = serde_json::from_str::<TaskInfo>(&j) {
                            tasks.push(task);
                        }
                    }
                }
                tasks
            }).unwrap_or_default()
        }

        fn update(&self, task_id: &str, f: Box<dyn FnOnce(&mut TaskInfo) + Send + 'static>) {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => { log::error!("Redis create runtime failed: {}", e); return; }
            };
            let _ = rt.block_on(async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let key = Self::task_key(task_id);
                let json: Option<String> = conn.get(&key).await.ok()?;
                if let Some(j) = json {
                    if let Ok(mut task) = serde_json::from_str::<TaskInfo>(&j) {
                        f(&mut task);
                        let updated = serde_json::to_string(&task).ok()?;
                        redis::cmd("SET")
                            .arg(&key)
                            .arg(&updated)
                            .exec_async(&mut conn)
                            .await.ok()?;
                    }
                }
                Option::<()>::None
            });
        }

        fn delete(&self, task_id: &str) -> bool {
            let rt = tokio::runtime::Runtime::new().ok()?;
            rt.block_on(async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let key = Self::task_key(task_id);
                let idx = Self::index_key();
                let removed: i32 = conn.del(&key).await.ok()?;
                conn.srem(&idx, task_id).await.ok();
                removed > 0
            }).unwrap_or(false)
        }
    }
}

#[cfg(feature = "redis")]
pub use redis_store::RedisTaskStore;
