use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use soma_core::utils;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_musics(param).await,
        "upload" => upload_music(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

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

async fn upload_music(_param: &RequestParameter) -> Result<Value> {
    Err(error!("BGM上传暂未实现，请通过文件系统直接放入songs目录"))
}
