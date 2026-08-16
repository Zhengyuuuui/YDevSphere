//! 项目记忆（`.ydevsphere/project.json`）相关数据结构。

use serde::{Deserialize, Serialize};

/// 项目记忆内容（对齐 `doc/` 中 project.json 的 `stack[]` 形态）。
///
/// 写入示例：
/// ```json
/// {
///   "name": "YDevSphere",
///   "stack": ["Vue3", "TypeScript", "Rust"],
///   "packageManager": "pnpm"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMemory {
    pub name: String,
    /// 技术栈列表：language + framework 合并去重（language 优先）。
    pub stack: Vec<String>,
    /// 包管理器（由 lockfile 检测得出；无 lockfile 则省略该字段）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,
}

/// 供项目记忆模块使用的项目摘要（从数据库 `ProjectDetail` 提取，解耦 memory 与数据库）。
#[derive(Debug, Clone)]
pub struct ProjectRef {
    pub name: String,
    pub path: String,
    pub language: Option<String>,
    pub framework: Option<String>,
}

impl ProjectRef {
    pub fn new(
        name: impl Into<String>,
        path: impl Into<String>,
        language: Option<String>,
        framework: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            language,
            framework,
        }
    }
}
