//! 共享测试 fixture 基础设施（供 PR2-4 复用）。
//!
//! 本模块为集成测试（`tests/`）提供：
//! - `TempDir`：自动清理的临时目录（抽取自各模块散落的 fake_app / tmp_dir 模式）。
//! - `vue_project()` / `express_project()`：Spec §13 测试矩阵基础项目。
//! - `frontend_backend()`：构造「藏蓝闪送」结构（前端 Vue + 后端 Express/SQLite）。
//! - `memory_db()`：创建已迁移的内存 SQLite 数据库，供「落库/读回」断言。
//!
//! 约定：集成测试在文件顶部 `mod common;` 引入，再 `use common::...;`。
//!
//! 每个测试二进制只用本模块的一个子集（common/mod.rs 按测试文件分别编译），
//! 故整体放行 dead_code。

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 自增计数器：避免并行测试临时目录/路径冲突。
static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 自动清理的临时目录句柄（Drop 时递归删除）。
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// 创建唯一临时目录（前缀区分用途，供 PR2-4 复用）。
    pub fn new(prefix: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "ydevsphere_fixture_{}_{}_{}",
            std::process::id(),
            prefix,
            n
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("创建临时目录失败");
        Self { path }
    }

    /// 临时目录根路径。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 临时目录根的字符串形式。
    pub fn path_str(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    /// 写入相对路径文件（自动创建父目录）。
    pub fn write(&self, rel: &str, content: &str) -> &Self {
        let path = self.path.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("创建父目录失败");
        }
        std::fs::write(path, content).expect("写入失败");
        self
    }

    /// 创建相对路径目录。
    pub fn mkdir(&self, rel: &str) -> &Self {
        std::fs::create_dir_all(self.path.join(rel)).expect("创建目录失败");
        self
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// 构造「藏蓝闪送」结构的 frontend-backend fixture（Spec §13 `frontend-backend/`）。
///
/// 结构：
/// ```text
/// package.json         ← 根（workspaces 指向 frontend/backend）
/// frontend/package.json   ← Vue + Pinia（前端）
/// backend/package.json    ← Express + better-sqlite3（后端）
/// ```
///
/// 根含 workspace 信号（P0-4 场景的根 + 子项目），供 PR3 边界识别复用。
pub fn frontend_backend() -> TempDir {
    let dir = TempDir::new("frontend_backend");
    dir.write(
        "package.json",
        r#"{"name":"canglan-fast-delivery","private":true,"workspaces":["frontend","backend"]}"#,
    )
    .write(
        "frontend/package.json",
        r#"{"name":"frontend","dependencies":{"vue":"^3.4.0","pinia":"^2.1.0"}}"#,
    )
    .write(
        "backend/package.json",
        r#"{"name":"backend","dependencies":{"express":"^4.18.0","better-sqlite3":"^9.0.0"}}"#,
    );
    dir
}

/// 构造 vue-project fixture（Spec §13）：Vue + TypeScript + Vite。
///
/// 预期识别（PR2 契约）：vue / typescript / vite / nodejs。
pub fn vue_project() -> TempDir {
    let dir = TempDir::new("vue_project");
    dir.write(
        "package.json",
        r#"{
            "name": "vue-project",
            "dependencies": { "vue": "^3.4.0", "pinia": "^2.1.0" },
            "devDependencies": { "typescript": "^5.4.0", "vite": "^5.0.0" }
        }"#,
    );
    dir
}

/// 构造 express-project fixture（Spec §13）：Express + better-sqlite3 + node 脚本。
///
/// 预期识别（PR2 契约）：nodejs / express / sqlite / javascript。
pub fn express_project() -> TempDir {
    let dir = TempDir::new("express_project");
    dir.write(
        "package.json",
        r#"{
            "name": "express-project",
            "dependencies": { "express": "^4.18.0", "better-sqlite3": "^9.0.0" },
            "scripts": { "start": "node server.js" }
        }"#,
    );
    dir
}

/// 构造已迁移的内存 SQLite 数据库（供「落库/读回」断言）。
///
/// 复用 `core::database::migrations`，与生产库结构一致（含 PR1 新增列）。
pub fn memory_db() -> ydevsphere_lib::core::database::Database {
    let conn = rusqlite::Connection::open_in_memory().expect("创建内存数据库失败");
    ydevsphere_lib::core::database::migrations::run(&conn).expect("迁移失败");
    ydevsphere_lib::core::database::Database::from_conn(conn)
}
