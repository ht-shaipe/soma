use soma_core::models::TaskInfo;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    pub static ref TASK_STORE: Mutex<HashMap<String, TaskInfo>> = Mutex::new(HashMap::new());
}

pub fn update_task(task_id: &str, state: Option<i32>, progress: Option<u32>) {
    let mut store = TASK_STORE.lock().unwrap();
    if let Some(task) = store.get_mut(task_id) {
        task.update(state, progress);
    }
}

pub fn update_task_data(task_id: &str, data: &TaskUpdateData) {
    let mut store = TASK_STORE.lock().unwrap();
    if let Some(task) = store.get_mut(task_id) {
        if let Some(ref s) = data.state { task.state = *s; }
        if let Some(p) = data.progress { task.progress = p; }
        if let Some(ref s) = data.script { task.script = Some(s.clone()); }
        if let Some(ref t) = data.terms { task.terms = Some(t.clone()); }
        if let Some(ref a) = data.audio_file { task.audio_file = Some(a.clone()); }
        if let Some(d) = data.audio_duration { task.audio_duration = Some(d); }
        if let Some(ref s) = data.subtitle_path { task.subtitle_path = Some(s.clone()); }
        if let Some(ref m) = data.materials { task.materials = Some(m.clone()); }
        if let Some(ref v) = data.videos { task.videos = Some(v.clone()); }
        if let Some(ref v) = data.combined_videos { task.combined_videos = Some(v.clone()); }
        task.updated_at = chrono::Utc::now();
    }
}

pub fn get_task(task_id: &str) -> Option<TaskInfo> {
    TASK_STORE.lock().unwrap().get(task_id).cloned()
}

pub fn get_all_tasks(page: usize, page_size: usize) -> (Vec<TaskInfo>, usize) {
    let store = TASK_STORE.lock().unwrap();
    let mut tasks: Vec<TaskInfo> = store.values().cloned().collect();
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
    TASK_STORE.lock().unwrap().remove(task_id).is_some()
}

pub fn create_task_entry(task_id: &str, params: soma_core::models::VideoParams) {
    let task = TaskInfo::new(task_id.to_string(), params);
    TASK_STORE.lock().unwrap().insert(task_id.to_string(), task);
}

#[derive(Debug, Default)]
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
