use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use tube::Error;
use tube_web::{
    response::{get_error, get_success},
};

pub async fn api_handler(req: HttpRequest, payload: web::Payload) -> Result<HttpResponse, ActixError> {
    let param = tube_web::parse_request(req, payload).await;

    let res = match param.module.to_lowercase().as_str() {
        "videos" => crate::handler::video::distribute(&param).await,
        "tasks" => crate::handler::video::distribute_tasks(&param).await,
        "scripts" => crate::handler::llm::distribute(&param).await,
        "terms" => crate::handler::llm::distribute_terms(&param).await,
        "social" => crate::handler::llm::distribute_social(&param).await,
        "musics" => crate::handler::music::distribute(&param).await,
        "materials" => crate::handler::material::distribute(&param).await,
        "stream" => crate::handler::stream::distribute(&param).await,
        _ => Err(error!("请求方法{}.{}系统未提供。", param.module, param.method)),
    };

    match res {
        Ok(v) => get_success(&v),
        Err(e) => get_error(e),
    }
}
