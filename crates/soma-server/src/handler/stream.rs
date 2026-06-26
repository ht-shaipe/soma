/// 视频流/下载 API 处理器
///
/// 处理视频播放和下载相关的 API 请求：
/// - "play" → 获取视频播放地址
/// - "download" → 获取视频下载地址（当前与播放逻辑相同）

use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use crate::state;

/// 流媒体模块请求分发
///
/// 根据 method 字段分发到对应的处理函数
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "play" => stream_video(param).await,
        "download" => download_video(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 获取视频播放/流式访问地址
///
/// 根据任务 ID 和视频索引（从1开始），返回视频文件的访问 URL 和本地路径。
/// 支持通过配置的 endpoint 生成完整的访问地址，未配置时使用相对路径。
///
/// 返回：包含 url（访问地址）和 path（本地路径）的 JSON 对象
async fn stream_video(param: &RequestParameter) -> Result<Value> {
    let task_id = param.value.get_def_string("taskId", "");
    if task_id.is_empty() {
        return Err(error!("taskId 不能为空"));
    }

    let task = state::get_task(&task_id).ok_or_else(|| error!("任务不存在: {}", task_id))?;

    // 视频索引（从1开始，0或越界默认取第一个）
    let video_index = param.value.get_i32("index", 0) as usize;

    let videos = task.videos.as_deref().unwrap_or(&[]);
    if videos.is_empty() {
        return Err(error!("视频尚未生成"));
    }

    // 将索引转换为数组下标（用户传入1-based，转为0-based）
    let idx = if video_index > 0 && video_index <= videos.len() {
        video_index - 1
    } else {
        0
    };

    let video_path = &videos[idx];
    let conf = crate::Config::get();
    let endpoint = conf.app.get_endpoint();

    // 根据 endpoint 配置生成完整或相对的访问 URL
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

/// 下载视频
///
/// 当前实现与播放相同，均返回视频文件的访问地址
async fn download_video(param: &RequestParameter) -> Result<Value> {
    stream_video(param).await
}
