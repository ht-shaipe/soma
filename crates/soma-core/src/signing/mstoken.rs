//! 抖音 msToken 生成
//!
//! 生成抖音 Web API 的 msToken 参数。

use rand::Rng;

/// 生成 msToken
///
/// msToken 是一个 128 位的随机十六进制字符串（32 字符），
/// 用于抖音 Web API 请求的额外验证。
pub fn generate_mstoken() -> String {
    let mut rng = rand::rng();
    (0..32)
        .map(|_| {
            let n: u8 = rng.random_range(0..16);
            if n < 10 {
                (b'0' + n) as char
            } else {
                (b'a' + n - 10) as char
            }
        })
        .collect()
}
