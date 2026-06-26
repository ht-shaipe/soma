/// 音乐（BGM）API 处理器
///
/// 处理背景音乐相关的 API 请求：
/// - "list" → 列出 songs 目录下的音乐文件
/// - "upload" → 上传 BGM 文件（暂未实现）

use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use soma_core::utils;

/// 音乐模块请求分发
///
/// 根据 method 字段分发到对应的处理函数
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_musics(param).await,
        "upload" => upload_music(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 列出所有可用音乐文件
///
/// 扫描 songs 目录下所有 mp3/wav/aac/ogg 格式的音频文件，
/// 返回文件名、路径和大小信息。
///
/// 返回：包含 list 数组和 total 数量的 JSON 对象
async fn list_musics(_param: &RequestParameter) -> Result<Value> {
    let song_dir = utils::song_dir();

    // 目录不存在时返回空列表
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
        // 仅列出支持的音频格式
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

/// 上传音乐文件（暂未实现）
///
/// 当前提示用户通过文件系统直接将文件放入 songs 目录
async fn upload_music(_param: &RequestParameter) -> Result<Value> {
    Err(error!("BGM上传暂未实现，请通过文件系统直接放入songs目录"))
}
