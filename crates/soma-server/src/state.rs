/// 全局任务状态管理模块
///
/// 通过 TaskStore trait 抽象任务存储，支持内存和 Redis 后端。
/// 提供任务的创建、查询、更新、删除等 CRUD 操作。

use soma_core::models::{TaskInfo, TaskStatus, StoryboardScene, VideoParams, AiVideoSegmentLog, DigitalHumanTaskInfo, DigitalHumanParams, ImageStoryTaskInfo, ImageStoryParams};
use crate::store::{TaskStore, InMemoryTaskStore, SqliteTaskStore};
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

/// 初始化 SQLite 持久化存储，失败时回退到内存存储
pub fn init_sqlite_store(db_path: &str) {
    match SqliteTaskStore::new(db_path) {
        Ok(store) => {
            log::info!("SQLite 任务存储已初始化: {}", db_path);
            init_store(Box::new(store));
        }
        Err(e) => {
            log::error!("SQLite 初始化失败 ({}), 回退到内存存储: {}", db_path, e);
        }
    }
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
    if let Some(ref sb) = d.storyboard { task.storyboard = Some(sb.clone()); }
    if let Some(ref n) = d.narration { task.narration = Some(n.clone()); }
    if let Some(ref a) = d.audio_file { task.audio_file = Some(a.clone()); }
        if let Some(dur) = d.audio_duration { task.audio_duration = Some(dur); }
        if let Some(ref s) = d.subtitle_path { task.subtitle_path = Some(s.clone()); }
        if let Some(ref m) = d.materials { task.materials = Some(m.clone()); }
        if let Some(ref v) = d.videos { task.videos = Some(v.clone()); }
    if let Some(ref v) = d.combined_videos { task.combined_videos = Some(v.clone()); }
    if let Some(ref v) = d.ai_video_logs { task.ai_video_logs = Some(v.clone()); }
    if let Some(ref e) = d.error_message { task.error_message = Some(e.clone()); }
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

pub fn create_task_entry(task_id: &str, params: VideoParams) {
    let task = TaskInfo::new(task_id.to_string(), params);
    lock_store().create(task);
}

pub fn create_draft_task_entry(task_id: &str, params: VideoParams) {
    let task = TaskInfo::with_status(task_id.to_string(), params, TaskStatus::Draft);
    lock_store().create(task);
}

pub fn update_task_params(task_id: &str, params: &VideoParams) {
    let p = params.clone();
    lock_store().update(task_id, Box::new(move |task| {
        // 允许 Draft 和 Failed 状态更新配置
        let allow_edit = task.state == TaskStatus::Draft.as_i32()
            || task.state == TaskStatus::Failed.as_i32();
        if !allow_edit {
            return;
        }
        if !p.video_subject.is_empty() { task.params.video_subject = p.video_subject.clone(); }
        if !p.video_script.is_empty() { task.params.video_script = p.video_script.clone(); }
        if p.video_terms.is_some() { task.params.video_terms = p.video_terms.clone(); }
        if p.video_aspect.is_some() { task.params.video_aspect = p.video_aspect.clone(); }
        if p.video_concat_mode.is_some() { task.params.video_concat_mode = p.video_concat_mode.clone(); }
        if p.video_transition_mode.is_some() { task.params.video_transition_mode = p.video_transition_mode.clone(); }
        if p.video_clip_duration.is_some() { task.params.video_clip_duration = p.video_clip_duration; }
        if p.match_materials_to_script.is_some() { task.params.match_materials_to_script = p.match_materials_to_script; }
        if p.video_count.is_some() { task.params.video_count = p.video_count; }
        if p.video_source.is_some() { task.params.video_source = p.video_source.clone(); }
        if p.video_materials.is_some() { task.params.video_materials = p.video_materials.clone(); }
        if p.custom_audio_file.is_some() { task.params.custom_audio_file = p.custom_audio_file.clone(); }
        if p.video_language.is_some() { task.params.video_language = p.video_language.clone(); }
        if p.voice_name.is_some() { task.params.voice_name = p.voice_name.clone(); }
        if p.voice_volume.is_some() { task.params.voice_volume = p.voice_volume; }
        if p.voice_rate.is_some() { task.params.voice_rate = p.voice_rate; }
        if p.bgm_type.is_some() { task.params.bgm_type = p.bgm_type.clone(); }
        if p.bgm_file.is_some() { task.params.bgm_file = p.bgm_file.clone(); }
        if p.bgm_volume.is_some() { task.params.bgm_volume = p.bgm_volume; }
        if p.subtitle_enabled.is_some() { task.params.subtitle_enabled = p.subtitle_enabled; }
        if p.subtitle_position.is_some() { task.params.subtitle_position = p.subtitle_position.clone(); }
        if p.custom_position.is_some() { task.params.custom_position = p.custom_position; }
        if p.font_name.is_some() { task.params.font_name = p.font_name.clone(); }
        if p.font_size.is_some() { task.params.font_size = p.font_size; }
        if p.text_fore_color.is_some() { task.params.text_fore_color = p.text_fore_color.clone(); }
        if p.stroke_color.is_some() { task.params.stroke_color = p.stroke_color.clone(); }
        if p.stroke_width.is_some() { task.params.stroke_width = p.stroke_width; }
        if p.text_background_color.is_some() { task.params.text_background_color = p.text_background_color.clone(); }
        if p.rounded_subtitle_background.is_some() { task.params.rounded_subtitle_background = p.rounded_subtitle_background; }
        if p.paragraph_number.is_some() { task.params.paragraph_number = p.paragraph_number; }
        if p.video_script_prompt.is_some() { task.params.video_script_prompt = p.video_script_prompt.clone(); }
        if p.custom_system_prompt.is_some() { task.params.custom_system_prompt = p.custom_system_prompt.clone(); }
        if p.use_custom_system_prompt.is_some() { task.params.use_custom_system_prompt = p.use_custom_system_prompt; }
        if p.video_encoder.is_some() { task.params.video_encoder = p.video_encoder.clone(); }
        if p.video_watermark.is_some() { task.params.video_watermark = p.video_watermark.clone(); }
        if p.video_intro.is_some() { task.params.video_intro = p.video_intro.clone(); }
        if p.video_outro.is_some() { task.params.video_outro = p.video_outro.clone(); }
        if p.portrait_image.is_some() { task.params.portrait_image = p.portrait_image.clone(); }
        if p.n_threads.is_some() { task.params.n_threads = p.n_threads; }
        if p.intent_style.is_some() { task.params.intent_style = p.intent_style.clone(); }
        if p.intent_mood.is_some() { task.params.intent_mood = p.intent_mood.clone(); }
        if p.intent_audience.is_some() { task.params.intent_audience = p.intent_audience.clone(); }
        task.updated_at = chrono::Utc::now();
    }));
}

pub fn update_task_params_from_json(task_id: &str, json: &serde_json::Value) {
    let j = json.clone();
    lock_store().update(task_id, Box::new(move |task| {
        // 允许 Draft 和 Failed 状态更新配置
        let allow_edit = task.state == TaskStatus::Draft.as_i32()
            || task.state == TaskStatus::Failed.as_i32();
        if !allow_edit {
            return;
        }
        if let Some(v) = j.get("video_subject").and_then(|v| v.as_str()) {
            if !v.is_empty() { task.params.video_subject = v.to_string(); }
        }
        if let Some(v) = j.get("video_script").and_then(|v| v.as_str()) {
            task.params.video_script = v.to_string();
        }
        if j.get("video_terms").is_some() {
            task.params.video_terms = j.get("video_terms").cloned();
        }
        if let Some(v) = j.get("video_aspect").and_then(|v| v.as_str()) {
            task.params.video_aspect = Some(v.to_string());
        }
        if let Some(v) = j.get("video_concat_mode").and_then(|v| v.as_str()) {
            task.params.video_concat_mode = Some(v.to_string());
        }
        if let Some(v) = j.get("video_transition_mode").and_then(|v| v.as_str()) {
            task.params.video_transition_mode = Some(v.to_string());
        }
        if let Some(v) = j.get("video_clip_duration").and_then(|v| v.as_u64()) {
            task.params.video_clip_duration = Some(v as u32);
        }
        if let Some(v) = j.get("match_materials_to_script").and_then(|v| v.as_bool()) {
            task.params.match_materials_to_script = Some(v);
        }
        if let Some(v) = j.get("video_count").and_then(|v| v.as_u64()) {
            task.params.video_count = Some(v as u32);
        }
        if let Some(v) = j.get("video_source").and_then(|v| v.as_str()) {
            task.params.video_source = Some(v.to_string());
        }
        if let Some(v) = j.get("video_materials") {
            if let Ok(m) = serde_json::from_value::<Vec<soma_core::models::MaterialInfo>>(v.clone()) {
                task.params.video_materials = Some(m);
            }
        }
        if let Some(v) = j.get("custom_audio_file").and_then(|v| v.as_str()) {
            task.params.custom_audio_file = Some(v.to_string());
        }
        if let Some(v) = j.get("video_language").and_then(|v| v.as_str()) {
            task.params.video_language = Some(v.to_string());
        }
        if let Some(v) = j.get("voice_name").and_then(|v| v.as_str()) {
            task.params.voice_name = Some(v.to_string());
        }
        if let Some(v) = j.get("voice_volume").and_then(|v| v.as_f64()) {
            task.params.voice_volume = Some(v as f32);
        }
        if let Some(v) = j.get("voice_rate").and_then(|v| v.as_f64()) {
            task.params.voice_rate = Some(v as f32);
        }
        if let Some(v) = j.get("bgm_type").and_then(|v| v.as_str()) {
            task.params.bgm_type = Some(v.to_string());
        }
        if let Some(v) = j.get("bgm_file").and_then(|v| v.as_str()) {
            task.params.bgm_file = Some(v.to_string());
        }
        if let Some(v) = j.get("bgm_volume").and_then(|v| v.as_f64()) {
            task.params.bgm_volume = Some(v as f32);
        }
        if let Some(v) = j.get("subtitle_enabled").and_then(|v| v.as_bool()) {
            task.params.subtitle_enabled = Some(v);
        }
        if let Some(v) = j.get("subtitle_position").and_then(|v| v.as_str()) {
            task.params.subtitle_position = Some(v.to_string());
        }
        if let Some(v) = j.get("custom_position").and_then(|v| v.as_f64()) {
            task.params.custom_position = Some(v);
        }
        if let Some(v) = j.get("font_name").and_then(|v| v.as_str()) {
            task.params.font_name = Some(v.to_string());
        }
        if let Some(v) = j.get("font_size").and_then(|v| v.as_u64()) {
            task.params.font_size = Some(v as u32);
        }
        if let Some(v) = j.get("text_fore_color").and_then(|v| v.as_str()) {
            task.params.text_fore_color = Some(v.to_string());
        }
        if let Some(v) = j.get("stroke_color").and_then(|v| v.as_str()) {
            task.params.stroke_color = Some(v.to_string());
        }
        if let Some(v) = j.get("stroke_width").and_then(|v| v.as_f64()) {
            task.params.stroke_width = Some(v as f32);
        }
        if j.get("text_background_color").is_some() {
            task.params.text_background_color = j.get("text_background_color").cloned();
        }
        if let Some(v) = j.get("rounded_subtitle_background").and_then(|v| v.as_bool()) {
            task.params.rounded_subtitle_background = Some(v);
        }
        if let Some(v) = j.get("paragraph_number").and_then(|v| v.as_u64()) {
            task.params.paragraph_number = Some(v as u32);
        }
        if let Some(v) = j.get("video_script_prompt").and_then(|v| v.as_str()) {
            task.params.video_script_prompt = Some(v.to_string());
        }
        if let Some(v) = j.get("custom_system_prompt").and_then(|v| v.as_str()) {
            task.params.custom_system_prompt = Some(v.to_string());
        }
        if let Some(v) = j.get("use_custom_system_prompt").and_then(|v| v.as_bool()) {
            task.params.use_custom_system_prompt = Some(v);
        }
        if let Some(v) = j.get("video_encoder").and_then(|v| v.as_str()) {
            task.params.video_encoder = Some(v.to_string());
        }
        if let Some(v) = j.get("video_watermark").and_then(|v| v.as_str()) {
            task.params.video_watermark = Some(v.to_string());
        }
        if let Some(v) = j.get("video_intro").and_then(|v| v.as_str()) {
            task.params.video_intro = Some(v.to_string());
        }
        if let Some(v) = j.get("video_outro").and_then(|v| v.as_str()) {
            task.params.video_outro = Some(v.to_string());
        }
        if let Some(v) = j.get("portrait_image").and_then(|v| v.as_str()) {
            task.params.portrait_image = Some(v.to_string());
        }
        if let Some(v) = j.get("n_threads").and_then(|v| v.as_u64()) {
            task.params.n_threads = Some(v as u32);
        }
        if let Some(v) = j.get("intent_style").and_then(|v| v.as_str()) {
            task.params.intent_style = Some(v.to_string());
        }
        if let Some(v) = j.get("intent_mood").and_then(|v| v.as_str()) {
            task.params.intent_mood = Some(v.to_string());
        }
        if let Some(v) = j.get("intent_audience").and_then(|v| v.as_str()) {
            task.params.intent_audience = Some(v.to_string());
        }
        task.updated_at = chrono::Utc::now();
    }));
}

pub fn set_task_state(task_id: &str, state: TaskStatus) {
    let state_val = state.as_i32();
    lock_store().update(task_id, Box::new(move |task| {
        task.state = state_val;
        task.updated_at = chrono::Utc::now();
    }));
}

#[derive(Debug, Default, Clone)]
pub struct TaskUpdateData {
    pub state: Option<i32>,
    pub progress: Option<u32>,
    pub script: Option<String>,
    pub terms: Option<Vec<String>>,
    pub storyboard: Option<Vec<StoryboardScene>>,
    pub narration: Option<String>,
    pub audio_file: Option<String>,
    pub audio_duration: Option<f64>,
    pub subtitle_path: Option<String>,
    pub materials: Option<Vec<String>>,
    pub videos: Option<Vec<String>>,
    pub combined_videos: Option<Vec<String>>,
    pub ai_video_logs: Option<Vec<AiVideoSegmentLog>>,
    pub error_message: Option<String>,
}

/// 数字人任务更新数据
#[derive(Debug, Default, Clone)]
pub struct DhTaskUpdateData {
    pub state: Option<i32>,
    pub progress: Option<u32>,
    pub audio_file: Option<String>,
    pub audio_duration: Option<f64>,
    pub subtitle_path: Option<String>,
    pub portrait_video_path: Option<String>,
    pub final_video_path: Option<String>,
    pub error_message: Option<String>,
    pub segment_count: Option<u32>,
    pub current_segment: Option<u32>,
    pub segment_audio_files: Option<Vec<String>>,
    pub segment_video_files: Option<Vec<String>>,
    pub merchant_id: Option<String>,
    pub heygem_task_code: Option<String>,
    pub live2d_model_id: Option<String>,
    pub frames_dir: Option<String>,
}

/// 创建数字人任务条目（状态 Processing）
pub fn create_dh_task_entry(task_id: &str, params: DigitalHumanParams) {
    let task = DigitalHumanTaskInfo::new(task_id.to_string(), params);
    lock_store().create_dh(task);
}

/// 创建数字人草稿任务条目（状态 Draft）
pub fn create_dh_draft_task_entry(task_id: &str, params: DigitalHumanParams) {
    let task = DigitalHumanTaskInfo::with_status(task_id.to_string(), params, TaskStatus::Draft);
    lock_store().create_dh(task);
}

/// 获取数字人任务
pub fn get_dh_task(task_id: &str) -> Option<DigitalHumanTaskInfo> {
    lock_store().get_dh(task_id)
}

/// 获取全部数字人任务（分页）
pub fn get_all_dh_tasks(page: usize, page_size: usize) -> (Vec<DigitalHumanTaskInfo>, usize) {
    let mut tasks = lock_store().get_all_dh();
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

/// 删除数字人任务
pub fn delete_dh_task(task_id: &str) -> bool {
    lock_store().delete_dh(task_id)
}

/// 设置数字人任务状态
pub fn set_dh_task_state(task_id: &str, state: TaskStatus) {
    let state_val = state.as_i32();
    lock_store().update_dh(task_id, Box::new(move |task| {
        task.state = state_val;
        task.updated_at = chrono::Utc::now();
    }));
}

/// 更新数字人任务数据
pub fn update_dh_task_data(task_id: &str, data: &DhTaskUpdateData) {
    let d = data.clone();
    lock_store().update_dh(task_id, Box::new(move |task| {
        if let Some(ref s) = d.state { task.state = *s; }
        if let Some(p) = d.progress { task.progress = p; }
        if let Some(ref a) = d.audio_file { task.audio_file = Some(a.clone()); }
        if let Some(dur) = d.audio_duration { task.audio_duration = Some(dur); }
        if let Some(ref s) = d.subtitle_path { task.subtitle_path = Some(s.clone()); }
        if let Some(ref p) = d.portrait_video_path { task.portrait_video_path = Some(p.clone()); }
        if let Some(ref f) = d.final_video_path { task.final_video_path = Some(f.clone()); }
        if let Some(ref e) = d.error_message { task.error_message = Some(e.clone()); }
        if let Some(sc) = d.segment_count { task.segment_count = Some(sc); }
        if let Some(cs) = d.current_segment { task.current_segment = Some(cs); }
        if let Some(ref sa) = d.segment_audio_files { task.segment_audio_files = Some(sa.clone()); }
        if let Some(ref sv) = d.segment_video_files { task.segment_video_files = Some(sv.clone()); }
        if let Some(ref mid) = d.merchant_id { task.merchant_id = Some(mid.clone()); }
        if let Some(ref tc) = d.heygem_task_code { task.heygem_task_code = Some(tc.clone()); }
        if let Some(ref lid) = d.live2d_model_id { task.live2d_model_id = Some(lid.clone()); }
        if let Some(ref fd) = d.frames_dir { task.frames_dir = Some(fd.clone()); }
        task.updated_at = chrono::Utc::now();
    }));
}

/// 更新数字人任务进度
pub fn update_dh_task(task_id: &str, state: Option<i32>, progress: Option<u32>) {
    let state_val = state;
    let progress_val = progress;
    lock_store().update_dh(task_id, Box::new(move |task| {
        task.update(state_val, progress_val);
    }));
}

// ===== 图片故事任务管理函数 =====

/// 创建图片故事任务条目（状态 Processing）
pub fn create_image_story_task_entry(task_id: &str, params: ImageStoryParams) {
    let task = ImageStoryTaskInfo::new(task_id.to_string(), params);
    lock_store().create_image_story(task);
}

/// 获取图片故事任务
pub fn get_image_story_task(task_id: &str) -> Option<ImageStoryTaskInfo> {
    lock_store().get_image_story(task_id)
}

/// 获取全部图片故事任务（分页）
pub fn get_all_image_story_tasks(page: usize, page_size: usize) -> (Vec<ImageStoryTaskInfo>, usize) {
    let mut tasks = lock_store().get_all_image_story();
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

/// 删除图片故事任务
pub fn delete_image_story_task(task_id: &str) -> bool {
    lock_store().delete_image_story(task_id)
}

/// 设置图片故事任务状态
pub fn set_image_story_task_state(task_id: &str, state: TaskStatus) {
    let state_val = state.as_i32();
    lock_store().update_image_story(task_id, Box::new(move |task| {
        task.state = state_val;
        task.updated_at = chrono::Utc::now();
    }));
}

/// 更新图片故事任务数据
pub fn update_image_story_task_data(task_id: &str, state: Option<i32>, progress: Option<u32>, video_path: Option<String>, error_message: Option<String>) {
    lock_store().update_image_story(task_id, Box::new(move |task| {
        if let Some(s) = state { task.state = s; }
        if let Some(p) = progress { task.progress = p; }
        if let Some(ref v) = video_path { task.video_path = Some(v.clone()); }
        if let Some(ref e) = error_message { task.error_message = Some(e.clone()); }
        task.updated_at = chrono::Utc::now();
    }));
}
