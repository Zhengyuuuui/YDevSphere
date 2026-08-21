//! PR2 — Technology Detection Engine 集成测试。
//!
//! 覆盖（见 `docs/V04-RECOGNITION-SPEC.md` §5，PR2 范围）：
//! - vue-project → Vue / TypeScript / Vite
//! - express-project → Node.js / Express / SQLite
//! - 藏蓝闪送前端/后端 fixture（frontend_backend）
//! - database 映射、packageManager 优先级、runtime 多来源
//! - Technology canonical id 去重（serialization）
//! - detect_stack 旧字段 language/framework 兼容
//! - 端到端：scan_workspace → DetectedProject.technologies → 落库读回
//!
//! 不涉及聚合 / 有限递归（PR3）。

mod common;

use common::{express_project, frontend_backend, vue_project};
use ydevsphere_lib::core::models::Technology;
use ydevsphere_lib::core::parser::{detect_stack, registry, ProjectMeta};

fn ids(meta: &ProjectMeta) -> Vec<&str> {
    meta.technologies.iter().map(|t| t.id.as_str()).collect()
}

/// vue-project fixture：Vue / TypeScript / Vite（+ nodejs runtime + pinia 库）。
#[test]
fn vue_project_detects_vue_typescript_vite() {
    let dir = vue_project();
    let meta = detect_stack(dir.path())
        .expect("解析应成功")
        .expect("应识别为项目");
    let got = ids(&meta);
    for expected in ["vue", "typescript", "vite", "nodejs", "pinia"] {
        assert!(got.contains(&expected), "vue-project 应识别 {expected}，实际 {got:?}");
    }
    // 旧字段兼容
    assert_eq!(meta.language.as_deref(), Some("Node"));
    assert_eq!(meta.framework.as_deref(), Some("Vue"));
}

/// express-project fixture：Node.js / Express / SQLite（better-sqlite3 映射）。
#[test]
fn express_project_detects_nodejs_express_sqlite() {
    let dir = express_project();
    let meta = detect_stack(dir.path())
        .expect("解析应成功")
        .expect("应识别为项目");
    let got = ids(&meta);
    for expected in ["nodejs", "express", "sqlite"] {
        assert!(got.contains(&expected), "express-project 应识别 {expected}，实际 {got:?}");
    }
    // 旧字段兼容：express 不在 v0.3 框架表 → framework None（行为不变）
    assert_eq!(meta.framework, None);
}

/// 藏蓝闪送前端 fixture：Vue / Pinia（+ nodejs）。
#[test]
fn canglan_frontend_stack() {
    let dir = frontend_backend();
    let meta = detect_stack(dir.path().join("frontend").as_path())
        .expect("解析应成功")
        .expect("应识别为项目");
    let got = ids(&meta);
    for expected in ["vue", "pinia", "nodejs"] {
        assert!(got.contains(&expected), "前端应识别 {expected}，实际 {got:?}");
    }
}

/// 藏蓝闪送后端 fixture：Node.js / Express / SQLite（对齐 Spec §1.4 验收示例）。
#[test]
fn canglan_backend_stack() {
    let dir = frontend_backend();
    let meta = detect_stack(dir.path().join("backend").as_path())
        .expect("解析应成功")
        .expect("应识别为项目");
    let got = ids(&meta);
    for expected in ["nodejs", "express", "sqlite"] {
        assert!(got.contains(&expected), "后端应识别 {expected}，实际 {got:?}");
    }
    // Spec §1.4 后端示例不含 JavaScript/TypeScript 语言标注
    assert!(!got.contains(&"javascript"), "后端无 TS 且无 node 脚本，不应标 JavaScript");
    assert!(!got.contains(&"typescript"));
}

/// database 映射表（Spec §5.7）经 detect_stack 端到端验证。
#[test]
fn database_mapping_end_to_end() {
    let cases: &[(&str, &str)] = &[
        ("better-sqlite3", "sqlite"),
        ("sqlite3", "sqlite"),
        ("pg", "postgresql"),
        ("postgres", "postgresql"),
        ("mysql2", "mysql"),
        ("mongoose", "mongodb"),
        ("redis", "redis"),
    ];
    for (dep, expected_id) in cases {
        let dir = common::TempDir::new("db_map");
        dir.write(
            "package.json",
            &format!(r#"{{"name":"x","dependencies":{{"{dep}":"^1.0.0"}}}}"#),
        );
        let meta = detect_stack(dir.path())
            .expect("解析应成功")
            .expect("应识别为项目");
        assert!(
            ids(&meta).contains(expected_id),
            "依赖 {dep} 应映射 {expected_id}，实际 {:?}",
            ids(&meta)
        );
    }
}

/// packageManager 优先级（§5.5）：字段 > lockfile；无字段时 lockfile 兜底。
#[test]
fn package_manager_priority_end_to_end() {
    // 场景 1：无 packageManager 字段 + pnpm-lock.yaml → pnpm（Spec §5.5 注）
    let dir = common::TempDir::new("pm_lockfile");
    dir.write("package.json", r#"{"name":"x"}"#);
    dir.write("pnpm-lock.yaml", "");
    let meta = detect_stack(dir.path()).expect("解析应成功").expect("应识别");
    assert!(ids(&meta).contains(&"pnpm"), "pnpm-lock.yaml 应识别 pnpm");

    // 场景 2：packageManager 字段与 lockfile 冲突 → 字段优先
    let dir = common::TempDir::new("pm_field");
    dir.write(
        "package.json",
        r#"{"name":"x","packageManager":"pnpm@9.1.0"}"#,
    );
    dir.write("package-lock.json", "");
    let meta = detect_stack(dir.path()).expect("解析应成功").expect("应识别");
    let got = ids(&meta);
    assert!(got.contains(&"pnpm"), "packageManager 字段应胜出");
    assert!(!got.contains(&"npm"), "冲突时不应产出 npm");
}

/// runtime 多来源（§5.6）：engines / lockfile / scripts 各来源均产出 nodejs。
#[test]
fn runtime_multi_source_end_to_end() {
    // 来源 1：engines.node
    let dir = common::TempDir::new("rt_engines");
    dir.write(
        "package.json",
        r#"{"name":"x","engines":{"node":">=18"}}"#,
    );
    let meta = detect_stack(dir.path()).expect("解析应成功").expect("应识别");
    assert!(ids(&meta).contains(&"nodejs"));

    // 来源 2：lockfile（无 engines）
    let dir = common::TempDir::new("rt_lockfile");
    dir.write("package.json", r#"{"name":"x"}"#);
    dir.write("yarn.lock", "");
    let meta = detect_stack(dir.path()).expect("解析应成功").expect("应识别");
    assert!(ids(&meta).contains(&"nodejs"));

    // 来源 3：scripts（无 engines / lockfile）
    let dir = common::TempDir::new("rt_scripts");
    dir.write(
        "package.json",
        r#"{"name":"x","scripts":{"start":"vite dev"}}"#,
    );
    let meta = detect_stack(dir.path()).expect("解析应成功").expect("应识别");
    assert!(ids(&meta).contains(&"nodejs"));
}

/// source_kind（Spec §8）：detailed API 携带来源；不落库、不展示。
#[test]
fn detection_result_carries_source_kind() {
    let dir = vue_project();
    let ctx = build_ctx_from_fixture(dir.path());
    let reg = registry::Registry::new();
    let detailed = reg.detect_detailed(&ctx);
    assert!(!detailed.is_empty());
    // vue 来自 ManifestDependency
    let vue = detailed
        .iter()
        .find(|r| r.technology.id == "vue")
        .expect("应有 vue");
    assert_eq!(vue.source_kind, registry::SourceKind::ManifestDependency);
}

/// canonical id 去重 + 序列化：registry 产出的 technologies 序列化后 id 无重复。
#[test]
fn technologies_serialization_ids_unique() {
    let dir = vue_project();
    let ctx = build_ctx_from_fixture(dir.path());
    let technologies: Vec<Technology> = registry::Registry::new().detect(&ctx);

    // 序列化 roundtrip 后 id 仍唯一
    let json = serde_json::to_string(&technologies).expect("序列化应成功");
    let back: Vec<Technology> = serde_json::from_str(&json).expect("反序列化应成功");
    let mut seen = Vec::new();
    for t in &back {
        assert!(!seen.contains(&t.id), "序列化后 id 重复: {}", t.id);
        seen.push(t.id.clone());
    }
    assert_eq!(back.len(), technologies.len());
}

/// 端到端：scan_workspace → DetectedProject.technologies → upsert 落库读回。
#[test]
fn scan_to_db_technologies_roundtrip() {
    use ydevsphere_lib::core::scanner;

    let ws = common::TempDir::new("e2e_ws");
    // 复制 vue_project 结构到工作区子目录（扫描语义：根的直接子目录）
    let vue_pkg = std::fs::read_to_string(vue_project().path().join("package.json"))
        .expect("读取 fixture 应成功");
    ws.write("my-vue-app/package.json", &vue_pkg);

    let out = scanner::scan_workspace(ws.path()).expect("扫描应成功");
    let project = out
        .projects
        .iter()
        .find(|p| p.path.ends_with("my-vue-app"))
        .expect("应识别 my-vue-app");
    let got: Vec<&str> = project.technologies.iter().map(|t| t.id.as_str()).collect();
    for expected in ["vue", "typescript", "vite", "nodejs"] {
        assert!(got.contains(&expected), "扫描应携带 {expected}，实际 {got:?}");
    }

    // 落库读回
    let db = common::memory_db();
    let inserted = db.upsert_projects(&out.projects).expect("upsert 应成功");
    let saved = inserted
        .iter()
        .find(|p| p.path.ends_with("my-vue-app"))
        .expect("应已入库");
    let saved_ids: Vec<&str> = saved.technologies.iter().map(|t| t.id.as_str()).collect();
    assert!(saved_ids.contains(&"vue"), "落库读回应含 vue");
    assert!(saved_ids.contains(&"typescript"), "落库读回应含 typescript");
}

/// 旧字段兼容：非 Node manifest（Rust / Go / Python）行为与 v0.3 一致，
/// technologies 为空列表（P1 detector 未落地）。
#[test]
fn non_node_manifests_keep_legacy_behavior() {
    let cases: &[(&str, &str, &str)] = &[
        ("Cargo.toml", "[package]\nname = \"x\"\n", "Rust"),
        ("go.mod", "module example.com/x\n", "Go"),
        ("pyproject.toml", "[project]\nname = \"x\"\n", "Python"),
        ("requirements.txt", "requests==2.31.0\n", "Python"),
    ];
    for (file, content, language) in cases {
        let dir = common::TempDir::new("legacy_manifest");
        dir.write(file, content);
        let meta = detect_stack(dir.path())
            .expect("解析应成功")
            .expect("应识别为项目");
        assert_eq!(meta.language.as_deref(), Some(*language), "{file} language 应为 {language}");
        assert_eq!(meta.framework, None);
        assert!(meta.technologies.is_empty(), "{file} PR2 阶段 technologies 应为空");
    }
}

/// 从 fixture 目录构建 DetectorContext（集成层走 node.rs 的 context 构建逻辑，
/// 此处经 detect_stack 间接覆盖；本函数直接构造以测 detailed API）。
fn build_ctx_from_fixture(path: &std::path::Path) -> registry::DetectorContext {
    let raw = std::fs::read_to_string(path.join("package.json")).expect("读取 package.json");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("解析 package.json");
    let mut deps = Vec::new();
    for key in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(map) = value.get(key).and_then(|v| v.as_object()) {
            deps.extend(map.keys().cloned());
        }
    }
    registry::DetectorContext {
        dependencies: deps,
        engines_node: value.get("engines").and_then(|e| e.get("node")).is_some(),
        package_manager_field: value
            .get("packageManager")
            .and_then(|v| v.as_str())
            .map(String::from),
        lockfiles: {
            let mut out = Vec::new();
            if path.join("pnpm-lock.yaml").is_file() {
                out.push(registry::PackageManagerId::Pnpm);
            }
            if path.join("package-lock.json").is_file() {
                out.push(registry::PackageManagerId::Npm);
            }
            if path.join("yarn.lock").is_file() {
                out.push(registry::PackageManagerId::Yarn);
            }
            if path.join("bun.lock").is_file() || path.join("bun.lockb").is_file() {
                out.push(registry::PackageManagerId::Bun);
            }
            out
        },
        script_commands: value
            .get("scripts")
            .and_then(|v| v.as_object())
            .map(|s| {
                s.values()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        has_typescript_config: path.join("tsconfig.json").is_file(),
    }
}
