//! PR1 — Recognition Model 集成测试。
//!
//! 覆盖（见 `docs/V04-RECOGNITION-SPEC.md` §12 PR1）：
//! - Technology / TechnologiesJson 序列化 roundtrip。
//! - ProjectKind serde（沿用既有 Real/AggregatedRoot/Category 契约）。
//! - frontend-backend fixture 构造。
//! - parent_id / kind / technologies_json 落库与读回。
//!
//! 本文件只做「模型 + 迁移 + fixture」验证，不涉及识别规则（PR2）。

mod common;

use common::{frontend_backend, memory_db};
use ydevsphere_lib::core::models::{
    DetectedProject, ProjectKind, TechnologiesJson, Technology, TechnologyCategory,
};

fn vue() -> Technology {
    Technology::new(
        "vue",
        "Vue",
        TechnologyCategory::Framework,
        Some("javascript".to_string()),
    )
}

fn node_runtime() -> Technology {
    Technology::new(
        "nodejs",
        "Node.js",
        TechnologyCategory::Runtime,
        None,
    )
}

/// Technology 序列化/反序列化 roundtrip（含 category snake_case、ecosystem 可空）。
#[test]
fn technology_roundtrip() {
    for tech in [vue(), node_runtime()] {
        let encoded = serde_json::to_string(&tech).expect("序列化应成功");
        let decoded: Technology = serde_json::from_str(&encoded).expect("反序列化应成功");
        assert_eq!(decoded, tech);
    }
}

/// technologies_json 含 schema_version 的序列化 roundtrip。
#[test]
fn technologies_json_roundtrip_with_schema_version() {
    let tj = TechnologiesJson::new(vec![vue(), node_runtime()]);
    let encoded = tj.encode();
    let value: serde_json::Value = serde_json::from_str(&encoded).expect("应为合法 JSON");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["technologies"].as_array().map(|a| a.len()), Some(2));

    let decoded = TechnologiesJson::decode(Some(&encoded));
    assert_eq!(decoded, tj);
}

/// ProjectKind serde：沿用既有 snake_case 契约（real/aggregated_root/category）。
#[test]
fn project_kind_serde() {
    for (kind, expected) in [
        (ProjectKind::Real, "real"),
        (ProjectKind::AggregatedRoot, "aggregated_root"),
        (ProjectKind::Category, "category"),
    ] {
        let s = serde_json::to_string(&kind).expect("序列化应成功");
        assert_eq!(s, format!("\"{expected}\""));
        let back: ProjectKind = serde_json::from_str(&s).expect("反序列化应成功");
        assert_eq!(back, kind);
    }
}

/// frontend-backend fixture：构造「藏蓝闪送」结构（根 + frontend/backend 子项目）。
#[test]
fn frontend_backend_fixture_builds() {
    let dir = frontend_backend();
    for rel in ["package.json", "frontend/package.json", "backend/package.json"] {
        assert!(
            dir.path().join(rel).is_file(),
            "fixture 应包含 {rel}"
        );
    }
    // 根含 workspaces 信号（PR3 依赖）
    let raw = std::fs::read_to_string(dir.path().join("package.json")).expect("读取应成功");
    assert!(raw.contains("workspaces"), "根应含 workspace 信号");
}

/// parent_id / kind / technologies_json 落库与读回（聚合根 + 子项目）。
#[test]
fn db_roundtrip_parent_kind_technologies() {
    let db = memory_db();

    // 聚合根（无技术）
    let agg = DetectedProject::new_with_kind(
        "agg",
        "/tmp/fb/agg",
        None,
        None,
        None,
        ProjectKind::AggregatedRoot,
        30,
        None,
    );

    // 前端子项目：携带 technologies + parent 指向聚合根
    let mut frontend = DetectedProject::new_with_kind(
        "frontend",
        "/tmp/fb/agg/frontend",
        Some("Node".into()),
        Some("Vue".into()),
        None,
        ProjectKind::Real,
        70,
        Some("/tmp/fb/agg".to_string()),
    );
    frontend.technologies = vec![vue()];

    // 后端子项目：Express + SQLite
    let mut backend = DetectedProject::new_with_kind(
        "backend",
        "/tmp/fb/agg/backend",
        Some("Node".into()),
        Some("Express".into()),
        None,
        ProjectKind::Real,
        65,
        Some("/tmp/fb/agg".to_string()),
    );
    backend.technologies = vec![
        node_runtime(),
        Technology::new(
            "express",
            "Express",
            TechnologyCategory::Framework,
            Some("javascript".into()),
        ),
        Technology::new(
            "sqlite",
            "SQLite",
            TechnologyCategory::Database,
            None,
        ),
    ];

    db.upsert_projects(&[agg, frontend, backend]).expect("upsert 应成功");

    // 默认只返回顶层（聚合根）
    let tops = db.get_projects(None, None, None, None).expect("读取应成功");
    assert_eq!(tops.len(), 1);
    assert_eq!(tops[0].kind, ProjectKind::AggregatedRoot);
    assert!(tops[0].parent_id.is_none());
    assert!(tops[0].technologies.is_empty());

    // 子项目经 parent_id 获取：kind / parent_id / technologies 均落库读回
    let children = db
        .get_projects(None, None, None, Some(tops[0].id))
        .expect("读取应成功");
    assert_eq!(children.len(), 2);
    let fe = children
        .iter()
        .find(|p| p.path.ends_with("/frontend"))
        .expect("应有 frontend");
    assert_eq!(fe.kind, ProjectKind::Real);
    assert_eq!(fe.parent_id, Some(tops[0].id));
    assert_eq!(fe.technologies, vec![vue()]);
    // 旧字段兼容
    assert_eq!(fe.language.as_deref(), Some("Node"));
    assert_eq!(fe.framework.as_deref(), Some("Vue"));

    let be = children
        .iter()
        .find(|p| p.path.ends_with("/backend"))
        .expect("应有 backend");
    assert_eq!(be.technologies.len(), 3);
    assert!(be.technologies.iter().any(|t| t.id == "sqlite"));
    assert!(be.technologies.iter().any(|t| t.id == "express"));
}
