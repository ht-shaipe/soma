/// 全局任务状态管理模块
///
/// 通过 TaskStore trait 抽象任务存储，支持内存和 Redis 后端。
/// 提供任务的创建、查询、更新、删除等 CRUD 操作。

use soma_core::models::TaskInfo;
use crate::store::{TaskStore, InMemoryTaskStore};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TASK_STORE: Mutex<Box<dyn TaskStore>> = Mutex::new(Box::new(InMemoryTaskStore::new()));
}

fn lock_store() -> std::sync::MutexGuard<'static, Box<dyn TaskStore>> {
    TASK_STORE.lock().unwrap_or_else(|e| {
        log::error!("TASK_STORE Mutex 中毒，强制恢复: {}", e);
        e.into_inner()
    })
}

/// 初始化任务存储后端（Redis 或内存）
pub fn init_store(store: Box<dyn TaskStore>) {
    *lock_store() = store;
}

pub fn update_task(task_id: &str, state: Option<i32>, progress: Option<u32>) {
    let state_val = state;
    let progress_val = progress;
    lock_store().update(task_id, Box::new(move |task| {
        task.update(state_val, progress_val);
    }));
}

pub fn update_task_data(task_id: &str, data: &TaskUpdateData) {
    let d = data.clone();
    lock_store().update(task_id, Box::new(move |task| {
        if let Some(ref s) = d.state { task.state = *s; }
        if let Some(p) = d.progress { task.progress = p; }
        if let Some(ref s) = d.script { task.script = Some(s.clone()); }
        if let Some(ref t) = d.terms { task.terms = Some(t.clone()); }
        if let Some(ref a) = d.audio_file { task.audio_file = Some(a.clone()); }
        if let Some(dur) = d.audio_duration { task.audio_duration = Some(dur); }
        if let Some(ref s) = d.subtitle_path { task.subtitle_path = Some(s.clone()); }
        if let Some(ref m) = d.materials { task.materials = Some(m.clone()); }
        if let Some(ref v) = d.videos { task.videos = Some(v.clone()); }
        if let Some(ref v) = d.combined_videos { task.combined_videos = Some(v.clone()); }
        task.updated_at = chrono::Utc::now();
    }));
}

pub fn get_task(task_id: &str) -> Option<TaskInfo> {
    lock_store().get(task_id)
}

pub fn get_all_tasks(page: usize, page_size: usize) -> (Vec<TaskInfo>, usize) {
    let mut tasks = lock_store().get_all();
    tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let total = tasks.len();
    let start = (page.saturating_sub(1)) * page_size;
    let end = (start + page_size).min(total);
    if start < total {
        (tasks[start..end].to_vec(), total)
    } else {
        (vec![], total)
    }
}

pub fn delete_task(task_id: &str) -> bool {
    lock_store().delete(task_id)
}

pub fn create_task_entry(task_id: &str, params: soma_core::models::VideoParams) {
    let task = TaskInfo::new(task_id.to_string(), params);
    lock_store().create(task);
}

#[derive(Debug, Default, Clone)]
pub struct TaskUpdateData {
    pub state: Option<i32>,
    pub progress: Option<u32>,
    pub script: Option<String>,
    pub terms: Option<Vec<String>>,
    pub audio_file: Option<String>,
    pub audio_duration: Option<f64>,
    pub subtitle_path: Option<String>,
    pub materials: Option<Vec<String>>,
    pub videos: Option<Vec<String>>,
    pub combined_videos: Option<Vec<String>>,
}
