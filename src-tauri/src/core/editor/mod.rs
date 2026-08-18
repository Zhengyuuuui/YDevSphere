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

/// 共享测试锁：串行化所有设置 `YDEVSPHERE_SETTINGS_PATH` env 的测试
/// （settings 模块内 + commands 层），避免并行测试的 env 竞态。
#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub use detect::{
    find_editor_by_id, is_available_editor, is_known_editor,
    list_available_editors, resolve_editor_by_id, EditorError,
};
pub use discover::{
    import_and_persist_custom_app, import_custom_app, list_installed_apps,
};
pub use open::{open_editor_by_id, open_editor_via, open_in_editor};
pub use settings::{
    clear_editor_cache, get_app_snapshot, get_custom_editors, get_editor_cache,
    get_editor_cache_version, get_editor_preference, get_ignore_dirs,
    get_installed_apps_cache, get_language_preference, get_workspace_preference,
    get_workspaces, is_custom_editor, reset_settings, set_app_snapshot,
    set_custom_editors, set_editor_cache, set_editor_cache_version,
    set_editor_preference, set_ignore_dirs, set_installed_apps_cache,
    set_language_preference, set_workspace_preference, set_workspaces,
    SettingsError, EDITOR_LOGIC_VERSION,
};
