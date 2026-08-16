//! # 纯业务核心层（Core）
//!
//! **硬性约束：本模块及其子模块禁止 `use tauri`**，与桌面框架完全解耦。
//! 未来 Desktop / CLI / MCP 将共享此 core。
//!
//! - `scanner`: 目录扫描（Sprint 2 实现）
//! - `parser`: 技术栈解析（Sprint 2 实现）
//! - `database`: SQLite 连接与迁移（Sprint 1 已落地骨架）
//! - `models`: 核心数据结构（Project / ProjectDetail）
//! - `memory`: 项目记忆（`.ydevsphere/project.json`，SPRINT4-01 实现）
//! - `git`: Git 分析（P1-1，SPRINT5-01 实现；只读）
//! - `editor`: 编辑器检测 / 打开 / 偏好（SPRINT5-02 实现；白名单执行）
//! - `workspace`: 系统工作区（Documents / Desktop，SPRINT5-04 实现；只解析不扫描）

pub mod database;
pub mod editor;
pub mod git;
pub mod memory;
pub mod models;
pub mod parser;
pub mod scanner;
pub mod workspace;

/// 启动 core 层：初始化数据库连接并完成建表迁移。
///
/// 返回数据库句柄，供上层（commands / 未来 CLI / MCP）使用。
pub fn init_core() -> rusqlite::Result<database::Database> {
    database::init()
}
