//! PR3 — Boundary + Aggregate Recognition 集成测试。
//!
//! 覆盖（见 `docs/V04-RECOGNITION-SPEC.md` §2/§4/§6.2/§13，PR3 范围）：
//! - 藏蓝闪送 frontend_backend：聚合根 + 2 子项目 + 父级 derived 聚合（§2.3）。
//! - root_plus_children：根有 package.json + frontend/backend 子项目（§4.1/§13）。
//! - 候选目录误报防护：docs/app/ 不判为子项目（§4.3）。
//! - 递归深度：非 workspace 不深入第三层（§4.2）。
//! - parent_id 落库读回（树恢复，§6.2）。
//! - 聚合规则：derived = Union(children.technologies) 按 id 去重（§2.2）。
//! - 单项目不回归：普通单项目 / 根有 manifest 无子项目行为保持 v0.2。

mod common;

use common::{frontend_backend, memory_db};
use ydevsphere_lib::core::models::{DetectedProject, ProjectKind, Technology};
use ydevsphere_lib::core::scanner;

fn tech_ids(project: &DetectedProject) -> Vec<&str> {
    project.technologies.iter().map(|t| t.id.as_str()).collect()
}

/// 藏蓝闪送验收（Spec §2.3/§14）：frontend_backend fixture →
/// 聚合根（kind=aggregated_root）+ 2 子项目 + 父级 derived 并集。
#[test]
fn canglan_fast_delivery_aggregate() {
    let dir = frontend_backend();
    // 工作区根 = 藏蓝闪送（含 workspaces + frontend/backend）
    let out = scanner::scan_workspace(dir.path()).expect("扫描应成功");

    // 根有 package.json（workspaces）→ 两阶段 → 聚合根
    let agg = out
        .projects
        .iter()
        .find(|p| p.kind == ProjectKind::AggregatedRoot)
        .expect("应有聚合根");
    assert_eq!(agg.path, dir.path_str());

    // 2 个子项目（frontend / backend）
    let children: Vec<_> = out
        .projects
        .iter()
        .filter(|p| p.parent_path.as_deref() == Some(agg.path.as_str()))
        .collect();
    assert_eq!(children.len(), 2, "聚合根应有 2 个子项目");
    assert!(children.iter().all(|c| c.kind == ProjectKind::Real));

    // 子项目各自技术栈（Source of Truth）
    let frontend = children.iter().find(|c| c.path.ends_with("/frontend")).expect("应有 frontend");
    let fe = tech_ids(frontend);
    assert!(fe.contains(&"vue"), "前端应含 vue，实际 {fe:?}");
    assert!(fe.contains(&"pinia"), "前端应含 pinia，实际 {fe:?}");

    let backend = children.iter().find(|c| c.path.ends_with("/backend")).expect("应有 backend");
    let be = tech_ids(backend);
    assert!(be.contains(&"express"), "后端应含 express，实际 {be:?}");
    assert!(be.contains(&"sqlite"), "后端应含 sqlite（better-sqlite3），实际 {be:?}");

    // 父级 derived = Union(children)（Spec §2.2），按 canonical id 去重
    let agg_ids = tech_ids(agg);
    assert!(agg_ids.contains(&"vue"), "derived 应含 vue");
    assert!(agg_ids.contains(&"express"), "derived 应含 express");
    assert!(agg_ids.contains(&"sqlite"), "derived 应含 sqlite");
    // 去重：vue 只出现一次
    assert_eq!(
        agg_ids.iter().filter(|id| **id == "vue").count(),
        1,
        "derived 中 vue 应去重"
    );
}

/// 根有 package.json + frontend/backend 子项目（Spec §4.1/§13 root-plus-children）。
/// 根有 manifest 也要继续找子项目；根升格为聚合根，derived 聚合。
#[test]
fn root_plus_children_aggregates() {
    let dir = common::TempDir::new("root_plus_children");
    // 根自身含 package.json（阶段 A：根自身技术栈）
    dir.write(
        "package.json",
        r#"{"name":"monorepo-root","private":true,"dependencies":{"lerna":"^8.0.0"}}"#,
    );
    dir.write(
        "frontend/package.json",
        r#"{"name":"frontend","dependencies":{"vue":"^3.4.0"}}"#,
    );
    dir.write(
        "backend/package.json",
        r#"{"name":"backend","dependencies":{"express":"^4.18.0"}}"#,
    );

    let out = scanner::scan_workspace(dir.path()).expect("扫描应成功");

    // 根有 manifest + 2 子项目 → 聚合根
    let agg = out
        .projects
        .iter()
        .find(|p| p.path == dir.path_str())
        .expect("根应被识别");
    assert_eq!(agg.kind, ProjectKind::AggregatedRoot, "根有 ≥2 子项目应升格聚合根");

    let children: Vec<_> = out
        .projects
        .iter()
        .filter(|p| p.parent_path.as_deref() == Some(dir.path_str().as_str()))
        .collect();
    assert_eq!(children.len(), 2);

    // derived = Union(frontend vue, backend express)
    let agg_ids = tech_ids(agg);
    assert!(agg_ids.contains(&"vue"));
    assert!(agg_ids.contains(&"express"));
}

/// 根有 manifest + 无子项目（src/ 非候选目录）→ 保持 Real，不拆内部（v0.2 回归）。
#[test]
fn root_without_subprojects_stays_real() {
    let dir = common::TempDir::new("root_plain");
    dir.write("package.json", r#"{"name":"plain-app","dependencies":{"vue":"^3.4.0"}}"#);
    dir.mkdir("src");
    dir.write("src/package.json", r#"{"name":"nested-in-src"}"#);

    let out = scanner::scan_workspace(dir.path()).expect("扫描应成功");
    assert_eq!(out.projects.len(), 1, "根有 manifest 无候选子项目应只识别根");
    assert_eq!(out.projects[0].kind, ProjectKind::Real);
    assert!(tech_ids(&out.projects[0]).contains(&"vue"));
}

/// 候选目录误报防护（§4.3）：docs/app/ 中 app 无 manifest → 不判为子项目。
#[test]
fn candidate_dir_requires_manifest_or_workspace() {
    let dir = common::TempDir::new("docs_app_fp");
    // docs/app 只有空目录，无 manifest / workspace signal
    dir.mkdir("docs/app");
    // 一个真实候选子项目（frontend 有 manifest）+ docs（含 app 但 app 无 manifest）
    dir.write(
        "frontend/package.json",
        r#"{"name":"frontend","dependencies":{"vue":"^3.0.0"}}"#,
    );
    dir.write(
        "backend/package.json",
        r#"{"name":"backend","dependencies":{"express":"^4.0.0"}}"#,
    );

    let out = scanner::scan_workspace(dir.path()).expect("扫描应成功");

    // app 无 manifest → 不应作为项目
    assert!(
        out.projects.iter().all(|p| !p.path.ends_with("/docs/app") && !p.path.ends_with("/app")),
        "docs/app 无 manifest 不应判为子项目"
    );
    // 但 frontend/backend 正常识别
    assert!(out.projects.iter().any(|p| p.path.ends_with("/frontend")));
    assert!(out.projects.iter().any(|p| p.path.ends_with("/backend")));
}

/// 递归深度（§4.2）：第三层（depth >= 2）仅当有 workspace/monorepo 信号才继续深入。
/// 结构：聚合根 agg（无 manifest，无 workspace 信号）→ frontend/backend 真项目 +
/// shared 分类目录（其内 lib 是第三层候选）。无 workspace 信号 → lib 不识别。
#[test]
fn recursion_limited_to_second_level_without_workspace() {
    let dir = common::TempDir::new("depth_limit");
    // 聚合根 agg（无 manifest）：frontend + backend 两个真项目 + shared 分类目录
    dir.write(
        "agg/frontend/package.json",
        r#"{"name":"frontend","dependencies":{"vue":"^3.0.0"}}"#,
    );
    dir.write(
        "agg/backend/package.json",
        r#"{"name":"backend","dependencies":{"express":"^4.0.0"}}"#,
    );
    // shared 是分类目录（无 manifest），其内部 lib 是第三层候选（有 manifest）
    dir.write(
        "agg/shared/lib/package.json",
        r#"{"name":"lib"}"#,
    );

    let out = scanner::scan_workspace(dir.path()).expect("扫描应成功");

    // 聚合根 agg 被识别（frontend + backend 2 真项目）
    let agg = out
        .projects
        .iter()
        .find(|p| p.path.ends_with("/agg"))
        .expect("应有聚合根 agg");
    assert_eq!(agg.kind, ProjectKind::AggregatedRoot);

    // 第三层 lib 无 workspace 信号 → 不应被识别
    assert!(
        out.projects.iter().all(|p| !p.path.ends_with("/lib") && !p.path.contains("shared/lib")),
        "非 workspace 不应深入第三层识别 lib"
    );
    // 项目：agg(聚合根) + frontend + backend + shared(分类目录)
    assert!(out.projects.iter().any(|p| p.path.ends_with("/shared")));
}

/// parent_id 落库读回（§6.2 树恢复）：聚合根下子项目 parent_id = 聚合根.id，
/// 顶层 parent_id = NULL；technologies_json 落库读回。
#[test]
fn parent_id_persisted_and_tree_recoverable() {
    let dir = frontend_backend();
    let out = scanner::scan_workspace(dir.path()).expect("扫描应成功");

    let db = memory_db();
    let _inserted = db.upsert_projects(&out.projects).expect("upsert 应成功");

    // 默认只返回顶层（聚合根，parent_id NULL）
    let tops = db.get_projects(None, None, None, None).expect("读取应成功");
    assert_eq!(tops.len(), 1);
    assert_eq!(tops[0].kind, ProjectKind::AggregatedRoot);
    assert!(tops[0].parent_id.is_none(), "顶层聚合根 parent_id 应为 NULL");

    // 经 parent_id_filter = 聚合根.id 恢复子项目（树恢复）
    let children = db
        .get_projects(None, None, None, Some(tops[0].id))
        .expect("读取子项目应成功");
    assert_eq!(children.len(), 2, "树应从 parent_id 恢复出 2 子项目");
    for c in &children {
        assert_eq!(c.parent_id, Some(tops[0].id), "子项目 parent_id 应指向聚合根");
    }

    // 子项目 technologies 落库读回（Source of Truth 保持）
    let backend = children.iter().find(|c| c.path.ends_with("/backend")).expect("应有 backend");
    let be: Vec<&str> = backend.technologies.iter().map(|t| t.id.as_str()).collect();
    assert!(be.contains(&"express"), "后端 technologies 落库读回应含 express");
    assert!(be.contains(&"sqlite"));

    // 聚合根 technologies（derived）落库读回
    let agg_ids: Vec<&str> = tops[0].technologies.iter().map(|t| t.id.as_str()).collect();
    assert!(agg_ids.contains(&"vue"));
    assert!(agg_ids.contains(&"express"));
}

/// 聚合 derived 去重：多个子项目共享同一技术（frontend/backend 都有 nodejs），
/// 聚合根 derived 中 nodejs 只出现一次（Spec §1.2 按 id 去重）。
#[test]
fn derived_dedupes_shared_technologies() {
    let dir = common::TempDir::new("derived_dedupe");
    // 聚合根目录（无 manifest）下两个 Node 子项目都含 nodejs runtime
    dir.write(
        "agg/frontend/package.json",
        r#"{"name":"frontend","dependencies":{"vue":"^3.0.0"},"engines":{"node":">=18"}}"#,
    );
    dir.write(
        "agg/backend/package.json",
        r#"{"name":"backend","dependencies":{"express":"^4.0.0"},"engines":{"node":">=18"}}"#,
    );

    let out = scanner::scan_workspace(dir.path()).expect("扫描应成功");
    let agg = out
        .projects
        .iter()
        .find(|p| p.kind == ProjectKind::AggregatedRoot)
        .expect("应有聚合根");
    let agg_ids = tech_ids(agg);
    assert_eq!(
        agg_ids.iter().filter(|id| **id == "nodejs").count(),
        1,
        "derived 中 nodejs 应去重，实际 {agg_ids:?}"
    );
}

/// 普通单目录（无 manifest、无子项目）不回归：空目录不识别。
#[test]
fn plain_empty_dir_not_detected() {
    let dir = common::TempDir::new("plain_empty");
    let out = scanner::scan_workspace(dir.path()).expect("扫描应成功");
    assert!(out.projects.is_empty());
}

/// 单项目（普通真项目目录作为工作区根）不回归：识别为 Real。
#[test]
fn single_real_project_no_regression() {
    let dir = common::TempDir::new("single_real");
    dir.write("package.json", r#"{"name":"solo","dependencies":{"vite":"^5.0.0"}}"#);
    dir.mkdir("src");
    let out = scanner::scan_workspace(dir.path()).expect("扫描应成功");
    assert_eq!(out.projects.len(), 1);
    assert_eq!(out.projects[0].kind, ProjectKind::Real);
    assert!(tech_ids(&out.projects[0]).contains(&"vite"));
}

// 辅助：确保 Technology 类型被引用（避免未使用导入）
#[allow(dead_code)]
fn _type_ref(_t: &Technology) {}
