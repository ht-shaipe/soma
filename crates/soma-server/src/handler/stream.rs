use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use crate::state;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "play" => stream_video(param).await,
        "download" => download_video(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

async fn stream_video(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_task(&task_id).ok_or_else(|| error!("任务不存在: {}", task_id))?;

    let video_index = param.value.get_i32("index", 0) as usize;

    let videos = task.videos.as_deref().unwrap_or(&[]);
    if videos.is_empty() {
        return Err(error!("视频尚未生成"));
    }

    let idx = if video_index > 0 && video_index <= videos.len() {
        video_index - 1
    } else {
        0
    };

    let video_path = &videos[idx];
    let conf = crate::Config::get();
    let endpoint = conf.app.get_endpoint();

    let url = if endpoint.is_empty() {
        format!("/storage/tasks/{}/final-{}.mp4", task_id, idx + 1)
    } else {
        format!("{}/storage/tasks/{}/final-{}.mp4", endpoint.trim_end_matches('/'), task_id, idx + 1)
    };

    Ok(value!({
        "url": url,
        "path": video_path.clone(),
    }))
}

async fn download_video(param: &RequestParameter) -> Result<Value> {
    stream_video(param).await
}
