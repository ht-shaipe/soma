//! 抖音 Web API 直连客户端
//!
//! 基于 a_bogus / msToken 签名算法直接调用抖音 Web API（不依赖浏览器）。
//! 当前支持：
//! - 视频详情：`/aweme/v1/web/aweme/detail/`
//! - 用户作品列表：`/aweme/v1/web/aweme/post/`
//!
//! 注意：抖音接口需要有效 Cookie（至少包含 ttwid），未携带时接口返回空数据。
//! Cookie 可从浏览器登录后复制，经 `DouyinClient::new(cookie)` 传入。

use soma_core::error::SomaError;
use soma_core::signing::douyin::{generate_abogus, generate_mstoken};
use std::time::{SystemTime, UNIX_EPOCH};

/// 抖音 Web API 基础地址
const BASE_URL: &str = "https://www.douyin.com";

/// 通用浏览器 UA（需与 a_bogus 签名时使用的 UA 一致）
const DEFAULT_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";

/// 抖音 Web API 客户端
pub struct DouyinClient {
    client: reqwest::Client,
    /// 登录 Cookie（须包含 ttwid 等字段）
    cookie: Option<String>,
    /// User-Agent
    ua: String,
}

impl DouyinClient {
    /// 创建客户端
    ///
    /// # 参数
    /// - `cookie`: 抖音网页版 Cookie，建议从浏览器复制完整值
    /// - `proxy`: 可选代理（如 `http://127.0.0.1:7890`）
    pub fn new(cookie: Option<String>, proxy: Option<String>) -> Result<Self, SomaError> {
        let mut builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(DEFAULT_UA);

        if let Some(p) = proxy {
            builder = builder.proxy(
                reqwest::Proxy::all(&p)
                    .map_err(|e| SomaError::Stock(format!("代理配置无效: {}", e)))?,
            );
        }

        Ok(Self {
            client: builder
                .build()
                .map_err(|e| SomaError::Stock(format!("HTTP 客户端创建失败: {}", e)))?,
            cookie,
            ua: DEFAULT_UA.to_string(),
        })
    }

    /// 获取视频详情
    ///
    /// # 参数
    /// - `aweme_id`: 视频 ID（可从分享链接/页面 URL 中提取）
    pub async fn video_detail(&self, aweme_id: &str) -> Result<serde_json::Value, SomaError> {
        let params = format!("aweme_id={}&device_platform=webapp&aid=6383&channel=channel_pc_web&pc_client_type=1&version_code=170400&version_name=17.4.0&cookie_enabled=true&platform=PC&downlink=10", aweme_id);

        let body = self
            .signed_get("/aweme/v1/web/aweme/detail/", &params)
            .await?;
        let detail = body.get("aweme_detail").cloned().ok_or_else(|| {
            SomaError::Stock("接口未返回视频详情（Cookie 失效或视频不存在）".to_string())
        })?;
        Ok(detail)
    }

    /// 获取用户作品列表
    ///
    /// # 参数
    /// - `sec_user_id`: 用户 sec_uid
    /// - `count`: 每页数量（1~20）
    /// - `max_cursor`: 翻页游标，首页传 0
    pub async fn user_posts(
        &self,
        sec_user_id: &str,
        count: u32,
        max_cursor: u64,
    ) -> Result<serde_json::Value, SomaError> {
        let count = count.clamp(1, 20);
        let params = format!(
            "sec_user_id={}&count={}&max_cursor={}&locate_query=false&show_live_replay_strategy=1&need_time_list=1&time_list_query=0&device_platform=webapp&aid=6383&channel=channel_pc_web&pc_client_type=1&version_code=170400&version_name=17.4.0&cookie_enabled=true&platform=PC&downlink=10",
            sec_user_id, count, max_cursor
        );

        let body = self
            .signed_get("/aweme/v1/web/aweme/post/", &params)
            .await?;
        if body.get("aweme_list").is_none() {
            return Err(SomaError::Stock(
                "接口未返回作品列表（Cookie 失效或用户不存在）".to_string(),
            ));
        }
        Ok(body)
    }

    /// 发起带签名的 GET 请求
    async fn signed_get(&self, path: &str, params: &str) -> Result<serde_json::Value, SomaError> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| SomaError::Stock(e.to_string()))?
            .as_millis() as u64;
        let a_bogus = generate_abogus(params, &self.ua, ts);
        let ms_token = generate_mstoken();

        let url = format!(
            "{}{}?{}&a_bogus={}&msToken={}",
            BASE_URL, path, params, a_bogus, ms_token
        );

        let mut req = self
            .client
            .get(&url)
            .header("Referer", "https://www.douyin.com/")
            .header("Accept", "application/json, text/plain, */*");

        if let Some(cookie) = &self.cookie {
            req = req.header("Cookie", cookie);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| SomaError::Http(format!("抖音接口请求失败: {}", e)))?;

        if !resp.status().is_success() {
            return Err(SomaError::Stock(format!(
                "抖音接口返回 HTTP {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| SomaError::Http(format!("抖音响应解析失败: {}", e)))
    }
}

/// 从抖音分享文本 / 页面 URL 中提取视频 ID
///
/// 支持格式：
/// - 短链（v.douyin.com）需先解析重定向，此处只处理直链
/// - `https://www.douyin.com/video/7xxxxxxxxxxxxxxxxxx`
/// - `https://www.iesdouyin.com/share/video/7xxxxxxxxxxxxxxxxxx`
pub fn extract_aweme_id(input: &str) -> Option<String> {
    let input = input.trim();

    // 纯数字 ID 直接返回
    if input.chars().all(|c| c.is_ascii_digit()) && input.len() >= 15 {
        return Some(input.to_string());
    }

    for marker in ["/video/", "share/video/", "modal_id="] {
        if let Some(pos) = input.find(marker) {
            let rest = &input[pos + marker.len()..];
            let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if id.len() >= 15 {
                return Some(id);
            }
        }
    }

    None
}

/// 从抖音用户主页 URL 中提取 sec_uid
///
/// 支持格式：`https://www.douyin.com/user/MS4wLjABAAAA...`
pub fn extract_sec_uid(input: &str) -> Option<String> {
    let input = input.trim();

    // 裸 sec_uid 直接返回
    if input.starts_with("MS4wLjABAAAA") {
        let uid: String = input
            .chars()
            .take_while(|c| *c != '?' && *c != '/')
            .collect();
        if !uid.is_empty() {
            return Some(uid);
        }
    }

    if let Some(pos) = input.find("/user/") {
        let rest = &input[pos + "/user/".len()..];
        // sec_uid 以 "MS4wLjABAAAA" 开头
        if rest.starts_with("MS4wLjABAAAA") {
            let uid: String = rest
                .chars()
                .take_while(|c| *c != '?' && *c != '/')
                .collect();
            if !uid.is_empty() {
                return Some(uid);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_aweme_id() {
        assert_eq!(
            extract_aweme_id("https://www.douyin.com/video/7341234567890123456"),
            Some("7341234567890123456".to_string())
        );
        assert_eq!(
            extract_aweme_id("https://www.iesdouyin.com/share/video/7341234567890123456/?mid=1"),
            Some("7341234567890123456".to_string())
        );
        assert_eq!(
            extract_aweme_id("7341234567890123456"),
            Some("7341234567890123456".to_string())
        );
        assert_eq!(extract_aweme_id("https://www.douyin.com/discover"), None);
    }

    #[test]
    fn test_extract_sec_uid() {
        assert_eq!(
            extract_sec_uid("https://www.douyin.com/user/MS4wLjABAAAAabcdefg123"),
            Some("MS4wLjABAAAAabcdefg123".to_string())
        );
        assert_eq!(
            extract_sec_uid("https://www.douyin.com/user/MS4wLjABAAAAabc?from=info"),
            Some("MS4wLjABAAAAabc".to_string())
        );
        assert_eq!(
            extract_sec_uid("MS4wLjABAAAAxyz"),
            Some("MS4wLjABAAAAxyz".to_string())
        );
        assert_eq!(extract_sec_uid("https://www.douyin.com/video/123"), None);
    }
}
