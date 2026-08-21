//! Swift / macOS 原生项目识别（A 基础，Spec §5.2 目录中的 swift 生态）。
//!
//! 识别信号（任一存在即 Swift 项目，跨平台纯文件存在性判断，无 macOS-only 逻辑）：
//! - `Package.swift`（SPM 清单）
//! - `.xcodeproj` / `.xcworkspace`（Xcode 工程）
//! - 目录含 `.swift` 源文件（顶层递归一层）
//!
//! 产出 Technology（canonical id 为前端/后续 PR 契约）：
//! - `swift`（language，恒产出）
//! - `xcode`（build_tool，.xcodeproj/.xcworkspace 存在时）
//! - `spm`（package_manager，Package.swift 存在时）
//!
//! 本期（A 基础）只做存在性识别，**不解析** Package.swift 依赖 / .xcodeproj
//! platform（Spec B，已记录 ROADMAP v0.4c，本期不做）。
//!
//! 硬性约束：禁止 `use tauri`；不引入第三方依赖。

use std::path::Path;

use crate::core::models::{Technology, TechnologyCategory};

use super::{ParseError, ProjectMeta};

/// Swift 技术（canonical id = `swift`）。
fn swift_tech() -> Technology {
    Technology::new("swift", "Swift", TechnologyCategory::Language, None)
}

/// Xcode 构建工具（canonical id = `xcode`）。
fn xcode_tech() -> Technology {
    Technology::new(
        "xcode",
        "Xcode",
        TechnologyCategory::BuildTool,
        Some("swift".to_string()),
    )
}

/// Swift Package Manager（canonical id = `spm`）。
fn spm_tech() -> Technology {
    Technology::new(
        "spm",
        "Swift Package Manager",
        TechnologyCategory::PackageManager,
        Some("swift".to_string()),
    )
}

/// 识别 Swift 项目并产出技术栈元数据。
///
/// - `language = Some("Swift")`（旧字段，兼容），`framework = None`。
/// - `technologies`：恒含 `swift`；`.xcodeproj`/`.xcworkspace` → 追加 `xcode`；
///   `Package.swift` → 追加 `spm`。
pub fn detect(project_dir: &Path) -> Result<Option<ProjectMeta>, ParseError> {
    let mut technologies = vec![swift_tech()];

    if has_xcodeproj(project_dir) {
        technologies.push(xcode_tech());
    }
    if project_dir.join("Package.swift").is_file() {
        technologies.push(spm_tech());
    }

    Ok(Some(ProjectMeta::with_technologies(
        Some("Swift".into()),
        None,
        technologies,
    )))
}

/// 目录是否含 `.xcodeproj` / `.xcworkspace`（Xcode 工程）。
pub fn has_xcodeproj(dir: &Path) -> bool {
    has_suffix_file(dir, ".xcodeproj") || has_suffix_file(dir, ".xcworkspace")
}

/// 目录是否含 `.swift` 源文件（顶层直接子项 + 递归一层）。
pub fn has_swift_files(dir: &Path) -> bool {
    walk_swift_files(dir, 0)
}

/// 递归查找 `.swift` 文件，`depth` 从 0 开始，最多下探一层子目录。
fn walk_swift_files(dir: &Path, depth: usize) -> bool {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth < 1 && walk_swift_files(&path, depth + 1) {
                return true;
            }
        } else if path.extension().map(|e| e == "swift").unwrap_or(false) {
            return true;
        }
    }
    false
}

/// 目录下是否含指定后缀的文件或目录（如 `.xcodeproj`）。
fn has_suffix_file(dir: &Path, suffix: &str) -> bool {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return false,
    };
    entries.flatten().any(|entry| {
        entry
            .file_name()
            .to_string_lossy()
            .ends_with(suffix)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ydevsphere_swift_test_{}_{}",
            std::process::id(),
            name
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("创建临时目录失败");
        dir
    }

    fn write(dir: &std::path::Path, file: &str, content: &str) {
        if let Some(parent) = std::path::Path::new(file).parent() {
            std::fs::create_dir_all(dir.join(parent)).expect("创建父目录失败");
        }
        std::fs::write(dir.join(file), content).expect("写入测试文件失败");
    }

    fn mkdir(dir: &std::path::Path, name: &str) {
        std::fs::create_dir_all(dir.join(name)).expect("创建目录失败");
    }

    /// Package.swift → Swift + SPM（canonical id: swift, spm）。
    #[test]
    fn package_swift_detects_swift_and_spm() {
        let dir = tmp_dir("pkg_swift");
        write(&dir, "Package.swift", "// swift-tools-version:5.9\n");
        let meta = detect(&dir).expect("解析应成功").expect("应识别");

        assert_eq!(meta.language.as_deref(), Some("Swift"));
        assert_eq!(meta.framework, None);

        let ids: Vec<&str> = meta.technologies.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"swift"), "应有 swift，实际 {ids:?}");
        assert!(ids.contains(&"spm"), "应有 spm，实际 {ids:?}");
        assert!(!ids.contains(&"xcode"), "无 .xcodeproj 不应有 xcode");
    }

    /// .xcodeproj → Swift + Xcode（canonical id: swift, xcode）。
    #[test]
    fn xcodeproj_detects_swift_and_xcode() {
        let dir = tmp_dir("xcodeproj");
        mkdir(&dir, "MyApp.xcodeproj");
        write(&dir, "MyApp/ContentView.swift", "import SwiftUI\nstruct ContentView: View {}\n");
        let meta = detect(&dir).expect("解析应成功").expect("应识别");

        let ids: Vec<&str> = meta.technologies.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"swift"), "应有 swift，实际 {ids:?}");
        assert!(ids.contains(&"xcode"), "应有 xcode，实际 {ids:?}");
        assert!(!ids.contains(&"spm"), "无 Package.swift 不应有 spm");
    }

    /// .xcworkspace → Swift + Xcode。
    #[test]
    fn xcworkspace_detects_xcode() {
        let dir = tmp_dir("xcworkspace");
        mkdir(&dir, "Workspace.xcworkspace");
        let meta = detect(&dir).expect("解析应成功").expect("应识别");
        let ids: Vec<&str> = meta.technologies.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"xcode"), "xcworkspace 应识别 xcode");
    }

    /// 含 .swift 源文件（递归一层子目录）→ Swift。
    #[test]
    fn swift_files_in_subdir_detected() {
        let dir = tmp_dir("swift_files");
        write(&dir, "Sources/main.swift", "print(\"hi\")\n");
        let meta = detect(&dir).expect("解析应成功").expect("应识别");
        let ids: Vec<&str> = meta.technologies.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"swift"), "递归一层应识别 swift");
    }

    /// 空目录 / 纯 JS 目录不误判为 Swift。
    #[test]
    fn empty_and_js_dir_not_swift() {
        let empty = tmp_dir("empty");
        assert!(!has_swift_files(&empty));
        assert!(!has_xcodeproj(&empty));

        let js = tmp_dir("js");
        write(&js, "package.json", r#"{"name":"x","dependencies":{"express":"^4.0.0"}}"#);
        write(&js, "server.js", "const express = require('express')\n");
        assert!(!has_swift_files(&js), "纯 JS 目录不应判为含 swift 文件");
    }

    /// 与 Node 共存：Package.swift 与 package.json 同时存在时，detect_stack 以
    /// package.json 优先（mod.rs 分支顺序保证），swift::detect 仅识别 Swift 信号。
    #[test]
    fn coexist_with_node_package_json() {
        let dir = tmp_dir("coexist");
        write(&dir, "Package.swift", "// swift-tools-version:5.9\n");
        write(&dir, "package.json", r#"{"name":"x","dependencies":{"express":"^4.0.0"}}"#);
        // 直接调 detect：Swift 信号存在 → Swift 识别（mod.rs 中 package.json 分支在前）
        let meta = detect(&dir).expect("解析应成功").expect("应识别");
        assert_eq!(meta.language.as_deref(), Some("Swift"));
    }

    /// canonical id 契约：swift / xcode / spm 的 category 正确。
    #[test]
    fn canonical_id_contract() {
        assert_eq!(swift_tech().category, TechnologyCategory::Language);
        assert_eq!(xcode_tech().category, TechnologyCategory::BuildTool);
        assert_eq!(spm_tech().category, TechnologyCategory::PackageManager);
    }

    /// has_swift_files 跨平台：不依赖 macOS-only 逻辑。
    #[test]
    fn detection_is_cross_platform() {
        // .swift 文件存在性判断与平台无关（纯 fs 操作）。
        let dir = tmp_dir("cross");
        write(&dir, "main.swift", "print(\"x\")\n");
        assert!(has_swift_files(&dir));
    }
}
