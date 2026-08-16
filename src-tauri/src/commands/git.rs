//! Git 相关命令（薄壳层）。
//!
//! 职责：`project_id` → 从库取项目路径 → 调 `core::git` 只读分析 → 返回 `GitInfo`。
//! 本层仅做适配，不含业务逻辑。

use std::sync::Mutex;

use tauri::State;

use crate::core::database::Database;
use crate::core::git;
use crate::core::models::GitInfo;

/// 获取指定项目的 git 信息（只读）。
///
/// - `project_id` 对应的项目不存在 → 返回 `Err("项目不存在: ...")`。
/// - 项目非 git 仓库 / `.git` 损坏 / 权限不足 → 返回 `Ok(None)`（优雅降级，
///   `GitInfo.is_git_repo` 为 `false`）。
///
/// 返回 `Result<Option<GitInfo>, String>`，沿用现有错误格式。
#[tauri::command]
pub fn get_project_git_info(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
) -> Result<Option<GitInfo>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let detail = db
        .get_project_detail(project_id)
        .map_err(|e| format!("查询项目失败: {e}"))?
        .ok_or_else(|| format!("项目不存在: {project_id}"))?;

    let info = git::analyze_git(std::path::Path::new(&detail.path));

    // 非 git 仓库：优雅降级返回 None
    if !info.is_git_repo {
        return Ok(None);
    }
    Ok(Some(info))
}
