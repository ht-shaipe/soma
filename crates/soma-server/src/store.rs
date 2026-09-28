/// 任务存储抽象模块
///
/// 提供 TaskStore trait，支持内存、Redis 和 SQLite 三种后端实现。
/// SQLite 后端为默认实现，任务数据持久化到本地数据库文件。
/// Redis 后端需启用 `redis` feature。
use soma_core::models::{DigitalHumanTaskInfo, ImageStoryTaskInfo, TaskInfo};
use std::collections::HashMap;
use std::sync::Mutex;

/// 任务存储 trait
///
/// 同时支持视频任务（TaskInfo）和数字人任务（DigitalHumanTaskInfo）两类任务，
/// 通过独立的方法集区分，不破坏存量 TaskInfo 接口。
pub trait TaskStore: Send + Sync {
    fn create(&self, task: TaskInfo);
    fn get(&self, task_id: &str) -> Option<TaskInfo>;
    fn get_all(&self) -> Vec<TaskInfo>;
    fn update(&self, task_id: &str, f: Box<dyn FnOnce(&mut TaskInfo) + Send + 'static>);
    fn delete(&self, task_id: &str) -> bool;

    /// 数字人任务：创建
    fn create_dh(&self, task: DigitalHumanTaskInfo);
    /// 数字人任务：查询单个
    fn get_dh(&self, task_id: &str) -> Option<DigitalHumanTaskInfo>;
    /// 数字人任务：查询全部
    fn get_all_dh(&self) -> Vec<DigitalHumanTaskInfo>;
    /// 数字人任务：更新
    fn update_dh(
        &self,
        task_id: &str,
        f: Box<dyn FnOnce(&mut DigitalHumanTaskInfo) + Send + 'static>,
    );
    /// 数字人任务：删除
    fn delete_dh(&self, task_id: &str) -> bool;

    /// 图片故事任务：创建
    fn create_image_story(&self, task: ImageStoryTaskInfo);
    /// 图片故事任务：查询单个
    fn get_image_story(&self, task_id: &str) -> Option<ImageStoryTaskInfo>;
    /// 图片故事任务：查询全部
    fn get_all_image_story(&self) -> Vec<ImageStoryTaskInfo>;
    /// 图片故事任务：更新
    fn update_image_story(
        &self,
        task_id: &str,
        f: Box<dyn FnOnce(&mut ImageStoryTaskInfo) + Send + 'static>,
    );
    /// 图片故事任务：删除
    fn delete_image_story(&self, task_id: &str) -> bool;
}

/// 内存任务存储
pub struct InMemoryTaskStore {
    store: Mutex<HashMap<String, TaskInfo>>,
    /// 数字人任务存储
    dh_store: Mutex<HashMap<String, DigitalHumanTaskInfo>>,
    /// 图片故事任务存储
    image_story_store: Mutex<HashMap<String, ImageStoryTaskInfo>>,
}

impl Default for InMemoryTaskStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryTaskStore {
    /// 创建内存任务存储（进程内缓存，供测试或无持久化场景）
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
            dh_store: Mutex::new(HashMap::new()),
            image_story_store: Mutex::new(HashMap::new()),
        }
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

    fn create_dh(&self, task: DigitalHumanTaskInfo) {
        let mut store = self.dh_store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore dh_store Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        store.insert(task.task_id.clone(), task);
    }

    fn get_dh(&self, task_id: &str) -> Option<DigitalHumanTaskInfo> {
        let store = self.dh_store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore dh_store Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        store.get(task_id).cloned()
    }

    fn get_all_dh(&self) -> Vec<DigitalHumanTaskInfo> {
        let store = self.dh_store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore dh_store Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        store.values().cloned().collect()
    }

    fn update_dh(&self, task_id: &str, f: Box<dyn FnOnce(&mut DigitalHumanTaskInfo) + Send>) {
        let mut store = self.dh_store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore dh_store Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        if let Some(task) = store.get_mut(task_id) {
            f(task);
        }
    }

    fn delete_dh(&self, task_id: &str) -> bool {
        let mut store = self.dh_store.lock().unwrap_or_else(|e| {
            log::error!("InMemoryTaskStore dh_store Mutex 中毒，强制恢复: {}", e);
            e.into_inner()
        });
        store.remove(task_id).is_some()
    }

    fn create_image_story(&self, task: ImageStoryTaskInfo) {
        let mut store = self.image_story_store.lock().unwrap_or_else(|e| {
            log::error!(
                "InMemoryTaskStore image_story_store Mutex 中毒，强制恢复: {}",
                e
            );
            e.into_inner()
        });
        store.insert(task.task_id.clone(), task);
    }

    fn get_image_story(&self, task_id: &str) -> Option<ImageStoryTaskInfo> {
        let store = self.image_story_store.lock().unwrap_or_else(|e| {
            log::error!(
                "InMemoryTaskStore image_story_store Mutex 中毒，强制恢复: {}",
                e
            );
            e.into_inner()
        });
        store.get(task_id).cloned()
    }

    fn get_all_image_story(&self) -> Vec<ImageStoryTaskInfo> {
        let store = self.image_story_store.lock().unwrap_or_else(|e| {
            log::error!(
                "InMemoryTaskStore image_story_store Mutex 中毒，强制恢复: {}",
                e
            );
            e.into_inner()
        });
        store.values().cloned().collect()
    }

    fn update_image_story(
        &self,
        task_id: &str,
        f: Box<dyn FnOnce(&mut ImageStoryTaskInfo) + Send>,
    ) {
        let mut store = self.image_story_store.lock().unwrap_or_else(|e| {
            log::error!(
                "InMemoryTaskStore image_story_store Mutex 中毒，强制恢复: {}",
                e
            );
            e.into_inner()
        });
        if let Some(task) = store.get_mut(task_id) {
            f(task);
        }
    }

    fn delete_image_story(&self, task_id: &str) -> bool {
        let mut store = self.image_story_store.lock().unwrap_or_else(|e| {
            log::error!(
                "InMemoryTaskStore image_story_store Mutex 中毒，强制恢复: {}",
                e
            );
            e.into_inner()
        });
        store.remove(task_id).is_some()
    }
}

/// SQLite 任务存储（持久化）
pub struct SqliteTaskStore {
    conn: Mutex<rusqlite::Connection>,
}

impl SqliteTaskStore {
    /// 打开或创建 SQLite 任务存储（WAL 模式）
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| format!("SQLite 打开失败 {}: {}", db_path, e))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("SQLite PRAGMA 设置失败: {}", e))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                task_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                state INTEGER NOT NULL,
                progress INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("SQLite 建表失败: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_state ON tasks(state)",
            [],
        )
        .map_err(|e| format!("SQLite 建索引失败: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_updated_at ON tasks(updated_at)",
            [],
        )
        .map_err(|e| format!("SQLite 建索引失败: {}", e))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dh_tasks (
                task_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                state INTEGER NOT NULL,
                progress INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("SQLite 建 dh_tasks 表失败: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dh_tasks_state ON dh_tasks(state)",
            [],
        )
        .map_err(|e| format!("SQLite 建 dh_tasks 索引失败: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dh_tasks_updated_at ON dh_tasks(updated_at)",
            [],
        )
        .map_err(|e| format!("SQLite 建 dh_tasks 索引失败: {}", e))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS image_story_tasks (
                task_id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                state INTEGER NOT NULL,
                progress INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| format!("SQLite 建 image_story_tasks 表失败: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_image_story_tasks_state ON image_story_tasks(state)",
            [],
        )
        .map_err(|e| format!("SQLite 建 image_story_tasks 索引失败: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_image_story_tasks_updated_at ON image_story_tasks(updated_at)",
            [],
        ).map_err(|e| format!("SQLite 建 image_story_tasks 索引失败: {}", e))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn task_to_row(task: &TaskInfo) -> (String, i32, u32, String, String) {
        let data = serde_json::to_string(task).unwrap_or_else(|e| {
            log::error!("任务序列化失败 {}: {}", task.task_id, e);
            "{}".to_string()
        });
        let created_at = task.created_at.to_rfc3339();
        let updated_at = task.updated_at.to_rfc3339();
        (data, task.state, task.progress, created_at, updated_at)
    }

    fn row_to_task(data: &str) -> Option<TaskInfo> {
        serde_json::from_str(data)
            .map_err(|e| {
                log::error!("任务反序列化失败: {}", e);
                e
            })
            .ok()
    }

    fn dh_task_to_row(task: &DigitalHumanTaskInfo) -> (String, i32, u32, String, String) {
        let data = serde_json::to_string(task).unwrap_or_else(|e| {
            log::error!("数字人任务序列化失败 {}: {}", task.task_id, e);
            "{}".to_string()
        });
        let created_at = task.created_at.to_rfc3339();
        let updated_at = task.updated_at.to_rfc3339();
        (data, task.state, task.progress, created_at, updated_at)
    }

    fn row_to_dh_task(data: &str) -> Option<DigitalHumanTaskInfo> {
        serde_json::from_str(data)
            .map_err(|e| {
                log::error!("数字人任务反序列化失败: {}", e);
                e
            })
            .ok()
    }

    fn image_story_task_to_row(task: &ImageStoryTaskInfo) -> (String, i32, u32, String, String) {
        let data = serde_json::to_string(task).unwrap_or_else(|e| {
            log::error!("图片故事任务序列化失败 {}: {}", task.task_id, e);
            "{}".to_string()
        });
        let created_at = task.created_at.to_rfc3339();
        let updated_at = task.updated_at.to_rfc3339();
        (data, task.state, task.progress, created_at, updated_at)
    }

    fn row_to_image_story_task(data: &str) -> Option<ImageStoryTaskInfo> {
        serde_json::from_str(data)
            .map_err(|e| {
                log::error!("图片故事任务反序列化失败: {}", e);
                e
            })
            .ok()
    }
}

impl TaskStore for SqliteTaskStore {
    fn create(&self, task: TaskInfo) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return;
            }
        };
        let (data, state, progress, created_at, updated_at) = Self::task_to_row(&task);
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO tasks (task_id, data, state, progress, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![task.task_id, data, state, progress, created_at, updated_at],
        ) {
            log::error!("SQLite 创建任务失败 {}: {}", task.task_id, e);
        }
    }

    fn get(&self, task_id: &str) -> Option<TaskInfo> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return None;
            }
        };
        let mut stmt = conn
            .prepare("SELECT data FROM tasks WHERE task_id = ?1")
            .ok()?;
        let data: String = stmt
            .query_row(rusqlite::params![task_id], |row| row.get(0))
            .ok()?;
        Self::row_to_task(&data)
    }

    fn get_all(&self) -> Vec<TaskInfo> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return vec![];
            }
        };
        let mut stmt = match conn.prepare("SELECT data FROM tasks ORDER BY updated_at DESC") {
            Ok(s) => s,
            Err(e) => {
                log::error!("SQLite 查询任务失败: {}", e);
                return vec![];
            }
        };
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        });
        let mut tasks = Vec::new();
        match rows {
            Ok(iter) => {
                for data in iter.flatten() {
                    if let Some(task) = Self::row_to_task(&data) {
                        tasks.push(task);
                    }
                }
            }
            Err(e) => log::error!("SQLite 遍历任务失败: {}", e),
        }
        tasks
    }

    fn update(&self, task_id: &str, f: Box<dyn FnOnce(&mut TaskInfo) + Send>) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return;
            }
        };
        let mut stmt = match conn.prepare("SELECT data FROM tasks WHERE task_id = ?1") {
            Ok(s) => s,
            Err(e) => {
                log::error!("SQLite 查询任务失败: {}", e);
                return;
            }
        };
        let data: String = match stmt.query_row(rusqlite::params![task_id], |row| row.get(0)) {
            Ok(d) => d,
            Err(e) => {
                log::error!("SQLite 获取任务 {} 失败: {}", task_id, e);
                return;
            }
        };
        let mut task = match Self::row_to_task(&data) {
            Some(t) => t,
            None => return,
        };
        f(&mut task);
        let (new_data, state, progress, _, updated_at) = Self::task_to_row(&task);
        if let Err(e) = conn.execute(
            "UPDATE tasks SET data = ?1, state = ?2, progress = ?3, updated_at = ?4 WHERE task_id = ?5",
            rusqlite::params![new_data, state, progress, updated_at, task_id],
        ) {
            log::error!("SQLite 更新任务 {} 失败: {}", task_id, e);
        }
    }

    fn delete(&self, task_id: &str) -> bool {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return false;
            }
        };
        match conn.execute(
            "DELETE FROM tasks WHERE task_id = ?1",
            rusqlite::params![task_id],
        ) {
            Ok(n) => n > 0,
            Err(e) => {
                log::error!("SQLite 删除任务 {} 失败: {}", task_id, e);
                false
            }
        }
    }

    fn create_dh(&self, task: DigitalHumanTaskInfo) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return;
            }
        };
        let (data, state, progress, created_at, updated_at) = Self::dh_task_to_row(&task);
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO dh_tasks (task_id, data, state, progress, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![task.task_id, data, state, progress, created_at, updated_at],
        ) {
            log::error!("SQLite 创建数字人任务失败 {}: {}", task.task_id, e);
        }
    }

    fn get_dh(&self, task_id: &str) -> Option<DigitalHumanTaskInfo> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return None;
            }
        };
        let mut stmt = conn
            .prepare("SELECT data FROM dh_tasks WHERE task_id = ?1")
            .ok()?;
        let data: String = stmt
            .query_row(rusqlite::params![task_id], |row| row.get(0))
            .ok()?;
        Self::row_to_dh_task(&data)
    }

    fn get_all_dh(&self) -> Vec<DigitalHumanTaskInfo> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return vec![];
            }
        };
        let mut stmt = match conn.prepare("SELECT data FROM dh_tasks ORDER BY updated_at DESC") {
            Ok(s) => s,
            Err(e) => {
                log::error!("SQLite 查询数字人任务失败: {}", e);
                return vec![];
            }
        };
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        });
        let mut tasks = Vec::new();
        match rows {
            Ok(iter) => {
                for data in iter.flatten() {
                    if let Some(task) = Self::row_to_dh_task(&data) {
                        tasks.push(task);
                    }
                }
            }
            Err(e) => log::error!("SQLite 遍历数字人任务失败: {}", e),
        }
        tasks
    }

    fn update_dh(&self, task_id: &str, f: Box<dyn FnOnce(&mut DigitalHumanTaskInfo) + Send>) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return;
            }
        };
        let mut stmt = match conn.prepare("SELECT data FROM dh_tasks WHERE task_id = ?1") {
            Ok(s) => s,
            Err(e) => {
                log::error!("SQLite 查询数字人任务失败: {}", e);
                return;
            }
        };
        let data: String = match stmt.query_row(rusqlite::params![task_id], |row| row.get(0)) {
            Ok(d) => d,
            Err(e) => {
                log::error!("SQLite 获取数字人任务 {} 失败: {}", task_id, e);
                return;
            }
        };
        let mut task = match Self::row_to_dh_task(&data) {
            Some(t) => t,
            None => return,
        };
        f(&mut task);
        let (new_data, state, progress, _, updated_at) = Self::dh_task_to_row(&task);
        if let Err(e) = conn.execute(
            "UPDATE dh_tasks SET data = ?1, state = ?2, progress = ?3, updated_at = ?4 WHERE task_id = ?5",
            rusqlite::params![new_data, state, progress, updated_at, task_id],
        ) {
            log::error!("SQLite 更新数字人任务 {} 失败: {}", task_id, e);
        }
    }

    fn delete_dh(&self, task_id: &str) -> bool {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return false;
            }
        };
        match conn.execute(
            "DELETE FROM dh_tasks WHERE task_id = ?1",
            rusqlite::params![task_id],
        ) {
            Ok(n) => n > 0,
            Err(e) => {
                log::error!("SQLite 删除数字人任务 {} 失败: {}", task_id, e);
                false
            }
        }
    }

    fn create_image_story(&self, task: ImageStoryTaskInfo) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return;
            }
        };
        let (data, state, progress, created_at, updated_at) = Self::image_story_task_to_row(&task);
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO image_story_tasks (task_id, data, state, progress, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![task.task_id, data, state, progress, created_at, updated_at],
        ) {
            log::error!("SQLite 创建图片故事任务失败 {}: {}", task.task_id, e);
        }
    }

    fn get_image_story(&self, task_id: &str) -> Option<ImageStoryTaskInfo> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return None;
            }
        };
        let mut stmt = conn
            .prepare("SELECT data FROM image_story_tasks WHERE task_id = ?1")
            .ok()?;
        let data: String = stmt
            .query_row(rusqlite::params![task_id], |row| row.get(0))
            .ok()?;
        Self::row_to_image_story_task(&data)
    }

    fn get_all_image_story(&self) -> Vec<ImageStoryTaskInfo> {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return vec![];
            }
        };
        let mut stmt =
            match conn.prepare("SELECT data FROM image_story_tasks ORDER BY updated_at DESC") {
                Ok(s) => s,
                Err(e) => {
                    log::error!("SQLite 查询图片故事任务失败: {}", e);
                    return vec![];
                }
            };
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        });
        let mut tasks = Vec::new();
        match rows {
            Ok(iter) => {
                for data in iter.flatten() {
                    if let Some(task) = Self::row_to_image_story_task(&data) {
                        tasks.push(task);
                    }
                }
            }
            Err(e) => log::error!("SQLite 遍历图片故事任务失败: {}", e),
        }
        tasks
    }

    fn update_image_story(
        &self,
        task_id: &str,
        f: Box<dyn FnOnce(&mut ImageStoryTaskInfo) + Send>,
    ) {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return;
            }
        };
        let mut stmt = match conn.prepare("SELECT data FROM image_story_tasks WHERE task_id = ?1") {
            Ok(s) => s,
            Err(e) => {
                log::error!("SQLite 查询图片故事任务失败: {}", e);
                return;
            }
        };
        let data: String = match stmt.query_row(rusqlite::params![task_id], |row| row.get(0)) {
            Ok(d) => d,
            Err(e) => {
                log::error!("SQLite 获取图片故事任务 {} 失败: {}", task_id, e);
                return;
            }
        };
        let mut task = match Self::row_to_image_story_task(&data) {
            Some(t) => t,
            None => return,
        };
        f(&mut task);
        let (new_data, state, progress, _, updated_at) = Self::image_story_task_to_row(&task);
        if let Err(e) = conn.execute(
            "UPDATE image_story_tasks SET data = ?1, state = ?2, progress = ?3, updated_at = ?4 WHERE task_id = ?5",
            rusqlite::params![new_data, state, progress, updated_at, task_id],
        ) {
            log::error!("SQLite 更新图片故事任务 {} 失败: {}", task_id, e);
        }
    }

    fn delete_image_story(&self, task_id: &str) -> bool {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(e) => {
                log::error!("SqliteTaskStore Mutex 中毒: {}", e);
                return false;
            }
        };
        match conn.execute(
            "DELETE FROM image_story_tasks WHERE task_id = ?1",
            rusqlite::params![task_id],
        ) {
            Ok(n) => n > 0,
            Err(e) => {
                log::error!("SQLite 删除图片故事任务 {} 失败: {}", task_id, e);
                false
            }
        }
    }
}

#[cfg(feature = "redis")]
mod redis_store {
    use super::TaskStore;
    use redis::AsyncCommands;
    use soma_core::models::{DigitalHumanTaskInfo, ImageStoryTaskInfo, TaskInfo};
    use std::sync::Arc;

    const KEY_PREFIX: &str = "soma:task:";
    const DH_KEY_PREFIX: &str = "soma:dh_task:";

    pub struct RedisTaskStore {
        client: redis::Client,
    }

    impl RedisTaskStore {
        /// 连接 Redis 任务存储（分布式部署场景）
        pub fn new(url: &str) -> Result<Self, String> {
            let client = redis::Client::open(url).map_err(|e| format!("Redis 连接失败: {}", e))?;
            Ok(Self { client })
        }

        fn task_key(task_id: &str) -> String {
            format!("{}{}", KEY_PREFIX, task_id)
        }

        fn index_key() -> String {
            format!("{}__index__", KEY_PREFIX)
        }

        fn dh_task_key(task_id: &str) -> String {
            format!("{}{}", DH_KEY_PREFIX, task_id)
        }

        fn dh_index_key() -> String {
            format!("{}__index__", DH_KEY_PREFIX)
        }
    }

    impl TaskStore for RedisTaskStore {
        fn create(&self, task: TaskInfo) {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    log::error!("Redis create runtime failed: {}", e);
                    return;
                }
            };
            let local = tokio::task::LocalSet::new();
            let _ = local.block_on(&rt, async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let json = serde_json::to_string(&task).ok()?;
                let key = Self::task_key(&task.task_id);
                redis::cmd("SET")
                    .arg(&key)
                    .arg(&json)
                    .exec_async(&mut conn)
                    .await
                    .ok()?;
                let idx = Self::index_key();
                conn.sadd(&idx, &task.task_id).await.ok()
            });
        }

        fn get(&self, task_id: &str) -> Option<TaskInfo> {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            let local = tokio::task::LocalSet::new();
            local.block_on(&rt, async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let key = Self::task_key(task_id);
                let json: Option<String> = conn.get(&key).await.ok()?;
                json.and_then(|j| serde_json::from_str(&j).ok())
            })
        }

        fn get_all(&self) -> Vec<TaskInfo> {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            let local = tokio::task::LocalSet::new();
            local
                .block_on(&rt, async {
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
                })
                .unwrap_or_default()
        }

        fn update(&self, task_id: &str, f: Box<dyn FnOnce(&mut TaskInfo) + Send + 'static>) {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    log::error!("Redis create runtime failed: {}", e);
                    return;
                }
            };
            let local = tokio::task::LocalSet::new();
            let _ = local.block_on(&rt, async {
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
                            .await
                            .ok()?;
                    }
                }
                Option::<()>::None
            });
        }

        fn delete(&self, task_id: &str) -> bool {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            let local = tokio::task::LocalSet::new();
            local
                .block_on(&rt, async {
                    let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                    let key = Self::task_key(task_id);
                    let idx = Self::index_key();
                    let removed: i32 = conn.del(&key).await.ok()?;
                    conn.srem(&idx, task_id).await.ok();
                    removed > 0
                })
                .unwrap_or(false)
        }

        fn create_dh(&self, task: DigitalHumanTaskInfo) {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    log::error!("Redis create_dh runtime failed: {}", e);
                    return;
                }
            };
            let local = tokio::task::LocalSet::new();
            let _ = local.block_on(&rt, async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let json = serde_json::to_string(&task).ok()?;
                let key = Self::dh_task_key(&task.task_id);
                redis::cmd("SET")
                    .arg(&key)
                    .arg(&json)
                    .exec_async(&mut conn)
                    .await
                    .ok()?;
                let idx = Self::dh_index_key();
                conn.sadd(&idx, &task.task_id).await.ok()
            });
        }

        fn get_dh(&self, task_id: &str) -> Option<DigitalHumanTaskInfo> {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            let local = tokio::task::LocalSet::new();
            local.block_on(&rt, async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let key = Self::dh_task_key(task_id);
                let json: Option<String> = conn.get(&key).await.ok()?;
                json.and_then(|j| serde_json::from_str(&j).ok())
            })
        }

        fn get_all_dh(&self) -> Vec<DigitalHumanTaskInfo> {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            let local = tokio::task::LocalSet::new();
            local
                .block_on(&rt, async {
                    let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                    let idx = Self::dh_index_key();
                    let ids: Vec<String> = conn.smembers(&idx).await.ok()?;
                    let mut tasks = Vec::new();
                    for id in ids {
                        let key = Self::dh_task_key(&id);
                        let json: Option<String> = conn.get(&key).await.ok()?;
                        if let Some(j) = json {
                            if let Ok(task) = serde_json::from_str::<DigitalHumanTaskInfo>(&j) {
                                tasks.push(task);
                            }
                        }
                    }
                    tasks
                })
                .unwrap_or_default()
        }

        fn update_dh(
            &self,
            task_id: &str,
            f: Box<dyn FnOnce(&mut DigitalHumanTaskInfo) + Send + 'static>,
        ) {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(r) => r,
                Err(e) => {
                    log::error!("Redis update_dh runtime failed: {}", e);
                    return;
                }
            };
            let local = tokio::task::LocalSet::new();
            let _ = local.block_on(&rt, async {
                let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                let key = Self::dh_task_key(task_id);
                let json: Option<String> = conn.get(&key).await.ok()?;
                if let Some(j) = json {
                    if let Ok(mut task) = serde_json::from_str::<DigitalHumanTaskInfo>(&j) {
                        f(&mut task);
                        let updated = serde_json::to_string(&task).ok()?;
                        redis::cmd("SET")
                            .arg(&key)
                            .arg(&updated)
                            .exec_async(&mut conn)
                            .await
                            .ok()?;
                    }
                }
                Option::<()>::None
            });
        }

        fn delete_dh(&self, task_id: &str) -> bool {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            let local = tokio::task::LocalSet::new();
            local
                .block_on(&rt, async {
                    let mut conn = self.client.get_multiplexed_async_connection().await.ok()?;
                    let key = Self::dh_task_key(task_id);
                    let idx = Self::dh_index_key();
                    let removed: i32 = conn.del(&key).await.ok()?;
                    conn.srem(&idx, task_id).await.ok();
                    removed > 0
                })
                .unwrap_or(false)
        }
    }
}

#[cfg(feature = "redis")]
pub use redis_store::RedisTaskStore;
