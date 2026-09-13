//! 内置功能点冒烟测试：注册、Schema 完整性与无需外部服务的运行路径

use soma_core::config::AppConfig;
use soma_feature::{FeatureRegistry, FeatureRequest, NoopProgress};

/// 创建带随机产物根目录的注册表
fn test_registry() -> (FeatureRegistry, std::path::PathBuf) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("soma-feature-builtin-{}", nanos));
    std::fs::create_dir_all(&root).unwrap();
    (FeatureRegistry::with_storage_root(root.clone()), root)
}

/// 流水线全部内置功能点可注册，且描述携带输入输出 Schema
#[test]
fn builtin_features_register_and_list() {
    let (registry, _root) = test_registry();
    soma_feature::features::register_builtin(&registry).unwrap();

    let ids: Vec<String> = registry.list().into_iter().map(|d| d.id).collect();
    for expected in [
        "llm.intent",
        "llm.script",
        "llm.storyboard",
        "llm.narration",
        "tts.synthesize",
        "subtitle.generate",
        "material.generate",
        "material.search",
        "material.download",
        "aivideo.generate",
        "video.compose",
        "llm.terms",
        "llm.social",
        "audio.concat",
        "video.concat",
        "video.render",
        "video.watermark",
        "video.join",
        "video.transition",
        "video.clip_resize",
        "video.info",
    ] {
        assert!(ids.iter().any(|id| id == expected), "缺少功能点 {expected}");
    }

    // TypedFeature 桥接层应自动生成输入输出 Schema
    for d in registry.list() {
        assert!(d.input_schema.is_some(), "{} 缺少 input schema", d.id);
        assert!(d.output_schema.is_some(), "{} 缺少 output schema", d.id);
    }
}

/// subtitle.generate 禁用字幕时返回空字符串（无需外部服务）
#[test]
fn subtitle_generate_disabled_returns_empty() {
    let (registry, _root) = test_registry();
    soma_feature::features::register_builtin(&registry).unwrap();

    let req = FeatureRequest {
        feature_id: "subtitle.generate".into(),
        run_id: None,
        input: serde_json::json!({
            "audio_file": "nonexistent.mp3",
            "text": "测试文本",
            "subtitle_enabled": false
        }),
    };
    let output = registry.run(&req, AppConfig::default(), &NoopProgress).unwrap();
    assert_eq!(output.status, soma_feature::FeatureStatus::Success);
    assert_eq!(output.data["subtitle_path"], "");
}

/// 入参校验：缺必填字段的 material.generate 调用失败并落盘失败记录
#[test]
fn material_generate_missing_terms_fails_gracefully() {
    let (registry, root) = test_registry();
    soma_feature::features::register_builtin(&registry).unwrap();

    let req = FeatureRequest {
        feature_id: "material.generate".into(),
        run_id: None,
        input: serde_json::json!({ "source": "pexels" }), // 缺少必填 terms
    };
    let output = registry.run(&req, AppConfig::default(), &NoopProgress).unwrap();
    assert_eq!(output.status, soma_feature::FeatureStatus::Failed);
    assert!(output.error.is_some());

    let record_path = root
        .join("material.generate")
        .join(&output.run_id)
        .join("run_record.json");
    assert!(record_path.exists(), "失败运行也应落盘运行记录");
}
