//! 项目技术栈解析模块。
//!
//! 职责：
//! - 依据项目内的清单文件识别语言与框架
//!   - Node：`package.json`（依赖中判断 Vue / React / Angular / Svelte 等）
//!   - Rust：`Cargo.toml`
//!   - Go：`go.mod`
//!   - Python：`requirements.txt` / `pyproject.toml`
//! - 纯只读，不修改源码、不执行命令、无网络请求。
//!
//! 硬性约束：本模块禁止 `use tauri`。

use std::path::Path;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectMeta {
    pub language: Option<String>,
    pub framework: Option<String>,
}

impl ProjectMeta {
    pub fn new(language: Option<String>, framework: Option<String>) -> Self {
        Self { language, framework }
    }
}

/// 根据清单文件推断技术栈。
///
/// 返回 `Ok(None)` 表示目录内无清单文件（即不是可识别的项目）。
pub fn detect_stack(project_dir: &Path) -> Result<Option<ProjectMeta>, ParseError> {
    if project_dir.join("package.json").is_file() {
        return detect_node(project_dir);
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
    Ok(None)
}

/// 解析 Node `package.json`，识别 language = Node + 前端框架。
fn detect_node(project_dir: &Path) -> Result<Option<ProjectMeta>, ParseError> {
    let raw = std::fs::read_to_string(project_dir.join("package.json"))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| ParseError::Malformed(format!("package.json: {e}")))?;

    let mut deps = Vec::new();
    // 收集 dependencies / devDependencies / peerDependencies 的所有键
    for key in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(map) = value.get(key).and_then(|v| v.as_object()) {
            deps.extend(map.keys().cloned());
        }
    }

    let framework = detect_node_framework(&deps);
    Ok(Some(ProjectMeta::new(Some("Node".into()), framework)))
}

/// 从 npm 依赖集合中推断前端框架。
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

    // 命中多个时按声明的优先级取第一个，保证确定性。
    FRAMEWORKS
        .iter()
        .find(|(needle, _)| deps.iter().any(|d| d.starts_with(needle)))
        .map(|(_, label)| label.to_string())
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
}
