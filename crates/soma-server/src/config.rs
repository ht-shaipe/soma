/// 全局配置模块
///
/// 提供配置的加载（从 TOML 文件）、缓存（通过全局 Mutex HashMap）和读取功能。
/// 使用 lazy_static 实现全局配置缓存，避免反复读取文件。

use soma_core::config::AppConfig;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    /// 全局配置缓存，以名称为键存储 Config 实例，使用 Mutex 保证线程安全
    pub static ref CONFIG_CACHE: Mutex<HashMap<String, Config>> = Mutex::new(HashMap::new());
    /// 配置文件路径缓存，用于保存时写回
    pub static ref CONF_PATH_CACHE: Mutex<String> = Mutex::new(String::new());
}

/// 服务端配置结构体
///
/// 封装了应用层配置（AppConfig），包含 LLM、TTS、视频处理等各模块的配置项
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// 应用层配置，包含所有业务相关配置（LLM、TTS、FFmpeg、存储等）
    pub app: AppConfig,
}

impl Config {
    /// 从 TOML 配置文件加载配置
    ///
    /// 参数：
    /// - `conf_path`: 配置文件的绝对路径
    ///
    /// 返回：加载成功返回 Config 实例，失败返回 SomaError
    pub fn load(conf_path: &str) -> std::result::Result<Config, soma_core::SomaError> {
        // 缓存配置文件路径
        if let Ok(mut cache) = CONF_PATH_CACHE.lock() {
            *cache = conf_path.to_string();
        }
        let app_config = AppConfig::load_toml(conf_path)?;
        Ok(Config { app: app_config })
    }

    /// 将配置写入全局缓存
    ///
    /// 以 "soma" 为键存储，供后续 get() 调用读取
    pub fn set(val: Config) {
        CONFIG_CACHE.lock().unwrap().insert("soma".to_owned(), val);
    }

    /// 从全局缓存读取配置
    ///
    /// 返回克隆的 Config 实例，若缓存中不存在则返回默认值
    pub fn get() -> Config {
        CONFIG_CACHE.lock().unwrap().get("soma").cloned().unwrap_or_default()
    }

    /// 获取当前配置文件路径
    pub fn get_conf_path() -> String {
        CONF_PATH_CACHE.lock().unwrap().clone()
    }
}
