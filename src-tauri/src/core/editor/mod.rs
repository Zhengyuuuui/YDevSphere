//! # 编辑器检测与打开模块（SPRINT5-02，Q2）+ 应用本地设置（SPRINT5-03）
//!
//! - `detect`：编辑器白名单检测 / 解析（只读，扫描 PATH 与常见安装路径）。
//! - `open`：白名单执行，仅启动已知编辑器。
//! - `settings`：应用本地设置持久化（默认编辑器偏好 + 最近工作区路径，
//!   `~/.ydevsphere/settings.json`）。
//!
//! ## 安全
//! 仅执行白名单内已知编辑器；未知 `editor_id` 直接拒绝，不执行任何进程。
//! 设置写入仅限应用自身配置目录，不触碰用户项目文件。
//!
//! 硬性约束：本模块禁止 `use tauri`。

pub mod detect;
pub mod discover;
pub mod open;
pub mod settings;

pub use detect::{
    find_editor_by_id, is_available_editor, is_known_editor,
    list_available_editors, resolve_editor_by_id, EditorError,
};
pub use open::{open_editor_by_id, open_editor_via, open_in_editor};
pub use settings::{
    clear_editor_cache, get_editor_cache, get_editor_preference, get_ignore_dirs,
    get_language_preference, get_workspace_preference, get_workspaces,
    set_editor_cache, set_editor_preference, set_ignore_dirs,
    set_language_preference, set_workspace_preference, set_workspaces,
    SettingsError,
};
