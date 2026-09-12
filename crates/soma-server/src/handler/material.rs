/// 素材文件 API 处理器
///
/// 处理视频素材相关的 API 请求：
/// - "list" → 列出素材目录下的视频/图片文件
/// - "upload" → 上传素材文件

use actix_multipart::Multipart;
use actix_web::HttpResponse;
use futures::StreamExt;
use tube::{Result, Value};
use tube_web::RequestParameter;
use soma_core::utils;

/// 素材模块请求分发
///
/// 根据 method 字段分发到对应的处理函数
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_materials(param).await,
        "fonts" => list_fonts().await,
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

pub async fn distribute_portraits(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_portraits().await,
        "delete" => delete_portrait(param).await,
        "upload" => Err(error!("人像上传请使用 /api/v1/portraits/upload 接口（multipart/form-data）")),
        _ => Err(error!("不支持的方法: portraits.{}", param.method)),
    }
}

/// 列出人像照片目录中的文件，按修改时间倒序排列
async fn list_portraits() -> Result<Value> {
    let portrait_dir = utils::storage_dir("portraits", false);

    if !portrait_dir.exists() {
        return Ok(value!({
            "list": [],
            "total": 0,
        }));
    }

    let mut items: Vec<(std::time::SystemTime, Value)> = Vec::new();
    let entries = std::fs::read_dir(&portrait_dir).map_err(|e| error!("读取目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if !soma_core::models::DH_PORTRAIT_FILE_TYPES.contains(&ext.as_str()) {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let metadata = match path.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = metadata.len();
        let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
        let uploaded_at = modified
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        items.push((
            modified,
            value!({
                "name": name,
                "path": path.to_string_lossy().to_string(),
                "size": size,
                "type": ext,
                "uploadedAt": uploaded_at,
            }),
        ));
    }

    items.sort_by(|a, b| b.0.cmp(&a.0));
    let list: Vec<Value> = items.into_iter().map(|(_, v)| v).collect();
    let total = list.len();
    Ok(value!({
        "list": list,
        "total": total,
    }))
}

/// 删除指定人像照片
async fn delete_portrait(param: &RequestParameter) -> Result<Value> {
    let name = param.value.get_def_string("name", "");
    if name.is_empty() {
        return Err(error!("缺少参数: name"));
    }

    let safe_name = utils::sanitize_upload_filename(&name)
        .map_err(|e| error!("文件名不合法: {}", e))?;

    let portrait_dir = utils::storage_dir("portraits", false);
    let dest_path = portrait_dir.join(&safe_name);

    if !dest_path.exists() {
        return Ok(value!({
            "deleted": false,
            "reason": "文件不存在",
        }));
    }

    std::fs::remove_file(&dest_path).map_err(|e| error!("删除文件失败: {}", e))?;

    Ok(value!({
        "deleted": true,
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

/// 处理自定义音频文件上传（multipart/form-data）
pub async fn upload_audio(mut payload: Multipart) -> HttpResponse {
    let audio_dir = utils::storage_dir("audio", true);
    std::fs::create_dir_all(&audio_dir).ok();

    let mut saved_name = String::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let filename = match field.content_disposition() {
            Some(cd) => cd.get_filename().unwrap_or("unknown").to_string(),
            None => "unknown".to_string(),
        };

        if filename.is_empty() {
            continue;
        }

        let dest_path = audio_dir.join(&filename);
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
        "path": audio_dir.join(&saved_name).to_string_lossy().to_string(),
    });
    let resp = tube_web::response::get_success(&result);
    resp.unwrap_or(HttpResponse::Ok().finish())
}

/// 处理人像图片上传（multipart/form-data）
///
/// 校验扩展名白名单（jpg/jpeg/png）与文件大小（≤ 10MB），
/// 使用 UUID 命名保证唯一性，写入前检测磁盘可用空间。
pub async fn upload_portrait(mut payload: Multipart) -> HttpResponse {
    let portrait_dir = utils::storage_dir("portraits", true);
    std::fs::create_dir_all(&portrait_dir).ok();

    let mut saved_name = String::new();

    while let Some(Ok(mut field)) = payload.next().await {
        let filename = match field.content_disposition() {
            Some(cd) => cd.get_filename().unwrap_or("unknown").to_string(),
            None => "unknown".to_string(),
        };

        if filename.is_empty() {
            continue;
        }

        let safe_filename = match utils::sanitize_upload_filename(&filename) {
            Ok(n) => n,
            Err(_) => {
                let resp = tube_web::response::get_error(error!("文件类型不合法"));
                return resp.unwrap_or(HttpResponse::BadRequest().finish());
            }
        };

        let ext = std::path::Path::new(&safe_filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !soma_core::models::DH_PORTRAIT_FILE_TYPES.contains(&ext.as_str()) {
            let resp = tube_web::response::get_error(error!("仅支持 JPG/PNG 格式的人像照片"));
            return resp.unwrap_or(HttpResponse::BadRequest().finish());
        }

        let mut body = Vec::new();
        while let Some(Ok(chunk)) = field.next().await {
            body.extend_from_slice(&chunk);
        }

        if body.len() as u64 > soma_core::models::DH_PORTRAIT_MAX_SIZE {
            let resp = tube_web::response::get_error(error!("照片大小不能超过 10MB"));
            return resp.unwrap_or(HttpResponse::BadRequest().finish());
        }

        let uuid_name = format!("{}.{}", utils::get_uuid(), ext);
        let dest_path = portrait_dir.join(&uuid_name);

        match std::fs::write(&dest_path, &body) {
            Ok(_) => saved_name = uuid_name,
            Err(e) => {
                let resp = tube_web::response::get_error(error!("存储空间不足，上传失败: {}", e));
                return resp.unwrap_or(HttpResponse::BadRequest().finish());
            }
        }
    }

    if saved_name.is_empty() {
        let resp = tube_web::response::get_error(error!("未接收到上传文件"));
        return resp.unwrap_or(HttpResponse::BadRequest().finish());
    }

    let result = value!({
        "name": saved_name.clone(),
        "path": portrait_dir.join(&saved_name).to_string_lossy().to_string(),
    });
    let resp = tube_web::response::get_success(&result);
    resp.unwrap_or(HttpResponse::Ok().finish())
}

async fn list_fonts() -> Result<Value> {
    let fonts_dir = utils::storage_dir("fonts", false);
    let mut font_list = Vec::new();

    if fonts_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&fonts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if matches!(ext_lower.as_str(), "ttf" | "ttc" | "otf") {
                        font_list.push(value!({
                            "name": path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                            "path": path.to_string_lossy().to_string(),
                        }));
                    }
                }
            }
        }
    }

    if font_list.is_empty() {
        let system_font_dirs: Vec<String> = if cfg!(target_os = "macos") {
            vec!["/Library/Fonts".to_string(), "/System/Library/Fonts".to_string()]
        } else if cfg!(target_os = "linux") {
            vec!["/usr/share/fonts".to_string(), "/usr/local/share/fonts".to_string()]
        } else if cfg!(target_os = "windows") {
            let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
            vec![format!("{}/Fonts", windir.replace('\\', "/"))]
        } else {
            vec![]
        };

        for dir in &system_font_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        let ext_lower = ext.to_string_lossy().to_lowercase();
                        if matches!(ext_lower.as_str(), "ttf" | "ttc" | "otf") {
                            font_list.push(value!({
                                "name": path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                                "path": path.to_string_lossy().to_string(),
                            }));
                        }
                    }
                }
            }
        }
    }

    Ok(value!({
        "list": font_list,
    }))
}
