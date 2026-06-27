/// 音乐（BGM）API 处理器
///
/// 处理背景音乐相关的 API 请求：
/// - "list" → 列出 songs 目录下的音乐文件
/// - "upload" → 上传 BGM 文件

use actix_multipart::Multipart;
use actix_web::HttpResponse;
use futures::StreamExt;
use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use soma_core::utils;

/// 音乐模块请求分发
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_musics(param).await,
        "upload" => Err(error!("音乐上传请使用 /api/v1/musics/upload 接口")),
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 列出所有可用音乐文件
async fn list_musics(_param: &RequestParameter) -> Result<Value> {
    let song_dir = utils::song_dir();

    if !song_dir.exists() {
        return Ok(value!({
            "list": [],
            "total": 0,
        }));
    }

    let mut items: Vec<Value> = Vec::new();
    let entries = std::fs::read_dir(&song_dir).map_err(|e| error!("读取目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if ext != "mp3" && ext != "wav" && ext != "aac" && ext != "ogg" {
            continue;
        }
        let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let size = path.metadata().map(|m| m.len()).unwrap_or(0);
        items.push(value!({
            "name": name,
            "file": file_name,
            "path": path.to_string_lossy().to_string(),
            "size": size,
        }));
    }

    let total = items.len();
    Ok(value!({
        "list": items,
        "total": total,
    }))
}

/// 处理音乐文件上传（multipart/form-data）
pub async fn upload_file(mut payload: Multipart) -> HttpResponse {
    let song_dir = utils::song_dir();
    std::fs::create_dir_all(&song_dir).ok();

    let mut saved_name = String::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let filename = match field.content_disposition() {
            Some(cd) => cd.get_filename().unwrap_or("unknown").to_string(),
            None => "unknown".to_string(),
        };

        if filename.is_empty() {
            continue;
        }

        let dest_path = song_dir.join(&filename);
        let mut body = Vec::new();
        while let Some(Ok(chunk)) = field.next().await {
            body.extend_from_slice(&chunk);
        }

        if std::fs::write(&dest_path, &body).is_ok() {
            saved_name = filename;
        }
    }

    if saved_name.is_empty() {
        let resp = tube_web::response::get_error(error!("未接收到上传文件"));
        return resp.unwrap_or(HttpResponse::BadRequest().finish());
    }

    let result = value!({
        "name": saved_name.clone(),
        "path": song_dir.join(&saved_name).to_string_lossy().to_string(),
    });
    let resp = tube_web::response::get_success(&result);
    resp.unwrap_or(HttpResponse::Ok().finish())
}
