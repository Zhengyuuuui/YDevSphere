//! 系统工作区（Documents / Desktop 快捷入口）数据结构（SPRINT5-04）。

use serde::{Deserialize, Serialize};

/// 系统工作区种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemWorkspaceKind {
    Documents,
    Desktop,
}

/// 一个系统工作区快捷入口。
///
/// 前端据此决定是否展示 / 禁用快捷入口：`exists == false` 时入口禁用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemWorkspace {
    /// `"documents"` / `"desktop"`。
    pub kind: SystemWorkspaceKind,
    /// 展示名 `"Documents"` / `"Desktop"`（英文，不本地化）。
    pub label: String,
    /// 解析出的绝对路径；目录不存在时为 `null`。
    pub path: Option<String>,
    /// 目录是否存在。
    pub exists: bool,
}

impl SystemWorkspace {
    pub fn new(kind: SystemWorkspaceKind, path: Option<String>) -> Self {
        let label = match kind {
            SystemWorkspaceKind::Documents => "Documents",
            SystemWorkspaceKind::Desktop => "Desktop",
        }
        .to_string();
        let exists = path.is_some();
        Self {
            kind,
            label,
            path,
            exists,
        }
    }
}
