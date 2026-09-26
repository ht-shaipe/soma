//! Soma 桌面应用（Tauri v2 宿主）
//!
//! 0.1.2 M2.2/M2.3：桌面模式直接复用 soma-server 业务层（handler/service/state），
//! 前端经 [`api`] 通用命令调用与 HTTP 完全同构的接口（{code, result, message} 信封），
//! Vue 前端零改动运行在浏览器（HTTP）与桌面（invoke）两种环境。
//!
//! 0.1.2 M2.4：配置迁移至系统配置目录（app_config_dir），存储迁移至系统数据目录
//! （app_data_dir/storage）；首次启动自动迁移项目内既有配置或初始化默认配置，
//! 桌面 App 在全新机器上开箱可用。服务器模式（soma-server）不受影响，
//! 仍使用项目相对的 conf/ 与 storage/。

// 引入 tube 宏（error! 等）
#[macro_use]
extern crate tube;

// error! 宏展开时引用 crate::Error，需在 crate 根引入
use tube::Error;
use tube_web::RequestParameter;

/// 项目根目录解析（开发态可执行文件位于 {root}/target/debug/，向上三级即根目录）
///
/// 可用环境变量 SOMA_ROOT 覆盖。仅项目模式（测试）使用；
/// 桌面运行时一律走系统目录（M2.4），严格版解析见 [`resolve_project_root_checked`]。
#[cfg(test)]
fn resolve_project_root() -> std::path::PathBuf {
    if let Ok(root) = std::env::var("SOMA_ROOT") {
        return root.into();
    }
    let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
    exe.ancestors()
        .nth(3)
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

/// 严格版项目根解析：要求根目录存在 conf/ 或 resource/ 标志。
///
/// 全新机器（打包运行）返回 None，调用方按"无项目环境"处理，
/// 相对路径资源（字体/歌曲/推理脚本）交由 M2.5 预检引导与 M3.4 资源打包解决。
fn resolve_project_root_checked() -> Option<std::path::PathBuf> {
    if let Ok(root) = std::env::var("SOMA_ROOT") {
        let p = std::path::PathBuf::from(root);
        return (p.join("conf").exists() || p.join("resource").exists()).then_some(p);
    }
    let exe = std::env::current_exe().ok()?;
    let root = exe.ancestors().nth(3)?.to_path_buf();
    (root.join("conf").exists() || root.join("resource").exists()).then_some(root)
}

/// 首启配置初始化/迁移（M2.4）：系统配置目录没有 config.toml 时依次尝试
///
/// ① 迁移项目内既有 `conf/config.toml`（开发机升级路径，保留用户密钥等全部配置）
/// ② 复制项目内 `conf/config.toml.example` 模板
/// ③ 写出出厂默认配置（AppConfig::default() 序列化）
fn ensure_desktop_config(config_dir: &std::path::Path) -> std::path::PathBuf {
    let _ = std::fs::create_dir_all(config_dir);
    let conf_path = config_dir.join("config.toml");
    if conf_path.exists() {
        return conf_path;
    }
    let root = resolve_project_root_checked();
    let legacy = root.as_ref().map(|r| r.join("conf/config.toml"));
    let example = root.as_ref().map(|r| r.join("conf/config.toml.example"));
    for candidate in [legacy, example].into_iter().flatten() {
        if candidate.exists() {
            match std::fs::copy(&candidate, &conf_path) {
                Ok(_) => {
                    eprintln!(
                        "[soma] 首启迁移配置 {} → {}",
                        candidate.display(),
                        conf_path.display()
                    );
                    return conf_path;
                }
                Err(e) => eprintln!("[soma] 迁移配置失败 {candidate:?}: {e}"),
            }
        }
    }
    let default_toml = toml::to_string_pretty(&soma_core::config::AppConfig::default())
        .unwrap_or_else(|_| "[app]\n".to_string());
    if let Err(e) = std::fs::write(&conf_path, default_toml) {
        eprintln!("[soma] 写出默认配置失败 {conf_path:?}: {e}");
    } else {
        eprintln!("[soma] 首启初始化默认配置 {}", conf_path.display());
    }
    conf_path
}

/// 后端初始化公共序列（配置 → 代理 → 队列 → 存储目录 → SQLite）
///
/// 与服务端唯一区别：不启动 HTTP 监听。`storage_root` 为 Some 时强制
/// `conf.app.storage_path` 指向该目录（桌面系统目录模式），并同步注入
/// soma-core 路径覆盖层；为 None 时保持项目相对语义（服务器/测试模式）。
fn init_backend_common(conf_path: &str, storage_root: Option<std::path::PathBuf>) {
    soma_core::utils::set_storage_root(storage_root.clone());

    let mut conf = match soma_server::Config::load(conf_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("配置加载失败 {conf_path}: {e:?}，使用默认配置");
            soma_server::Config::default()
        }
    };
    if let Some(ref root) = storage_root {
        conf.app.app.storage_path = Some(root.to_string_lossy().to_string());
    }
    soma_server::Config::set(conf.clone());

    let proxy_map = conf.app.get_proxy_map();
    if let Some(ref http_proxy) = proxy_map.get("http") {
        if !http_proxy.is_empty() {
            std::env::set_var("HTTP_PROXY", http_proxy);
        }
    }
    if let Some(ref https_proxy) = proxy_map.get("https") {
        if !https_proxy.is_empty() {
            std::env::set_var("HTTPS_PROXY", https_proxy);
        }
    }

    let max_concurrent = conf.app.get_max_concurrent_tasks();
    let max_queued = conf.app.get_max_queued_tasks();
    soma_server::task::init_queue(max_concurrent, max_queued);

    let storage_path = conf.app.get_storage_path().to_string();
    std::fs::create_dir_all(&storage_path).ok();
    std::fs::create_dir_all(soma_core::utils::storage_dir("songs", true)).ok();
    std::fs::create_dir_all(soma_core::utils::storage_dir("fonts", true)).ok();
    std::fs::create_dir_all(soma_core::utils::storage_dir("cache_videos", true)).ok();

    let db_path = format!("{storage_path}/tasks.db");
    soma_server::state::init_sqlite_store(&db_path);
}

/// 项目模式初始化（仅测试使用）：保持项目相对路径（conf/ 与 storage/）；
/// 桌面运行时一律走 [`init_backend_system`]（M2.4）
#[cfg(test)]
fn init_backend() {
    let root = resolve_project_root();
    if let Ok(cwd) = std::env::current_dir() {
        if cwd != root {
            let _ = std::env::set_current_dir(&root);
        }
    }

    let conf_path = tube_web::utils::get_abs_path("conf/config.toml");
    init_backend_common(&conf_path, None);
}

/// 桌面模式初始化（M2.4）：配置 → app_config_dir，存储 → app_data_dir/storage
///
/// 开发态仍尝试把工作目录切到项目根，保证 resource/ 相对路径
/// （字体/歌曲/推理脚本）继续可用；全新机器无项目目录时静默跳过。
fn init_backend_system(config_dir: &std::path::Path, data_dir: &std::path::Path) {
    if let Some(root) = resolve_project_root_checked() {
        if let Ok(cwd) = std::env::current_dir() {
            if cwd != root {
                let _ = std::env::set_current_dir(&root);
            }
        }
    }

    let conf_path = ensure_desktop_config(config_dir);
    let storage_root = data_dir.join("storage");
    init_backend_common(&conf_path.to_string_lossy(), Some(storage_root));
}

/// 示例命令：验证前端 → Rust 的 invoke 链路
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好，{name}！Soma 桌面版已就绪（Tauri v2）。")
}

/// 桌面端文件上传命令
///
/// 前端把 FormData 中的文件转为 base64 后 invoke 本命令，
/// 写入 `storage/uploads/` 并返回与 `/materials/upload` 兼容的 `{path}` 形状，
/// 使图片故事等上传流程在桌面模式下可用。
#[tauri::command]
fn upload_file(file_name: String, data_base64: String) -> Result<serde_json::Value, String> {
    use base64::Engine as _;
    let safe_name: String = file_name
        .replace(['\\', '/'], "_")
        .replace("..", "_")
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let dir = soma_core::utils::storage_dir("uploads", true);
    let path = dir.join(format!("{}-{}", soma_core::utils::get_uuid(), safe_name));
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64.as_bytes())
        .map_err(|e| format!("base64 解码失败: {e}"))?;
    std::fs::write(&path, bytes).map_err(|e| format!("写入失败: {e}"))?;
    Ok(serde_json::json!({
        "code": 200,
        "message": "",
        "result": { "path": path.to_string_lossy(), "url": "" }
    }))
}

/// 通用 API 分发命令
///
/// 与 HTTP 端点完全同构：前端 `POST /api/v1/{module}/{method}` 的请求
/// 在桌面模式转为 `invoke('api', { module, method, payload })`，
/// 返回相同形状的 `{code, result, message}` 信封，前端拦截器逻辑不变。
#[tauri::command]
async fn api(
    module: String,
    method: String,
    payload: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let value = tube_value::Value::from_serialize(&payload).unwrap_or(tube_value::Value::Null);
    let text = serde_json::to_string(&payload).ok();
    // RequestParameter 存在私有字段，跨 crate 只能 Default 构造后赋值公开字段
    let mut param = RequestParameter::default();
    param.module = module.clone();
    param.method = method.clone();
    param.value = value;
    param.text = text;

    // handler 层的 future 非 Send（actix ?Send async_trait + LocalSet），
    // 因此在独立原生线程上以 current-thread runtime + LocalSet 执行，
    // 与任务队列的线程模型一致，结果经 oneshot 送回 async 命令。
    let (tx, rx) = tokio::sync::oneshot::channel::<serde_json::Value>();
    std::thread::spawn(move || {
        let envelope = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => {
                let local = tokio::task::LocalSet::new();
                match local.block_on(&rt, dispatch(&module, &param)) {
                    Ok(v) => serde_json::json!({ "code": 200, "result": v, "message": "" }),
                    Err(e) => {
                        serde_json::json!({ "code": 40001, "result": "", "message": e.to_string() })
                    }
                }
            }
            Err(e) => serde_json::json!({
                "code": 40001, "result": "", "message": format!("runtime 创建失败: {e}")
            }),
        };
        let _ = tx.send(envelope);
    });
    rx.await.map_err(|e| format!("桌面后端线程异常: {e}"))
}

/// 业务分发：与 soma-server 路由表（router.rs）保持一致
///
/// 例外：upload（multipart 文件上传）依赖 HTTP 服务器环境，桌面模式暂不开放，
/// 前端会收到明确提示（M2.4/M3.3 通过本地文件对话框解决）。
async fn dispatch(module: &str, param: &RequestParameter) -> tube::Result<tube::Value> {
    match module.to_lowercase().as_str() {
        "videos" => soma_server::handler::video::distribute(param).await,
        "tasks" => soma_server::handler::video::distribute_tasks(param).await,
        "digital_human" => soma_server::handler::digital_human::distribute(param).await,
        "dh_tasks" => soma_server::handler::digital_human::distribute_tasks(param).await,
        "scripts" => soma_server::handler::llm::distribute(param).await,
        "terms" => soma_server::handler::llm::distribute_terms(param).await,
        "social" => soma_server::handler::llm::distribute_social(param).await,
        "intent" => soma_server::handler::llm::distribute_intent(param).await,
        "storyboard" => soma_server::handler::llm::distribute_storyboard(param).await,
        "musics" => soma_server::handler::music::distribute(param).await,
        "materials" => soma_server::handler::material::distribute(param).await,
        "config" => soma_server::handler::config::distribute(param).await,
        "voices" => soma_server::handler::voice::distribute(param).await,
        "portraits" => soma_server::handler::material::distribute_portraits(param).await,
        "features" => soma_server::handler::features::distribute(param).await,
        "system" => soma_server::handler::system::distribute(param).await,
        "image_story" => soma_server::handler::image_story::distribute(param).await,
        "download" => soma_server::handler::download::distribute(param).await,
        "notify" => soma_server::handler::notify::distribute(param).await,
        "subtitle" => soma_server::handler::subtitle::distribute(param).await,
        "jianying" => soma_server::handler::jianying::distribute(param).await,
        "platform" => soma_server::handler::platform::distribute(param).await,
        "dataexport" => soma_server::handler::dataexport::distribute(param).await,
        "watermark" => soma_server::handler::watermark::distribute(param).await,
        // stream 仅返回播放/下载 URL 字符串；桌面模式下实际媒体播放
        // 需要本机 soma-server 提供静态服务，或等 M3 接入 asset 协议
        "stream" => soma_server::handler::stream::distribute(param).await,
        _ => Err(error!("请求模块{module}桌面模式未提供。")),
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;
            // M2.4：后端初始化移入 setup 钩子，配置/存储落系统目录
            // （macOS: ~/Library/Application Support/com.soma.desktop/）
            let config_dir = app.path().app_config_dir().expect("解析系统配置目录失败");
            let data_dir = app.path().app_data_dir().expect("解析系统数据目录失败");
            init_backend_system(&config_dir, &data_dir);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, api, upload_file])
        .run(tauri::generate_context!())
        .expect("Soma 桌面应用启动失败");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 初始化类测试共享进程级全局状态（工作目录 / 路径覆盖 / 配置缓存），
    /// 用互斥锁串行化，避免并行互踩
    static INIT_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// 验证 features/list 的桌面分发链路：应返回全部功能点（含原子化新增）
    #[test]
    fn test_features_list_dispatch() {
        let _guard = INIT_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init_backend();
        let mut param = RequestParameter::default();
        param.module = "features".to_string();
        param.method = "list".to_string();
        param.value = tube_value::Value::from_serialize(serde_json::json!({}))
            .unwrap_or(tube_value::Value::Null);
        param.text = Some("{}".to_string());

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let local = tokio::task::LocalSet::new();
        let res = local
            .block_on(&rt, dispatch("features", &param))
            .expect("dispatch 失败");
        let json = serde_json::to_value(&res).expect("序列化失败");
        let list = json.as_array().expect("features/list 应返回数组");
        assert!(list.len() >= 24, "功能点数量应 ≥ 24，实际 {}", list.len());
        let ids: Vec<&str> = list
            .iter()
            .filter_map(|f| f.get("id").and_then(|v| v.as_str()))
            .collect();
        for expected in [
            "video.info",
            "video.transition",
            "digitalhuman.portrait",
            "llm.terms",
            "material.download",
        ] {
            assert!(ids.contains(&expected), "缺少功能点 {expected}");
        }
    }

    /// 验证真实 api 命令（线程 + oneshot 包装 + 信封）的完整路径
    #[test]
    fn test_api_command_envelope() {
        let _guard = INIT_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init_backend();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let envelope = rt
            .block_on(api(
                "features".to_string(),
                "list".to_string(),
                serde_json::json!({}),
            ))
            .expect("api 命令失败");
        assert_eq!(
            envelope["code"], 200,
            "信封 code 应为 200: {}",
            envelope["message"]
        );
        let list = envelope["result"].as_array().expect("result 应为数组");
        assert!(list.len() >= 24, "功能点数量应 ≥ 24，实际 {}", list.len());
    }

    /// 回归测试：router.rs 新增模块必须同步进桌面 dispatch，
    /// 否则桌面模式调用会命中"桌面模式未提供"错误。
    /// 以空参数调用，只要错误不是"模块未提供"即视为接线正确
    /// （各 handler 会返回自己的参数校验错误）。
    #[test]
    fn test_dispatch_covers_new_modules() {
        let _guard = INIT_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init_backend();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let local = tokio::task::LocalSet::new();

        for module in [
            "image_story",
            "download",
            "notify",
            "subtitle",
            "jianying",
            "platform",
            "dataexport",
            "watermark",
        ] {
            let mut param = RequestParameter::default();
            param.module = module.to_string();
            param.method = "list".to_string();
            param.value = tube_value::Value::Null;
            param.text = Some("{}".to_string());

            let res = local.block_on(&rt, dispatch(module, &param));
            match res {
                Ok(_) => { /* 接线正确且成功返回 */ }
                Err(e) => {
                    let msg = e.to_string();
                    assert!(
                        !msg.contains("桌面模式未提供"),
                        "模块 {module} 未接入桌面 dispatch: {msg}"
                    );
                }
            }
        }
    }

    /// 桌面模式 dataexport/preview 端到端（纯本地，无外部依赖）
    #[test]
    fn test_dispatch_dataexport_preview() {
        let _guard = INIT_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init_backend();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let local = tokio::task::LocalSet::new();

        let mut param = RequestParameter::default();
        param.module = "dataexport".to_string();
        param.method = "preview".to_string();
        param.value = tube_value::Value::from_serialize(
            serde_json::json!({ "data": [{ "k": "v", "x": "y" }] }),
        )
        .unwrap_or(tube_value::Value::Null);
        param.text = Some(r#"{"data":[{"k":"v","x":"y"}]}"#.to_string());

        let res = local
            .block_on(&rt, dispatch("dataexport", &param))
            .expect("dataexport/preview dispatch 失败");
        let json = serde_json::to_value(&res).expect("序列化失败");
        assert_eq!(json["total"], 1, "preview 应返回 total=1: {json}");
        let csv = json["csvPreview"].as_str().unwrap_or("");
        assert!(csv.contains("k,x"), "CSV 表头应含 k,x: {csv}");
        assert!(csv.contains("v,y"), "CSV 数据行应含 v,y: {csv}");
    }

    /// M2.4：桌面系统目录模式首启——配置自动初始化/迁移、存储落系统目录、分发链路可用
    #[test]
    fn test_system_mode_first_run() {
        let _guard = INIT_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let base =
            std::env::temp_dir().join(format!("soma-app-m24-{}", soma_core::utils::get_uuid()));
        let config_dir = base.join("config");
        let data_dir = base.join("data");

        assert!(
            !config_dir.join("config.toml").exists(),
            "前置：系统配置目录应为空（临时目录）"
        );
        init_backend_system(&config_dir, &data_dir);

        let conf_path = config_dir.join("config.toml");
        assert!(
            conf_path.exists(),
            "首启应在系统配置目录产出 config.toml: {conf_path:?}"
        );
        assert!(
            data_dir.join("storage/tasks.db").exists(),
            "SQLite 应落在系统存储目录"
        );
        assert_eq!(
            soma_core::utils::storage_dir("portraits", false),
            data_dir.join("storage/portraits"),
            "storage_dir 应被覆盖到系统目录"
        );
        assert_eq!(
            soma_server::Config::get_conf_path(),
            conf_path.to_string_lossy(),
            "配置缓存应指向系统路径（设置页保存写回此处）"
        );

        // 分发链路在系统模式下可用
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let envelope = rt
            .block_on(api(
                "features".to_string(),
                "list".to_string(),
                serde_json::json!({}),
            ))
            .expect("api 命令失败");
        assert_eq!(
            envelope["code"], 200,
            "系统模式 features/list 失败: {}",
            envelope["message"]
        );
        assert!(
            envelope["result"].as_array().is_some_and(|l| l.len() >= 24),
            "系统模式应返回 ≥24 个功能点"
        );

        // 清理：恢复项目相对语义，避免影响后续断言全局状态的用例
        soma_core::utils::set_storage_root(None);
        let _ = std::fs::remove_dir_all(&base);
    }
}
