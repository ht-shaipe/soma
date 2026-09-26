//! 通知推送模块
//!
//! 提供多渠道消息通知能力，支持：
//! - Bark：iOS 推送通知
//! - 钉钉：群机器人 Webhook
//! - Telegram：Bot 消息推送
//!
//! 所有渠道均为 HTTP API 调用，无需额外依赖。

use serde::{Deserialize, Serialize};

/// 通知渠道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyChannel {
    /// 渠道类型：bark / dingtalk / telegram
    pub channel: String,
    /// Bark: 服务器地址（如 "https://api.day.app/你的key"）
    /// 钉钉: Webhook URL
    /// Telegram: Bot API URL（如 "https://api.telegram.org/bot<TOKEN>"）
    pub webhook: String,
    /// Bark: 可选的加密 key
    /// 钉钉: 可选的加签 secret
    /// Telegram: Chat ID
    pub secret: Option<String>,
}

/// 通知消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyMessage {
    /// 消息标题
    pub title: String,
    /// 消息内容
    pub body: String,
    /// 可选的 URL（点击跳转）
    pub url: Option<String>,
    /// 可选的分组/标签
    pub group: Option<String>,
}

/// 通知发送结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyResult {
    pub success: bool,
    pub channel: String,
    pub message: String,
}

/// 发送通知到指定渠道
pub async fn send(channel: &NotifyChannel, msg: &NotifyMessage) -> NotifyResult {
    match channel.channel.to_lowercase().as_str() {
        "bark" => send_bark(channel, msg).await,
        "dingtalk" => send_dingtalk(channel, msg).await,
        "telegram" => send_telegram(channel, msg).await,
        other => NotifyResult {
            success: false,
            channel: other.to_string(),
            message: format!("不支持的通知渠道: {}", other),
        },
    }
}

/// 批量发送通知到多个渠道
pub async fn send_batch(channels: &[NotifyChannel], msg: &NotifyMessage) -> Vec<NotifyResult> {
    let mut results = Vec::new();
    for ch in channels {
        results.push(send(ch, msg).await);
    }
    results
}

/// Bark 推送
///
/// API: POST {webhook}/{title}/{body}
/// 或: POST {webhook} with JSON body
async fn send_bark(channel: &NotifyChannel, msg: &NotifyMessage) -> NotifyResult {
    let client = reqwest::Client::new();
    let base_url = channel.webhook.trim_end_matches('/');

    let mut payload = serde_json::json!({
        "title": msg.title,
        "body": msg.body,
    });
    if let Some(url) = &msg.url {
        payload["url"] = serde_json::Value::String(url.clone());
    }
    if let Some(group) = &msg.group {
        payload["group"] = serde_json::Value::String(group.clone());
    }
    if let Some(key) = &channel.secret {
        payload["key"] = serde_json::Value::String(key.clone());
    }

    match client.post(base_url).json(&payload).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                NotifyResult {
                    success: true,
                    channel: "bark".into(),
                    message: "推送成功".into(),
                }
            } else {
                NotifyResult {
                    success: false,
                    channel: "bark".into(),
                    message: format!("HTTP {}", status),
                }
            }
        }
        Err(e) => NotifyResult {
            success: false,
            channel: "bark".into(),
            message: format!("请求失败: {}", e),
        },
    }
}

/// 钉钉群机器人推送
///
/// API: POST {webhook} with JSON body
/// 支持文本消息和 Markdown 消息
async fn send_dingtalk(channel: &NotifyChannel, msg: &NotifyMessage) -> NotifyResult {
    let client = reqwest::Client::new();

    let payload = if msg.body.contains("**") || msg.body.contains("##") {
        serde_json::json!({
            "msgtype": "markdown",
            "markdown": {
                "title": msg.title,
                "text": format!("## {}\n\n{}", msg.title, msg.body),
            }
        })
    } else {
        serde_json::json!({
            "msgtype": "text",
            "text": {
                "content": format!("{}\n{}", msg.title, msg.body),
            }
        })
    };

    let url = if let Some(secret) = &channel.secret {
        sign_dingtalk_url(&channel.webhook, secret)
    } else {
        channel.webhook.clone()
    };

    match client.post(&url).json(&payload).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let errcode = body.get("errcode").and_then(|v| v.as_i64()).unwrap_or(0);
                if errcode == 0 {
                    NotifyResult {
                        success: true,
                        channel: "dingtalk".into(),
                        message: "推送成功".into(),
                    }
                } else {
                    let errmsg = body
                        .get("errmsg")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误");
                    NotifyResult {
                        success: false,
                        channel: "dingtalk".into(),
                        message: format!("钉钉错误: {} {}", errcode, errmsg),
                    }
                }
            } else {
                NotifyResult {
                    success: false,
                    channel: "dingtalk".into(),
                    message: format!("HTTP {}", status),
                }
            }
        }
        Err(e) => NotifyResult {
            success: false,
            channel: "dingtalk".into(),
            message: format!("请求失败: {}", e),
        },
    }
}

/// Telegram Bot 推送
///
/// API: POST {webhook}/sendMessage
async fn send_telegram(channel: &NotifyChannel, msg: &NotifyMessage) -> NotifyResult {
    let client = reqwest::Client::new();
    let chat_id = channel.secret.as_deref().unwrap_or("");
    if chat_id.is_empty() {
        return NotifyResult {
            success: false,
            channel: "telegram".into(),
            message: "缺少 Chat ID（请配置 secret 字段）".into(),
        };
    }

    let url = format!("{}/sendMessage", channel.webhook.trim_end_matches('/'));
    let text = format!("{}\n\n{}", msg.title, msg.body);

    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "HTML",
    });

    match client.post(&url).json(&payload).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let ok = body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                if ok {
                    NotifyResult {
                        success: true,
                        channel: "telegram".into(),
                        message: "推送成功".into(),
                    }
                } else {
                    let description = body
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误");
                    NotifyResult {
                        success: false,
                        channel: "telegram".into(),
                        message: format!("Telegram 错误: {}", description),
                    }
                }
            } else {
                NotifyResult {
                    success: false,
                    channel: "telegram".into(),
                    message: format!("HTTP {}", status),
                }
            }
        }
        Err(e) => NotifyResult {
            success: false,
            channel: "telegram".into(),
            message: format!("请求失败: {}", e),
        },
    }
}

/// 钉钉 Webhook 加签
fn sign_dingtalk_url(webhook: &str, secret: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let string_to_sign = format!("{}\n{}", timestamp, secret);
    let hmac = hmac_sha256(secret.as_bytes(), string_to_sign.as_bytes());
    let sign = base64_url_encode(&hmac);

    format!("{}&timestamp={}&sign={}", webhook, timestamp, sign)
}

/// HMAC-SHA256
fn hmac_sha256(key: &[u8], message: &[u8]) -> Vec<u8> {
    const BLOCK_SIZE: usize = 64;

    let key = if key.len() > BLOCK_SIZE {
        sha256(key).to_vec()
    } else {
        key.to_vec()
    };

    let mut padded_key = key.clone();
    padded_key.resize(BLOCK_SIZE, 0);

    let mut o_key_pad = vec![0u8; BLOCK_SIZE];
    let mut i_key_pad = vec![0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        o_key_pad[i] = padded_key[i] ^ 0x5c;
        i_key_pad[i] = padded_key[i] ^ 0x36;
    }

    let mut inner = i_key_pad;
    inner.extend_from_slice(message);
    let inner_hash = sha256(&inner);

    let mut outer = o_key_pad;
    outer.extend_from_slice(&inner_hash);
    sha256(&outer).to_vec()
}

/// SHA-256
fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

/// Base64 URL 编码
fn base64_url_encode(data: &[u8]) -> String {
    let b64 = base64_encode(data);
    b64.replace('+', "-").replace('/', "_").replace('=', "")
}

/// 简易 Base64 编码
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        result.push(CHARS[(b0 >> 2) as usize] as char);
        result.push(CHARS[((b0 << 4) | (b1 >> 4)) as usize & 0x3f] as char);
        if i + 1 < data.len() {
            result.push(CHARS[((b1 << 2) | (b2 >> 6)) as usize & 0x3f] as char);
        } else {
            result.push('=');
        }
        if i + 2 < data.len() {
            result.push(CHARS[b2 as usize & 0x3f] as char);
        } else {
            result.push('=');
        }
        i += 3;
    }
    result
}

// ── SHA-256 实现 ──

struct Sha256 {
    state: [u32; 8],
    buffer: Vec<u8>,
    length: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buffer: Vec::new(),
            length: 0,
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.length += data.len() as u64 * 8;
        self.buffer.extend_from_slice(data);
        while self.buffer.len() >= 64 {
            let block: [u8; 64] = self.buffer[..64].try_into().unwrap();
            self.process_block(&block);
            self.buffer.drain(..64);
        }
    }

    fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.length;
        self.buffer.push(0x80);
        while self.buffer.len() % 64 != 56 {
            self.buffer.push(0);
        }
        self.buffer.extend_from_slice(&bit_len.to_be_bytes());

        while !self.buffer.is_empty() {
            let block: [u8; 64] = self.buffer[..64].try_into().unwrap();
            self.process_block(&block);
            self.buffer.drain(..64);
        }

        let mut result = [0u8; 32];
        for (i, &s) in self.state.iter().enumerate() {
            result[i * 4..i * 4 + 4].copy_from_slice(&s.to_be_bytes());
        }
        result
    }

    fn process_block(&mut self, block: &[u8; 64]) {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];

        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16] + s0 + w[i - 7] + s1;
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}
