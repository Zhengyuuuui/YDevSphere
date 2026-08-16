//! SQLite 连接初始化。
//!
//! 目标数据库路径：`~/.ydevsphere/database.sqlite`（对齐 spec 第 7 节）。

use std::path::PathBuf;

use rusqlite::Connection;

use super::migrations;

/// YDevSphere 应用数据目录名（位于用户主目录下）。
const APP_DIR: &str = ".ydevsphere";
/// SQLite 数据库文件名。
const DB_FILE: &str = "database.sqlite";

/// 计算应用数据目录：`~/.ydevsphere`
pub fn app_data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR)
}

/// 计算数据库文件完整路径：`~/.ydevsphere/database.sqlite`
pub fn database_path() -> PathBuf {
    app_data_dir().join(DB_FILE)
}

/// 数据库封装：持有已初始化（已建表）的连接。
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 打开（必要时创建目录与文件）并初始化连接、执行建表迁移。
    pub fn open() -> rusqlite::Result<Self> {
        let path = database_path();

        // 确保应用数据目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(e))
            })?;
        }

        let conn = Connection::open(&path)?;
        // 显式开启外键约束
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        migrations::run(&conn)?;

        Ok(Self { conn })
    }

    /// 暴露底层连接（供 CRUD / 查询使用）。
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// 从已有连接构造（供测试 / 复用连接场景使用）。
    pub fn from_conn(conn: Connection) -> Self {
        Self { conn }
    }

    /// 占位连接：在正式连接失败时提供，保证应用可启动。
    /// 使用内存数据库，仅用于保持程序不崩溃，不持久化。
    pub fn open_placeholder() -> Self {
        let conn = Connection::open_in_memory().expect("无法创建内存数据库");
        Self { conn }
    }
}

/// 便捷函数：打开数据库并完成迁移。
pub fn init() -> rusqlite::Result<Database> {
    Database::open()
}
