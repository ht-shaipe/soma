//! 数据导出模块
//!
//! 提供通用数据导出功能，支持 CSV / JSON / JSONL 格式。
//! 用于爬虫数据、任务结果、统计报表等场景。

use serde::Serialize;
use std::path::Path;

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Csv,
    Json,
    Jsonl,
}

impl ExportFormat {
    pub fn from_ext(ext: &str) -> Self {
        match ext.to_lowercase().trim_start_matches('.') {
            "json" => ExportFormat::Json,
            "jsonl" => ExportFormat::Jsonl,
            _ => ExportFormat::Csv,
        }
    }
}

/// 导出数据行（键值对）
#[derive(Debug, Clone, Serialize)]
pub struct ExportRow {
    pub fields: Vec<(String, String)>,
}

impl ExportRow {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn add(mut self, key: &str, value: &str) -> Self {
        self.fields.push((key.to_string(), value.to_string()));
        self
    }
}

/// 导出器
pub struct DataExporter {
    rows: Vec<ExportRow>,
    headers: Vec<String>,
}

impl DataExporter {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            headers: Vec::new(),
        }
    }

    /// 设置列头
    pub fn set_headers(&mut self, headers: Vec<String>) {
        self.headers = headers;
    }

    /// 添加数据行
    pub fn add_row(&mut self, row: ExportRow) {
        if self.headers.is_empty() {
            self.headers = row.fields.iter().map(|(k, _)| k.clone()).collect();
        }
        self.rows.push(row);
    }

    /// 批量添加数据行
    pub fn add_rows(&mut self, rows: Vec<ExportRow>) {
        for row in rows {
            self.add_row(row);
        }
    }

    /// 导出为 CSV
    pub fn to_csv(&self) -> String {
        let mut output = String::new();

        // 表头
        if !self.headers.is_empty() {
            output.push_str(&self.headers.join(","));
            output.push('\n');
        }

        // 数据行
        for row in &self.rows {
            let values: Vec<String> = if self.headers.is_empty() {
                row.fields.iter().map(|(_, v)| csv_escape(v)).collect()
            } else {
                self.headers
                    .iter()
                    .map(|h| {
                        let val = row
                            .fields
                            .iter()
                            .find(|(k, _)| k == h)
                            .map(|(_, v)| v.as_str())
                            .unwrap_or("");
                        csv_escape(val)
                    })
                    .collect()
            };
            output.push_str(&values.join(","));
            output.push('\n');
        }

        output
    }

    /// 导出为 JSON 数组
    pub fn to_json(&self) -> String {
        let objects: Vec<serde_json::Value> = self
            .rows
            .iter()
            .map(|row| {
                let mut obj = serde_json::Map::new();
                for (k, v) in &row.fields {
                    obj.insert(k.clone(), serde_json::Value::String(v.clone()));
                }
                serde_json::Value::Object(obj)
            })
            .collect();

        serde_json::to_string_pretty(&objects).unwrap_or_else(|_| "[]".to_string())
    }

    /// 导出为 JSONL（每行一个 JSON 对象）
    pub fn to_jsonl(&self) -> String {
        let mut output = String::new();
        for row in &self.rows {
            let mut obj = serde_json::Map::new();
            for (k, v) in &row.fields {
                obj.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            let line = serde_json::to_string(&serde_json::Value::Object(obj))
                .unwrap_or_else(|_| "{}".to_string());
            output.push_str(&line);
            output.push('\n');
        }
        output
    }

    /// 导出到文件
    pub fn export(&self, path: &str, format: ExportFormat) -> Result<usize, String> {
        let content = match format {
            ExportFormat::Csv => self.to_csv(),
            ExportFormat::Json => self.to_json(),
            ExportFormat::Jsonl => self.to_jsonl(),
        };

        let dir = Path::new(path).parent();
        if let Some(d) = dir {
            if !d.exists() {
                std::fs::create_dir_all(d).map_err(|e| format!("创建目录失败: {}", e))?;
            }
        }

        std::fs::write(path, &content).map_err(|e| format!("写入文件失败: {}", e))?;
        Ok(self.rows.len())
    }

    /// 自动检测格式并导出
    pub fn export_auto(&self, path: &str) -> Result<usize, String> {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("csv");
        self.export(path, ExportFormat::from_ext(ext))
    }

    /// 获取行数
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// CSV 字段转义
fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// 从 JSON 数组导入数据
pub fn import_json(content: &str) -> Result<Vec<ExportRow>, String> {
    let arr: Vec<serde_json::Value> =
        serde_json::from_str(content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

    Ok(arr
        .iter()
        .map(|v| {
            let mut row = ExportRow::new();
            if let Some(obj) = v.as_object() {
                for (k, v) in obj {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Null => String::new(),
                        _ => v.to_string(),
                    };
                    row = row.add(k, &val);
                }
            }
            row
        })
        .collect())
}

/// 从 JSONL 导入数据
pub fn import_jsonl(content: &str) -> Vec<ExportRow> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let mut row = ExportRow::new();
            let v: serde_json::Value = serde_json::from_str(l).ok()?;
            if let Some(obj) = v.as_object() {
                for (k, v) in obj {
                    let val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Null => String::new(),
                        _ => v.to_string(),
                    };
                    row = row.add(k, &val);
                }
            }
            Some(row)
        })
        .collect()
}

/// 从 CSV 导入数据（支持带引号的转义字段，含字段内换行）
pub fn import_csv(content: &str) -> Vec<ExportRow> {
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut fields: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                fields.push(std::mem::take(&mut current));
            }
            '\r' if !in_quotes && chars.peek() == Some(&'\n') => {
                chars.next();
                fields.push(std::mem::take(&mut current));
                if !fields.is_empty() {
                    records.push(std::mem::take(&mut fields));
                }
            }
            '\n' if !in_quotes => {
                fields.push(std::mem::take(&mut current));
                if !fields.is_empty() {
                    records.push(std::mem::take(&mut fields));
                }
            }
            _ => current.push(c),
        }
    }
    // 末尾无换行的最后一条记录
    if !current.is_empty() || !fields.is_empty() {
        fields.push(current);
        records.push(fields);
    }

    let mut rows = Vec::new();
    let mut headers: Vec<String> = Vec::new();

    for rec in records {
        if headers.is_empty() {
            headers = rec;
            continue;
        }
        let mut row = ExportRow::new();
        for (i, val) in rec.iter().enumerate() {
            let key = headers.get(i).cloned().unwrap_or_else(|| format!("col_{}", i));
            row = row.add(&key, val);
        }
        rows.push(row);
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_roundtrip() {
        let mut exporter = DataExporter::new();
        exporter.add_row(ExportRow::new().add("name", "soma").add("desc", "video, generator"));
        exporter.add_row(ExportRow::new().add("name", "quote\"d").add("desc", "line\nbreak"));

        let csv = exporter.to_csv();

        let rows = import_csv(&csv);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].fields[0], ("name".to_string(), "soma".to_string()));
        assert_eq!(rows[0].fields[1], ("desc".to_string(), "video, generator".to_string()));
        assert_eq!(rows[1].fields[0], ("name".to_string(), "quote\"d".to_string()));
        assert_eq!(rows[1].fields[1], ("desc".to_string(), "line\nbreak".to_string()));
    }

    #[test]
    fn test_import_jsonl() {
        let rows = import_jsonl("{\"a\":1,\"b\":\"x\"}\n{\"a\":2,\"b\":\"y\"}\n");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].fields[0], ("a".to_string(), "1".to_string()));
        assert_eq!(rows[1].fields[1], ("b".to_string(), "y".to_string()));
    }

    #[test]
    fn test_export_formats() {
        let mut exporter = DataExporter::new();
        exporter.add_row(ExportRow::new().add("k", "v"));

        let json = exporter.to_json();
        assert!(json.starts_with('['));

        let jsonl = exporter.to_jsonl();
        assert!(jsonl.trim_end().ends_with('}'));
    }
}
