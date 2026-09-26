//! 敏感词过滤模块
//!
//! 提供敏感词库加载与文本命中检查能力，用于在创建口播任务时
//! 对文案进行敏感词校验，命中则拒绝创建。
//!
//! 词库格式：UTF-8 文本文件，每行一个词，`#` 开头为注释行，空行忽略。

use std::sync::OnceLock;

use crate::error::SomaError;

/// 敏感词检查结果
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckResult {
    /// 是否命中敏感词
    pub hit: bool,
    /// 命中的敏感词列表
    pub words: Vec<String>,
}

impl CheckResult {
    /// 创建未命中的结果
    pub fn not_hit() -> Self {
        Self {
            hit: false,
            words: vec![],
        }
    }
}

/// 敏感词过滤器
///
/// 加载敏感词库后可对文本进行命中匹配。首期采用逐词子串匹配，
/// 预留正则扩展点。
#[derive(Debug, Clone, Default)]
pub struct SensitiveWordFilter {
    /// 敏感词列表（已去除注释与空行）
    words: Vec<String>,
}

impl SensitiveWordFilter {
    /// 创建空的敏感词过滤器
    pub fn empty() -> Self {
        Self { words: vec![] }
    }

    /// 从文本文件加载敏感词库
    ///
    /// 文件不存在时返回空词库（不阻塞任务创建，按未命中处理）。
    /// 每行一个词，忽略空行与 `#` 开头的注释行。
    ///
    /// # 参数
    /// - `path`: 敏感词库文件路径
    pub fn load(path: &str) -> Result<Self, SomaError> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(Self::empty()),
        };

        let mut words = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            words.push(trimmed.to_string());
        }

        Ok(Self { words })
    }

    /// 对文本进行敏感词命中检查
    ///
    /// 首期采用逐词子串匹配，返回命中的敏感词列表。
    ///
    /// # 参数
    /// - `text`: 待检查文本
    pub fn check(&self, text: &str) -> CheckResult {
        if text.is_empty() || self.words.is_empty() {
            return CheckResult::not_hit();
        }

        let mut hit_words = Vec::new();
        for word in &self.words {
            if text.contains(word.as_str()) {
                hit_words.push(word.clone());
            }
        }

        if hit_words.is_empty() {
            CheckResult::not_hit()
        } else {
            CheckResult {
                hit: true,
                words: hit_words,
            }
        }
    }

    /// 返回词库大小
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// 词库是否为空
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

/// 全局缓存的敏感词过滤器（首次加载后复用，避免重复读盘）
static GLOBAL_FILTER: OnceLock<SensitiveWordFilter> = OnceLock::new();

/// 获取全局缓存的敏感词过滤器
///
/// 首次调用时从指定路径加载并缓存，后续调用直接复用。
/// 加载失败时缓存空过滤器。
pub fn global_filter(path: &str) -> &'static SensitiveWordFilter {
    GLOBAL_FILTER.get_or_init(|| {
        SensitiveWordFilter::load(path).unwrap_or_else(|_| SensitiveWordFilter::empty())
    })
}

/// 对文本进行敏感词检查（使用全局缓存的过滤器）
///
/// 便捷方法，首次调用时从 `path` 加载词库并缓存。
pub fn check_text(text: &str, path: &str) -> CheckResult {
    global_filter(path).check(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_words(content: &str) -> String {
        let path = format!("/tmp/soma_test_sw_{}.txt", uuid::Uuid::new_v4());
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_empty_filter() {
        let filter = SensitiveWordFilter::empty();
        let result = filter.check("任意文本");
        assert!(!result.hit);
        assert!(result.words.is_empty());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let filter = SensitiveWordFilter::load("/nonexistent/path/words.txt").unwrap();
        assert!(filter.is_empty());
        let result = filter.check("任意文本");
        assert!(!result.hit);
    }

    #[test]
    fn test_hit_and_miss() {
        let path = write_temp_words("敏感词A\n敏感词B\n# 注释\n\n");
        let filter = SensitiveWordFilter::load(&path).unwrap();
        assert_eq!(filter.len(), 2);

        let result = filter.check("包含敏感词A的文案");
        assert!(result.hit);
        assert_eq!(result.words, vec!["敏感词A".to_string()]);

        let result = filter.check("干净文案");
        assert!(!result.hit);

        let result = filter.check("");
        assert!(!result.hit);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_multiple_hits() {
        let path = write_temp_words("词A\n词B\n词C\n");
        let filter = SensitiveWordFilter::load(&path).unwrap();

        let result = filter.check("包含词A和词C的文案");
        assert!(result.hit);
        assert_eq!(result.words.len(), 2);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_comment_lines_ignored() {
        let path = write_temp_words("# 这是注释\n#另一个注释\n实际词\n");
        let filter = SensitiveWordFilter::load(&path).unwrap();
        assert_eq!(filter.len(), 1);

        std::fs::remove_file(&path).ok();
    }
}
