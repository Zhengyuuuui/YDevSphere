//! Node 生态 manifest 加载（package.json → DetectorContext，Spec §5.2/§5.3，PR2）。
//!
//! 职责：
//! - 读取并解析 `package.json`
//! - 收集 dependencies / devDependencies / peerDependencies / scripts / engines /
//!   packageManager 字段（Spec §5.3 检测来源）
//! - 探测 lockfile（§5.5）与 tsconfig.json
//! - 构造 `DetectorContext` 交给 `Registry` 分发
//! - **旧字段兼容**：`language` / `framework` 沿用 v0.3 识别规则，不受 registry 影响
//!
//! 纯只读；禁止 `use tauri`。

use std::path::Path;

use super::registry::{DetectorContext, PackageManagerId, Registry};
use super::ProjectMeta;

/// 解析 Node `package.json`，产出技术栈元数据。
///
/// - `language` / `framework`：沿用 v0.3 规则（**不变**，旧字段兼容）。
/// - `technologies`：由 Detector Registry 产出（PR2 新链路）。
pub fn detect(project_dir: &Path) -> Result<Option<ProjectMeta>, ParseError> {
    let raw = std::fs::read_to_string(project_dir.join("package.json"))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| ParseError::Malformed(format!("package.json: {e}")))?;

    // ---- 旧字段：依赖收集 + 框架判定（规则与 v0.3 完全一致）----
    let deps = collect_dependency_names(&value);
    let framework = detect_node_framework(&deps);

    // ---- 新链路：构造 DetectorContext，交给 Registry ----
    let ctx = build_context(project_dir, &value, deps);
    let technologies = Registry::new().detect(&ctx);

    Ok(Some(ProjectMeta::with_technologies(
        Some("Node".into()),
        framework,
        technologies,
    )))
}

/// 收集 dependencies / devDependencies / peerDependencies 的所有依赖名。
fn collect_dependency_names(value: &serde_json::Value) -> Vec<String> {
    let mut deps = Vec::new();
    for key in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(map) = value.get(key).and_then(|v| v.as_object()) {
            deps.extend(map.keys().cloned());
        }
    }
    deps
}

/// 从 npm 依赖集合中推断前端框架（v0.3 旧规则，**保持不变**）。
///
/// 命中多个时按声明的优先级取第一个，保证确定性。
fn detect_node_framework(deps: &[String]) -> Option<String> {
    const FRAMEWORKS: &[(&str, &str)] = &[
        ("vue", "Vue"),
        ("@vue", "Vue"),
        ("react", "React"),
        ("next", "Next.js"),
        ("nuxt", "Nuxt"),
        ("svelte", "Svelte"),
        ("@angular", "Angular"),
    ];

    FRAMEWORKS
        .iter()
        .find(|(needle, _)| deps.iter().any(|d| d.starts_with(needle)))
        .map(|(_, label)| label.to_string())
}

/// 从解析后的 package.json + 项目目录构建 `DetectorContext`。
///
/// 文件系统探测（lockfile / tsconfig.json）在此层完成，detector 保持纯函数。
fn build_context(
    project_dir: &Path,
    value: &serde_json::Value,
    deps: Vec<String>,
) -> DetectorContext {
    DetectorContext {
        dependencies: deps,
        engines_node: value
            .get("engines")
            .and_then(|e| e.get("node"))
            .is_some(),
        package_manager_field: value
            .get("packageManager")
            .and_then(|v| v.as_str())
            .map(String::from),
        lockfiles: detect_lockfiles(project_dir),
        script_commands: collect_script_commands(value),
        has_typescript_config: project_dir.join("tsconfig.json").is_file(),
    }
}

/// 探测项目根的 lockfile → 包管理器（Spec §5.5）。
fn detect_lockfiles(project_dir: &Path) -> Vec<PackageManagerId> {
    let mut out = Vec::new();
    if project_dir.join("pnpm-lock.yaml").is_file() {
        out.push(PackageManagerId::Pnpm);
    }
    if project_dir.join("package-lock.json").is_file() {
        out.push(PackageManagerId::Npm);
    }
    if project_dir.join("yarn.lock").is_file() {
        out.push(PackageManagerId::Yarn);
    }
    if project_dir.join("bun.lock").is_file() || project_dir.join("bun.lockb").is_file() {
        out.push(PackageManagerId::Bun);
    }
    out
}

/// 收集 scripts 对象的命令内容。
fn collect_script_commands(value: &serde_json::Value) -> Vec<String> {
    value
        .get("scripts")
        .and_then(|v| v.as_object())
        .map(|scripts| {
            scripts
                .values()
                .filter_map(|v| v.as_str())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

/// 解析失败错误（与 parser 根模块共用）。
use super::ParseError;

/// 便捷断言辅助（测试用）：提取技术 id 列表。
#[cfg(test)]
pub(crate) fn tech_ids(technologies: &[crate::core::models::Technology]) -> Vec<&str> {
    technologies.iter().map(|t| t.id.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ydevsphere_node_test_{}_{}",
            std::process::id(),
            name
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("创建临时目录失败");
        dir
    }

    fn write(dir: &std::path::Path, file: &str, content: &str) {
        std::fs::write(dir.join(file), content).expect("写入测试文件失败");
    }

    /// Vue 项目（vue + typescript + vite 依赖）→ vue/typescript/vite 均识别，
    /// 旧字段 language=Node / framework=Vue 保持不变。
    #[test]
    fn vue_project_full_stack() {
        let dir = tmp_dir("vue_full");
        write(
            &dir,
            "package.json",
            r#"{"name":"vue-app","dependencies":{"vue":"^3.4.0"},
                "devDependencies":{"typescript":"^5.4.0","vite":"^5.0.0"}}"#,
        );
        let meta = detect(&dir).expect("解析应成功").expect("应识别");

        // 旧字段兼容
        assert_eq!(meta.language.as_deref(), Some("Node"));
        assert_eq!(meta.framework.as_deref(), Some("Vue"));

        let ids = tech_ids(&meta.technologies);
        assert!(ids.contains(&"vue"), "应有 vue，实际 {ids:?}");
        assert!(ids.contains(&"typescript"), "应有 typescript，实际 {ids:?}");
        assert!(ids.contains(&"vite"), "应有 vite，实际 {ids:?}");
        assert!(ids.contains(&"nodejs"), "应有 nodejs，实际 {ids:?}");
    }

    /// Express 后端（express + better-sqlite3 + node 脚本）→
    /// nodejs/express/sqlite/javascript；framework 旧字段同步为 None。
    #[test]
    fn express_backend_stack() {
        let dir = tmp_dir("express_backend");
        write(
            &dir,
            "package.json",
            r#"{"name":"api","dependencies":{"express":"^4.18.0","better-sqlite3":"^9.0.0"},
                "scripts":{"start":"node server.js"}}"#,
        );
        let meta = detect(&dir).expect("解析应成功").expect("应识别");

        let ids = tech_ids(&meta.technologies);
        assert!(ids.contains(&"nodejs"));
        assert!(ids.contains(&"express"));
        assert!(ids.contains(&"sqlite"), "better-sqlite3 应映射 sqlite，实际 {ids:?}");
        assert!(ids.contains(&"javascript"), "node 脚本应产出 javascript");
        // 旧字段：express 不在旧框架规则表 → framework None（v0.3 行为不变）
        assert_eq!(meta.framework, None);
    }

    /// packageManager 字段 + pnpm lockfile 共存 → pnpm（字段优先级）。
    #[test]
    fn package_manager_detection() {
        let dir = tmp_dir("pm");
        write(
            &dir,
            "package.json",
            r#"{"name":"x","packageManager":"pnpm@9.1.0"}"#,
        );
        write(&dir, "pnpm-lock.yaml", "");
        let meta = detect(&dir).expect("解析应成功").expect("应识别");
        let ids = tech_ids(&meta.technologies);
        assert_eq!(
            ids.iter().filter(|id| **id == "pnpm").count(),
            1,
            "pnpm 应恰好出现一次（去重），实际 {ids:?}"
        );
    }

    /// tsconfig.json（无 typescript 依赖）→ TypeScript。
    #[test]
    fn tsconfig_without_dep() {
        let dir = tmp_dir("tsconfig");
        write(&dir, "package.json", r#"{"name":"x"}"#);
        write(&dir, "tsconfig.json", "{}");
        let meta = detect(&dir).expect("解析应成功").expect("应识别");
        assert!(tech_ids(&meta.technologies).contains(&"typescript"));
    }

    /// engines.node 存在 → nodejs runtime（ManifestField 证据路径覆盖）。
    #[test]
    fn engines_node_runtime() {
        let dir = tmp_dir("engines");
        write(
            &dir,
            "package.json",
            r#"{"name":"x","engines":{"node":">=18.0.0"}}"#,
        );
        let meta = detect(&dir).expect("解析应成功").expect("应识别");
        assert!(tech_ids(&meta.technologies).contains(&"nodejs"));
    }

    /// malformed package.json → 沿用 Malformed 错误（不 panic）。
    #[test]
    fn malformed_errors() {
        let dir = tmp_dir("bad");
        write(&dir, "package.json", "{not valid json");
        assert!(matches!(detect(&dir), Err(ParseError::Malformed(_))));
    }

    /// canonical id 去重：vue + @vue/* 同时声明 → technologies 只有一个 vue。
    #[test]
    fn canonical_id_dedupe_across_rules() {
        let dir = tmp_dir("dedupe");
        write(
            &dir,
            "package.json",
            r#"{"name":"x","dependencies":{"vue":"^3.4.0","@vue/compiler-sfc":"^3.4.0"}}"#,
        );
        let meta = detect(&dir).expect("解析应成功").expect("应识别");
        let ids = tech_ids(&meta.technologies);
        assert_eq!(ids.iter().filter(|id| **id == "vue").count(), 1);
    }
}
