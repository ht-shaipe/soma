//! SM3 密码哈希算法
//!
//! 中国国密标准 SM3，用于抖音 a_bogus 签名。

const IV: [u32; 8] = [
    0x7380166f, 0x49142869, 0x6445e8c3, 0x77e5f0ff,
    0x6e6e6e6e, 0x8c8c8c8c, 0x9c9c9c9c, 0xbcdef0ff,
];

fn t(j: usize) -> u32 {
    if j < 16 { 0x79cc4519 } else { 0x7a879d8a }
}

fn ff(x: u32, y: u32, z: u32, j: usize) -> u32 {
    if j < 16 { x ^ y ^ z } else { (x & y) | (x & z) | (y & z) }
}

fn gg(x: u32, y: u32, z: u32, j: usize) -> u32 {
    if j < 16 { x ^ y ^ z } else { (x & y) | (!x & z) }
}

fn p0(x: u32) -> u32 {
    x ^ x.rotate_left(9) ^ x.rotate_left(17)
}

fn p1(x: u32) -> u32 {
    x ^ x.rotate_left(15) ^ x.rotate_left(23)
}

/// 计算 SM3 哈希，返回 32 字节摘要
pub fn sm3(data: &[u8]) -> [u8; 32] {
    let mut padded = data.to_vec();
    let bit_len = (data.len() as u64) * 8;

    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut v: [u32; 8] = IV;

    for chunk in padded.chunks(64) {
        let mut w: [u32; 68] = [0; 68];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..68 {
            w[i] = p1(w[i - 16] ^ w[i - 9] ^ w[i - 2].rotate_left(15))
                ^ w[i - 13]
                ^ w[i - 6];
        }

        let mut a = v[0];
        let mut b = v[1];
        let mut c = v[2];
        let mut d = v[3];
        let mut e = v[4];
        let mut f = v[5];
        let mut g = v[6];
        let mut h = v[7];

        for j in 0..64 {
            let ss1 = a
                .rotate_left(12)
                .wrapping_add(e)
                .wrapping_add(t(j).rotate_left(j as u32))
                .rotate_left(7);
            let ss2 = ss1 ^ a.rotate_left(12);
            let tt1 = ff(a, b, c, j)
                .wrapping_add(d)
                .wrapping_add(ss2)
                .wrapping_add(w[j + 4]);
            let tt2 = gg(e, f, g, j)
                .wrapping_add(h)
                .wrapping_add(ss1)
                .wrapping_add(w[j]);

            d = c;
            c = b.rotate_left(9);
            b = a;
            a = tt1;
            h = g;
            g = f.rotate_left(19);
            f = e;
            e = p0(tt2);
        }

        v[0] ^= a;
        v[1] ^= b;
        v[2] ^= c;
        v[3] ^= d;
        v[4] ^= e;
        v[5] ^= f;
        v[6] ^= g;
        v[7] ^= h;
    }

    let mut result = [0u8; 32];
    for (i, &val) in v.iter().enumerate() {
        result[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
    }
    result
}

/// 计算 SM3 哈希，返回十六进制字符串
pub fn sm3_hex(data: &[u8]) -> String {
    let hash = sm3(data);
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}
