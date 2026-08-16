//! # Git 分析模块（P1-1）
//!
//! 读取项目仓库的 git 信息（branch / last_commit / status / last_update /
//! is_git_repo）。
//!
//! ## 约束
//! - **只读**：仅使用 `git2` 只读 API，绝不修改任何 git 状态
//!   （不 commit / checkout / push / pull / reset）。
//! - **容错**：非 git 仓库、`.git` 损坏、权限不足、无 commit 等一律优雅降级为
//!   `is_git_repo: false` 或字段 `None`，不得 panic。
//! - **跨平台**：git2 由系统 libgit2 驱动，三平台可用；无 macOS-only 逻辑。
//!
//! 硬性约束：本模块禁止 `use tauri`。

pub mod analyzer;

pub use analyzer::analyze_git;
