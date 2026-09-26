//! 视频下载 API 处理器
//!
//! 基于 yt-dlp 的通用视频下载，支持 1000+ 站点。

use tube::{Result, Value};
use tube_web::RequestParameter;
use soma_stock::ytdlp;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "check" => check(param).await,
        "info" => info(param).await,
        "download" => download(param).await,
        "batch" => batch(param).await,
        "extract_urls" => extract_urls(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 检查 yt-dlp 是否安装
async fn check(_param: &RequestParameter) -> Result<Value> {
    match ytdlp::check_ytdlp() {
        Ok(version) => Ok(value!({
            "installed": true,
            "version": version,
        })),
        Err(e) => Ok(value!({
            "installed": false,
            "version": "",
            "error": format!("{:?}", e),
        })),
    }
}

/// 获取视频信息（不下载）
async fn info(param: &RequestParameter) -> Result<Value> {
    let url = param.value.get_def_string("url", "");
    if url.is_empty() {
        return Err(error!("缺少 url 参数"));
    }

    let proxy = param.value.get_def_string("proxy", "");
    let proxy = if proxy.is_empty() { None } else { Some(proxy.as_str()) };

    let vinfo = ytdlp::get_info(&url, proxy).await
        .map_err(|e| error!("获取视频信息失败: {:?}", e))?;

    let formats: Vec<Value> = vinfo.formats.iter().map(|f| {
        value!({
            "formatId": f.format_id.clone(),
            "ext": f.ext.clone(),
            "resolution": f.resolution.as_deref().unwrap_or(""),
            "filesize": f.filesize.unwrap_or(0),
            "hasVideo": f.has_video,
            "hasAudio": f.has_audio,
        })
    }).collect();

    Ok(value!({
        "id": vinfo.id,
        "title": vinfo.title,
        "duration": vinfo.duration.unwrap_or(0.0),
        "uploader": vinfo.uploader.as_deref().unwrap_or(""),
        "uploadDate": vinfo.upload_date.as_deref().unwrap_or(""),
        "description": vinfo.description.as_deref().unwrap_or(""),
        "thumbnail": vinfo.thumbnail.as_deref().unwrap_or(""),
        "formats": formats,
    }))
}

/// 下载视频
async fn download(param: &RequestParameter) -> Result<Value> {
    let url = param.value.get_def_string("url", "");
    if url.is_empty() {
        return Err(error!("缺少 url 参数"));
    }

    let save_dir = param.value.get_def_string("saveDir", "");
    if save_dir.is_empty() {
        return Err(error!("缺少 saveDir 参数"));
    }

    let params = ytdlp::YtdlpParams {
        url: url.clone(),
        save_dir,
        output_template: param.value.get("outputTemplate").and_then(|v| v.as_str()).map(String::from),
        format: param.value.get("format").and_then(|v| v.as_str()).map(String::from),
        audio_only: param.value.get("audioOnly").and_then(|v| v.as_bool()).unwrap_or(false),
        audio_quality: param.value.get("audioQuality").and_then(|v| v.as_str()).map(String::from),
        proxy: param.value.get("proxy").and_then(|v| v.as_str()).map(String::from),
        rate_limit: param.value.get("rateLimit").and_then(|v| v.as_str()).map(String::from),
        skip_existing: param.value.get("skipExisting").and_then(|v| v.as_bool()).unwrap_or(true),
    };

    let path = ytdlp::download(&params).await
        .map_err(|e| error!("下载失败: {:?}", e))?;

    Ok(value!({
        "success": true,
        "path": path,
        "platform": ytdlp::detect_platform(&url),
    }))
}

/// 批量下载
async fn batch(param: &RequestParameter) -> Result<Value> {
    let urls_val = param.value.get("urls").and_then(|v| v.as_array());
    let urls: Vec<String> = if let Some(arr) = urls_val {
        arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
    } else {
        return Err(error!("缺少 urls 参数"));
    };

    if urls.is_empty() {
        return Err(error!("URL 列表为空"));
    }

    let save_dir = param.value.get_def_string("saveDir", "");
    if save_dir.is_empty() {
        return Err(error!("缺少 saveDir 参数"));
    }

    let proxy = param.value.get_def_string("proxy", "");
    let audio_only = param.value.get("audioOnly").and_then(|v| v.as_bool()).unwrap_or(false);
    let proxy = if proxy.is_empty() { None } else { Some(proxy.as_str()) };

    let paths = ytdlp::download_batch(&urls, &save_dir, proxy, audio_only).await
        .map_err(|e| error!("批量下载失败: {:?}", e))?;

    Ok(value!({
        "success": true,
        "count": paths.len(),
        "paths": paths,
    }))
}

/// 从分享文本中提取 URL
async fn extract_urls(param: &RequestParameter) -> Result<Value> {
    let text = param.value.get_def_string("text", "");
    if text.is_empty() {
        return Err(error!("缺少 text 参数"));
    }

    let urls = ytdlp::extract_urls(&text);
    let results: Vec<Value> = urls.iter().map(|u| {
        value!({
            "url": u,
            "platform": ytdlp::detect_platform(u),
        })
    }).collect();

    Ok(value!({
        "urls": results,
    }))
}
