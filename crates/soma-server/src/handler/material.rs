/// 素材文件 API 处理器
///
/// 处理视频素材相关的 API 请求：
/// - "list" → 列出素材目录下的视频/图片文件
/// - "upload" → 上传素材文件（暂未实现）

use tube::{Error, Result, Value};
use tube_web::RequestParameter;
use soma_core::utils;

/// 素材模块请求分发
///
/// 根据 method 字段分发到对应的处理函数
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "list" => list_materials(param).await,
        "upload" => upload_material(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 列出素材目录中的文件
///
/// 扫描指定目录（默认为 materials 存储目录）下的视频和图片文件，
/// 支持通过 directory 参数自定义扫描路径。
/// 仅列出 FILE_TYPE_VIDEOS 和 FILE_TYPE_IMAGES 中定义的格式。
///
/// 返回：包含 list 数组和 total 数量的 JSON 对象
async fn list_materials(param: &RequestParameter) -> Result<Value> {
    // 支持自定义目录，默认使用标准素材存储目录
    let custom_dir = param.value.get_def_string("directory", "");
    let material_dir = if custom_dir.is_empty() {
        utils::storage_dir("materials", true).to_string_lossy().to_string()
    } else {
        custom_dir
    };

    let dir = std::path::Path::new(&material_dir);
    // 目录不存在时返回空列表
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
        // 仅列出视频和图片格式的文件
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

/// 上传素材文件（暂未实现）
///
/// 当前提示用户通过文件系统直接将文件放入素材目录
async fn upload_material(_param: &RequestParameter) -> Result<Value> {
    Err(error!("素材上传暂未实现，请通过文件系统直接放入素材目录"))
}
