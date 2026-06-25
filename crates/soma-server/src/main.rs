use actix_cors::Cors;
use actix_web::{guard, middleware, web, App, HttpServer};
use actix_files as afs;
use clap::Parser;

const DEF_CONFIG_PATH: &str = "conf/config.toml";

#[derive(Parser, Debug)]
#[command(name = "soma", about = "Soma - AI video generator")]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let conf_path = tube_web::utils::get_abs_path(&args.config.unwrap_or(DEF_CONFIG_PATH.to_owned()));

    let conf = match soma_server::Config::load(&conf_path) {
        Ok(c) => c,
        Err(e) => {
            println!("配置加载失败 {}: {:?}", conf_path, e);
            soma_server::Config::default()
        }
    };

    soma_server::Config::set(conf.clone());

    tube_web::logs::initialize_logging("");

    let max_concurrent = conf.app.get_max_concurrent_tasks();
    let max_queued = conf.app.get_max_queued_tasks();
    soma_server::task::init_queue(max_concurrent, max_queued);

    let storage_path = conf.app.get_storage_path().to_string();
    std::fs::create_dir_all(&storage_path).ok();
    std::fs::create_dir_all(soma_core::utils::storage_dir("songs", true)).ok();
    std::fs::create_dir_all(soma_core::utils::storage_dir("fonts", true)).ok();
    std::fs::create_dir_all(soma_core::utils::storage_dir("cache_videos", true)).ok();

    let ip = format!("{}:{}", conf.app.get_listen_host(), conf.app.get_listen_port());
    println!("Soma server starting at {}", ip);

    let storage_path_clone = storage_path.clone();
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .max_age(3600),
            )
            .service(
                afs::Files::new("/storage", &storage_path_clone)
                    .show_files_listing()
                    .redirect_to_slash_directory(),
            )
            .service(
                web::scope("/api/v1")
                    .service(
                        web::resource("/{cls}")
                            .route(web::to(soma_server::router::api_handler)),
                    )
                    .service(
                        web::resource("/{cls}/{tail:.*}")
                            .route(web::to(soma_server::router::api_handler)),
                    ),
            )
    })
    .bind(ip)?
    .run()
    .await
}
