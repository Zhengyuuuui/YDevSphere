//! Infrastructure detector（Spec §5.5/§5.6，PR2）。
//!
//! - **Package Manager 优先级（§5.5，P1-8）**：
//!   1. `package.json` 的 `packageManager` 字段（最高优先级，如 `"pnpm@9.1.0"`）
//!   2. lockfile：`pnpm-lock.yaml` → pnpm；`package-lock.json` → npm；
//!      `yarn.lock` → yarn；`bun.lock` / `bun.lockb` → bun
//!   > 没写 packageManager + 存在 pnpm-lock.yaml → 仍识别为 pnpm。
//!
//! - **Runtime 多来源（§5.6，P1-9）**：Node.js 不能只靠 `engines.node`，
//!   综合判定 `engines.node` + Node-specific manifest/dependencies + lockfile/scripts。
//!   V0.4 只定义**来源优先级**（不做 Evidence 数值）：
//!   `engines.node`（ManifestField）> lockfile（Lockfile）> scripts（Script）。
//!
//! 说明：本 detector 仅在 package.json manifest 存在时被调用（由 `node.rs`
//! 构建 context），故 Node.js runtime 的产出前提即「存在 Node manifest」。

use crate::core::models::TechnologyCategory as Cat;

use super::super::registry::{
    tech, DetectionResult, Detector, DetectorContext, PackageManagerId, SourceKind,
};

/// Infrastructure detector（packageManager + runtime）。
pub struct InfrastructureDetector;

impl Detector for InfrastructureDetector {
    fn name(&self) -> &'static str {
        "infrastructure"
    }

    fn detect(&self, ctx: &DetectorContext) -> Vec<DetectionResult> {
        let mut results = Vec::new();

        // ---- Package Manager（§5.5 优先级）----
        if let Some(pm) = detect_package_manager(ctx) {
            results.push(DetectionResult::new(
                tech(
                    pm.id.canonical_id(),
                    pm.id.display_name(),
                    Cat::PackageManager,
                    Some("javascript"),
                ),
                pm.source_kind,
            ));
        }

        // ---- Runtime：Node.js 多来源（§5.6）----
        // 来源优先级（仅决定 source_kind，不做 evidence 数值）：
        // engines.node > lockfile > scripts > manifest 本身。
        let source = if ctx.engines_node {
            SourceKind::ManifestField
        } else if !ctx.lockfiles.is_empty() {
            SourceKind::Lockfile
        } else if !ctx.script_commands.is_empty() {
            SourceKind::Script
        } else {
            SourceKind::ManifestField
        };
        // ecosystem 为 None（Node.js 即运行时本身，对齐 Spec §1.4 示例）。
        results.push(DetectionResult::new(
            tech("nodejs", "Node.js", Cat::Runtime, None),
            source,
        ));

        results
    }
}

/// Package Manager 识别结果（含命中来源）。
struct PackageManagerHit {
    id: PackageManagerId,
    source_kind: SourceKind,
}

/// Package Manager 识别（§5.5 优先级：packageManager 字段 > lockfile）。
///
/// - `packageManager` 字段值形如 `"pnpm@9.1.0"`，取 `@` 前的名字；
///   仅接受 pnpm/npm/yarn/bun（未知值忽略，回退 lockfile 判定）。
/// - 字段与 lockfile 冲突时**字段优先**（如 packageManager=pnpm + package-lock.json → pnpm）。
fn detect_package_manager(ctx: &DetectorContext) -> Option<PackageManagerHit> {
    // 1. packageManager 字段（最高优先级）
    if let Some(field) = &ctx.package_manager_field {
        let name = field.split('@').next().unwrap_or("").trim();
        let id = match name {
            "pnpm" => Some(PackageManagerId::Pnpm),
            "npm" => Some(PackageManagerId::Npm),
            "yarn" => Some(PackageManagerId::Yarn),
            "bun" => Some(PackageManagerId::Bun),
            _ => None,
        };
        if let Some(id) = id {
            return Some(PackageManagerHit {
                id,
                source_kind: SourceKind::ManifestField,
            });
        }
    }

    // 2. lockfile（次优先级；多个 lockfile 并存时取第一个命中，稳定即可）
    ctx.lockfiles.first().map(|id| PackageManagerHit {
        id: *id,
        source_kind: SourceKind::Lockfile,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(ctx: &DetectorContext) -> Vec<DetectionResult> {
        InfrastructureDetector.detect(ctx)
    }

    fn ids(results: &[DetectionResult]) -> Vec<&str> {
        results.iter().map(|r| r.technology.id.as_str()).collect()
    }

    /// packageManager 字段最高优先级。
    #[test]
    fn package_manager_field_wins() {
        let ctx = DetectorContext {
            package_manager_field: Some("pnpm@9.1.0".into()),
            lockfiles: vec![PackageManagerId::Npm], // 冲突：lockfile 说 npm
            ..Default::default()
        };
        let out = detect(&ctx);
        assert!(ids(&out).contains(&"pnpm"), "字段应胜过 lockfile");
        assert!(!ids(&out).contains(&"npm"), "冲突时不应产出 npm");
    }

    /// 无 packageManager 字段 → lockfile 判定（pnpm-lock.yaml → pnpm）。
    #[test]
    fn lockfile_fallback_pnpm() {
        let ctx = DetectorContext {
            lockfiles: vec![PackageManagerId::Pnpm],
            ..Default::default()
        };
        let out = detect(&ctx);
        let pm = out.iter().find(|r| r.technology.id == "pnpm").expect("应有 pnpm");
        assert_eq!(pm.source_kind, SourceKind::Lockfile);
    }

    /// 各 lockfile → 对应包管理器。
    #[test]
    fn each_lockfile_maps_to_package_manager() {
        for (lock, id) in [
            (PackageManagerId::Pnpm, "pnpm"),
            (PackageManagerId::Npm, "npm"),
            (PackageManagerId::Yarn, "yarn"),
            (PackageManagerId::Bun, "bun"),
        ] {
            let ctx = DetectorContext {
                lockfiles: vec![lock],
                ..Default::default()
            };
            assert!(ids(&detect(&ctx)).contains(&id), "{id} 应被识别");
        }
    }

    /// 未知 packageManager 字段（如 deno@2）→ 忽略，回退 lockfile。
    #[test]
    fn unknown_package_manager_field_falls_back_to_lockfile() {
        let ctx = DetectorContext {
            package_manager_field: Some("deno@2.0.0".into()),
            lockfiles: vec![PackageManagerId::Npm],
            ..Default::default()
        };
        assert!(ids(&detect(&ctx)).contains(&"npm"));
    }

    /// 无 packageManager 字段、无 lockfile → 不产出包管理器。
    #[test]
    fn no_package_manager_evidence() {
        let ctx = DetectorContext::default();
        let out = detect(&ctx);
        assert!(!ids(&out).contains(&"pnpm"));
        assert!(!ids(&out).contains(&"npm"));
        assert!(!ids(&out).contains(&"yarn"));
        assert!(!ids(&out).contains(&"bun"));
    }

    /// Node.js runtime 始终产出（manifest 存在前提），来源优先级：
    /// engines.node（ManifestField）。
    #[test]
    fn runtime_source_engines_first() {
        let ctx = DetectorContext {
            engines_node: true,
            lockfiles: vec![PackageManagerId::Pnpm],
            script_commands: vec!["vite dev".into()],
            ..Default::default()
        };
        let out = detect(&ctx);
        let node = out.iter().find(|r| r.technology.id == "nodejs").expect("应有 nodejs");
        assert_eq!(node.technology.category, Cat::Runtime);
        assert_eq!(node.source_kind, SourceKind::ManifestField, "engines.node 最优先");
        assert_eq!(node.technology.ecosystem, None);
    }

    /// 无 engines → lockfile 作为 runtime 证据。
    #[test]
    fn runtime_source_lockfile_second() {
        let ctx = DetectorContext {
            lockfiles: vec![PackageManagerId::Npm],
            script_commands: vec!["start".into()],
            ..Default::default()
        };
        let out = detect(&ctx);
        let node = out.iter().find(|r| r.technology.id == "nodejs").expect("应有 nodejs");
        assert_eq!(node.source_kind, SourceKind::Lockfile);
    }

    /// 无 engines、无 lockfile → scripts 作为 runtime 证据。
    #[test]
    fn runtime_source_script_third() {
        let ctx = DetectorContext {
            script_commands: vec!["node server.js".into()],
            ..Default::default()
        };
        let out = detect(&ctx);
        let node = out.iter().find(|r| r.technology.id == "nodejs").expect("应有 nodejs");
        assert_eq!(node.source_kind, SourceKind::Script);
    }

    /// 无任何附加证据 → manifest 本身（ManifestField）。
    #[test]
    fn runtime_source_manifest_fallback() {
        let ctx = DetectorContext::default();
        let out = detect(&ctx);
        assert!(ids(&out).contains(&"nodejs"));
    }
}
