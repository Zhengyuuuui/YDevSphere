//! 核心数据结构。
//!
//! 本模块仅依赖纯 Rust 与 serde，不依赖 tauri / 数据库驱动，
//! 便于 Desktop / CLI / MCP 共享同一套数据结构。
//!
//! 注意：字段对齐 `doc/spec.md` 第 7 节 `projects` / `scan_history` 表。

pub mod editor;
pub mod error;
pub mod git;
pub mod memory;
pub mod project;
pub mod workspace;

pub use editor::{AvailableEditor, EditorCategory, EditorSource, OpenMethod};
pub use error::ScanCommandError;
pub use git::{CommitInfo, GitInfo, GitStatus};
pub use memory::{ProjectMemory, ProjectRef};
pub use project::{
    DetectedProject, DirNode, Project, ProjectDetail, ProjectKind, ScanHistory,
    ScanResult,
};
pub use workspace::{SystemWorkspace, SystemWorkspaceKind};
