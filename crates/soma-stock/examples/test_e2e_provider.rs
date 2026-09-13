//! 素材供应商端到端示例：对指定来源执行一次真实搜索并打印结果。
//!
//! 用法：`cargo run -p soma-stock --example test_e2e_provider`（需已配置对应密钥）。

use soma_core::config::AppConfig;

fn main() {
    let config = AppConfig::load_toml("conf/config.toml").unwrap();
    let dh = &config.digital_human;
    println!("provider: {}", dh.get_provider());
    
    let emv3 = &dh.echomimic_v3;
    println!("script_path: {}", emv3.get_script_path());
    println!("model_path: {}", emv3.get_model_path());
    println!("device: {}", emv3.get_device());
    println!("resolution: {}", emv3.get_resolution());
    println!("infer_steps: {}", emv3.get_infer_steps());
    println!("timeout: {}", emv3.get_timeout());
    println!("preflight_check: {}", emv3.get_preflight_check());
    
    let _provider = soma_stock::digital_human::create_provider(dh).unwrap();
    println!("Provider created successfully");
}
