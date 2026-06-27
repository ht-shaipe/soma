/// 素材文件 API 处理器
///
/// 处理视频素材相关的 API 请求：
/// - "list" → 列出素材目录下的视频/图片文件
/// - "upload" → 上传素材文件

use actix_multipart::Multipart;
use actix_web::HttpResponse;
use futures::StreamExt;
use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use soma_core::utils;

/// 素材模块请求分发
///
/// 根据 method 字段分发到对应的处理函数
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_materials(param).await,
        "upload" => Err(error!("素材上传请使用 /api/v1/materials/upload 接口")),
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 列出素材目录中的文件
async fn list_materials(param: &RequestParameter) -> Result<Value> {
    let custom_dir = param.value.get_def_string("directory", "");
    let material_dir = if custom_dir.is_empty() {
        utils::storage_dir("materials", true).to_string_lossy().to_string()
    } else {
        custom_dir
    };

    let dir = std::path::Path::new(&material_dir);
    if !dir.exists() {
        return Ok(value!({
            "list": [],
            "total": 0,
        }));
    }

    let mut items: Vec<Value> = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| error!("读取目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !soma_core::models::FILE_TYPE_VIDEOS.contains(&ext.as_str())
            && !soma_core::models::FILE_TYPE_IMAGES.contains(&ext.as_str())
        {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let size = path.metadata().map(|m| m.len()).unwrap_or(0);
        items.push(value!({
            "name": name,
            "path": path.to_string_lossy().to_string(),
            "size": size,
            "type": ext,
        }));
    }

    let total = items.len();
    Ok(value!({
        "list": items,
        "total": total,
    }))
}

/// 处理素材文件上传（multipart/form-data）
pub async fn upload_file(mut payload: Multipart) -> HttpResponse {
    let material_dir = utils::storage_dir("materials", true);
    std::fs::create_dir_all(&material_dir).ok();

    let mut saved_name = String::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let filename = match field.content_disposition() {
            Some(cd) => cd.get_filename().unwrap_or("unknown").to_string(),
            None => "unknown".to_string(),
        };

        if filename.is_empty() {
            continue;
        }

        let dest_path = material_dir.join(&filename);
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
        "path": material_dir.join(&saved_name).to_string_lossy().to_string(),
    });
    let resp = tube_web::response::get_success(&result);
    resp.unwrap_or(HttpResponse::Ok().finish())
}
