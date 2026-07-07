/// API 路由分发模块
///
/// 作为所有 /api/v1/* 请求的统一入口，根据请求中的 module 名称
/// 将请求分发到对应的 handler 处理函数。

use actix_web::{web, Error as ActixError, HttpRequest, HttpResponse};
use tube_web::{
    response::{get_error, get_success},
};

/// API 请求统一处理函数
///
/// 解析请求参数后，根据 param.module 字段匹配对应的业务处理器：
/// - "videos" → 视频创建/草稿/配置更新/启动
/// - "tasks" → 任务列表/查询/删除
/// - "scripts" → LLM 脚本生成
/// - "terms" → LLM 关键词提取
/// - "social" → LLM 社交元数据生成
/// - "intent" → LLM 需求理解/意图解析
/// - "storyboard" → LLM 分镜脚本生成
/// - "musics" → 音乐文件管理
/// - "materials" → 素材文件管理
/// - "stream" → 视频流播放/下载
/// - "config" → 配置读写
///
/// 参数：
/// - `req`: HTTP 请求
/// - `payload`: 请求体
///
/// 返回：成功返回业务数据，失败返回错误信息
pub async fn api_handler(req: HttpRequest, payload: web::Payload) -> Result<HttpResponse, ActixError> {
    let param = tube_web::parse_request(req, payload).await;

    // 根据 module 名称分发到对应 handler
    let res = match param.module.to_lowercase().as_str() {
        "videos" => crate::handler::video::distribute(&param).await,
        "tasks" => crate::handler::video::distribute_tasks(&param).await,
        "scripts" => crate::handler::llm::distribute(&param).await,
        "terms" => crate::handler::llm::distribute_terms(&param).await,
        "social" => crate::handler::llm::distribute_social(&param).await,
        "intent" => crate::handler::llm::distribute_intent(&param).await,
        "storyboard" => crate::handler::llm::distribute_storyboard(&param).await,
        "musics" => crate::handler::music::distribute(&param).await,
        "materials" => crate::handler::material::distribute(&param).await,
        "stream" => crate::handler::stream::distribute(&param).await,
        "config" => crate::handler::config::distribute(&param).await,
        "voices" => crate::handler::voice::distribute(&param).await,
        "upload" => crate::handler::upload::distribute(&param).await,
        "portraits" => crate::handler::material::distribute_portraits(&param).await,
        _ => Err(error!("请求方法{}.{}系统未提供。", param.module, param.method)),
    };

    // 统一封装成功/失败响应
    match res {
        Ok(v) => get_success(&v),
        Err(e) => get_error(e),
    }
}
