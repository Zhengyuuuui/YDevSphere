//! 工作区相关命令（薄壳层）。
//!
//! 职责：打开原生目录选择器，返回绝对路径。仅做参数转发，不含业务逻辑。

use std::sync::Mutex;

use tauri::State;

use crate::core::database::Database;
use crate::core::models::SystemWorkspace;
use crate::core::workspace;

/// 打开原生目录选择器，返回用户选择的绝对路径。
///
/// 若用户取消选择，返回 `None`（前端据此判断未选择）。
///
/// 必须是 `async fn`：macOS 上同步 command 运行在主线程，而
/// `blocking_pick_folder` 内部会阻塞等待对话框回调，导致主线程事件循环无法
/// 处理 → 应用冻结。`async fn` 让 command 运行在 async runtime 线程上，
/// 主线程保持空闲，对话框可正常完成回调。
#[tauri::command]
pub async fn select_workspace(
    _db: State<'_, Mutex<Database>>,
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    // 临时诊断日志：确认 command 生命周期与线程模型。
    eprintln!("[YDevSphere] select_workspace START");

    // 阻塞式打开原生目录选择器。
    let picked = app
        .dialog()
        .file()
        .blocking_pick_folder();

    match picked {
        Some(path) => {
            // 归一化：统一使用 `/` 分隔符，并去除末尾斜杠。
            let normalized = normalize_path(&path.to_string());
            eprintln!("[YDevSphere] select_workspace picked={normalized}");
            eprintln!("[YDevSphere] select_workspace END");
            Ok(Some(normalized))
        }
        None => {
            eprintln!("[YDevSphere] select_workspace picked=None");
            eprintln!("[YDevSphere] select_workspace END");
            Ok(None)
        }
    }
}

/// 获取 Documents / Desktop 两个系统工作区入口。
///
/// 前端据此决定是否展示 / 禁用快捷入口；实际扫描由前端直接调 `scan_projects(path)`。
#[tauri::command]
pub fn get_system_workspaces() -> Vec<SystemWorkspace> {
    workspace::get_system_workspaces()
}

/// 路径归一化：转换为绝对路径字符串，统一分隔符为 `/`，去除末尾斜杠。
fn normalize_path(raw: &str) -> String {
    let mut out = raw.replace('\\', "/");
    // 去除末尾斜杠（保留根路径 "/"）
    while out.len() > 1 && out.ends_with('/') {
        out.pop();
    }
    out
}
