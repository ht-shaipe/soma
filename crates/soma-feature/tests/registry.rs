//! FeatureRegistry 集成测试：注册、描述、运行、进度、失败路径与历史记录

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use soma_core::config::AppConfig;
use soma_core::error::SomaError;
use soma_feature::{
    ArtifactKind, FeatureContext, FeatureKind, FeatureMeta, FeatureRegistry, FeatureRequest,
    FnReporter, NoopProgress, ProgressReporter, TypedFeature,
};
use std::sync::{Arc, Mutex};

// ---------- 测试用功能点 ----------

#[derive(Debug, Deserialize, JsonSchema)]
struct EchoInput {
    text: String,
    #[serde(default)]
    repeat: u32,
}

#[derive(Debug, Serialize, JsonSchema)]
struct EchoOutput {
    text: String,
    repeat: u32,
}

/// 正常功能点：上报进度 + 写产物文件 + 回显输入
struct EchoFeature;

impl TypedFeature for EchoFeature {
    type Input = EchoInput;
    type Output = EchoOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "test.echo".into(),
            name: "回声测试".into(),
            description: "回显输入文本并产出 echo.txt".into(),
            kind: FeatureKind::Utility,
        }
    }

    fn run(
        &self,
        ctx: &mut FeatureContext,
        input: EchoInput,
        progress: &dyn ProgressReporter,
    ) -> Result<EchoOutput, SomaError> {
        progress.report(
            soma_feature::FeatureProgress::new(ctx.feature_id(), ctx.run_id(), 50)
                .with_step("echo")
                .with_message("处理中"),
        );
        let path = ctx.artifact_path("echo.txt");
        std::fs::write(&path, &input.text)?;
        ctx.add_artifact("echo.txt", &path, ArtifactKind::Text);
        Ok(EchoOutput {
            text: input.text,
            repeat: input.repeat.max(1),
        })
    }
}

/// 直接返回 Err 的功能点
struct FailFeature;

impl TypedFeature for FailFeature {
    type Input = EchoInput;
    type Output = EchoOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "test.fail".into(),
            name: "失败测试".into(),
            description: "总是返回错误".into(),
            kind: FeatureKind::Utility,
        }
    }

    fn run(
        &self,
        _ctx: &mut FeatureContext,
        _input: EchoInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<EchoOutput, SomaError> {
        Err(SomaError::Config("故意的失败".into()))
    }
}

/// panic 的功能点
struct PanicFeature;

impl TypedFeature for PanicFeature {
    type Input = EchoInput;
    type Output = EchoOutput;

    fn meta(&self) -> FeatureMeta {
        FeatureMeta {
            id: "test.panic".into(),
            name: "panic 测试".into(),
            description: "执行时 panic".into(),
            kind: FeatureKind::Utility,
        }
    }

    fn run(
        &self,
        _ctx: &mut FeatureContext,
        _input: EchoInput,
        _progress: &dyn ProgressReporter,
    ) -> Result<EchoOutput, SomaError> {
        panic!("boom");
    }
}

// ---------- 辅助 ----------

/// 创建带随机产物根目录的注册表，并返回（registry, 根目录路径）
fn test_registry() -> (FeatureRegistry, std::path::PathBuf) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("soma-feature-test-{}", nanos));
    std::fs::create_dir_all(&root).unwrap();
    (FeatureRegistry::with_storage_root(root.clone()), root)
}

/// 抑制 panic 测试时的默认 panic 输出噪音由测试框架处理，这里无需额外处理
fn echo_request(input: serde_json::Value) -> FeatureRequest {
    FeatureRequest {
        feature_id: "test.echo".into(),
        run_id: None,
        input,
    }
}

// ---------- 测试 ----------

#[test]
fn register_and_duplicate_rejected() {
    let (registry, _root) = test_registry();
    registry.register(Arc::new(EchoFeature)).unwrap();
    let err = registry.register(Arc::new(EchoFeature)).unwrap_err();
    assert!(err.to_string().contains("重复注册"), "unexpected: {err}");
    assert!(registry.get("test.echo").is_some());
    assert!(registry.get("test.missing").is_none());
}

#[test]
fn list_contains_descriptor_with_schemas() {
    let (registry, _root) = test_registry();
    registry.register(Arc::new(EchoFeature)).unwrap();
    let list = registry.list();
    assert_eq!(list.len(), 1);
    let desc = &list[0];
    assert_eq!(desc.id, "test.echo");
    assert_eq!(desc.name, "回声测试");
    assert_eq!(desc.kind, FeatureKind::Utility);
    // TypedFeature 的输入输出 Schema 自动生成
    let input_schema = desc.input_schema.as_ref().expect("input schema 应存在");
    assert_eq!(input_schema["type"], "object");
    assert!(input_schema["properties"]["text"].is_object());
    assert!(desc.output_schema.is_some());
}

#[test]
fn run_success_produces_output_artifacts_and_records() {
    let (registry, root) = test_registry();
    registry.register(Arc::new(EchoFeature)).unwrap();

    let req = echo_request(serde_json::json!({ "text": "你好世界", "repeat": 3 }));
    let output = registry
        .run(&req, AppConfig::default(), &NoopProgress)
        .unwrap();

    assert_eq!(output.status, soma_feature::FeatureStatus::Success);
    assert_eq!(output.data["text"], "你好世界");
    assert_eq!(output.data["repeat"], 3);
    assert!(output.error.is_none());
    // 入参回显
    assert_eq!(output.input["text"], "你好世界");
    // 耗时字段有效
    assert!(output.finished_at >= output.started_at);

    // 产物登记与文件
    assert_eq!(output.artifacts.len(), 1);
    let artifact = &output.artifacts[0];
    assert_eq!(artifact.name, "echo.txt");
    assert_eq!(artifact.kind, ArtifactKind::Text);
    assert!(artifact.size > 0, "产物文件大小应大于 0");
    assert!(std::path::Path::new(&artifact.path).exists());

    // 产物目录归档：echo.txt / input.json / run_record.json
    let run_dir = root.join("test.echo").join(&output.run_id);
    assert!(run_dir.join("echo.txt").exists());
    assert!(run_dir.join("input.json").exists());
    assert!(run_dir.join("run_record.json").exists());

    // run_record.json 可反序列化且与返回值一致
    let record = std::fs::read_to_string(run_dir.join("run_record.json")).unwrap();
    let parsed: soma_feature::FeatureOutput = serde_json::from_str(&record).unwrap();
    assert_eq!(parsed.run_id, output.run_id);
    assert_eq!(parsed.status, soma_feature::FeatureStatus::Success);
}

#[test]
fn run_emits_progress_events() {
    let (registry, _root) = test_registry();
    registry.register(Arc::new(EchoFeature)).unwrap();

    let events: Arc<Mutex<Vec<soma_feature::FeatureProgress>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    let reporter = FnReporter(move |p: soma_feature::FeatureProgress| {
        sink.lock().unwrap().push(p);
    });

    let req = echo_request(serde_json::json!({ "text": "hi" }));
    let output = registry.run(&req, AppConfig::default(), &reporter).unwrap();

    let got = events.lock().unwrap();
    assert_eq!(got.len(), 1);
    let event = &got[0];
    assert_eq!(event.feature_id, "test.echo");
    assert_eq!(event.run_id, output.run_id);
    assert_eq!(event.percent, 50);
    assert_eq!(event.step.as_deref(), Some("echo"));
    assert_eq!(event.message.as_deref(), Some("处理中"));
}

#[test]
fn run_unknown_feature_returns_err() {
    let (registry, _root) = test_registry();
    let req = FeatureRequest {
        feature_id: "test.nope".into(),
        run_id: None,
        input: serde_json::json!({}),
    };
    let err = registry
        .run(&req, AppConfig::default(), &NoopProgress)
        .unwrap_err();
    assert!(
        err.to_string().contains("功能点不存在"),
        "unexpected: {err}"
    );
}

#[test]
fn run_invalid_input_records_failure() {
    let (registry, root) = test_registry();
    registry.register(Arc::new(EchoFeature)).unwrap();

    // 缺少必填字段 text → 反序列化失败 → 状态 Failed
    let req = echo_request(serde_json::json!({ "repeat": 2 }));
    let output = registry
        .run(&req, AppConfig::default(), &NoopProgress)
        .unwrap();
    assert_eq!(output.status, soma_feature::FeatureStatus::Failed);
    assert!(output.error.is_some());
    assert!(output.data.is_null());

    // 失败同样持久化运行记录
    let record_path = root
        .join("test.echo")
        .join(&output.run_id)
        .join("run_record.json");
    let record = std::fs::read_to_string(record_path).unwrap();
    let parsed: soma_feature::FeatureOutput = serde_json::from_str(&record).unwrap();
    assert_eq!(parsed.status, soma_feature::FeatureStatus::Failed);
}

#[test]
fn run_feature_error_records_failure() {
    let (registry, _root) = test_registry();
    registry.register(Arc::new(FailFeature)).unwrap();

    let req = FeatureRequest {
        feature_id: "test.fail".into(),
        run_id: None,
        input: serde_json::json!({ "text": "x" }),
    };
    let output = registry
        .run(&req, AppConfig::default(), &NoopProgress)
        .unwrap();
    assert_eq!(output.status, soma_feature::FeatureStatus::Failed);
    assert!(output.error.unwrap().contains("故意的失败"));
}

#[test]
fn run_panicking_feature_is_caught() {
    let (registry, _root) = test_registry();
    registry.register(Arc::new(PanicFeature)).unwrap();

    let req = FeatureRequest {
        feature_id: "test.panic".into(),
        run_id: None,
        input: serde_json::json!({ "text": "x" }),
    };
    let output = registry
        .run(&req, AppConfig::default(), &NoopProgress)
        .unwrap();
    assert_eq!(output.status, soma_feature::FeatureStatus::Failed);
    assert!(output.error.unwrap().contains("panic"));
}

#[test]
fn list_runs_returns_records_sorted_desc() {
    let (registry, _root) = test_registry();
    registry.register(Arc::new(EchoFeature)).unwrap();

    for text in ["第一次", "第二次"] {
        let req = echo_request(serde_json::json!({ "text": text }));
        registry
            .run(&req, AppConfig::default(), &NoopProgress)
            .unwrap();
    }

    let runs = registry.list_runs("test.echo").unwrap();
    assert_eq!(
        runs.len(),
        2,
        "应恰有 2 条运行记录，实际 {}：{:?}",
        runs.len(),
        runs.iter()
            .map(|r| (&r.run_id, &r.input))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        (&runs[0].input["text"], &runs[1].input["text"]),
        (&serde_json::json!("第二次"), &serde_json::json!("第一次")),
        "倒序排序不应颠倒两次运行的先后（run_id: {:#?}）",
        runs.iter().map(|r| &r.run_id).collect::<Vec<_>>()
    );

    // 未运行过的功能点返回空列表而非错误
    assert!(registry.list_runs("test.empty").unwrap().is_empty());
}

#[test]
fn percent_is_clamped() {
    let p = soma_feature::FeatureProgress::new("f", "r", 150);
    assert_eq!(p.percent, 100);
    let p = soma_feature::FeatureProgress::new("f", "r", 0);
    assert_eq!(p.percent, 0);
}
