//! 数据库 migrations。
//!
//! Sprint 1：落地 `projects` / `scan_history` 两张表。
//! SPRINT3-01：`projects` 表新增 `file_count` / `last_scan_at` 列（供前端
//! 「最近更新」排序与列表展示），以幂等 ALTER 兼容已建表的旧库。
//!
//! 迁移策略：每个变更一个 `&str`，逐个 `execute_batch`；所有 ALTER 均先检查
//! 目标列是否存在，保证幂等。

use rusqlite::Connection;

/// 执行全部迁移（幂等）。
pub fn run(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA_SQL)?;
    migrate_001_add_project_metrics(conn)?;
    migrate_002_add_workspace(conn)?;
    migrate_003_add_project_kind(conn)?;
    migrate_004_backfill_workspace(conn)?;
    Ok(())
}

/// Migration 001：`projects` 表补充扫描元数据列。
/// - `file_count`  ：项目文件数（扫描时统计，非实时）
/// - `last_scan_at`：最近一次扫描时间
///
/// 在 Rust 侧检查列是否存在后再 `ALTER TABLE ADD COLUMN`，保证幂等且兼容旧库。
fn migrate_001_add_project_metrics(conn: &Connection) -> rusqlite::Result<()> {
    add_column_if_missing(conn, "projects", "file_count", "INTEGER DEFAULT 0")?;
    add_column_if_missing(conn, "projects", "last_scan_at", "DATETIME")?;
    Ok(())
}

/// Migration 002：`projects` 表新增 `workspace` 列（SPRINT5-05）。
/// 记录项目归属的工作区根路径（扫描时写入）。
/// 兼容旧库：已存在的项目该列为 NULL（归入「全部」，不属于 Documents/Desktop）。
fn migrate_002_add_workspace(conn: &Connection) -> rusqlite::Result<()> {
    add_column_if_missing(conn, "projects", "workspace", "TEXT")?;
    Ok(())
}

/// Migration 003：`projects` 表新增项目边界 / 健康度列（v0.2 Scanner 迭代）。
/// - `kind`         ：项目类型（`real` / `aggregated_root` / `category`），默认 `real`
/// - `health_score` ：健康度评分（0-100），默认 0
/// - `parent_id`    ：父项目 id（聚合根 / 分类目录树形归属），可空
///
/// 兼容旧库：已存在项目 `kind` 回退 `real`、`health_score` 为 0、`parent_id` 为 NULL。
fn migrate_003_add_project_kind(conn: &Connection) -> rusqlite::Result<()> {
    add_column_if_missing(conn, "projects", "kind", "TEXT DEFAULT 'real'")?;
    add_column_if_missing(conn, "projects", "health_score", "INTEGER DEFAULT 0")?;
    add_column_if_missing(conn, "projects", "parent_id", "INTEGER")?;
    Ok(())
}

/// Migration 004：旧数据回溯回填 `workspace`（V02-BUG-BACKEND，阻塞级）。
///
/// 背景：v0.1 手动选择工作区扫描入库的项目，当时 `projects` 表还没有
/// `workspace` 列（v0.2 Phase 1 才加），导致这些旧项目 `workspace = NULL`，
/// 前端按 Documents/Desktop 筛选时漏掉它们。
///
/// 本迁移对 `workspace IS NULL` 的旧项目，按 `path` 前缀回填：
/// - `path` 以 `~/Documents/` 开头 → `workspace = ~/Documents`
/// - `path` 以 `~/Desktop/` 开头  → `workspace = ~/Desktop`
/// 基于 `dirs::home_dir()` 拼接（不硬编码用户名）；无法匹配的保持 NULL。
///
/// **幂等**：仅更新 `workspace IS NULL` 的行；已回填（非 NULL）的行不处理，
/// 因此迁移重跑不会重复或覆盖。`workspace` 列由 migration 002 保证已存在。
fn migrate_004_backfill_workspace(conn: &Connection) -> rusqlite::Result<()> {
    let Some(home) = dirs::home_dir() else {
        // 无法解析 home（极端环境），跳过回溯，不阻断启动。
        return Ok(());
    };

    let documents = home.join("Documents");
    let desktop = home.join("Desktop");

    // 仅处理 workspace IS NULL 的行，保证幂等（已回填的不再改动）。
    backfill_by_prefix(conn, &documents, "~/Documents")?;
    backfill_by_prefix(conn, &desktop, "~/Desktop")?;
    Ok(())
}

/// 将 `path` 以 `prefix`（规范化系统目录）开头的 `workspace IS NULL` 项目，
/// 回填 workspace 为 `prefix` 的字符串形式。
///
/// 前缀匹配保证：`~/Documents/foo` 匹配，但 `~/Documents2` 不匹配（需带路径分隔符）。
fn backfill_by_prefix(
    conn: &Connection,
    sys_dir: &std::path::Path,
    _label: &str,
) -> rusqlite::Result<()> {
    let prefix_str = sys_dir.to_string_lossy().to_string();
    // 匹配：path == prefix（理论不出现）或 path 以 prefix + '/' 开头。
    let like = format!("{}/%", prefix_str.trim_end_matches('/'));
    conn.execute(
        "UPDATE projects
         SET workspace = ?1
         WHERE workspace IS NULL
           AND (path = ?2 OR path LIKE ?3)",
        rusqlite::params![prefix_str, prefix_str, like],
    )?;
    Ok(())
}

/// 若 `table` 不存在名为 `column` 的列，则执行 `ALTER TABLE ... ADD COLUMN`。
fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    decl: &str,
) -> rusqlite::Result<()> {
    let exists: i64 = conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2"
        ),
        rusqlite::params![table, column],
        |r| r.get(0),
    )?;
    if exists == 0 {
        let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {decl}");
        conn.execute_batch(&sql)?;
    }
    Ok(())
}

/// 建表 SQL（对齐 spec 第 7 节）。
const SCHEMA_SQL: &str = r#"
-- 项目表
CREATE TABLE IF NOT EXISTS projects (
    id         INTEGER PRIMARY KEY,
    name       TEXT,
    path       TEXT UNIQUE,
    language   TEXT,
    framework  TEXT,
    created_at DATETIME,
    updated_at DATETIME
);

-- 扫描历史表
CREATE TABLE IF NOT EXISTS scan_history (
    id         INTEGER PRIMARY KEY,
    workspace  TEXT,
    scan_time  DATETIME,
    status     TEXT
);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_create_expected_tables() {
        let conn = Connection::open_in_memory().expect("无法创建内存数据库");
        run(&conn).expect("迁移应成功");

        // 校验 projects 表结构（含新增列）
        let project_cols = conn
            .prepare("PRAGMA table_info(projects)")
            .expect("准备查询失败")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("查询失败")
            .collect::<Result<Vec<_>, _>>()
            .expect("读取失败");
        assert_eq!(
            project_cols,
            vec![
                "id",
                "name",
                "path",
                "language",
                "framework",
                "created_at",
                "updated_at",
                "file_count",
                "last_scan_at",
                "workspace",
                "kind",
                "health_score",
                "parent_id"
            ]
        );

        // 校验 scan_history 表结构
        let history_cols = conn
            .prepare("PRAGMA table_info(scan_history)")
            .expect("准备查询失败")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("查询失败")
            .collect::<Result<Vec<_>, _>>()
            .expect("读取失败");
        assert_eq!(history_cols, vec!["id", "workspace", "scan_time", "status"]);
    }

    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().expect("无法创建内存数据库");
        // 跑两次，应不报错且列不重复
        run(&conn).expect("第一次迁移应成功");
        run(&conn).expect("第二次迁移应成功");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('projects') WHERE name IN ('file_count','last_scan_at','workspace','kind','health_score','parent_id')",
                [],
                |r| r.get(0),
            )
            .expect("查询应成功");
        assert_eq!(count, 6, "新增列应各只有一列");
    }

    // ---- V02-BUG-BACKEND：旧数据回溯回填 workspace ----

    /// 插入一条指定 workspace 状态的项目行。
    fn insert_project(conn: &Connection, path: &str, workspace: Option<&str>) {
        conn.execute(
            "INSERT INTO projects (name, path, workspace) VALUES (?1, ?2, ?3)",
            rusqlite::params!["p", path, workspace],
        )
        .expect("插入应成功");
    }

    /// 读取单条 path 的 workspace 值。
    fn workspace_of(conn: &Connection, path: &str) -> Option<String> {
        conn.query_row(
            "SELECT workspace FROM projects WHERE path = ?1",
            rusqlite::params![path],
            |r| r.get(0),
        )
        .expect("查询应成功")
    }

    /// 回溯迁移：Documents 旧项目（workspace=NULL）按 path 前缀回填为 ~/Documents；
    /// 已回填/无法匹配的项目保持原状。
    #[test]
    fn backfill_workspace_for_legacy_documents_and_desktop() {
        let home = dirs::home_dir().expect("应能解析 home");
        let docs = home.join("Documents").to_string_lossy().to_string();
        let desk = home.join("Desktop").to_string_lossy().to_string();

        // 用真实 home 下的路径模拟旧数据（避免依赖硬编码用户名）。
        let doc_proj = format!("{docs}/liquid glass demo/app");
        let desk_proj = format!("{desk}/proj");
        let unmatched = "/tmp/somewhere-else/proj"; // 无法匹配，应保持 NULL
        let already_filled = format!("{docs}/already"); // 已有 workspace，不应被覆盖

        let conn = Connection::open_in_memory().expect("无法创建内存数据库");
        run(&conn).expect("首次迁移应成功"); // 空表，004 无操作

        // 注入旧数据（workspace=NULL）+ 一个已有 workspace 的行
        insert_project(&conn, &doc_proj, None);
        insert_project(&conn, &desk_proj, None);
        insert_project(&conn, unmatched, None);
        insert_project(&conn, &already_filled, Some(&desk)); // 故意放 Desktop，验证不被覆盖

        // 再次跑迁移触发 004 回填
        run(&conn).expect("二次迁移应成功");

        assert_eq!(workspace_of(&conn, &doc_proj).as_deref(), Some(docs.as_str()), "Documents 旧项目应回填 ~/Documents");
        assert_eq!(workspace_of(&conn, &desk_proj).as_deref(), Some(desk.as_str()), "Desktop 旧项目应回填 ~/Desktop");
        assert_eq!(workspace_of(&conn, unmatched), None, "无法匹配的项目应保持 NULL");
        assert_eq!(workspace_of(&conn, &already_filled).as_deref(), Some(desk.as_str()), "已有 workspace 不应被覆盖");
    }

    /// 回溯迁移幂等：重跑不会重复处理，也不会覆盖已回填的值。
    #[test]
    fn backfill_workspace_is_idempotent() {
        let home = dirs::home_dir().expect("应能解析 home");
        let docs = home.join("Documents").to_string_lossy().to_string();
        let doc_proj = format!("{docs}/legacy/app");

        let conn = Connection::open_in_memory().expect("无法创建内存数据库");
        run(&conn).expect("首次迁移应成功");
        insert_project(&conn, &doc_proj, None);

        // 跑多次迁移，结果应一致且稳定
        run(&conn).expect("二次迁移应成功");
        run(&conn).expect("三次迁移应成功");
        assert_eq!(workspace_of(&conn, &doc_proj).as_deref(), Some(docs.as_str()));

        // 回填后，故意再插一个 NULL 新行，再次迁移仍能回填
        let doc_proj2 = format!("{docs}/another");
        insert_project(&conn, &doc_proj2, None);
        run(&conn).expect("四次迁移应成功");
        assert_eq!(workspace_of(&conn, &doc_proj2).as_deref(), Some(docs.as_str()), "新 NULL 行也应被回填");
    }
}
