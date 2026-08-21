//! 数据库 CRUD 业务层。
//!
//! 实现：
//! - `upsert_projects`：按 `path` 唯一键 upsert，大批量写入用单事务
//! - `get_projects`：读取项目列表
//! - `get_project_detail`：读取单个项目详情
//! - `insert_scan_history`：写入扫描历史
//!
//! 本模块方法挂在 `connection::Database` 上，但不依赖 tauri。

use rusqlite::params;

use super::connection::Database;
use crate::core::models::{
    DetectedProject, Project, ProjectDetail, ProjectKind, ScanHistory,
    TechnologiesJson,
};

/// 当前进程统一的「当前时间」字符串（RFC3339，UTC），
/// 供一次扫描内创建/更新时间保持一致。
fn now_string() -> String {
    // 用 chrono 需引入额外依赖；这里用系统时间格式化，避免引入 chrono。
    // 采用人类可读的 UTC 时间格式：YYYY-MM-DD HH:MM:SS（与 DATETIME 字段对齐）。
    format_utc_now()
}

/// 生成当前 UTC 时间字符串（YYYY-MM-DD HH:MM:SS）。
fn format_utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 0 到 2106 年之间均可，避免 i64 溢出。
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days as i64);
    let (hh, mm, ss) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}")
}

/// 从 UNIX 天数转换为公历日期（Howard Hinnant 算法）。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

impl Database {
    /// 批量 upsert 项目（按 `path` 唯一）。全部写入在单个事务内完成。
    ///
    /// 扫描时会为每个项目统计 `file_count`、写入 `last_scan_at`（当前时间），
    /// 以及 v0.2 新增的 `kind` / `health_score` / `parent_id`（由 `parent_path`
    /// 关联到父项目 id）。
    ///
    /// 流程：先 upsert 全部项目（`parent_id` 置 NULL）→ 再按 `parent_path` 回填
    /// `parent_id`，保证父/子 id 均已确定。
    ///
    /// 返回最新 upsert 后带 id 的项目列表（保持输入顺序）。
    pub fn upsert_projects(
        &self,
        detected: &[DetectedProject],
    ) -> rusqlite::Result<Vec<Project>> {
        let conn = self.connection();
        let now = now_string();

        // 使用显式 BEGIN/COMMIT 事务（rusqlite execute_batch 接受 &self，
        // 避免对 &Connection 做 &mut 借用的复杂性）。
        conn.execute_batch("BEGIN IMMEDIATE;")?;
        let tx_result = (|| -> rusqlite::Result<()> {
            let mut stmt = conn.prepare_cached(
                "INSERT INTO projects
                   (name, path, language, framework, created_at, updated_at, file_count, last_scan_at, workspace, kind, health_score, parent_id, technologies_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, NULL, ?12)
                 ON CONFLICT(path) DO UPDATE SET
                    name = excluded.name,
                    language = excluded.language,
                    framework = excluded.framework,
                    updated_at = excluded.updated_at,
                    file_count = excluded.file_count,
                    last_scan_at = excluded.last_scan_at,
                    workspace = excluded.workspace,
                    kind = excluded.kind,
                    health_score = excluded.health_score,
                    technologies_json = excluded.technologies_json",
            )?;

            for p in detected {
                let file_count = count_files(&p.path);
                let tech_json = TechnologiesJson::new(p.technologies.clone()).encode();
                stmt.execute(params![
                    p.name,
                    p.path,
                    p.language,
                    p.framework,
                    now,
                    now,
                    file_count,
                    now,
                    p.workspace,
                    p.kind.as_db(),
                    p.health_score,
                    tech_json,
                ])?;
            }

            // 回填 parent_id：按 parent_path 关联父项目 id。
            for p in detected {
                if let Some(parent_path) = &p.parent_path {
                    conn.execute(
                        "UPDATE projects SET parent_id = (
                            SELECT id FROM projects WHERE path = ?1
                         ) WHERE path = ?2",
                        params![parent_path, p.path],
                    )?;
                }
            }
            Ok(())
        })();
        // 无论成功与否都结束事务
        conn.execute_batch("COMMIT;")?;
        tx_result?;

        // 读取最新 id 列表，按输入顺序返回完整 Project。
        let mut projects = Vec::with_capacity(detected.len());
        for p in detected {
            projects.push(self.get_project_by_path(&p.path)?);
        }
        Ok(projects)
    }

    /// 按 path 读取单个项目。
    fn get_project_by_path(&self, path: &str) -> rusqlite::Result<Project> {
        let sql = format!("{SELECT_PROJECT_SQL} WHERE path = ?1");
        self.connection()
            .query_row(&sql, params![path], project_from_row)
    }

    /// 读取项目列表。
    ///
    /// - `sort_by`：`"name"`（名称升序）或 `"updated_at"`（默认，最近扫描倒序）；非法值回退默认。
    /// - `workspace_filter`：`"all"`（默认，不过滤）/ `"documents"` / `"desktop"`；`null`/不传/非法值回退 `all`。
    /// - `kind_filter`（v0.2）：`None` 不过滤；`Some("real" / "aggregated_root" / "category")` 按类型过滤。
    /// - `parent_id_filter`（v0.2）：见下方语义。
    ///
    /// ## 父项目边界优先的落地（v0.2，`docs/v0.2-scanner-plan.md` §2.1）
    ///
    /// 聚合根 / 分类目录的**后代不生成卡片**，但为支持树形展开仍入库（带 `parent_id`）。
    /// 因此列表默认只返回顶层项目，子项目按需获取：
    /// - `None`（默认）→ 只返回顶层项目（`parent_id IS NULL`），即一张卡片对应一个顶层项目。
    /// - `Some(id)`（`id >= 0`）→ 返回该父项目下的直接子项目（前端展开聚合根/分类目录时用）。
    /// - `Some(-1)` → 显式「顶层」（等价默认，防御性保留）。
    /// - `Some(i64::MIN)` → 返回全部（含子项目，供统计 / 调试，非列表常规用途）。
    ///
    /// 筛选规则：
    /// - `documents`：`workspace == ~/Documents` 或以 `~/Documents/` 开头。
    /// - `desktop`：`workspace == ~/Desktop` 或以 `~/Desktop/` 开头。
    /// - `all`：不过滤（含 NULL workspace 的旧项目 / 手动目录）。
    pub fn get_projects(
        &self,
        sort_by: Option<&str>,
        workspace_filter: Option<&str>,
        kind_filter: Option<&str>,
        parent_id_filter: Option<i64>,
    ) -> rusqlite::Result<Vec<Project>> {
        let order = match sort_by {
            Some("name") => "name COLLATE NOCASE ASC",
            // 最近更新：按扫描时间倒序；`id DESC` 作为同一秒内插入顺序的兜底
            _ => "last_scan_at DESC, updated_at DESC, id DESC",
        };

        // 逐步累积 WHERE 条件与绑定参数。
        let mut conds: Vec<String> = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some((cond, prefix)) = workspace_filter_condition(workspace_filter) {
            conds.push(cond);
            if let Some(p) = prefix {
                params.push(Box::new(p));
            }
        }

        if let Some(kind) = kind_filter {
            // 仅接受合法 kind 字符串，非法回退不过滤（向后兼容）。
            if matches!(kind, "real" | "aggregated_root" | "category") {
                conds.push("kind = ?".to_string());
                params.push(Box::new(kind.to_string()));
            }
        }

        // parent_id_filter（v0.2 语义，见方法注释）：
        // - `None`（默认）→ 只返回顶层项目（`parent_id IS NULL`）
        // - `Some(id)`（id >= 0）→ 返回该父项目下的直接子项目
        // - `Some(-1)` → 显式「顶层」（等价默认，防御性保留）
        // - `Some(i64::MIN)` → 返回全部（含子项目，供统计/调试）
        match parent_id_filter {
            Some(i64::MIN) => {}
            Some(-1) | None => conds.push("parent_id IS NULL".to_string()),
            Some(parent) => {
                conds.push("parent_id = ?".to_string());
                params.push(Box::new(parent));
            }
        }

        let where_sql = if conds.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conds.join(" AND "))
        };

        let sql = format!("{SELECT_PROJECT_SQL}{where_sql} ORDER BY {order}");
        let mut stmt = self.connection().prepare(&sql)?;

        // 将参数转为 &dyn ToSql 切片引用。
        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(&param_refs[..], project_from_row)?;
        rows.collect()
    }

    /// 读取单个项目详情（`file_count` / `last_scan_at` / `kind` 等直接读库，非实时计算）。
    pub fn get_project_detail(&self, id: i64) -> rusqlite::Result<Option<ProjectDetail>> {
        let conn = self.connection();
        let result = conn.query_row(
            "SELECT id, name, path, language, framework, created_at, updated_at, file_count, last_scan_at, workspace, kind, health_score, parent_id, technologies_json
             FROM projects WHERE id = ?1",
            params![id],
            project_detail_from_row,
        );
        match result {
            Ok(d) => Ok(Some(d)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 同步清理（reconciliation，v0.2）：删除指定工作区下磁盘上已不存在的项目。
    ///
    /// 场景：用户删除项目目录后重新扫描，此前入库的「幽灵项目」仍留在列表中。
    /// 本方法按 `workspace` 归属（等于工作区根或其子路径）查出项目，逐个检查
    /// 磁盘路径是否存在，删除已消失的项目。
    ///
    /// 返回被删除的项目数量。
    pub fn delete_missing_projects(&self, workspace: &str) -> rusqlite::Result<usize> {
        let conn = self.connection();
        let ws = workspace.trim_end_matches('/');

        // 查询该工作区下的所有项目 id + path。
        let mut stmt = conn.prepare(
            "SELECT id, path FROM projects
             WHERE workspace = ?1 OR workspace LIKE ?2",
        )?;
        let prefix = format!("{}/%", ws);
        let rows = stmt.query_map(params![ws, prefix], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut ids_to_delete: Vec<i64> = Vec::new();
        for row in rows {
            let (id, path) = row?;
            if !std::path::Path::new(&path).exists() {
                ids_to_delete.push(id);
            }
        }

        let mut deleted = 0usize;
        let mut del_stmt = conn.prepare("DELETE FROM projects WHERE id = ?1")?;
        for id in &ids_to_delete {
            deleted += del_stmt.execute(params![id])?;
        }
        Ok(deleted)
    }

    /// 写入一条扫描历史记录，返回带 id 的完整记录。
    pub fn insert_scan_history(
        &self,
        workspace: &str,
        status: &str,
    ) -> rusqlite::Result<ScanHistory> {
        let scan_time = now_string();
        self.connection().execute(
            "INSERT INTO scan_history (workspace, scan_time, status)
             VALUES (?1, ?2, ?3)",
            params![workspace, scan_time, status],
        )?;
        let id = self.connection().last_insert_rowid();
        Ok(ScanHistory {
            id,
            workspace: workspace.to_string(),
            scan_time,
            status: status.to_string(),
        })
    }

    /// 读取最近 `limit` 条扫描历史（按时间倒序）。
    pub fn get_scan_history(&self, limit: usize) -> rusqlite::Result<Vec<ScanHistory>> {
        let limit = limit.clamp(1, 200) as i64;
        let mut stmt = self.connection().prepare(
            "SELECT id, workspace, scan_time, status
             FROM scan_history
             ORDER BY scan_time DESC, id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], scan_history_from_row)?;
        rows.collect()
    }
}

/// 查询项目字段的基础 SELECT 前缀（含新增元数据列）。
const SELECT_PROJECT_SQL: &str =
    "SELECT id, name, path, language, framework, created_at, updated_at, file_count, last_scan_at, workspace, kind, health_score, parent_id, technologies_json FROM projects";

/// 计算 `workspace_filter` 对应的 WHERE 条件与绑定前缀。
///
/// 返回 `(cond, prefix)`；`prefix` 为 `~/Documents` 或 `~/Desktop` 的规范化路径
/// （基于 `dirs::home_dir()`，不硬编码用户名）。「全部」（all / null / 非法值）
/// 或无法解析系统目录时返回 `None`。
fn workspace_filter_condition(
    filter: Option<&str>,
) -> Option<(String, Option<String>)> {
    let dir = match filter {
        Some("documents") => system_dir("Documents"),
        Some("desktop") => system_dir("Desktop"),
        _ => return None, // "all" / null / 非法值 → 不过滤
    };
    let dir = dir?;
    // workspace == dir 或以 dir/ 开头
    Some((
        "(workspace = ?1 OR workspace LIKE ?1 || '/%')".to_string(),
        Some(dir),
    ))
}

/// 解析 `~/<name>` 目录（不存在返回 `None`）。
fn system_dir(name: &str) -> Option<String> {
    let home = dirs::home_dir()?;
    Some(home.join(name).to_string_lossy().to_string())
}

/// 从结果集行构造 `Project`（列序见 `SELECT_PROJECT_SQL`）。
fn project_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        language: row.get(3)?,
        framework: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        file_count: row.get(7)?,
        last_scan_at: row.get(8)?,
        workspace: row.get(9)?,
        kind: ProjectKind::from_db(row.get::<_, Option<String>>(10)?.as_deref()),
        health_score: row.get(11)?,
        parent_id: row.get(12)?,
        technologies: technologies_from_row(row),
    })
}

/// 从结果集行读取 `technologies_json`（索引 13）并解析为技术列表。
fn technologies_from_row(row: &rusqlite::Row<'_>) -> Vec<crate::core::models::Technology> {
    TechnologiesJson::decode(row.get::<_, Option<String>>(13).ok().flatten().as_deref())
        .technologies
        .to_vec()
}

/// 从结果集行构造 `ProjectDetail`（列序与 `project_from_row` 一致）。
fn project_detail_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectDetail> {
    Ok(ProjectDetail {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        language: row.get(3)?,
        framework: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        file_count: row.get(7)?,
        last_scan_at: row.get(8)?,
        workspace: row.get(9)?,
        kind: ProjectKind::from_db(row.get::<_, Option<String>>(10)?.as_deref()),
        health_score: row.get(11)?,
        parent_id: row.get(12)?,
        technologies: technologies_from_row(row),
    })
}

/// 从结果集行构造 `ScanHistory`。
fn scan_history_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ScanHistory> {
    Ok(ScanHistory {
        id: row.get(0)?,
        workspace: row.get(1)?,
        scan_time: row.get(2)?,
        status: row.get(3)?,
    })
}

/// 统计目录下文件数（只读，跳过隐藏目录与忽略目录，不递归进项目根内的清单目录）。
fn count_files(root: &str) -> i64 {
    const IGNORED: &[&str] = &[
        "node_modules",
        ".git",
        "target",
        "dist",
        "build",
        "vendor",
        ".cache",
    ];

    fn walk(dir: &std::path::Path, ignored: &[&str], count: &mut i64) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                if ignored.contains(&name.as_str()) {
                    continue;
                }
                walk(&path, ignored, count);
            } else if path.is_file() {
                *count += 1;
            }
        }
    }

    let mut count = 0i64;
    walk(std::path::Path::new(root), IGNORED, &mut count);
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// 创建内存数据库并完成迁移。
    fn memory_db() -> Database {
        let conn = Connection::open_in_memory().expect("创建内存库失败");
        super::super::migrations::run(&conn).expect("迁移失败");
        Database::from_conn(conn)
    }

    /// 构造一个测试项目。
    fn proj(name: &str, path: &str, lang: &str, fw: Option<&str>) -> DetectedProject {
        DetectedProject::new(
            name,
            path,
            Some(lang.to_string()),
            fw.map(String::from),
        )
    }

    #[test]
    fn upsert_inserts_and_updates_by_path() {
        let db = memory_db();

        let p1 = proj("app", "/tmp/app", "Node", Some("Vue"));
        let inserted = db
            .upsert_projects(&[p1])
            .expect("upsert 应成功");
        assert_eq!(inserted.len(), 1);
        assert_eq!(inserted[0].id, 1);
        assert_eq!(inserted[0].name, "app");

        // 同 path 再次 upsert，应更新而非新增
        let p2 = proj("app-renamed", "/tmp/app", "Node", Some("React"));
        let updated = db
            .upsert_projects(&[p2])
            .expect("再次 upsert 应成功");
        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].name, "app-renamed");
        assert_eq!(updated[0].framework.as_deref(), Some("React"));

        // 总数仍为 1
        let all = db.get_projects(None, None, None, None).expect("读取列表应成功");
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn get_projects_sorted_by_name() {
        let db = memory_db();
        db.upsert_projects(&[
            proj("zeta", "/tmp/zeta", "Rust", None),
            proj("alpha", "/tmp/alpha", "Go", None),
        ])
        .expect("upsert 应成功");

        let all = db.get_projects(Some("name"), None, None, None).expect("读取应成功");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].name, "alpha");
        assert_eq!(all[1].name, "zeta");
    }

    #[test]
    fn get_projects_default_sorted_by_recent_scan() {
        let db = memory_db();
        // 两次不同时间 upsert：后 upsert 的 last_scan_at 更大，应排前面
        db.upsert_projects(&[proj("older", "/tmp/older", "Node", None)])
            .expect("upsert 应成功");
        std::thread::sleep(std::time::Duration::from_millis(20));
        db.upsert_projects(&[proj("newer", "/tmp/newer", "Go", None)])
            .expect("upsert 应成功");

        // 默认按最近扫描倒序
        let all = db.get_projects(None, None, None, None).expect("读取应成功");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].name, "newer");
        assert_eq!(all[1].name, "older");

        // last_scan_at 应已填充
        assert!(all[0].last_scan_at.is_some());
    }

    #[test]
    fn get_projects_invalid_sort_falls_back() {
        let db = memory_db();
        db.upsert_projects(&[proj("app", "/tmp/app", "Node", None)])
            .expect("upsert 应成功");
        let all = db.get_projects(Some("bogus"), None, None, None).expect("读取应成功");
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn upsert_fills_file_count_and_last_scan_at() {
        let db = memory_db();
        let inserted = db
            .upsert_projects(&[proj("app", "/tmp/app", "Node", Some("Vue"))])
            .expect("upsert 应成功");
        assert_eq!(inserted[0].file_count, 0, "临时路径无文件");
        assert!(inserted[0].last_scan_at.is_some(), "last_scan_at 应被填充");
    }

    #[test]
    fn get_project_detail_missing_returns_none() {
        let db = memory_db();
        let detail = db.get_project_detail(999).expect("查询应成功");
        assert!(detail.is_none());
    }

    #[test]
    fn get_project_detail_present() {
        let db = memory_db();
        let proj = proj("app", "/tmp/app", "Node", Some("Vue"));
        let inserted = db.upsert_projects(&[proj]).expect("upsert 应成功");
        let detail = db
            .get_project_detail(inserted[0].id)
            .expect("查询应成功")
            .expect("应存在");
        assert_eq!(detail.name, "app");
        assert!(detail.file_count >= 0);
    }

    #[test]
    fn scan_history_insert_and_read_back() {
        let db = memory_db();
        let h = db
            .insert_scan_history("/tmp/ws", "success")
            .expect("插入应成功");
        assert_eq!(h.status, "success");
        assert_eq!(h.workspace, "/tmp/ws");
        assert!(h.id >= 1);

        // 读取验证
        let count: i64 = db
            .connection()
            .query_row("SELECT COUNT(*) FROM scan_history", [], |r| r.get(0))
            .expect("查询应成功");
        assert_eq!(count, 1);
    }

    #[test]
    fn get_scan_history_returns_recent_and_limited() {
        let db = memory_db();
        for i in 0..5 {
            db.insert_scan_history(&format!("/tmp/ws{i}"), "success")
                .expect("插入应成功");
        }

        // 默认读全部（limit 较大）
        let all = db.get_scan_history(100).expect("读取应成功");
        assert_eq!(all.len(), 5);

        // 限流
        let limited = db.get_scan_history(2).expect("读取应成功");
        assert_eq!(limited.len(), 2);
        // 时间倒序：最近一次（ws4）在最前
        assert_eq!(limited[0].workspace, "/tmp/ws4");
        assert_eq!(limited[1].workspace, "/tmp/ws3");

        // limit 上下界钳制
        assert_eq!(db.get_scan_history(0).expect("读取应成功").len(), 1);
    }

    #[test]
    fn batch_upsert_is_atomic() {
        let db = memory_db();
        let mut batch = Vec::new();
        for i in 0..200 {
            batch.push(proj(&format!("proj{i}"), &format!("/tmp/proj{i}"), "Node", None));
        }
        db.upsert_projects(&batch).expect("批量 upsert 应成功");
        assert_eq!(db.get_projects(None, None, None, None).expect("读取应成功").len(), 200);
    }

    // ---- workspace 筛选（SPRINT5-05） ----

    /// 带 workspace 的项目构造。
    fn proj_ws(name: &str, path: &str, workspace: Option<&str>) -> DetectedProject {
        DetectedProject::new_with_workspace(
            name,
            path,
            Some("Node".into()),
            None,
            workspace.map(String::from),
        )
    }

    #[test]
    fn upsert_writes_workspace_column() {
        let db = memory_db();
        db.upsert_projects(&[proj_ws("app", "/tmp/app", Some("/tmp/ws") )])
            .expect("upsert 应成功");

        // 读回 project 应含 workspace
        let all = db.get_projects(None, None, None, None).expect("读取应成功");
        assert_eq!(all[0].workspace.as_deref(), Some("/tmp/ws"));

        // 详情也应含 workspace
        let detail = db.get_project_detail(all[0].id).expect("读取应成功").expect("应存在");
        assert_eq!(detail.workspace.as_deref(), Some("/tmp/ws"));
    }

    // ---- V02-BUG-BACKEND：workspace 更新与扫描写入 ----

    /// B2：同 path 项目再次 upsert 时，workspace 应被刷新（ON CONFLICT 更新 workspace）。
    #[test]
    fn upsert_refreshes_workspace_on_existing_path() {
        let db = memory_db();

        // 首次：workspace = A
        db.upsert_projects(&[proj_ws("app", "/tmp/app", Some("/tmp/ws-A"))])
            .expect("首次 upsert 应成功");

        // 再次：同一 path，workspace = B（模拟项目挪到另一个工作区后重新扫描）
        db.upsert_projects(&[proj_ws("app", "/tmp/app", Some("/tmp/ws-B"))])
            .expect("再次 upsert 应成功");

        let all = db.get_projects(None, None, None, None).expect("读取应成功");
        assert_eq!(all.len(), 1, "同 path 不应重复");
        assert_eq!(all[0].workspace.as_deref(), Some("/tmp/ws-B"), "workspace 应被刷新为 B");
    }

    /// B3：一次扫描（批量 upsert）为每个项目（含聚合根/分类目录/子项目）写入 workspace。
    #[test]
    fn scan_upsert_writes_workspace_for_all_kinds() {
        let db = memory_db();
        let ws = "/Users/me/Documents";

        // 模拟一次扫描结果：聚合根 + 子项目 + 分类目录 + 独立真项目，全部归属同一工作区。
        let detected = vec![
            proj_kind_ws("sub2api", "/Users/me/Documents/sub2api", ProjectKind::AggregatedRoot, 30, None, Some(ws)),
            proj_kind_ws("frontend", "/Users/me/Documents/sub2api/frontend", ProjectKind::Real, 70, Some("/Users/me/Documents/sub2api"), Some(ws)),
            proj_kind_ws("backend", "/Users/me/Documents/sub2api/backend", ProjectKind::Real, 65, Some("/Users/me/Documents/sub2api"), Some(ws)),
            proj_kind_ws("学习", "/Users/me/Documents/学习", ProjectKind::Category, 5, None, Some(ws)),
            proj_kind_ws("standalone", "/Users/me/Documents/standalone", ProjectKind::Real, 60, None, Some(ws)),
        ];
        db.upsert_projects(&detected).expect("upsert 应成功");

        // 读全部（含子项目），验证每个项目的 workspace 都是 ws
        let all = db.get_projects(None, None, None, Some(i64::MIN)).expect("读取应成功");
        assert_eq!(all.len(), 5);
        for p in &all {
            assert_eq!(
                p.workspace.as_deref(),
                Some(ws),
                "项目 {} 的 workspace 应写入工作区",
                p.name
            );
        }
    }

    #[test]
    fn workspace_filter_documents_desktop_and_all() {
        let db = memory_db();
        let home = dirs::home_dir().expect("应能解析 home");
        let docs = home.join("Documents").to_string_lossy().to_string();
        let desk = home.join("Desktop").to_string_lossy().to_string();

        db.upsert_projects(&[
            proj_ws("doc-proj", "/tmp/doc-proj", Some(&docs)),
            proj_ws("doc-proj-nested", "/tmp/doc-nested", Some(&format!("{docs}/sub"))),
            proj_ws("desk-proj", "/tmp/desk-proj", Some(&desk)),
            proj_ws("manual", "/tmp/manual", Some("/tmp/manual-ws")),
            // 无 workspace（旧库 / 手动目录）
            proj("legacy", "/tmp/legacy", "Node", None),
        ])
        .expect("upsert 应成功");

        // all：全部 5 个
        let all = db.get_projects(None, Some("all"), None, None).expect("读取应成功");
        assert_eq!(all.len(), 5);

        // documents：2（doc-proj + doc-proj-nested，含前缀）
        let docs_only = db.get_projects(None, Some("documents"), None, None).expect("读取应成功");
        let mut names: Vec<_> = docs_only.iter().map(|p| p.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["doc-proj", "doc-proj-nested"]);

        // desktop：1
        let desk_only = db.get_projects(None, Some("desktop"), None, None).expect("读取应成功");
        assert_eq!(desk_only.len(), 1);
        assert_eq!(desk_only[0].name, "desk-proj");

        // NULL workspace 不归入 documents/desktop
        assert!(!docs_only.iter().any(|p| p.name == "legacy"));
        assert!(!desk_only.iter().any(|p| p.name == "legacy"));
    }

    #[test]
    fn workspace_filter_null_and_invalid_falls_back_to_all() {
        let db = memory_db();
        let home = dirs::home_dir().expect("应能解析 home");
        let docs = home.join("Documents").to_string_lossy().to_string();
        db.upsert_projects(&[
            proj_ws("doc-proj", "/tmp/doc-proj", Some(&docs)),
            proj("manual", "/tmp/manual", "Go", None),
        ])
        .expect("upsert 应成功");

        // null / 不传 → all
        assert_eq!(db.get_projects(None, None, None, None).expect("读取应成功").len(), 2);
        // 非法 filter → all
        assert_eq!(db.get_projects(None, Some("bogus"), None, None).expect("读取应成功").len(), 2);
        // 空串 → all
        assert_eq!(db.get_projects(None, Some(""), None, None).expect("读取应成功").len(), 2);
    }

    // ---- v0.2：kind / health_score / parent_id 落库与读取 ----

    /// 带 v0.2 元数据的项目构造。
    fn proj_kind(
        name: &str,
        path: &str,
        kind: ProjectKind,
        health: i64,
        parent_path: Option<&str>,
    ) -> DetectedProject {
        DetectedProject::new_with_kind(
            name,
            path,
            Some("Node".into()),
            None,
            None,
            kind,
            health,
            parent_path.map(String::from),
        )
    }

    /// 带 v0.2 元数据 + workspace 归属的项目构造（供同步清理测试）。
    fn proj_kind_ws(
        name: &str,
        path: &str,
        kind: ProjectKind,
        health: i64,
        parent_path: Option<&str>,
        workspace: Option<&str>,
    ) -> DetectedProject {
        DetectedProject::new_with_kind(
            name,
            path,
            Some("Node".into()),
            None,
            workspace.map(String::from),
            kind,
            health,
            parent_path.map(String::from),
        )
    }

    #[test]
    fn upsert_and_read_kind_health_parent() {
        let db = memory_db();

        // 聚合根 + 两个子项目
        db.upsert_projects(&[
            proj_kind("agg", "/tmp/agg", ProjectKind::AggregatedRoot, 30, None),
            proj_kind("frontend", "/tmp/agg/frontend", ProjectKind::Real, 70, Some("/tmp/agg")),
            proj_kind("backend", "/tmp/agg/backend", ProjectKind::Real, 65, Some("/tmp/agg")),
        ])
        .expect("upsert 应成功");

        // 默认只返回顶层项目（父项目边界优先）：仅聚合根 1 个。
        let tops = db.get_projects(None, None, None, None).expect("读取应成功");
        assert_eq!(tops.len(), 1, "默认列表应只含顶层项目");

        let agg = &tops[0];
        assert_eq!(agg.path, "/tmp/agg");
        assert_eq!(agg.kind, ProjectKind::AggregatedRoot);
        assert_eq!(agg.health_score, 30);
        assert!(agg.parent_id.is_none(), "聚合根应为顶层（parent_id None）");

        // 子项目经 parent_id_filter 获取，且 parent_id 指向聚合根 id
        let children = db.get_projects(None, None, None, Some(agg.id)).expect("读取应成功");
        assert_eq!(children.len(), 2, "聚合根下应有 2 个子项目");
        for c in &children {
            assert_eq!(c.parent_id, Some(agg.id), "子项目 parent_id 应指向聚合根 id");
        }
        let frontend = children.iter().find(|p| p.path.ends_with("/frontend")).expect("应有 frontend");
        assert_eq!(frontend.kind, ProjectKind::Real);
        assert_eq!(frontend.health_score, 70);

        // i64::MIN 返回全部（含子项目）
        let all = db.get_projects(None, None, None, Some(i64::MIN)).expect("读取应成功");
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn get_projects_filter_by_kind() {
        let db = memory_db();
        db.upsert_projects(&[
            proj_kind("real1", "/tmp/real1", ProjectKind::Real, 70, None),
            proj_kind("cat1", "/tmp/cat1", ProjectKind::Category, 5, None),
        ])
        .expect("upsert 应成功");

        // 按 kind 过滤
        let reals = db.get_projects(None, None, Some("real"), None).expect("读取应成功");
        assert_eq!(reals.len(), 1);
        assert_eq!(reals[0].name, "real1");

        let cats = db.get_projects(None, None, Some("category"), None).expect("读取应成功");
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, "cat1");

        // 非法 kind 回退不过滤
        assert_eq!(db.get_projects(None, None, Some("bogus"), None).expect("读取应成功").len(), 2);
    }

    #[test]
    fn get_projects_filter_by_parent_id() {
        let db = memory_db();
        db.upsert_projects(&[
            proj_kind("agg", "/tmp/agg", ProjectKind::AggregatedRoot, 30, None),
            proj_kind("frontend", "/tmp/agg/frontend", ProjectKind::Real, 70, Some("/tmp/agg")),
            proj_kind("backend", "/tmp/agg/backend", ProjectKind::Real, 65, Some("/tmp/agg")),
            proj_kind("top", "/tmp/top", ProjectKind::Real, 60, None),
        ])
        .expect("upsert 应成功");

        let agg = db
            .get_projects(None, None, None, None)
            .expect("读取应成功")
            .into_iter()
            .find(|p| p.path == "/tmp/agg")
            .expect("应找到聚合根");

        // 按 parent_id = agg.id 过滤：应返回 2 个子项目
        let children = db.get_projects(None, None, None, Some(agg.id)).expect("读取应成功");
        assert_eq!(children.len(), 2);
        assert!(children.iter().all(|p| p.parent_id == Some(agg.id)));

        // 顶层项目（parent_id IS NULL，哨兵 -1）：agg + top
        let tops = db.get_projects(None, None, None, Some(-1)).expect("读取应成功");
        let mut names: Vec<_> = tops.iter().map(|p| p.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["agg", "top"]);
    }

    #[test]
    fn delete_missing_projects_removes_ghost() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let ws = std::env::temp_dir().join(format!(
            "ydevsphere_reconcile_ws_{}_{}",
            std::process::id(),
            n
        ));
        std::fs::create_dir_all(&ws).expect("创建工作区失败");

        // 磁盘上真实存在的项目
        let existing = ws.join("existing");
        std::fs::create_dir_all(&existing).expect("创建 existing 失败");

        // 磁盘上不存在的「幽灵项目」
        let ghost = ws.join("ghost").to_string_lossy().to_string();

        let db = memory_db();
        let ws_str = ws.to_string_lossy().to_string();
        db.upsert_projects(&[
            proj_ws("existing", existing.to_str().unwrap(), Some(&ws_str)),
            proj_ws("ghost", &ghost, Some(&ws_str)),
            // 另一个工作区的项目，不应被误删
            proj_ws("other", "/tmp/other-ws/proj", Some("/tmp/other-ws")),
        ])
        .expect("upsert 应成功");
        assert_eq!(db.get_projects(None, None, None, None).expect("读取").len(), 3);

        // 同步清理：ghost 不存在应被删除，other（不同工作区）保留
        let removed = db.delete_missing_projects(&ws_str).expect("清理应成功");
        assert_eq!(removed, 1, "应删除 1 个幽灵项目");

        let remaining = db.get_projects(None, None, None, Some(i64::MIN)).expect("读取应成功");
        let mut names: Vec<_> = remaining.iter().map(|p| p.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["existing", "other"]);
    }

    /// 聚合根父子级联删除：删除聚合根整个目录 → 父 + 子项目均被清理。
    #[test]
    fn delete_missing_projects_cascades_aggregated_root() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let ws = std::env::temp_dir().join(format!(
            "ydevsphere_reconcile_agg_{}_{}",
            std::process::id(),
            n
        ));
        std::fs::create_dir_all(&ws).expect("创建工作区失败");

        // 聚合根目录（磁盘上不存在，模拟被删除）+ 其子项目（也不存在）
        let ws_str = ws.to_string_lossy().to_string();
        let agg = format!("{}/sub2api", ws_str);
        let frontend = format!("{}/sub2api/frontend", ws_str);
        let backend = format!("{}/sub2api/backend", ws_str);

        // 磁盘上保留一个真实存在的顶层项目，验证不被误删
        let existing = ws.join("existing");
        std::fs::create_dir_all(&existing).expect("创建 existing 失败");

        let db = memory_db();
        db.upsert_projects(&[
            proj_kind_ws("sub2api", &agg, ProjectKind::AggregatedRoot, 30, None, Some(&ws_str)),
            proj_kind_ws("frontend", &frontend, ProjectKind::Real, 70, Some(&agg), Some(&ws_str)),
            proj_kind_ws("backend", &backend, ProjectKind::Real, 65, Some(&agg), Some(&ws_str)),
            proj_kind_ws("existing", existing.to_str().unwrap(), ProjectKind::Real, 60, None, Some(&ws_str)),
        ])
        .expect("upsert 应成功");

        // 同步清理：agg + frontend + backend 均不存在 → 删除 3 个，existing 保留
        let removed = db.delete_missing_projects(&ws_str).expect("清理应成功");
        assert_eq!(removed, 3, "聚合根 + 2 子项目 + 应删除 3 个");

        let remaining = db.get_projects(None, None, None, Some(i64::MIN)).expect("读取应成功");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].name, "existing");
    }

    /// 聚合根部分删除：仅删除聚合根下某个子目录 → 父保留、对应子项目被清理。
    #[test]
    fn delete_missing_projects_keeps_parent_removes_child() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let ws = std::env::temp_dir().join(format!(
            "ydevsphere_reconcile_child_{}_{}",
            std::process::id(),
            n
        ));
        std::fs::create_dir_all(&ws).expect("创建工作区失败");

        let ws_str = ws.to_string_lossy().to_string();
        let agg = format!("{}/sub2api", ws_str);
        // 聚合根目录 + frontend 子目录在磁盘上存在
        std::fs::create_dir_all(format!("{}/sub2api/frontend", ws_str)).expect("创建 frontend 失败");
        // backend 子目录不存在（模拟被删除）
        let backend = format!("{}/sub2api/backend", ws_str);

        let db = memory_db();
        db.upsert_projects(&[
            proj_kind_ws("sub2api", &agg, ProjectKind::AggregatedRoot, 30, None, Some(&ws_str)),
            proj_kind_ws("frontend", &format!("{}/frontend", agg), ProjectKind::Real, 70, Some(&agg), Some(&ws_str)),
            proj_kind_ws("backend", &backend, ProjectKind::Real, 65, Some(&agg), Some(&ws_str)),
        ])
        .expect("upsert 应成功");

        // 同步清理：仅 backend 不存在 → 删除 1 个；父 + frontend 保留
        let removed = db.delete_missing_projects(&ws_str).expect("清理应成功");
        assert_eq!(removed, 1, "仅 backend 被删除");

        let remaining = db.get_projects(None, None, None, Some(i64::MIN)).expect("读取应成功");
        let mut names: Vec<_> = remaining.iter().map(|p| p.name.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["frontend", "sub2api"]);
    }

    // ---- V04-RECOGNITION（PR1）：parent_id / kind / technologies_json 落库与读回 ----

    use crate::core::models::{
        TechnologiesJson, Technology, TechnologyCategory,
    };

    fn vue_tech() -> Technology {
        Technology::new(
            "vue",
            "Vue",
            TechnologyCategory::Framework,
            Some("javascript".into()),
        )
    }

    /// technologies_json 落库并读回：DetectedProject 携带 technologies，
    /// upsert 后 Project / ProjectDetail 应能读回同样的技术列表。
    #[test]
    fn upsert_persists_and_reads_back_technologies() {
        let db = memory_db();

        // 构造带技术栈的真项目（parent = None，顶层）。
        let mut detected = DetectedProject::new_with_kind(
            "frontend",
            "/tmp/agg/frontend",
            Some("Node".into()),
            Some("Vue".into()),
            None,
            ProjectKind::Real,
            70,
            None,
        );
        detected.technologies = vec![vue_tech()];

        db.upsert_projects(&[detected]).expect("upsert 应成功");

        // 列表读回：technologies 应包含 vue
        let all = db.get_projects(None, None, None, None).expect("读取应成功");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].technologies, vec![vue_tech()]);
        // 旧字段 language/framework 仍兼容（已回读）
        assert_eq!(all[0].language.as_deref(), Some("Node"));
        assert_eq!(all[0].framework.as_deref(), Some("Vue"));

        // 详情读回：technologies 一致
        let detail = db
            .get_project_detail(all[0].id)
            .expect("查询应成功")
            .expect("应存在");
        assert_eq!(detail.technologies, vec![vue_tech()]);

        // 数据库原始列应含 schema_version
        let raw: String = db
            .connection()
            .query_row(
                "SELECT technologies_json FROM projects WHERE id = ?1",
                params![all[0].id],
                |r| r.get(0),
            )
            .expect("读取原始列应成功");
        let decoded = TechnologiesJson::decode(Some(&raw));
        assert_eq!(decoded.schema_version, 1);
        assert_eq!(decoded.technologies, vec![vue_tech()]);
    }

    /// 旧数据兼容：technologies_json 为 NULL（旧库）时，读回空技术列表而非报错，
    /// 且 language/framework 仍可用（前端 fallback 来源）。
    #[test]
    fn legacy_row_without_technologies_reads_empty() {
        let db = memory_db();

        // 绕过 upsert，直接插入一条「旧数据」行（technologies_json 为 NULL）
        db.connection()
            .execute(
                "INSERT INTO projects (name, path, language, framework, kind, parent_id)
                 VALUES ('legacy', '/tmp/legacy', 'Rust', NULL, 'real', NULL)",
                [],
            )
            .expect("插入旧数据行应成功");

        let all = db.get_projects(None, None, None, None).expect("读取应成功");
        assert_eq!(all.len(), 1);
        assert!(all[0].technologies.is_empty(), "旧数据应回退空技术列表");
        assert_eq!(all[0].language.as_deref(), Some("Rust"));
        assert_eq!(all[0].parent_id, None);
    }
}
