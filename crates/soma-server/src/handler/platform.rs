//! 平台直连 API 处理器
//!
//! 基于签名算法直接调用各平台 Web API（不依赖浏览器）。
//! 当前支持抖音（a_bogus 签名）。

use soma_stock::douyin::{self, DouyinClient};
use tube::{Result, Value};
use tube_web::RequestParameter;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "douyin_detail" => douyin_detail(param).await,
        "douyin_posts" => douyin_posts(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 构建抖音客户端
fn build_client(param: &RequestParameter) -> Result<DouyinClient> {
    let cookie = param.value.get_def_string("cookie", "");
    let cookie = if cookie.is_empty() {
        None
    } else {
        Some(cookie)
    };

    let proxy = param.value.get_def_string("proxy", "");
    let proxy = if proxy.is_empty() { None } else { Some(proxy) };

    DouyinClient::new(cookie, proxy).map_err(|e| error!("{:?}", e))
}

/// 获取抖音视频详情
async fn douyin_detail(param: &RequestParameter) -> Result<Value> {
    let input = param.value.get_def_string("awemeId", "");
    if input.is_empty() {
        return Err(error!("缺少 awemeId 参数（视频 ID 或视频页 URL）"));
    }

    let aweme_id = douyin::extract_aweme_id(&input)
        .ok_or_else(|| error!("无法从输入中提取视频 ID: {}", input))?;

    let client = build_client(param)?;
    let detail = client
        .video_detail(&aweme_id)
        .await
        .map_err(|e| error!("获取视频详情失败: {}", e))?;

    let desc = detail.get("desc").and_then(|v| v.as_str()).unwrap_or("");
    let create_time = detail
        .get("create_time")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let stats = detail.get("statistics").cloned().unwrap_or_default();
    let author = detail.get("author").cloned().unwrap_or_default();
    let video = detail.get("video").cloned().unwrap_or_default();

    Ok(value!({
        "awemeId": detail.get("aweme_id").and_then(|v| v.as_str()).unwrap_or(""),
        "desc": desc,
        "createTime": create_time,
        "author": {
            "nickname": author.get("nickname").and_then(|v| v.as_str()).unwrap_or(""),
            "secUid": author.get("sec_uid").and_then(|v| v.as_str()).unwrap_or(""),
        },
        "statistics": {
            "diggCount": stats.get("digg_count").and_then(|v| v.as_i64()).unwrap_or(0),
            "commentCount": stats.get("comment_count").and_then(|v| v.as_i64()).unwrap_or(0),
            "shareCount": stats.get("share_count").and_then(|v| v.as_i64()).unwrap_or(0),
            "collectCount": stats.get("collect_count").and_then(|v| v.as_i64()).unwrap_or(0),
            "playCount": stats.get("play_count").and_then(|v| v.as_i64()).unwrap_or(0),
        },
        "video": {
            "cover": video.get("cover").and_then(|v| v.get("url_list")).and_then(|l| l.as_array()).and_then(|l| l.first()).and_then(|u| u.as_str()).unwrap_or(""),
            "durationMs": video.get("duration").and_then(|v| v.as_i64()).unwrap_or(0),
        },
    }))
}

/// 获取抖音用户作品列表
async fn douyin_posts(param: &RequestParameter) -> Result<Value> {
    let input = param.value.get_def_string("secUserId", "");
    if input.is_empty() {
        return Err(error!("缺少 secUserId 参数（sec_uid 或用户主页 URL）"));
    }

    let sec_uid = douyin::extract_sec_uid(&input).unwrap_or_else(|| input.trim().to_string());

    let count = param
        .value
        .get("count")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as u32;
    let cursor = param
        .value
        .get("cursor")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let client = build_client(param)?;
    let body = client
        .user_posts(&sec_uid, count, cursor)
        .await
        .map_err(|e| error!("获取作品列表失败: {}", e))?;

    let posts: Vec<Value> = body
        .get("aweme_list")
        .and_then(|v| v.as_array())
        .map(|list| {
            list.iter()
                .map(|a| {
                    value!({
                        "awemeId": a.get("aweme_id").and_then(|v| v.as_str()).unwrap_or(""),
                        "desc": a.get("desc").and_then(|v| v.as_str()).unwrap_or(""),
                        "createTime": a.get("create_time").and_then(|v| v.as_i64()).unwrap_or(0),
                        "diggCount": a.get("statistics").and_then(|s| s.get("digg_count")).and_then(|v| v.as_i64()).unwrap_or(0),
                        "commentCount": a.get("statistics").and_then(|s| s.get("comment_count")).and_then(|v| v.as_i64()).unwrap_or(0),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(value!({
        "hasMore": body.get("has_more").and_then(|v| v.as_i64()).unwrap_or(0) == 1,
        "cursor": body.get("max_cursor").and_then(|v| v.as_i64()).unwrap_or(0),
        "count": posts.len(),
        "posts": posts,
    }))
}
