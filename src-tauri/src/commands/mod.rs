//! # Tauri IPC 命令层（薄壳层）
//!
//! 仅做：参数解析 + 调用 `core` 转发，**不含业务逻辑**。
//! 本层是唯一被 tauri 直接引用 command 的模块。

pub mod editor;
pub mod git;
pub mod memory;
pub mod project;
pub mod workspace;

pub use editor::{
    confirm_custom_editor, get_editor_preference, get_ignore_rules,
    get_language_preference, get_workspace_preference, get_workspaces,
    import_custom_app, list_app_candidates, list_editors, list_installed_apps,
    open_in_editor, open_in_file_manager, rescan_editors, scan_editors_once,
    set_editor_preference, set_ignore_rules, set_language_preference,
    set_workspace_preference, set_workspaces,
};
pub use git::get_project_git_info;
pub use memory::{
    ensure_project_memory, get_project_memory, update_project_memory,
};
pub use project::{
    get_dir_children, get_project_detail, get_projects, get_scan_history, scan_projects,
};
pub use workspace::{get_system_workspaces, select_workspace};
