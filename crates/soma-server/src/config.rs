use soma_core::config::AppConfig;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    pub static ref CONFIG_CACHE: Mutex<HashMap<String, Config>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub app: AppConfig,
}

impl Config {
    pub fn load(conf_path: &str) -> std::result::Result<Config, soma_core::SomaError> {
        let app_config = AppConfig::load_toml(conf_path)?;
        Ok(Config { app: app_config })
    }

    pub fn set(val: Config) {
        CONFIG_CACHE.lock().unwrap().insert("soma".to_owned(), val);
    }

    pub fn get() -> Config {
        CONFIG_CACHE.lock().unwrap().get("soma").cloned().unwrap_or_default()
    }
}
