//! B站 wbi 签名算法
//!
//! 生成 B站 Web API 的 wbi 签名参数。

/// wbi 签名
///
/// # 参数
/// - `params`: 原始请求参数（已排序的 key=value&key=value 格式）
/// - `img_key`: wbi 图片 key（从导航 API 获取）
/// - `sub_key`: wbi 子 key
/// - `ts`: 时间戳（秒）
///
/// # 返回
/// 签名后的完整参数字符串（包含 w_rid 和 wts）
pub fn wbi_sign(params: &str, img_key: &str, sub_key: &str, ts: u64) -> String {
    let mixin_key = get_mixin_key(img_key, sub_key);

    let mut full = String::from(params);
    if !full.is_empty() && !full.ends_with('&') {
        full.push('&');
    }
    full.push_str(&format!("wts={}", ts));

    let w_rid = md5_hex(format!("{}{}", full, mixin_key).as_bytes());

    format!("{}w_rid={}", full, w_rid)
}

/// 获取 mixin key
///
/// 将 img_key + sub_key 拼接后按特定索引表重排
fn get_mixin_key(img_key: &str, sub_key: &str) -> String {
    const MIXIN_KEY_ENC_TAB: [usize; 64] = [
        46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 0, 5, 60, 43, 36, 40, 38, 30, 41, 12, 39,
        37, 45, 24, 44, 9, 25, 48, 51, 55, 7, 16, 56, 14, 27, 20, 29, 42, 1, 35, 57, 11, 4, 33, 49,
        17, 3, 21, 34, 52, 26, 13, 6, 28, 59, 22, 54, 19, 61, 62, 63,
    ];

    let raw: Vec<char> = format!("{}{}", img_key, sub_key).chars().collect();
    MIXIN_KEY_ENC_TAB
        .iter()
        .filter_map(|&i| raw.get(i))
        .take(32)
        .collect()
}

/// MD5 哈希（十六进制）
fn md5_hex(data: &[u8]) -> String {
    let hash = md5(data);
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// MD5 实现
fn md5(data: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];

    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8f1c0cdc, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    let mut msg = data.to_vec();
    let orig_len = data.len() as u64 * 8;

    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&orig_len.to_le_bytes());

    let mut a0: u32 = 0x67452301;
    let mut b0: u32 = 0xefcdab89;
    let mut c0: u32 = 0x98badcfe;
    let mut d0: u32 = 0x10325476;

    for chunk in msg.chunks(64) {
        let mut m = [0u32; 16];
        for i in 0..16 {
            m[i] = u32::from_le_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let (f, g) = if i < 16 {
                ((b & c) | (!b & d), i)
            } else if i < 32 {
                ((d & b) | (!d & c), (5 * i + 1) % 16)
            } else if i < 48 {
                (b ^ c ^ d, (3 * i + 5) % 16)
            } else {
                (c ^ (b | !d), (7 * i) % 16)
            };

            let temp = d;
            d = c;
            c = b;
            b = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(m[g])
                    .rotate_left(S[i]),
            );
            a = temp;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut result = [0u8; 16];
    result[0..4].copy_from_slice(&a0.to_le_bytes());
    result[4..8].copy_from_slice(&b0.to_le_bytes());
    result[8..12].copy_from_slice(&c0.to_le_bytes());
    result[12..16].copy_from_slice(&d0.to_le_bytes());
    result
}
