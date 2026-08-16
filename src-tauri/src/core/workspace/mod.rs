//! # 系统工作区模块（SPRINT5-04）
//!
//! 解析标准文档 / 桌面目录（`~/Documents` / `~/Desktop`），作为「一键导入」
//! 的快捷工作区入口。
//!
//! ## 设计要点
//! - 只负责**解析路径**；实际扫描复用 `core::scanner`（前端选中入口后调 `scan_projects`）。
//! - 基于 `dirs::home_dir()` 拼英文目录名（`Documents` / `Desktop`），**不做本地化**
//!   （产品决策）。
//! - 跨平台：macOS 为主场景；Windows / Linux 同样解析（目录不存在则 `exists=false`），
//!   **禁止 macOS-only 硬编码**。
//!
//! 硬性约束：本模块禁止 `use tauri`。

use std::path::PathBuf;

use crate::core::models::{SystemWorkspace, SystemWorkspaceKind};

/// `~/Documents` 目录路径；不存在或无法解析时返回 `None`。
pub fn documents_dir() -> Option<PathBuf> {
    resolve_system_dir("Documents")
}

/// `~/Desktop` 目录路径；不存在或无法解析时返回 `None`。
pub fn desktop_dir() -> Option<PathBuf> {
    resolve_system_dir("Desktop")
}

/// 解析 `~/<name>` 目录（英文目录名，不本地化）。
///
/// 返回 `Some` 当且仅当该目录存在且是目录；否则 `None`。
fn resolve_system_dir(name: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let path = home.join(name);
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

/// 获取 Documents / Desktop 两个系统工作区入口。
pub fn get_system_workspaces() -> Vec<SystemWorkspace> {
    let documents = documents_dir().map(|p| p.to_string_lossy().to_string());
    let desktop = desktop_dir().map(|p| p.to_string_lossy().to_string());

    vec![
        SystemWorkspace::new(SystemWorkspaceKind::Documents, documents),
        SystemWorkspace::new(SystemWorkspaceKind::Desktop, desktop),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 串行化：`HOME` 是进程级 env var，避免并行测试互相覆盖。
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// 临时 HOME，用于隔离测试目录解析。
    /// 返回临时 HOME 路径 + 清理闭包。
    fn with_temp_home() -> (PathBuf, PathBuf, Box<dyn FnOnce()>) {
        let home = std::env::temp_dir().join(format!(
            "ydevsphere_ws_home_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).expect("创建临时 HOME 失败");
        let prev = std::env::var_os("HOME");
        std::env::set_var("HOME", &home);
        let home_for_cleanup = home.clone();
        let cleanup = move || {
            match prev {
                Some(p) => std::env::set_var("HOME", p),
                None => std::env::remove_var("HOME"),
            }
            let _ = std::fs::remove_dir_all(&home_for_cleanup);
        };
        (home.clone(), home, Box::new(cleanup))
    }

    #[test]
    fn documents_dir_returns_none_when_missing() {
        let _guard = LOCK.lock().unwrap();
        let (_home, _path, cleanup) = with_temp_home();
        assert!(documents_dir().is_none());
        cleanup();
    }

    #[test]
    fn desktop_dir_returns_none_when_missing() {
        let _guard = LOCK.lock().unwrap();
        let (_home, _path, cleanup) = with_temp_home();
        assert!(desktop_dir().is_none());
        cleanup();
    }

    #[test]
    fn resolves_documents_when_exists() {
        let _guard = LOCK.lock().unwrap();
        let (home, _path, cleanup) = with_temp_home();
        let docs = home.join("Documents");
        std::fs::create_dir_all(&docs).expect("创建 Documents 失败");

        let resolved = documents_dir().expect("应解析出 Documents");
        assert_eq!(resolved, docs);
        cleanup();
    }

    #[test]
    fn resolves_desktop_when_exists() {
        let _guard = LOCK.lock().unwrap();
        let (home, _path, cleanup) = with_temp_home();
        let desk = home.join("Desktop");
        std::fs::create_dir_all(&desk).expect("创建 Desktop 失败");

        let resolved = desktop_dir().expect("应解析出 Desktop");
        assert_eq!(resolved, desk);
        cleanup();
    }

    #[test]
    fn ignores_regular_file_as_dir() {
        let _guard = LOCK.lock().unwrap();
        let (home, _path, cleanup) = with_temp_home();
        // 用同名文件而非目录：应视为不存在
        std::fs::write(home.join("Documents"), "not a dir").expect("写文件失败");
        assert!(documents_dir().is_none());
        cleanup();
    }

    #[test]
    fn get_system_workspaces_serializes_and_flags_exists() {
        let _guard = LOCK.lock().unwrap();
        let (home, _path, cleanup) = with_temp_home();
        // 只建 Documents，Desktop 不建
        std::fs::create_dir_all(home.join("Documents")).expect("创建 Documents 失败");

        let workspaces = get_system_workspaces();
        assert_eq!(workspaces.len(), 2);

        // Documents 存在
        let docs = &workspaces[0];
        assert_eq!(docs.kind, SystemWorkspaceKind::Documents);
        assert_eq!(docs.label, "Documents");
        assert!(docs.exists);
        assert!(docs.path.is_some());

        // Desktop 不存在
        let desk = &workspaces[1];
        assert_eq!(desk.kind, SystemWorkspaceKind::Desktop);
        assert_eq!(desk.label, "Desktop");
        assert!(!desk.exists);
        assert!(desk.path.is_none());

        // serde 序列化：kind 小写字符串
        let json = serde_json::to_string(&docs).expect("序列化应成功");
        assert!(json.contains("\"kind\":\"documents\""));
        cleanup();
    }

    #[test]
    fn uses_english_names_not_localized() {
        let _guard = LOCK.lock().unwrap();
        let (home, _path, cleanup) = with_temp_home();
        std::fs::create_dir_all(home.join("文档")).expect("创建本地化目录失败");
        // 本地化目录名不应被识别
        assert!(documents_dir().is_none());
        cleanup();
    }
}
