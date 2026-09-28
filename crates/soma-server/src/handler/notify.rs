//! 通知推送 API 处理器
//!
//! 支持 Bark / 钉钉 / Telegram 多渠道消息通知。

use soma_core::notify::{self, NotifyChannel, NotifyMessage};
use tube::{Result, Value};
use tube_web::RequestParameter;

/// 通知推送模块分发入口：`notify.send`（Bark / 钉钉 / Telegram）
pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "send" => send(param).await,
        "send_batch" => send_batch(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 发送通知到单个渠道
async fn send(param: &RequestParameter) -> Result<Value> {
    let channel = param.value.get_def_string("channel", "");
    let webhook = param.value.get_def_string("webhook", "");
    let title = param.value.get_def_string("title", "");
    let body = param.value.get_def_string("body", "");

    if channel.is_empty() || webhook.is_empty() {
        return Err(error!("缺少 channel 或 webhook 参数"));
    }

    let ch = NotifyChannel {
        channel: channel.to_string(),
        webhook: webhook.to_string(),
        secret: param.value.get("secret").and_then(|v| v.as_str()),
    };

    let msg = NotifyMessage {
        title: title.to_string(),
        body: body.to_string(),
        url: param.value.get("url").and_then(|v| v.as_str()),
        group: param.value.get("group").and_then(|v| v.as_str()),
    };

    let result = notify::send(&ch, &msg).await;

    Ok(value!({
        "success": result.success,
        "channel": result.channel,
        "message": result.message,
    }))
}

/// 批量发送通知到多个渠道
async fn send_batch(param: &RequestParameter) -> Result<Value> {
    let title = param.value.get_def_string("title", "");
    let body = param.value.get_def_string("body", "");

    let channels_val = param.value.get("channels").and_then(|v| v.as_array());
    let channels: Vec<NotifyChannel> = if let Some(arr) = channels_val {
        arr.iter()
            .filter_map(|v| {
                let channel = v.get("channel").and_then(|v| v.as_str())?;
                let webhook = v.get("webhook").and_then(|v| v.as_str())?;
                Some(NotifyChannel {
                    channel: channel.to_string(),
                    webhook: webhook.to_string(),
                    secret: v.get("secret").and_then(|v| v.as_str()),
                })
            })
            .collect()
    } else {
        return Err(error!("缺少 channels 参数"));
    };

    let msg = NotifyMessage {
        title: title.to_string(),
        body: body.to_string(),
        url: param.value.get("url").and_then(|v| v.as_str()),
        group: param.value.get("group").and_then(|v| v.as_str()),
    };

    let results = notify::send_batch(&channels, &msg).await;
    let results_val: Vec<Value> = results
        .iter()
        .map(|r| {
            value!({
                "success": r.success,
                "channel": r.channel.clone(),
                "message": r.message.clone(),
            })
        })
        .collect();

    Ok(value!({
        "results": results_val,
    }))
}
