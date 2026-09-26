//! 字幕处理 API 处理器
//!
//! 提供字幕翻译、校正、合并和格式转换功能。

use soma_core::subtitle;
use tube::{Result, Value};
use tube_web::RequestParameter;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "parse" => parse(param).await,
        "convert" => convert(param).await,
        "merge" => merge(param).await,
        "translate" => translate(param).await,
        "correct" => correct(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 解析字幕文件
async fn parse(param: &RequestParameter) -> Result<Value> {
    let path = param.value.get_def_string("path", "");
    if path.is_empty() {
        return Err(error!("缺少 path 参数"));
    }

    let entries = subtitle::load_subtitle(&path).map_err(|e| error!("{}", e))?;

    let cues: Vec<Value> = entries
        .iter()
        .map(|e| {
            value!({
                "index": e.index,
                "startMs": e.start_ms,
                "endMs": e.end_ms,
                "text": e.text.clone(),
            })
        })
        .collect();

    Ok(value!({
        "count": entries.len(),
        "entries": cues,
    }))
}

/// 格式转换
async fn convert(param: &RequestParameter) -> Result<Value> {
    let input_path = param.value.get_def_string("inputPath", "");
    let output_path = param.value.get_def_string("outputPath", "");
    if input_path.is_empty() || output_path.is_empty() {
        return Err(error!("缺少 inputPath 或 outputPath 参数"));
    }

    let entries = subtitle::load_subtitle(&input_path).map_err(|e| error!("{}", e))?;

    subtitle::save_subtitle(&entries, &output_path).map_err(|e| error!("{}", e))?;

    Ok(value!({
        "success": true,
        "count": entries.len(),
        "outputPath": output_path,
    }))
}

/// 合并多个字幕文件
async fn merge(param: &RequestParameter) -> Result<Value> {
    let output_path = param.value.get_def_string("outputPath", "");
    if output_path.is_empty() {
        return Err(error!("缺少 outputPath 参数"));
    }

    let paths_val = param.value.get("paths").and_then(|v| v.as_array());
    let paths: Vec<String> = if let Some(arr) = paths_val {
        arr.iter().filter_map(|v| v.as_str()).collect()
    } else {
        return Err(error!("缺少 paths 参数"));
    };

    let merged = subtitle::merge_srt_files(&paths).map_err(|e| error!("{}", e))?;

    subtitle::save_subtitle(&merged, &output_path).map_err(|e| error!("{}", e))?;

    Ok(value!({
        "success": true,
        "count": merged.len(),
        "outputPath": output_path,
    }))
}

/// 翻译字幕
async fn translate(param: &RequestParameter) -> Result<Value> {
    let input_path = param.value.get_def_string("inputPath", "");
    let output_path = param.value.get_def_string("outputPath", "");
    let target_lang = param.value.get_def_string("targetLang", "");
    if input_path.is_empty() || output_path.is_empty() || target_lang.is_empty() {
        return Err(error!("缺少 inputPath / outputPath / targetLang 参数"));
    }

    let conf = crate::Config::get();
    let api_key = conf.app.app.openai_api_key.clone().unwrap_or_default();
    let base_url = conf
        .app
        .app
        .openai_base_url
        .clone()
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
    let model = conf
        .app
        .app
        .openai_model_name
        .clone()
        .unwrap_or_else(|| "gpt-4o-mini".to_string());

    if api_key.is_empty() {
        return Err(error!("LLM API Key 未配置"));
    }

    let entries = subtitle::load_subtitle(&input_path).map_err(|e| error!("{}", e))?;

    let params = subtitle::TranslateParams {
        api_key,
        base_url,
        model,
        target_lang: target_lang.to_string(),
        batch_size: param
            .value
            .get("batchSize")
            .and_then(|v| v.as_u64())
            .unwrap_or(20) as usize,
    };

    let translated = subtitle::translate_subtitles(&entries, &params)
        .await
        .map_err(|e| error!("{}", e))?;

    subtitle::save_subtitle(&translated, &output_path).map_err(|e| error!("{}", e))?;

    Ok(value!({
        "success": true,
        "count": translated.len(),
        "outputPath": output_path,
    }))
}

/// 校正字幕
async fn correct(param: &RequestParameter) -> Result<Value> {
    let input_path = param.value.get_def_string("inputPath", "");
    let output_path = param.value.get_def_string("outputPath", "");
    if input_path.is_empty() || output_path.is_empty() {
        return Err(error!("缺少 inputPath 或 outputPath 参数"));
    }

    let conf = crate::Config::get();
    let api_key = conf.app.app.openai_api_key.clone().unwrap_or_default();
    let base_url = conf
        .app
        .app
        .openai_base_url
        .clone()
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
    let model = conf
        .app
        .app
        .openai_model_name
        .clone()
        .unwrap_or_else(|| "gpt-4o-mini".to_string());

    if api_key.is_empty() {
        return Err(error!("LLM API Key 未配置"));
    }

    let entries = subtitle::load_subtitle(&input_path).map_err(|e| error!("{}", e))?;

    let correct_type = param.value.get_def_string("correctType", "all");
    let params = subtitle::CorrectParams {
        api_key,
        base_url,
        model,
        correct_type: correct_type.to_string(),
    };

    let corrected = subtitle::correct_subtitles(&entries, &params)
        .await
        .map_err(|e| error!("{}", e))?;

    subtitle::save_subtitle(&corrected, &output_path).map_err(|e| error!("{}", e))?;

    Ok(value!({
        "success": true,
        "count": corrected.len(),
        "outputPath": output_path,
    }))
}
