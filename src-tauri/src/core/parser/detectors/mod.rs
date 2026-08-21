//! P0 detector 实现（Spec §5.2/§5.4，PR2）。
//!
//! - `javascript`：JS/Node 语言、框架、构建工具、平台、库（Spec §5.4 P0 列表）。
//! - `database`：依赖级数据库映射（Spec §5.7）。
//! - `infrastructure`：packageManager 优先级（§5.5）+ Runtime 多来源（§5.6）。
//!
//! 全部为纯函数 detector（消费 `DetectorContext`，不访问文件系统）。
//! 禁止 `use tauri`。

pub mod database;
pub mod infrastructure;
pub mod javascript;
