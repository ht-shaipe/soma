//! 抖音 a_bogus 签名算法
//!
//! 生成抖音 Web API 的 a_bogus 签名参数。
//! 基于 SM3 哈希 + RC4 加密 + UA/时间戳/参数序列化。

use super::sm3::sm3_hex;
use super::rc4::rc4;

/// 生成 a_bogus 签名
///
/// # 参数
/// - `params`: 请求参数字符串（URL query string）
/// - `user_agent`: User-Agent 字符串
/// - `timestamp`: 时间戳（毫秒）
///
/// # 返回
/// a_bogus 签名字符串
pub fn generate_abogus(params: &str, user_agent: &str, timestamp: u64) -> String {
    // 1. 构造待签名数据
    let mut data = String::new();
    data.push_str(params);
    data.push_str(&timestamp.to_string());
    data.push_str(user_agent);

    // 2. SM3 哈希
    let hash = sm3_hex(data.as_bytes());

    // 3. RC4 加密
    let key = format!("{}_{}", timestamp, user_agent.len());
    let encrypted = rc4(key.as_bytes(), hash.as_bytes());

    // 4. Base64 编码并替换字符
    let b64 = base64_encode(&encrypted);
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
