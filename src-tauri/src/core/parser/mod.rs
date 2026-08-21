//! 项目技术栈解析模块。
//!
//! 职责：
//! - 依据项目内的清单文件识别语言与框架
//!   - Node：`package.json`（dependencies / devDependencies / scripts / engines /
//!     packageManager，经 Detector Registry，PR2）
//!   - Rust：`Cargo.toml`
//!   - Go：`go.mod`
//!   - Python：`requirements.txt` / `pyproject.toml`
//! - 纯只读，不修改源码、不执行命令、无网络请求。
//!
//! V0.4 Detection Engine（PR2，Spec §5）：Manifest → Detector Registry →
//! Vec&lt;Technology&gt;。识别规则集中在 `detectors/`（注册表模式），
//! 禁止 if vue... if express... 堆砌。
//!
//! 硬性约束：本模块禁止 `use tauri`。

pub mod detectors;
pub mod node;
pub mod registry;
pub mod swift;

use std::path::Path;

use crate::core::models::Technology;

/// 解析失败错误。
#[derive(Debug)]
pub enum ParseError {
    /// 项目目录下没有可识别的清单文件。
    NoManifestFound,
    /// 读取清单文件失败。
    Io(std::io::Error),
    /// 清单文件内容解析失败。
    Malformed(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NoManifestFound => write!(f, "未找到可识别的清单文件"),
            ParseError::Io(e) => write!(f, "读取清单文件失败: {e}"),
            ParseError::Malformed(m) => write!(f, "清单文件解析失败: {m}"),
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::Io(e)
    }
}

/// 识别出的技术栈元数据。
///
/// V0.4 Recognition Model（PR1）升级：从 `{ language, framework }` 扩展为
/// `{ language, framework, technologies }`。`language` / `framework` 为旧字段，
/// **保留兼容**（现有调用点不破坏）；`technologies` 为新的多技术列表，由
/// PR2 的 detector 填充，PR1 阶段保持空列表。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectMeta {
    pub language: Option<String>,
    pub framework: Option<String>,
    /// 技术栈列表（V0.4 Recognition Model；PR1 阶段恒为空，PR2 填充）。
    pub technologies: Vec<Technology>,
}

impl ProjectMeta {
    pub fn new(language: Option<String>, framework: Option<String>) -> Self {
        Self {
            language,
            framework,
            technologies: Vec::new(),
        }
    }

    /// 带技术栈列表构造（供 PR2+ detector 使用）。
    pub fn with_technologies(
        language: Option<String>,
        framework: Option<String>,
        technologies: Vec<Technology>,
    ) -> Self {
        Self {
            language,
            framework,
            technologies,
        }
    }
}

/// 根据清单文件推断技术栈。
///
/// 返回 `Ok(None)` 表示目录内无清单文件（即不是可识别的项目）。
///
/// - Node：`package.json` → `node::detect`（经 Detector Registry 产出
///   `technologies`；旧字段 `language`/`framework` 规则不变）。
/// - Rust / Go / Python：暂沿用 v0.3 单值规则（P1 detector 未落地，
///   `technologies` 为空列表）。
pub fn detect_stack(project_dir: &Path) -> Result<Option<ProjectMeta>, ParseError> {
    if project_dir.join("package.json").is_file() {
        return node::detect(project_dir);
    }
    if project_dir.join("Cargo.toml").is_file() {
        return Ok(Some(ProjectMeta::new(Some("Rust".into()), None)));
    }
    if project_dir.join("go.mod").is_file() {
        return Ok(Some(ProjectMeta::new(Some("Go".into()), None)));
    }
    if project_dir.join("pyproject.toml").is_file() {
        return Ok(Some(ProjectMeta::new(Some("Python".into()), None)));
    }
    if project_dir.join("requirements.txt").is_file() {
        return Ok(Some(ProjectMeta::new(Some("Python".into()), None)));
    }
    // Swift / macOS 原生（A 基础）：Package.swift / .xcodeproj / .xcworkspace / .swift 文件。
    // 放在 Rust/Go/Python 之后、Ok(None) 之前；不与现有清单冲突
    // （package.json/Cargo.toml 等分支在前，优先返回）。
    if project_dir.join("Package.swift").is_file()
        || swift::has_xcodeproj(project_dir)
        || swift::has_swift_files(project_dir)
    {
        return swift::detect(project_dir);
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ydevsphere_parser_test_{}_{}",
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

    #[test]
    fn detects_rust() {
        let dir = tmp_dir("rust");
        write(&dir, "Cargo.toml", "[package]\nname = \"x\"\n");
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Rust"));
        assert_eq!(meta.framework, None);
    }

    #[test]
    fn detects_go() {
        let dir = tmp_dir("go");
        write(&dir, "go.mod", "module example.com/x\n\ngo 1.21\n");
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Go"));
    }

    #[test]
    fn detects_python_requirements() {
        let dir = tmp_dir("py_req");
        write(&dir, "requirements.txt", "requests==2.31.0\n");
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Python"));
    }

    #[test]
    fn detects_python_pyproject() {
        let dir = tmp_dir("py_pyproj");
        write(&dir, "pyproject.toml", "[project]\nname = \"x\"\n");
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Python"));
    }

    #[test]
    fn detects_vue_from_dependencies() {
        let dir = tmp_dir("vue");
        write(
            &dir,
            "package.json",
            r#"{"name":"app","dependencies":{"vue":"^3.4.0","axios":"^1.0.0"}}"#,
        );
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Node"));
        assert_eq!(meta.framework.as_deref(), Some("Vue"));
    }

    #[test]
    fn detects_react_from_dev_dependencies() {
        let dir = tmp_dir("react");
        write(
            &dir,
            "package.json",
            r#"{"name":"app","devDependencies":{"react":"^18.0.0"}}"#,
        );
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.framework.as_deref(), Some("React"));
    }

    #[test]
    fn node_without_framework_returns_none_framework() {
        let dir = tmp_dir("node_plain");
        write(
            &dir,
            "package.json",
            r#"{"name":"cli","dependencies":{"commander":"^11.0.0"}}"#,
        );
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Node"));
        assert_eq!(meta.framework, None);
    }

    #[test]
    fn returns_none_for_empty_dir() {
        let dir = tmp_dir("empty");
        let meta = detect_stack(&dir).expect("解析应成功");
        assert!(meta.is_none());
    }

    #[test]
    fn malformed_package_json_errors() {
        let dir = tmp_dir("bad_json");
        write(&dir, "package.json", "{not valid json");
        assert!(matches!(
            detect_stack(&dir),
            Err(ParseError::Malformed(_))
        ));
    }

    // ---- Swift / macOS 原生（V0.4 Swift detector A）----

    #[test]
    fn detects_swift_via_package_swift() {
        let dir = tmp_dir("swift_pkg");
        write(&dir, "Package.swift", "// swift-tools-version:5.9\n");
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Swift"));
        assert_eq!(meta.framework, None);
        let ids: Vec<&str> = meta.technologies.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"swift"));
        assert!(ids.contains(&"spm"));
    }

    #[test]
    fn detects_swift_via_xcodeproj() {
        let dir = tmp_dir("swift_xcode");
        std::fs::create_dir_all(dir.join("App.xcodeproj")).expect("创建失败");
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Swift"));
        let ids: Vec<&str> = meta.technologies.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"xcode"));
    }

    #[test]
    fn detects_swift_via_source_file() {
        let dir = tmp_dir("swift_src");
        write(&dir, "main.swift", "print(\"hi\")\n");
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Swift"));
    }

    #[test]
    fn empty_dir_not_swift() {
        let dir = tmp_dir("empty_not_swift");
        let meta = detect_stack(&dir).expect("解析应成功");
        assert!(meta.is_none(), "空目录不应识别为 Swift");
    }

    #[test]
    fn pure_js_dir_not_swift() {
        let dir = tmp_dir("js_not_swift");
        write(&dir, "package.json", r#"{"name":"x","dependencies":{"express":"^4.0.0"}}"#);
        write(&dir, "server.js", "const express = require('express')\n");
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Node"), "纯 JS 目录应识别为 Node 而非 Swift");
    }

    #[test]
    fn package_json_wins_over_swift_signals() {
        // package.json 与 Package.swift 共存 → package.json 分支优先（不冲突）。
        let dir = tmp_dir("node_wins_swift");
        write(&dir, "package.json", r#"{"name":"x","dependencies":{"express":"^4.0.0"}}"#);
        write(&dir, "Package.swift", "// swift-tools-version:5.9\n");
        let meta = detect_stack(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Node"), "package.json 应优先");
    }
}
