use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use soma_core::utils;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_materials(param).await,
        "upload" => upload_material(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

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

async fn upload_material(_param: &RequestParameter) -> Result<Value> {
    Err(error!("素材上传暂未实现，请通过文件系统直接放入素材目录"))
}
