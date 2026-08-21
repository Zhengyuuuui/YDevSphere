//! JavaScript / Node detector（Spec §5.3/§5.4 P0，PR2）。
//!
//! 从 `DetectorContext`（package.json 解析产物）识别：
//! - Language：TypeScript（typescript 依赖或 tsconfig.json）/ JavaScript（scripts 直跑 node）
//! - Framework：Vue / React / Next.js / Nuxt / Svelte / Angular / Express / NestJS / Fastify
//! - BuildTool：Vite / Webpack / Rspack
//! - Platform：uni-app / Tauri / Electron
//! - Library：Pinia / JWT / Axios（Spec §1.4/§5.3 示例的最小库集）
//!
//! 全部规则走注册表（依赖规则表），**禁止 if vue... if express... 堆砌**。
//! canonical id 见各规则表（Spec §1.2/§1.4）。

use crate::core::models::TechnologyCategory as Cat;

use super::super::registry::{tech, DetectionResult, Detector, DetectorContext, SourceKind};

/// 依赖规则：`dep` 命中（精确名或 scope 前缀）即产出 `Technology`。
struct DepRule {
    dep: &'static str,
    id: &'static str,
    name: &'static str,
    category: Cat,
}

/// P0 框架规则（Spec §5.4；id 为 canonical id）。
const FRAMEWORKS: &[DepRule] = &[
    DepRule { dep: "vue", id: "vue", name: "Vue", category: Cat::Framework },
    DepRule { dep: "@vue", id: "vue", name: "Vue", category: Cat::Framework },
    DepRule { dep: "react", id: "react", name: "React", category: Cat::Framework },
    DepRule { dep: "next", id: "next", name: "Next.js", category: Cat::Framework },
    DepRule { dep: "nuxt", id: "nuxt", name: "Nuxt", category: Cat::Framework },
    DepRule { dep: "svelte", id: "svelte", name: "Svelte", category: Cat::Framework },
    DepRule { dep: "@angular", id: "angular", name: "Angular", category: Cat::Framework },
    DepRule { dep: "express", id: "express", name: "Express", category: Cat::Framework },
    DepRule { dep: "@nestjs", id: "nestjs", name: "NestJS", category: Cat::Framework },
    DepRule { dep: "fastify", id: "fastify", name: "Fastify", category: Cat::Framework },
];

/// P0 构建工具规则。
const BUILD_TOOLS: &[DepRule] = &[
    DepRule { dep: "vite", id: "vite", name: "Vite", category: Cat::BuildTool },
    DepRule { dep: "webpack", id: "webpack", name: "Webpack", category: Cat::BuildTool },
    DepRule { dep: "rspack", id: "rspack", name: "Rspack", category: Cat::BuildTool },
    DepRule { dep: "@rspack", id: "rspack", name: "Rspack", category: Cat::BuildTool },
];

/// P0 平台规则（Spec §1.4：uni-app 的 canonical id 为 `uniapp`）。
const PLATFORMS: &[DepRule] = &[
    DepRule { dep: "@dcloudio/uni-app", id: "uniapp", name: "uni-app", category: Cat::Platform },
    DepRule { dep: "@tauri-apps", id: "tauri", name: "Tauri", category: Cat::Platform },
    DepRule { dep: "tauri", id: "tauri", name: "Tauri", category: Cat::Platform },
    DepRule { dep: "electron", id: "electron", name: "Electron", category: Cat::Platform },
];

/// P0 库规则（Spec §1.4/§5.3 示例的最小库集：Pinia / JWT / Axios）。
const LIBRARIES: &[DepRule] = &[
    DepRule { dep: "pinia", id: "pinia", name: "Pinia", category: Cat::Library },
    DepRule { dep: "jsonwebtoken", id: "jwt", name: "JWT", category: Cat::Library },
    DepRule { dep: "axios", id: "axios", name: "Axios", category: Cat::Library },
];

/// JavaScript / Node 技术栈 detector。
pub struct JavaScriptDetector;

impl Detector for JavaScriptDetector {
    fn name(&self) -> &'static str {
        "javascript"
    }

    fn detect(&self, ctx: &DetectorContext) -> Vec<DetectionResult> {
        let mut results = Vec::new();

        // ---- Language ----
        // TypeScript：typescript 依赖（强证据）或 tsconfig.json 存在。
        // 任一命中即产出；source_kind 取最强来源（依赖优先）。
        let has_ts_dep = ctx.has_dependency("typescript");
        if has_ts_dep || ctx.has_typescript_config {
            let source = if has_ts_dep {
                SourceKind::ManifestDependency
            } else {
                SourceKind::ManifestField // tsconfig.json（清单文件级证据）
            };
            results.push(DetectionResult::new(
                tech("typescript", "TypeScript", Cat::Language, Some("javascript")),
                source,
            ));
        } else if ctx.any_script_contains("node ") {
            // JavaScript：无 TypeScript 且 scripts 直接以 node 执行（Script 证据）。
            //（决策：不默认为所有 package.json 产出 JavaScript，避免「纯后端项目
            // 被标注 JS 语言」——对齐 Spec §1.4/§14 验收示例，后端仅标 Node.js。）
            results.push(DetectionResult::new(
                tech("javascript", "JavaScript", Cat::Language, Some("javascript")),
                SourceKind::Script,
            ));
        }

        // ---- Framework / BuildTool / Platform / Library（依赖规则表）----
        // 多条规则可能命中同一 canonical id（如 vue 与 @vue/*）→ 组内去重。
        let mut seen: Vec<&'static str> = Vec::new();
        for group in [FRAMEWORKS, BUILD_TOOLS, PLATFORMS, LIBRARIES] {
            for rule in group {
                if seen.contains(&rule.id) {
                    continue;
                }
                if ctx.has_dependency(rule.dep) {
                    seen.push(rule.id);
                    results.push(DetectionResult::new(
                        tech(
                            rule.id,
                            rule.name,
                            rule.category,
                            Some("javascript"),
                        ),
                        SourceKind::ManifestDependency,
                    ));
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with_deps(deps: &[&str]) -> DetectorContext {
        DetectorContext {
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    fn detect(deps: &[&str]) -> Vec<DetectionResult> {
        JavaScriptDetector.detect(&ctx_with_deps(deps))
    }

    fn ids(results: &[DetectionResult]) -> Vec<&str> {
        results.iter().map(|r| r.technology.id.as_str()).collect()
    }

    #[test]
    fn detects_vue_framework() {
        let out = detect(&["vue", "pinia"]);
        assert!(ids(&out).contains(&"vue"));
        assert!(ids(&out).contains(&"pinia"));
    }

    #[test]
    fn vue_via_scoped_package() {
        let out = detect(&["@vue/compiler-sfc"]);
        assert!(ids(&out).contains(&"vue"), "@vue scope 应命中 vue");
    }

    #[test]
    fn all_p0_frameworks_detected() {
        for (dep, id) in [
            ("vue", "vue"),
            ("react", "react"),
            ("next", "next"),
            ("nuxt", "nuxt"),
            ("svelte", "svelte"),
            ("@angular/core", "angular"),
            ("express", "express"),
            ("@nestjs/common", "nestjs"),
            ("fastify", "fastify"),
        ] {
            let out = detect(&[dep]);
            assert!(
                ids(&out).contains(&id),
                "依赖 {dep} 应识别为 {id}"
            );
        }
    }

    #[test]
    fn build_tools_detected() {
        for (dep, id) in [
            ("vite", "vite"),
            ("webpack", "webpack"),
            ("rspack", "rspack"),
            ("@rspack/core", "rspack"),
        ] {
            let out = detect(&[dep]);
            assert!(ids(&out).contains(&id), "依赖 {dep} 应识别为 {id}");
        }
    }

    #[test]
    fn platforms_detected() {
        for (dep, id) in [
            ("@dcloudio/uni-app", "uniapp"),
            ("@tauri-apps/api", "tauri"),
            ("tauri", "tauri"),
            ("electron", "electron"),
        ] {
            let out = detect(&[dep]);
            assert!(ids(&out).contains(&id), "依赖 {dep} 应识别为 {id}");
        }
    }

    #[test]
    fn libraries_detected() {
        for (dep, id) in [("pinia", "pinia"), ("jsonwebtoken", "jwt"), ("axios", "axios")] {
            let out = detect(&[dep]);
            assert!(ids(&out).contains(&id), "依赖 {dep} 应识别为 {id}");
        }
    }

    /// typescript 依赖 → TypeScript 语言（ManifestDependency）。
    #[test]
    fn typescript_from_dependency() {
        let out = detect(&["typescript", "vue"]);
        let ts = out.iter().find(|r| r.technology.id == "typescript").expect("应有 typescript");
        assert_eq!(ts.technology.category, Cat::Language);
        assert_eq!(ts.source_kind, SourceKind::ManifestDependency);
    }

    /// tsconfig.json 存在（无 typescript 依赖）→ TypeScript（ManifestField）。
    #[test]
    fn typescript_from_tsconfig_only() {
        let ctx = DetectorContext {
            has_typescript_config: true,
            ..Default::default()
        };
        let out = JavaScriptDetector.detect(&ctx);
        let ts = out.iter().find(|r| r.technology.id == "typescript").expect("应有 typescript");
        assert_eq!(ts.source_kind, SourceKind::ManifestField);
    }

    /// scripts 直跑 node 且无 TS → JavaScript（Script）。
    #[test]
    fn javascript_from_node_script() {
        let ctx = DetectorContext {
            script_commands: vec!["node server.js".into()],
            ..Default::default()
        };
        let out = JavaScriptDetector.detect(&ctx);
        let js = out.iter().find(|r| r.technology.id == "javascript").expect("应有 javascript");
        assert_eq!(js.source_kind, SourceKind::Script);
    }

    /// TypeScript 优先：有 TS 时不产出 JavaScript。
    #[test]
    fn typescript_suppresses_javascript() {
        let ctx = DetectorContext {
            script_commands: vec!["node server.js".into()],
            has_typescript_config: true,
            ..Default::default()
        };
        let out = JavaScriptDetector.detect(&ctx);
        assert!(ids(&out).contains(&"typescript"));
        assert!(!ids(&out).contains(&"javascript"), "有 TypeScript 不应再标 JavaScript");
    }

    /// 无 TS、无 node 脚本 → 不产出语言类技术（如藏蓝闪送后端场景）。
    #[test]
    fn backend_without_language() {
        let out = detect(&["express", "better-sqlite3"]);
        assert!(!ids(&out).contains(&"javascript"));
        assert!(!ids(&out).contains(&"typescript"));
    }

    /// 相似前缀不误报：vitest ≠ vite、express-rate-limit ≠ express。
    #[test]
    fn no_false_positive_on_similar_prefix() {
        let out = detect(&["vitest", "express-rate-limit"]);
        assert!(!ids(&out).contains(&"vite"));
        assert!(!ids(&out).contains(&"express"));
    }

    /// 多条规则命中同一 canonical id（vue + @vue/*、rspack + @rspack/*）→
    /// detector 输出去重（同 id 只产出一次）。
    #[test]
    fn dedupes_output_across_rules() {
        let out = detect(&["vue", "@vue/compiler-sfc", "rspack", "@rspack/core"]);
        let ids = ids(&out);
        assert_eq!(ids.iter().filter(|id| **id == "vue").count(), 1, "vue 应只出现一次");
        assert_eq!(ids.iter().filter(|id| **id == "rspack").count(), 1, "rspack 应只出现一次");
    }

    /// 规则表覆盖 Spec §5.4 P0 全部技术（防漏注册）。
    #[test]
    fn rules_cover_all_p0_technologies() {
        let all_rules: Vec<&'static str> = [FRAMEWORKS, BUILD_TOOLS, PLATFORMS, LIBRARIES]
            .iter()
            .flat_map(|g| g.iter().map(|r| r.id))
            .collect();
        for id in [
            "vue", "react", "next", "nuxt", "svelte", "angular",
            "express", "nestjs", "fastify", "vite", "webpack", "rspack",
            "uniapp", "tauri", "electron",
            "pinia", "jwt", "axios",
        ] {
            assert!(all_rules.contains(&id), "P0 技术 {id} 缺少规则");
        }
    }
}
