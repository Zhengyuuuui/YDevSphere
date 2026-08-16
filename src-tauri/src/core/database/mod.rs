//! 数据库模块（SQLite）。
//!
//! 职责：
//! - 连接初始化（目标路径 `~/.ydevsphere/database.sqlite`）
//! - migrations（`projects` / `scan_history` 建表）
//! - CRUD 业务（upsert / 查询 / 扫描历史）
//!
//! 数据库驱动固定为 `rusqlite`（bundled）。

pub mod connection;
pub mod crud;
pub mod migrations;

pub use connection::{init, Database};
