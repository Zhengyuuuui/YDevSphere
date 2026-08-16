//! # 项目记忆模块
//!
//! 管理用户项目目录下的 `.ydevsphere/project.json`（P0-4「项目记忆」）。
//!
//! ## 安全红线（RESTRICTIONS.md 第 3 节）
//! 写用户项目目录属于写操作，本模块默认只读：
//! - **仅**允许写 `<project>/.ydevsphere/project.json` 及其临时文件。
//! - 创建 `.ydevsphere/` 目录（若不存在）；绝不触碰其他源码 / 配置 / 文件。
//! - 所有写入函数均要求上层传入 `authorized: bool` 标志（由前端点击「启用」后为 `true`），
//!   未授权一律拒绝并返回 `MemoryError::Unauthorized`。
//!
//! 硬性约束：本模块禁止 `use tauri`。

pub mod project_memory;

pub use project_memory::{
    ensure_project_memory, get_project_memory, update_project_memory, MemoryError,
};
