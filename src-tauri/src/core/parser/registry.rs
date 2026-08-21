//! Detector Registry（Spec §5.1/§5.2，PR2）。
//!
//! 核心链路：`Manifest → Detector Registry → Vec<Technology>`。
//!
//! - `Detector` trait：单个 detector 的识别规则（纯函数，输入 `DetectorContext`）。
//! - `Registry`：注册 + 分发 + **按 canonical id 去重**（Spec §1.2）。
//! - `DetectionResult`：detector 内部返回，携带 `source_kind`（Spec §8，
//!   为 V0.5 Evidence 留接口；**不落库、不展示、不做 confidence**）。
//!
//! 硬性约束：本模块禁止 `use tauri`；不引入第三方依赖（手写 registry）。

use crate::core::models::{Technology, TechnologyCategory};

/// 检测来源（Spec §8）：detector 内部保留，为 V0.5 Evidence 留接口。
///
/// **不落库、不展示、不做 confidence**，仅出现在 detector API 返回值中。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// manifest 依赖声明（dependencies / devDependencies 等）。
    ManifestDependency,
    /// manifest 显式字段（packageManager / engines 等）。
    ManifestField,
    /// lockfile 文件存在（pnpm-lock.yaml 等）。
    Lockfile,
    /// scripts 命令内容。
    Script,
}

/// 单条检测结果（Spec §8）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionResult {
    pub technology: Technology,
    pub source_kind: SourceKind,
}

impl DetectionResult {
    pub fn new(technology: Technology, source_kind: SourceKind) -> Self {
        Self {
            technology,
            source_kind,
        }
    }
}

/// 依赖匹配到的包管理器（lockfile 证据，Spec §5.5）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManagerId {
    Pnpm,
    Npm,
    Yarn,
    Bun,
}

impl PackageManagerId {
    /// canonical id（Spec §1.2，与 packageManager 字段统一）。
    pub fn canonical_id(&self) -> &'static str {
        match self {
            PackageManagerId::Pnpm => "pnpm",
            PackageManagerId::Npm => "npm",
            PackageManagerId::Yarn => "yarn",
            PackageManagerId::Bun => "bun",
        }
    }

    /// 展示名。
    pub fn display_name(&self) -> &'static str {
        match self {
            PackageManagerId::Pnpm => "pnpm",
            PackageManagerId::Npm => "npm",
            PackageManagerId::Yarn => "Yarn",
            PackageManagerId::Bun => "Bun",
        }
    }
}

/// Detector 输入上下文：manifest 已解析的结构化数据（不含文件系统访问）。
///
/// 由 `node.rs` 等清单加载层构造，detector 作为纯函数消费，
/// 便于单测与未来多语言 manifest（rust.rs / go.rs / python.rs）复用。
#[derive(Debug, Clone, Default)]
pub struct DetectorContext {
    /// 依赖名集合（dependencies + devDependencies + peerDependencies 的键）。
    pub dependencies: Vec<String>,
    /// `engines.node` 是否存在（Runtime 证据，Spec §5.6）。
    pub engines_node: bool,
    /// `package.json` 的 `packageManager` 字段原始值（如 `"pnpm@9.1.0"`）。
    pub package_manager_field: Option<String>,
    /// 项目根存在的 lockfile 对应的包管理器（Spec §5.5）。
    pub lockfiles: Vec<PackageManagerId>,
    /// scripts 的命令内容（如 `"vite dev"`、`"node server.js"`）。
    pub script_commands: Vec<String>,
    /// `tsconfig.json` 是否存在（TypeScript 证据）。
    pub has_typescript_config: bool,
}

impl DetectorContext {
    /// 判断是否声明了指定依赖（精确名或 scope 前缀：`@vue` 命中 `@vue/core`）。
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies
            .iter()
            .any(|d| d == name || d.starts_with(&format!("{name}/")))
    }

    /// 判断任一 script 命令包含 `needle`（如 `"node "`，JavaScript 证据）。
    pub fn any_script_contains(&self, needle: &str) -> bool {
        self.script_commands.iter().any(|s| s.contains(needle))
    }
}

/// Detector trait（Spec §5.1）：禁止 `if vue... if express...` 堆砌，
/// 一切识别规则以 detector 实现注册进 Registry。
pub trait Detector {
    /// detector 名称（诊断 / 测试用）。
    fn name(&self) -> &'static str;
    /// 对清单上下文执行识别，返回检测结果（含 source_kind）。
    fn detect(&self, ctx: &DetectorContext) -> Vec<DetectionResult>;
}

/// Detector Registry（Spec §5.1）：注册 + 分发 + 去重。
///
/// 去重规则（Spec §1.2）：不同 detector 对同一技术必须产出**同一个 canonical id**，
/// registry 按 `id` 去重，**先注册的 detector 优先**（注册顺序即优先级）。
pub struct Registry {
    detectors: Vec<Box<dyn Detector>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// 创建带 PR2 P0 detector 的 registry（javascript + database + infrastructure）。
    pub fn new() -> Self {
        let mut registry = Self::empty();
        registry.register(Box::new(super::detectors::javascript::JavaScriptDetector));
        registry.register(Box::new(super::detectors::database::DatabaseDetector));
        registry.register(Box::new(super::detectors::infrastructure::InfrastructureDetector));
        registry
    }

    /// 空 registry（测试自定义 detector 组合用）。
    pub fn empty() -> Self {
        Self {
            detectors: Vec::new(),
        }
    }

    /// 注册 detector（追加到末尾；先注册者优先）。
    pub fn register(&mut self, detector: Box<dyn Detector>) {
        self.detectors.push(detector);
    }

    /// 已注册 detector 数量。
    pub fn len(&self) -> usize {
        self.detectors.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.detectors.is_empty()
    }

    /// 分发到全部 detector 并汇总结果。
    ///
    /// 返回 `DetectionResult` 列表（含 source_kind，供内部/测试使用），
    /// 已按 canonical id 去重（保留先注册 detector 的结果）。
    pub fn detect_detailed(&self, ctx: &DetectorContext) -> Vec<DetectionResult> {
        let mut seen_ids: Vec<String> = Vec::new();
        let mut results: Vec<DetectionResult> = Vec::new();
        for detector in &self.detectors {
            for result in detector.detect(ctx) {
                if !seen_ids.contains(&result.technology.id) {
                    seen_ids.push(result.technology.id.clone());
                    results.push(result);
                }
            }
        }
        results
    }

    /// 便捷入口：只取去重后的技术列表（`detect_stack` 使用）。
    pub fn detect(&self, ctx: &DetectorContext) -> Vec<Technology> {
        self.detect_detailed(ctx)
            .into_iter()
            .map(|r| r.technology)
            .collect()
    }
}

/// 便捷构造：从规则表生成 `Technology`。
pub(crate) fn tech(
    id: &str,
    name: &str,
    category: TechnologyCategory,
    ecosystem: Option<&str>,
) -> Technology {
    Technology::new(
        id,
        name,
        category,
        ecosystem.map(|e| e.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::TechnologyCategory as Cat;

    /// 测试用 detector：固定返回给定技术。
    struct FixedDetector {
        name: &'static str,
        techs: Vec<Technology>,
    }

    impl Detector for FixedDetector {
        fn name(&self) -> &'static str {
            self.name
        }
        fn detect(&self, _ctx: &DetectorContext) -> Vec<DetectionResult> {
            self.techs
                .clone()
                .into_iter()
                .map(|t| DetectionResult::new(t, SourceKind::ManifestDependency))
                .collect()
        }
    }

    fn ctx() -> DetectorContext {
        DetectorContext::default()
    }

    /// 注册顺序即优先级：同 id 时保留先注册 detector 的结果。
    #[test]
    fn dedupes_by_canonical_id_first_wins() {
        let mut reg = Registry::empty();
        reg.register(Box::new(FixedDetector {
            name: "first",
            techs: vec![tech("vue", "Vue", Cat::Framework, Some("javascript"))],
        }));
        reg.register(Box::new(FixedDetector {
            name: "second",
            techs: vec![tech("vue", "Vue (dup)", Cat::Framework, None)],
        }));

        let out = reg.detect(&ctx());
        assert_eq!(out.len(), 1, "同 canonical id 应去重");
        assert_eq!(out[0].id, "vue");
        assert_eq!(out[0].name, "Vue", "先注册者胜出");
    }

    /// 不同 id 不互相吞并。
    #[test]
    fn keeps_distinct_ids() {
        let mut reg = Registry::empty();
        reg.register(Box::new(FixedDetector {
            name: "a",
            techs: vec![
                tech("vue", "Vue", Cat::Framework, Some("javascript")),
                tech("vite", "Vite", Cat::BuildTool, Some("javascript")),
            ],
        }));
        let out = reg.detect(&ctx());
        assert_eq!(out.len(), 2);
    }

    /// 同一 detector 内重复 id 也去重。
    #[test]
    fn dedupes_within_single_detector() {
        let mut reg = Registry::empty();
        reg.register(Box::new(FixedDetector {
            name: "dup",
            techs: vec![
                tech("vue", "Vue", Cat::Framework, Some("javascript")),
                tech("vue", "Vue", Cat::Framework, Some("javascript")),
            ],
        }));
        assert_eq!(reg.detect(&ctx()).len(), 1);
    }

    /// 默认 registry 注册了 PR2 三个 P0 detector。
    #[test]
    fn default_registry_has_p0_detectors() {
        let reg = Registry::new();
        assert_eq!(reg.len(), 3);
        let names: Vec<_> = reg.detectors.iter().map(|d| d.name()).collect();
        assert_eq!(names, vec!["javascript", "database", "infrastructure"]);
    }

    /// has_dependency：精确名与 scope 前缀匹配，不误命中相似前缀。
    #[test]
    fn context_dependency_matching() {
        let ctx = DetectorContext {
            dependencies: vec![
                "vue".into(),
                "@vue/compiler-sfc".into(),
                "@angular/core".into(),
                "vite".into(),
            ],
            ..Default::default()
        };
        assert!(ctx.has_dependency("vue"));
        assert!(ctx.has_dependency("@vue"));
        assert!(ctx.has_dependency("@angular"));
        assert!(ctx.has_dependency("vite"));
        // 前缀相似不误命中：vitest 不应命中 vite
        let ctx2 = DetectorContext {
            dependencies: vec!["vitest".into()],
            ..Default::default()
        };
        assert!(!ctx2.has_dependency("vite"), "vitest 不应命中 vite");
        // express 不应命中 express-rate-limit 之外……反之 rate-limit 不命中 express
        let ctx3 = DetectorContext {
            dependencies: vec!["express-rate-limit".into()],
            ..Default::default()
        };
        assert!(!ctx3.has_dependency("express"), "express-rate-limit 不应命中 express");
    }

    /// any_script_contains。
    #[test]
    fn context_script_matching() {
        let ctx = DetectorContext {
            script_commands: vec!["node server.js".into(), "vite build".into()],
            ..Default::default()
        };
        assert!(ctx.any_script_contains("node "));
        assert!(ctx.any_script_contains("vite"));
        assert!(!ctx.any_script_contains("deno "));
    }
}
