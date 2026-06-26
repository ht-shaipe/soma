/// 全局任务状态管理模块
///
/// 使用内存 HashMap（TASK_STORE）存储所有任务信息，提供任务的创建、查询、
/// 更新、删除等 CRUD 操作。所有操作通过 Mutex 保证线程安全。

use soma_core::models::TaskInfo;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    /// 全局任务存储，以 task_id 为键，TaskInfo 为值，线程安全
    pub static ref TASK_STORE: Mutex<HashMap<String, TaskInfo>> = Mutex::new(HashMap::new());
}

/// 快速更新任务状态和进度
///
/// 仅更新 state 和 progress 两个字段，用于流水线各步骤间标记进展
///
/// 参数：
/// - `task_id`: 任务 ID
/// - `state`: 任务状态码（Some 更新，None 不变）
/// - `progress`: 任务进度百分比（Some 更新，None 不变）
pub fn update_task(task_id: &str, state: Option<i32>, progress: Option<u32>) {
    let mut store = TASK_STORE.lock().unwrap();
    if let Some(task) = store.get_mut(task_id) {
        task.update(state, progress);
    }
}

/// 使用 TaskUpdateData 批量更新任务数据
///
/// 可更新任务的各个字段（脚本、关键词、音频、字幕、素材、视频等），
/// 只有非 None 的字段才会被更新，实现了部分更新语义
///
/// 参数：
/// - `task_id`: 任务 ID
/// - `data`: 包含待更新字段的 TaskUpdateData 实例
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
        // 更新最后修改时间
        task.updated_at = chrono::Utc::now();
    }
}

/// 根据 task_id 查询单个任务
///
/// 参数：
/// - `task_id`: 任务 ID
///
/// 返回：找到则返回 Some(TaskInfo)，否则返回 None
pub fn get_task(task_id: &str) -> Option<TaskInfo> {
    TASK_STORE.lock().unwrap().get(task_id).cloned()
}

/// 分页查询所有任务
///
/// 返回结果按 updated_at 降序排列（最近更新的排在前面）
///
/// 参数：
/// - `page`: 页码（从1开始）
/// - `page_size`: 每页数量
///
/// 返回：(当前页任务列表, 总任务数)
pub fn get_all_tasks(page: usize, page_size: usize) -> (Vec<TaskInfo>, usize) {
    let store = TASK_STORE.lock().unwrap();
    let mut tasks: Vec<TaskInfo> = store.values().cloned().collect();
    // 按更新时间降序排列
    tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let total = tasks.len();
    // 计算分页范围
    let start = (page.saturating_sub(1)) * page_size;
    let end = (start + page_size).min(total);
    if start < total {
        (tasks[start..end].to_vec(), total)
    } else {
        (vec![], total)
    }
}

/// 根据 task_id 删除任务
///
/// 参数：
/// - `task_id`: 任务 ID
///
/// 返回：是否成功删除（true 表示任务存在并已删除）
pub fn delete_task(task_id: &str) -> bool {
    TASK_STORE.lock().unwrap().remove(task_id).is_some()
}

/// 创建新任务并写入存储
///
/// 使用 VideoParams 构造 TaskInfo，插入到 TASK_STORE 中
///
/// 参数：
/// - `task_id`: 新任务的唯一 ID
/// - `params`: 视频生成参数
pub fn create_task_entry(task_id: &str, params: soma_core::models::VideoParams) {
    let task = TaskInfo::new(task_id.to_string(), params);
    TASK_STORE.lock().unwrap().insert(task_id.to_string(), task);
}

/// 任务数据更新结构体
///
/// 所有字段均为 Option 类型，仅非 None 的字段会在 update_task_data 中被更新，
/// 实现部分更新语义，避免覆盖已有数据
#[derive(Debug, Default)]
pub struct TaskUpdateData {
    /// 任务状态码（对应 TaskStatus 枚举值）
    pub state: Option<i32>,
    /// 任务进度百分比（0-100）
    pub progress: Option<u32>,
    /// 生成的视频脚本内容
    pub script: Option<String>,
    /// 搜索关键词列表
    pub terms: Option<Vec<String>>,
    /// 生成的音频文件路径
    pub audio_file: Option<String>,
    /// 音频时长（秒）
    pub audio_duration: Option<f64>,
    /// 字幕文件路径
    pub subtitle_path: Option<String>,
    /// 视频素材文件路径列表
    pub materials: Option<Vec<String>>,
    /// 最终生成的视频文件路径列表
    pub videos: Option<Vec<String>>,
    /// 合成后的视频文件路径列表（含音频但未加字幕/BGM）
    pub combined_videos: Option<Vec<String>>,
}
