//! YDevSphere Rust 后端库入口。
//!
//! 依赖注入：初始化 core（数据库）后放入 Tauri managed state，
//! 并注册 commands 层命令。
//!
//! 说明：rusqlite::Connection 非 `Sync`，故用 `Mutex<Database>` 包装
//! 以满足 Tauri managed state 的 `Send + Sync` 约束。

pub mod commands;
pub mod core;

use std::sync::Mutex;

use tauri::Manager;

use core::database::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化数据库（若失败则打印错误并使用内存占位，避免阻断 UI）。
    let db = match Database::open() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("[YDevSphere] 数据库初始化失败: {e}");
            Database::open_placeholder()
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(db))
        .setup(|app| {
            // 开发模式（tauri dev）下，窗口/程序坞图标需显式从 bundle icon 应用。
            // 打包后 .app 的 Info.plist 会自动使用 icon.icns，此处主要覆盖开发调试场景。
            if let Some(icon) = app.default_window_icon() {
                for window in app.webview_windows().values() {
                    let _ = window.set_icon(icon.clone());
                }
            }

            // 首次启动自动扫描编辑器（后台线程，不阻塞启动；有缓存则跳过）。
            tauri::async_runtime::spawn_blocking(|| {
                commands::editor::scan_editors_once();
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::select_workspace,
            commands::project::scan_projects,
            commands::project::get_projects,
            commands::project::get_project_detail,
            commands::project::get_scan_history,
            commands::project::get_dir_children,
            commands::memory::ensure_project_memory,
            commands::memory::get_project_memory,
            commands::memory::update_project_memory,
            commands::git::get_project_git_info,
            commands::editor::list_editors,
            commands::editor::rescan_editors,
            commands::editor::list_app_candidates,
            commands::editor::confirm_custom_editor,
            commands::editor::list_installed_apps,
            commands::editor::import_custom_app,
            commands::editor::open_in_editor,
            commands::editor::open_in_file_manager,
            commands::editor::get_editor_preference,
            commands::editor::set_editor_preference,
            commands::editor::get_workspace_preference,
            commands::editor::set_workspace_preference,
            commands::editor::get_ignore_rules,
            commands::editor::set_ignore_rules,
            commands::editor::get_workspaces,
            commands::editor::set_workspaces,
            commands::editor::get_language_preference,
            commands::editor::set_language_preference,
            commands::editor::reset_app_state,
            commands::workspace::get_system_workspaces,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
