use crate::Config;
use std::process::Command;
/// 系统环境检测处理器
///
/// - preflight: 依赖与配置体检（FFmpeg/ffprobe/edge-tts 可用性、存储可写、各服务密钥状态）
///
/// 供设置页「环境状态」卡片与桌面端启动提示使用（0.1.2 M2.5）。
use tube::{Result, Value};
use tube_web::RequestParameter;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "preflight" => preflight().await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 单项检测：命令是否可执行（存在且退出码为 0/--version）
fn check_binary(name: &str, program: &str, args: &[&str]) -> (bool, String) {
    let output = Command::new(program).args(args).output();
    match output {
        Ok(o) if o.status.success() => {
            let version = String::from_utf8_lossy(&o.stdout);
            let first = version
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect::<String>();
            (true, first)
        }
        Ok(o) => (false, format!("{} 退出码异常: {}", program, o.status)),
        Err(e) => (false, format!("未找到 {}（{}），请安装后重试", name, e)),
    }
}

/// 构造单项检测结果的便捷宏
macro_rules! check {
    ($name:expr, $ok:expr, $detail:expr, $hint:expr) => {
        (
            $name.to_string(),
            $ok,
            $detail.to_string(),
            $hint.to_string(),
        )
    };
}

/// 环境体检
///
/// 返回 checks 数组（name/ok/detail/hint），前端据此渲染 ✓/✗ 与安装指引。
async fn preflight() -> Result<Value> {
    let conf = Config::get();
    let ffmpeg_bin = conf.app.get_ffmpeg_binary();

    let (ffmpeg_ok, ffmpeg_detail) = check_binary("FFmpeg", &ffmpeg_bin, &["-version"]);
    let (ffprobe_ok, ffprobe_detail) = check_binary("ffprobe", "ffprobe", &["-version"]);
    let (edge_ok, edge_detail) = check_binary("edge-tts", "edge-tts", &["--version"]);

    // 存储可写
    let storage_path = conf.app.get_storage_path().to_string();
    let probe = std::path::Path::new(&storage_path).join(".write_probe");
    let storage_ok = std::fs::write(&probe, b"ok").is_ok();
    let _ = std::fs::remove_file(&probe);
    let storage_detail = if storage_ok {
        format!("{} 可写", storage_path)
    } else {
        format!("{} 不可写，请检查目录权限", storage_path)
    };

    // 各服务密钥状态（只报告配置与否，不校验有效性）
    let llm_provider = conf.app.app.llm_provider.as_deref().unwrap_or("openai");
    let llm_key = get_llm_key(&conf, llm_provider);
    let llm_ok = !llm_key.is_empty();
    let pexels_ok = conf
        .app
        .app
        .pexels_api_keys
        .as_ref()
        .map(|v| v.iter().any(|k| !k.is_empty()))
        .unwrap_or(false);
    let aivideo_ok = conf.app.app.zhipu_video_api_key.is_some()
        || conf.app.app.kling_access_key.is_some()
        || conf.app.app.minimax_video_api_key.is_some();

    let checks = [
        check!(
            "FFmpeg",
            ffmpeg_ok,
            ffmpeg_detail,
            "brew install ffmpeg 或 apt install ffmpeg"
        ),
        check!("ffprobe", ffprobe_ok, ffprobe_detail, "随 FFmpeg 一同安装"),
        check!(
            "edge-tts",
            edge_ok,
            edge_detail,
            "pip install edge-tts（免费配音引擎）"
        ),
        check!(
            "存储目录",
            storage_ok,
            storage_detail,
            "检查磁盘权限或更换存储路径"
        ),
        check!(
            "LLM 密钥",
            llm_ok,
            format!(
                "提供商 {}，{}",
                llm_provider,
                if llm_ok { "已配置" } else { "未配置" }
            ),
            "系统设置 → LLM 配置 中填入 API Key"
        ),
        check!(
            "素材站密钥",
            pexels_ok,
            if pexels_ok {
                "Pexels 已配置".to_string()
            } else {
                "Pexels/Pixabay/Coverr 均未配置".to_string()
            },
            "系统设置 → 素材源 API Key"
        ),
        check!(
            "AI 视频密钥",
            aivideo_ok,
            if aivideo_ok {
                "至少一家已配置".to_string()
            } else {
                "智谱/可灵/MiniMax 均未配置".to_string()
            },
            "系统设置 → AI 视频生成"
        ),
    ];

    let all_ok = checks.iter().all(|c| c.1);
    Ok(value!({
        "allOk": all_ok,
        "checks": checks.iter().map(|(name, ok, detail, hint)| value!({
            "name": name, "ok": ok, "detail": detail, "hint": hint,
        })).collect::<Vec<_>>(),
    }))
}

/// 读取当前 LLM 提供商对应的 API Key（与 handler/config.rs 的路由规则一致）
fn get_llm_key(conf: &Config, provider: &str) -> String {
    let a = &conf.app.app;
    match provider {
        "openai" => a.openai_api_key.as_deref().unwrap_or(""),
        "deepseek" => a.deepseek_api_key.as_deref().unwrap_or(""),
        "qwen" => a.qwen_api_key.as_deref().unwrap_or(""),
        "moonshot" | "kimi" => a.moonshot_api_key.as_deref().unwrap_or(""),
        "zhipu" => a.zhipu_api_key.as_deref().unwrap_or(""),
        "doubao" => a.doubao_api_key.as_deref().unwrap_or(""),
        "hunyuan" => a.hunyuan_api_key.as_deref().unwrap_or(""),
        "wenxin" => a.wenxin_api_key.as_deref().unwrap_or(""),
        "xunfei" => a.xunfei_api_key.as_deref().unwrap_or(""),
        "minimax" => a.minimax_api_key.as_deref().unwrap_or(""),
        "groq" => a.groq_api_key.as_deref().unwrap_or(""),
        "grok" => a.grok_api_key.as_deref().unwrap_or(""),
        "gemini" => a.gemini_api_key.as_deref().unwrap_or(""),
        "azure" => a.azure_api_key.as_deref().unwrap_or(""),
        "mimo" => a.mimo_api_key.as_deref().unwrap_or(""),
        "modelscope" => a.modelscope_api_key.as_deref().unwrap_or(""),
        "aihubmix" => a.aihubmix_api_key.as_deref().unwrap_or(""),
        "evolink" => a.evolink_api_key.as_deref().unwrap_or(""),
        "aiml" | "aimlapi" => a.aimlapi_api_key.as_deref().unwrap_or(""),
        "litellm" => a.oneapi_api_key.as_deref().unwrap_or(""),
        "ollama" | "g4f" | "pollinations" => "ok", // 免密钥提供商
        _ => "",
    }
    .to_string()
}
