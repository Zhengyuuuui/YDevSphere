//! 项目相关命令（薄壳层，真实业务转发到 core）。
//!
//! 职责：参数解析 + 调用 `core`（scanner / parser / database）转发。
//! 本层仅做适配，不含业务逻辑，`core/` 保持无 `use tauri`。

use std::sync::Mutex;

use tauri::{Manager, State};

use crate::core::database::Database;
use crate::core::editor;
use crate::core::models::{
    DirNode, Project, ProjectDetail, ScanCommandError, ScanHistory, ScanResult,
};
use crate::core::scanner;

/// 扫描指定工作区目录，识别项目、解析技术栈、写入数据库。
///
/// 返回 `ScanResult`（含项目列表、扫描历史、扫描统计）。
///
/// v0.2：读取用户自定义忽略规则（`~/.ydevsphere/settings.json` 的 `ignore_dirs`），
/// 叠加到 scanner 预设规则上；扫描后执行同步清理删除幽灵项目。
///
/// 错误类型为 `ScanCommandError`（结构化 `{ code, message }`），前端按 `code`
/// 分支处理，不依赖中文字符串：
/// - `INVALID_DIRECTORY`：工作区路径不存在/不可读
/// - `IO_ERROR`：目录遍历失败
/// - `DB_ERROR`：数据库写入失败
/// - `INTERNAL_ERROR`：内部异常（锁/线程）
///
/// `async fn` + `spawn_blocking`：`scan_workspace`（递归目录遍历）、
/// `upsert_projects`（含 `count_files` 递归文件计数）与数据库写入均为同步 IO /
/// CPU 密集工作。整体移入阻塞线程池，避免占用 macOS 主线程导致 UI 冻结。
#[tauri::command]
pub async fn scan_projects(
    app: tauri::AppHandle,
    workspace_path: String,
) -> Result<ScanResult, ScanCommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = std::path::Path::new(&workspace_path);

        // 1. 读取用户自定义忽略规则（读取失败当作空列表，不阻断扫描）。
        let ignore_dirs = editor::get_ignore_dirs().unwrap_or_default();

        // 2. 扫描目录，识别项目（预设忽略 + 用户自定义忽略）。
        let options = scanner::ScanOptions {
            max_depth: 6,
            extra_ignored_dirs: ignore_dirs,
        };
        let output = scanner::scan_workspace_with_options(path, &options).map_err(|e| {
            ScanCommandError {
                code: e.code().to_string(),
                message: e.to_string(),
            }
        })?;
        let ignored_count = output.ignored_count;

        // 3. 为识别到的每个项目标记归属的工作区根路径（SPRINT5-05）
        let mut detected = output.projects;
        for p in detected.iter_mut() {
            p.workspace = Some(workspace_path.clone());
        }

        // 4. upsert 入库（批量事务），得到带 id 的项目列表
        let db = app.state::<Mutex<Database>>();
        let db = db.lock().map_err(|e| ScanCommandError::internal(format!("锁数据库失败: {e}")))?;
        let projects = db
            .upsert_projects(&detected)
            .map_err(|e| ScanCommandError::db(format!("写入数据库失败: {e}")))?;

        // 5. 同步清理（v0.2）：删除该工作区下磁盘已不存在的幽灵项目。
        let _removed = db
            .delete_missing_projects(&workspace_path)
            .map_err(|e| ScanCommandError::db(format!("同步清理失败: {e}")))?;

        // 6. 写入扫描历史
        let status = if projects.is_empty() {
            "empty"
        } else {
            "success"
        };
        let history = db
            .insert_scan_history(&workspace_path, status)
            .map_err(|e| ScanCommandError::db(format!("写入扫描历史失败: {e}")))?;

        let scanned_count = projects.len();
        Ok(ScanResult {
            projects,
            history,
            scanned_count,
            ignored_count,
        })
    })
    .await
    .map_err(|e| ScanCommandError::internal(format!("扫描线程异常: {e}")))?
}

/// 按需返回指定目录的直接子项（`DirNode[]`），供前端懒加载目录树。
///
/// 仅返回直接子项，不递归；隐藏项与预设忽略目录（node_modules 等）跳过。
/// 目录不存在 / 不可读返回空列表（静默降级）。
#[tauri::command]
pub fn get_dir_children(path: String) -> Vec<DirNode> {
    scanner::list_dir_children(std::path::Path::new(&path))
}

/// 从数据库读取项目列表。
///
/// - `sort_by`：`"name"`（名称升序）或 `"updated_at"`（默认，最近扫描倒序）；不传/非法值回退默认。
/// - `workspace_filter`：`"all"`（默认，不过滤）/ `"documents"` / `"desktop"`；不传/`null`/非法值回退 `all`。
/// - `kind_filter`（v0.2）：`"real"` / `"aggregated_root"` / `"category"`；不传不过滤。
/// - `parent_id_filter`（v0.2）：默认（`null`）只返回顶层项目（`parent_id IS NULL`）；
///   传父项目 id 返回其直接子项目；`-1` 等价默认顶层；`i64::MIN` 返回全部（含子项目）。
#[tauri::command]
pub fn get_projects(
    db: State<'_, Mutex<Database>>,
    sort_by: Option<String>,
    workspace_filter: Option<String>,
    kind_filter: Option<String>,
    parent_id_filter: Option<i64>,
) -> Result<Vec<Project>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_projects(
        sort_by.as_deref(),
        workspace_filter.as_deref(),
        kind_filter.as_deref(),
        parent_id_filter,
    )
    .map_err(|e| e.to_string())
}

/// 读取最近扫描历史（供前端「最近扫描摘要」）。
///
/// `limit` 可选，默认 20，钳制在 1..=200。
#[tauri::command]
pub fn get_scan_history(
    db: State<'_, Mutex<Database>>,
    limit: Option<usize>,
) -> Result<Vec<ScanHistory>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_scan_history(limit.unwrap_or(20))
        .map_err(|e| e.to_string())
}

/// 读取单个项目详情。
#[tauri::command]
pub fn get_project_detail(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
) -> Result<Option<ProjectDetail>, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    db.get_project_detail(project_id).map_err(|e| e.to_string())
}
