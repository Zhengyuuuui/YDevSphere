//! 项目记忆相关命令（薄壳层）。
//!
//! 职责：参数解析 + 查询数据库项目信息 + 转发到 `core::memory`。
//! 本层仅做适配，不含业务逻辑。
//!
//! ⚠️ 安全红线：所有写操作要求前端显式传入 `authorized: true` 才执行，
//! 且 `core::memory` 仅写 `<project>/.ydevsphere/project.json`。

use std::sync::Mutex;

use tauri::State;

use crate::core::database::Database;
use crate::core::memory::{self, MemoryError};
use crate::core::models::{ProjectMemory, ProjectRef};

/// 为指定项目生成（或刷新）`.ydevsphere/project.json`。
///
/// - `authorized` 必须为 `true`，否则拒绝写入（安全红线）。
/// - `package_manager` 可选；`null` 时由 lockfile 自动检测。
///
/// 返回生成后的 `ProjectMemory`。
#[tauri::command]
pub fn ensure_project_memory(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    package_manager: Option<String>,
    authorized: bool,
) -> Result<ProjectMemory, String> {
    let project_ref = fetch_project_ref(&db, project_id)?;
    memory::ensure_project_memory(&project_ref, authorized, package_manager.as_deref())
        .map_err(map_memory_err)
}

/// 读取项目记忆；不存在返回 `null`。
#[tauri::command]
pub fn get_project_memory(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
) -> Result<Option<ProjectMemory>, String> {
    let project_ref = fetch_project_ref(&db, project_id)?;
    memory::get_project_memory(std::path::Path::new(&project_ref.path))
        .map_err(map_memory_err)
}

/// 更新项目记忆字段（幂等刷新）。
///
/// - `authorized` 必须为 `true`。
/// - `package_manager` / `stack` 可选；`null` 时保留既有值。
#[tauri::command]
pub fn update_project_memory(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    package_manager: Option<String>,
    stack: Option<Vec<String>>,
    authorized: bool,
) -> Result<ProjectMemory, String> {
    let project_ref = fetch_project_ref(&db, project_id)?;
    memory::update_project_memory(
        &project_ref,
        authorized,
        package_manager.as_deref(),
        stack,
    )
    .map_err(map_memory_err)
}

/// 从数据库查询项目，构造 `ProjectRef`。
fn fetch_project_ref(db: &Mutex<Database>, project_id: i64) -> Result<ProjectRef, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let detail = db
        .get_project_detail(project_id)
        .map_err(|e| format!("查询项目失败: {e}"))?
        .ok_or_else(|| format!("项目不存在: {project_id}"))?;
    Ok(ProjectRef::new(
        detail.name,
        detail.path,
        detail.language,
        detail.framework,
    ))
}

/// 将 `MemoryError` 映射为前端可读的 `String`。
fn map_memory_err(e: MemoryError) -> String {
    e.to_string()
}
