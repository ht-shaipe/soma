/// Soma 服务端启动入口
///
/// 负责加载配置、初始化日志、创建存储目录、初始化任务队列，
/// 并启动 Actix Web HTTP 服务，挂载 CORS 中间件、静态文件服务和 API 路由。

use actix_cors::Cors;
use actix_web::{guard, middleware, web, App, HttpServer};
use actix_files as afs;
use clap::Parser;

/// 默认配置文件路径（相对于工作目录）
const DEF_CONFIG_PATH: &str = "conf/config.toml";

/// 命令行参数结构体
#[derive(Parser, Debug)]
#[command(name = "soma", about = "Soma - AI video generator")]
struct Args {
    /// 可选的配置文件路径参数，通过 -c/--config 指定
    #[arg(short, long)]
    config: Option<String>,
}

/// 服务端主函数 - 加载配置并启动 HTTP 服务
///
/// 启动流程：
/// 1. 解析命令行参数，确定配置文件路径
/// 2. 加载配置文件（失败则使用默认配置）
/// 3. 将配置写入全局缓存
/// 4. 初始化日志系统
/// 5. 初始化任务队列（并发数和排队数）
/// 6. 创建所需存储目录（songs、fonts、cache_videos 等）
/// 7. 绑定 IP:端口，启动 Actix Web 服务
///
/// 返回：IO 操作结果
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // 获取配置文件绝对路径，未指定则使用默认路径
    let conf_path = tube_web::utils::get_abs_path(&args.config.unwrap_or(DEF_CONFIG_PATH.to_owned()));

    // 加载配置，加载失败时打印错误并使用默认配置
    let conf = match soma_server::Config::load(&conf_path) {
        Ok(c) => c,
        Err(e) => {
            println!("配置加载失败 {}: {:?}", conf_path, e);
            soma_server::Config::default()
        }
    };

    // 将配置写入全局缓存，供其他模块随时读取
    soma_server::Config::set(conf.clone());

    // 初始化日志系统
    tube_web::logs::initialize_logging("");

    // 读取并发与排队配置，初始化任务队列
    let max_concurrent = conf.app.get_max_concurrent_tasks();
    let max_queued = conf.app.get_max_queued_tasks();
    soma_server::task::init_queue(max_concurrent, max_queued);

    // 创建存储相关目录
    let storage_path = conf.app.get_storage_path().to_string();
    std::fs::create_dir_all(&storage_path).ok();
    std::fs::create_dir_all(soma_core::utils::storage_dir("songs", true)).ok();
    std::fs::create_dir_all(soma_core::utils::storage_dir("fonts", true)).ok();
    std::fs::create_dir_all(soma_core::utils::storage_dir("cache_videos", true)).ok();

    // 构建监听地址
    let ip = format!("{}:{}", conf.app.get_listen_host(), conf.app.get_listen_port());
    println!("Soma server starting at {}", ip);

    // 启动 HTTP 服务，使用闭包构建 App 实例
    let storage_path_clone = storage_path.clone();
    HttpServer::new(move || {
        App::new()
            // 请求日志中间件
            .wrap(middleware::Logger::default())
            // CORS 跨域配置：允许所有来源、指定方法、缓存预检1小时
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .max_age(3600),
            )
            // 静态文件服务：/storage 路径映射到本地存储目录
            .service(
                afs::Files::new("/storage", &storage_path_clone)
                    .show_files_listing()
                    .redirect_to_slash_directory(),
            )
            // API 路由：所有 /api/v1/* 请求统一由 router::api_handler 分发
            .service(
                web::scope("/api/v1")
                    .service(
                        web::resource("/{cls}")
                            .route(web::to(soma_server::router::api_handler)),
                    )
                    .service(
                        web::resource("/{cls}/{tail:.*}")
                            .route(web::to(soma_server::router::api_handler)),
                    )
                    // 文件上传接口（multipart/form-data，不走统一分发）
                    .service(
                        web::resource("/materials/upload")
                            .route(web::post().to(soma_server::handler::material::upload_file)),
                    )
                    .service(
                        web::resource("/musics/upload")
                            .route(web::post().to(soma_server::handler::music::upload_file)),
                    )
            )
    })
    .bind(ip)?
    .run()
    .await
}
