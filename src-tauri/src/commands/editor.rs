//! 编辑器相关命令（薄壳层）。
//!
//! 职责：参数解析 + 调用 `core::editor` 转发。
//! - 编辑器检测 / 打开走 `core::editor`（白名单）。
//! - 文件管理器打开走 `tauri-plugin-opener`（系统默认文件管理器）。
//!
//! ## 安全
//! - 仅执行白名单内已知编辑器（`core::editor` 强制）。
//! - 非法 `editor_id` 直接拒绝，不执行任何进程。

use std::sync::Mutex;

use tauri::State;

use crate::core::database::Database;
use crate::core::editor::{self, EditorError, SettingsError};
use crate::core::models::AvailableEditor;

/// 列出检测到的可用编辑器。
///
/// v0.2：优先读缓存（`~/.ydevsphere/settings.json` 的 `editor_cache`），
/// 无缓存则扫描并写缓存（首次调用自动触发扫描）。
#[tauri::command]
pub fn list_editors() -> Vec<AvailableEditor> {
    scan_and_cache(false)
}

/// 重新扫描编辑器（清缓存 → 扫描 → 写缓存）。
#[tauri::command]
pub fn rescan_editors() -> Vec<AvailableEditor> {
    scan_and_cache(true)
}

/// 扫描编辑器并（可选）先清缓存。
///
/// - `force`：是否强制清缓存重扫。
/// - 读缓存优先；命中则直接返回，不触发扫描。
fn scan_and_cache(force: bool) -> Vec<AvailableEditor> {
    // 非强制：读缓存
    if !force {
        if let Ok(Some(cache)) = editor::get_editor_cache() {
            return cache;
        }
    } else {
        // 强制重扫：清缓存
        let _ = editor::clear_editor_cache();
    }

    // 扫描（白名单 + 动态发现）
    let editors = editor::list_available_editors();
    // 写缓存（失败不阻断返回）
    let _ = editor::set_editor_cache(editors.clone());
    editors
}

/// 首次启动自动扫描一次（后台写缓存，不阻塞启动）。
///
/// 由 `lib.rs` 的 setup 钩子调用；仅当缓存不存在时扫描。
pub fn scan_editors_once() {
    if let Ok(Some(_)) = editor::get_editor_cache() {
        return; // 已有缓存，跳过
    }
    let editors = editor::list_available_editors();
    let _ = editor::set_editor_cache(editors);
}

/// 在指定编辑器内打开项目。
///
/// v0.2：改为「动态发现优先 + 白名单兜底」——按 id 在「白名单 + 动态发现」
/// 中查找编辑器，命中后按 `open_method` 分级打开（cli / open-a / unsupported）。
///
/// - 项目不存在 → 返回明确错误。
/// - `editor_id` 未知（列表找不到）→ `UnknownEditor`（前端提示「编辑器不存在」）。
/// - `open_method = Unsupported` → `UnsupportedMethod`（前端提示「请手动选择目录」）。
#[tauri::command]
pub fn open_in_editor(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    editor_id: String,
) -> Result<(), String> {
    let project_path = fetch_project_path(&db, project_id)?;
    editor::open_editor_by_id(std::path::Path::new(&project_path), &editor_id)
        .map_err(map_editor_err)
}

/// 用系统文件管理器打开项目目录。
#[tauri::command]
pub fn open_in_file_manager(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let project_path = fetch_project_path(&db, project_id)?;
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(project_path, None::<&str>)
        .map_err(|e| format!("打开文件管理器失败: {e}"))
}

/// 读取默认编辑器偏好（未设置返回 `null`）。
#[tauri::command]
pub fn get_editor_preference() -> Result<Option<String>, String> {
    editor::get_editor_preference().map_err(map_settings_err)
}

/// 设置默认编辑器偏好（白名单校验通过才写入 `~/.ydevsphere/settings.json`）。
#[tauri::command]
pub fn set_editor_preference(editor_id: String) -> Result<(), String> {
    editor::set_editor_preference(&editor_id).map_err(map_settings_err)
}

/// 从库读取项目路径。
/// 读取最近保存的工作区路径（未设置返回 `null`）。
#[tauri::command]
pub fn get_workspace_preference() -> Result<Option<String>, String> {
    editor::get_workspace_preference().map_err(map_settings_err)
}

/// 保存工作区路径。
///
/// - `path` 非空：写入（启动时自动恢复直达 Dashboard）。
/// - `path` 空串 / 空白：清除工作区偏好。
#[tauri::command]
pub fn set_workspace_preference(path: String) -> Result<(), String> {
    editor::set_workspace_preference(&path).map_err(map_settings_err)
}

/// 读取用户自定义忽略目录列表（未设置返回空数组）。
#[tauri::command]
pub fn get_ignore_rules() -> Result<Vec<String>, String> {
    editor::get_ignore_dirs().map_err(map_settings_err)
}

/// 设置用户自定义忽略目录列表（整表替换，去重 + 去空白项）。
#[tauri::command]
pub fn set_ignore_rules(dirs: Vec<String>) -> Result<(), String> {
    editor::set_ignore_dirs(&dirs).map_err(map_settings_err)
}

/// 读取工作区路径集合（v0.2 多工作区模型权威源）。
///
/// 兼容迁移：集合为空但旧单值 `workspace_path` 有值时，返回 `[workspace_path]`。
#[tauri::command]
pub fn get_workspaces() -> Result<Vec<String>, String> {
    editor::get_workspaces().map_err(map_settings_err)
}

/// 设置工作区路径集合（整表替换，去重 + 去空白项）。
#[tauri::command]
pub fn set_workspaces(dirs: Vec<String>) -> Result<(), String> {
    editor::set_workspaces(&dirs).map_err(map_settings_err)
}

/// 读取界面语言偏好（未设置返回 `null`）。
#[tauri::command]
pub fn get_language_preference() -> Result<Option<String>, String> {
    editor::get_language_preference().map_err(map_settings_err)
}

/// 设置界面语言偏好（空串/空白清除偏好）。
#[tauri::command]
pub fn set_language_preference(lng: String) -> Result<(), String> {
    editor::set_language_preference(&lng).map_err(map_settings_err)
}

fn fetch_project_path(db: &Mutex<Database>, project_id: i64) -> Result<String, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let detail = db
        .get_project_detail(project_id)
        .map_err(|e| format!("查询项目失败: {e}"))?
        .ok_or_else(|| format!("项目不存在: {project_id}"))?;
    Ok(detail.path)
}

fn map_editor_err(e: EditorError) -> String {
    e.to_string()
}

fn map_settings_err(e: SettingsError) -> String {
    e.to_string()
}
