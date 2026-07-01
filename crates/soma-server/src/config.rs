/// 全局配置模块
///
/// 提供配置的加载（从 TOML 文件）、缓存（通过全局 Mutex HashMap）和读取功能。
/// 使用 lazy_static 实现全局配置缓存，避免反复读取文件。

use soma_core::config::AppConfig;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    pub static ref CONFIG_CACHE: Mutex<HashMap<String, Config>> = Mutex::new(HashMap::new());
    pub static ref CONF_PATH_CACHE: Mutex<String> = Mutex::new(String::new());
}

fn lock_config_cache() -> std::sync::MutexGuard<'static, HashMap<String, Config>> {
    CONFIG_CACHE.lock().unwrap_or_else(|e| {
        log::error!("CONFIG_CACHE Mutex 中毒，强制恢复: {}", e);
        e.into_inner()
    })
}

fn lock_conf_path() -> std::sync::MutexGuard<'static, String> {
    CONF_PATH_CACHE.lock().unwrap_or_else(|e| {
        log::error!("CONF_PATH_CACHE Mutex 中毒，强制恢复: {}", e);
        e.into_inner()
    })
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub app: AppConfig,
}

impl Config {
    pub fn load(conf_path: &str) -> std::result::Result<Config, soma_core::SomaError> {
        if let Ok(mut cache) = CONF_PATH_CACHE.lock() {
            *cache = conf_path.to_string();
        }
        let app_config = AppConfig::load_toml(conf_path)?;
        Ok(Config { app: app_config })
    }

    pub fn set(val: Config) {
        lock_config_cache().insert("soma".to_owned(), val);
    }

    pub fn get() -> Config {
        lock_config_cache().get("soma").cloned().unwrap_or_default()
    }

    pub fn get_conf_path() -> String {
        lock_conf_path().clone()
    }

    pub fn save() -> std::result::Result<(), String> {
        let conf = Self::get();
        let conf_path = Self::get_conf_path();
        if conf_path.is_empty() {
            return Err("配置文件路径未设置".to_string());
        }
        let toml_str = toml::to_string_pretty(&conf.app).map_err(|e| format!("序列化失败: {}", e))?;
        std::fs::write(&conf_path, toml_str).map_err(|e| format!("写入失败: {}", e))
    }
}
