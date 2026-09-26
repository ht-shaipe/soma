//! 数据导出 API 处理器
//!
//! 通用数据导出：JSON/JSONL 数据 → CSV / JSON / JSONL 文件。

use soma_core::export::{self, DataExporter, ExportFormat, ExportRow};
use tube::{Map, Result, Value};
use tube_web::RequestParameter;

pub async fn distribute(param: &RequestParameter) -> Result<Value> {
    match param.method.to_lowercase().as_str() {
        "export" => export_data(param).await,
        "import" => import_data(param).await,
        "preview" => preview_data(param).await,
        _ => Err(error!("不支持的方法: {}", param.method)),
    }
}

/// 从参数中的 JSON 数组构建导出行
fn rows_from_json_value(arr: &[Value]) -> Vec<ExportRow> {
    arr.iter()
        .map(|v| {
            let mut row = ExportRow::new();
            if let Some(obj) = v.as_object() {
                for (k, val) in obj.iter() {
                    row = row.add(k, &value_to_string(val));
                }
            }
            row
        })
        .collect()
}

/// tube Value 转字符串
fn value_to_string(v: &Value) -> String {
    match v {
        Value::Text(s) => s.clone(),
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Int64(i) => i.to_string(),
        Value::U32(u) => u.to_string(),
        Value::U64(u) => u.to_string(),
        Value::Double(d) => d.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Object(o) => o.to_string(),
        Value::Array(a) => a.iter().map(value_to_string).collect::<Vec<_>>().join(","),
        other => other.to_string(),
    }
}

/// 导出数据到文件
///
/// 数据来源二选一：
/// - `data`: JSON 数组（对象数组）
/// - `inputPath`: JSON / JSONL 文件路径
///
/// 输出：`outputPath`（扩展名决定格式：csv / json / jsonl）
async fn export_data(param: &RequestParameter) -> Result<Value> {
    let output_path = param.value.get_def_string("outputPath", "");
    if output_path.is_empty() {
        return Err(error!("缺少 outputPath 参数"));
    }

    let mut exporter = DataExporter::new();

    if let Some(arr) = param.value.get("data").and_then(|v| v.as_array()) {
        if arr.is_empty() {
            return Err(error!("data 数据为空"));
        }
        exporter.add_rows(rows_from_json_value(&arr));
    } else {
        let input_path = param.value.get_def_string("inputPath", "");
        if input_path.is_empty() {
            return Err(error!("缺少 data 或 inputPath 参数"));
        }

        let content = std::fs::read_to_string(&input_path)
            .map_err(|e| error!("读取输入文件失败: {}", e))?;

        let rows = if input_path.to_lowercase().ends_with(".jsonl") {
            export::import_jsonl(&content)
        } else if input_path.to_lowercase().ends_with(".csv") {
            export::import_csv(&content)
        } else {
            export::import_json(&content).map_err(|_| error!("解析 JSON 失败，请确认输入文件是 JSON 数组格式"))?
        };

        if rows.is_empty() {
            return Err(error!("输入文件中没有数据行"));
        }
        exporter.add_rows(rows);
    }

    // 可选列头（按给定顺序输出 CSV 列）
    if let Some(headers) = param.value.get("headers").and_then(|v| v.as_array()) {
        let hs: Vec<String> = headers.iter().filter_map(|v| v.as_str()).collect();
        if !hs.is_empty() {
            exporter.set_headers(hs);
        }
    }

    // 格式：显式指定优先，否则按输出文件扩展名
    let format = match param.value.get_def_string("format", "").to_lowercase().as_str() {
        "csv" => Some(ExportFormat::Csv),
        "json" => Some(ExportFormat::Json),
        "jsonl" => Some(ExportFormat::Jsonl),
        _ => None,
    };

    let count = match format {
        Some(f) => exporter.export(&output_path, f).map_err(|e| error!("{}", e))?,
        None => exporter.export_auto(&output_path).map_err(|e| error!("{}", e))?,
    };

    Ok(value!({
        "success": true,
        "count": count,
        "outputPath": output_path,
    }))
}

/// 导入文件并返回数据预览（前 N 行）
async fn import_data(param: &RequestParameter) -> Result<Value> {
    let path = param.value.get_def_string("path", "");
    if path.is_empty() {
        return Err(error!("缺少 path 参数"));
    }

    let rows = load_rows(&path)?;

    Ok(value!({
        "total": rows.len(),
        "rows": rows_to_value(&rows),
    }))
}

/// 预览待导出的 JSON 数据（不落盘，用于前端确认格式）
async fn preview_data(param: &RequestParameter) -> Result<Value> {
    let arr = param.value.get("data").and_then(|v| v.as_array());
    let rows = match arr {
        Some(arr) if !arr.is_empty() => rows_from_json_value(&arr),
        _ => return Err(error!("缺少 data 参数或 data 为空")),
    };

    let total = rows.len();
    let preview_rows = rows_to_value(&rows);

    let mut exporter = DataExporter::new();
    exporter.add_rows(rows);

    Ok(value!({
        "total": total,
        "rows": preview_rows,
        "csvPreview": exporter.to_csv(),
    }))
}

fn load_rows(path: &str) -> Result<Vec<ExportRow>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| error!("读取文件失败: {}", e))?;

    if path.to_lowercase().ends_with(".jsonl") {
        Ok(export::import_jsonl(&content))
    } else if path.to_lowercase().ends_with(".csv") {
        Ok(export::import_csv(&content))
    } else {
        export::import_json(&content).map_err(|_| error!("解析 JSON 失败，请确认文件是 JSON 数组格式"))
    }
}

fn rows_to_value(rows: &[ExportRow]) -> Vec<Value> {
    rows.iter()
        .take(50)
        .map(|row| {
            let mut obj = Map::new();
            for (k, v) in &row.fields {
                obj.insert_str(k, v);
            }
            Value::Object(obj)
        })
        .collect()
}
