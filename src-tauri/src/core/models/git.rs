//! Git 分析结果数据结构（SPRINT5-01，P1-1 Git 分析）。

use serde::{Deserialize, Serialize};

/// Git 仓库分析结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    /// 是否为 git 仓库（`Repository::open` 失败 / 非仓库时为 `false`）。
    pub is_git_repo: bool,
    /// 当前分支名（HEAD detached 时为 `None`）。
    pub branch: Option<String>,
    /// 最近一次提交信息（无任何 commit 时为 `None`）。
    pub last_commit: Option<CommitInfo>,
    /// 工作区状态（Clean / Dirty）。
    pub status: Option<GitStatus>,
    /// 最近一次 commit 时间（ISO 8601 / RFC3339）；无 commit 时为 `None`。
    pub last_update: Option<String>,
}

/// 最近一次提交信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// 短 hash（前 8 位）。
    pub hash: String,
    /// 提交 message（首行）。
    pub message: String,
    /// 提交作者（name，回退到 email）。
    pub author: String,
    /// 提交时间（RFC3339）。
    pub time: String,
}

/// 工作区状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitStatus {
    Clean,
    /// 有未提交的变更（含 staged / unstaged / untracked）。
    Dirty {
        changed_files: usize,
    },
}
