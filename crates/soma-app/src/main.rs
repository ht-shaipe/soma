//! Soma 桌面应用（Tauri v2 宿主）
//!
//! 0.1.2 M2.2/M2.3：桌面模式直接复用 soma-server 业务层（handler/service/state），
//! 前端经 [`api`] 通用命令调用与 HTTP 完全同构的接口（{code, result, message} 信封），
//! Vue 前端零改动运行在浏览器（HTTP）与桌面（invoke）两种环境。
//!
//! 进度推送：任务进度仍以前端轮询（tasks/get）为主，
//! event 推送（progress://{task_id}）在任务中心 UI 改造时启用。

// 引入 tube 宏（error! 等）
#[macro_use]
extern crate tube;

// error! 宏展开时引用 crate::Error，需在 crate 根引入
use tube::Error;
use tube_web::RequestParameter;

/// 项目根目录解析（开发态可执行文件位于 {root}/target/debug/，向上三级即根目录）
///
/// 可用环境变量 SOMA_ROOT 覆盖；打包发布（M3.4）后由资源目录约定替代。
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

/// 后端初始化：复刻服务端启动序列（配置 → 代理 → 队列 → 存储目录 → SQLite）
///
/// 与服务端唯一区别：不启动 HTTP 监听。配置/存储路径迁移到系统目录（M2.4）时，
/// 仅需替换本函数中的根目录解析与 conf/storage 路径。
fn init_backend() {
    let root = resolve_project_root();
    if let Ok(cwd) = std::env::current_dir() {
        if cwd != root {
            let _ = std::env::set_current_dir(&root);
        }
    }

    let conf_path = tube_web::utils::get_abs_path("conf/config.toml");
    let conf = match soma_server::Config::load(&conf_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("配置加载失败 {conf_path}: {e:?}，使用默认配置");
            soma_server::Config::default()
        }
    };
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
        .replace('\\', "_")
        .replace('/', "_")
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
        // stream 仅返回播放/下载 URL 字符串；桌面模式下实际媒体播放
        // 需要本机 soma-server 提供静态服务，或等 M3 接入 asset 协议
        "stream" => soma_server::handler::stream::distribute(param).await,
        _ => Err(error!("请求模块{module}桌面模式未提供。")),
    }
}

fn main() {
    init_backend();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, api, upload_file])
        .run(tauri::generate_context!())
        .expect("Soma 桌面应用启动失败");
}


#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 features/list 的桌面分发链路：应返回全部功能点（含原子化新增）
    #[test]
    fn test_features_list_dispatch() {
        init_backend();
        let mut param = RequestParameter::default();
        param.module = "features".to_string();
        param.method = "list".to_string();
        param.value = tube_value::Value::from_serialize(&serde_json::json!({})).unwrap_or(tube_value::Value::Null);
        param.text = Some("{}".to_string());

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let local = tokio::task::LocalSet::new();
        let res = local.block_on(&rt, dispatch("features", &param)).expect("dispatch 失败");
        let json = serde_json::to_value(&res).expect("序列化失败");
        let list = json.as_array().expect("features/list 应返回数组");
        assert!(list.len() >= 24, "功能点数量应 ≥ 24，实际 {}", list.len());
        let ids: Vec<&str> = list.iter().filter_map(|f| f.get("id").and_then(|v| v.as_str())).collect();
        for expected in ["video.info", "video.transition", "digitalhuman.portrait", "llm.terms", "material.download"] {
            assert!(ids.contains(&expected), "缺少功能点 {expected}");
        }
    }

    /// 验证真实 api 命令（线程 + oneshot 包装 + 信封）的完整路径
    #[test]
    fn test_api_command_envelope() {
        init_backend();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let envelope = rt.block_on(api(
            "features".to_string(),
            "list".to_string(),
            serde_json::json!({}),
        ))
        .expect("api 命令失败");
        assert_eq!(envelope["code"], 200, "信封 code 应为 200: {}", envelope["message"]);
        let list = envelope["result"].as_array().expect("result 应为数组");
        assert!(list.len() >= 24, "功能点数量应 ≥ 24，实际 {}", list.len());
    }
}
