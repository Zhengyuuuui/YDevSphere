//! 项目相关数据结构，与数据库 `projects` 表及前端 TS 类型对齐。
//!
//! 本模块仅依赖 serde，不依赖 tauri / 数据库驱动，便于 Desktop / CLI / MCP 共享。

use serde::{Deserialize, Serialize};

use super::technology::Technology;

/// 项目类型（v0.2 Scanner 迭代）。
///
/// 判定语义见 `docs/v0.2-scanner-plan.md` §2.1：
/// - `Real`：真项目（含清单文件，如 package.json / Cargo.toml）。
/// - `AggregatedRoot`：聚合根（无清单，但直接子目录含 ≥2 个真项目/聚合根）。
/// - `Category`：分类目录（无清单/子项目，但含子目录，作可折叠容器）。
///
/// 普通目录（无任何项目特征）不入库、不生成卡片。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    /// 真项目（含清单文件）。
    Real,
    /// 聚合根（多项目容器，如 sub2api）。
    AggregatedRoot,
    /// 分类目录（可折叠容器，如 学习/工作）。
    Category,
}

impl ProjectKind {
    /// 从数据库 TEXT 列解析；未知 / 空值回退 `Real`（向后兼容旧数据）。
    pub fn from_db(s: Option<&str>) -> Self {
        match s {
            Some("real") => ProjectKind::Real,
            Some("aggregated_root") => ProjectKind::AggregatedRoot,
            Some("category") => ProjectKind::Category,
            _ => ProjectKind::Real,
        }
    }

    /// 序列化为数据库 TEXT 列值。
    pub fn as_db(&self) -> &'static str {
        match self {
            ProjectKind::Real => "real",
            ProjectKind::AggregatedRoot => "aggregated_root",
            ProjectKind::Category => "category",
        }
    }
}

impl std::fmt::Display for ProjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_db())
    }
}

/// 目录树节点（按需返回，供前端懒加载目录树）。
///
/// 仅描述某一目录的**直接**子项；不递归携带全量树，性能友好。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirNode {
    /// 子项名称（目录 / 文件名）。
    pub name: String,
    /// 子项绝对路径。
    pub path: String,
    /// 是否为目录。
    pub is_dir: bool,
    /// 是否为「真项目根」（该目录含清单文件）。
    pub has_manifest: bool,
    /// 直接子项数量（仅目录时有效，前端据此判断能否展开；文件为 0）。
    pub children_count: usize,
}

/// 项目基础信息（对齐 `projects` 表 / 前端 `Project`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub language: Option<String>,
    pub framework: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    /// 项目文件总数（扫描时统计落库，非实时；供列表展示）。
    pub file_count: i64,
    /// 最近一次扫描时间（供前端「最近更新」排序）。
    pub last_scan_at: Option<String>,
    /// 项目归属的工作区根路径（扫描时写入）；旧库 / 手动目录为 `None`（归「全部」）。
    pub workspace: Option<String>,
    /// 项目类型（v0.2：真项目 / 聚合根 / 分类目录）。
    pub kind: ProjectKind,
    /// 健康度评分（0-100，v0.2 Scanner 迭代）。
    pub health_score: i64,
    /// 父项目 id（聚合根 / 分类目录下的树形归属；顶层为 `None`）。
    pub parent_id: Option<i64>,
    /// 技术栈列表（V0.4 Recognition Model，PR1）。
    ///
    /// 序列化为 `technologies[]`（camelCase 无关，本字段为单词）；
    /// 由 `projects.technologies_json` 列（含 schema_version）落库并读回。
    /// 旧数据（technologies_json 为空）回退为空列表，前端可 fallback
    /// `language` / `framework`。
    #[serde(default)]
    pub technologies: Vec<Technology>,
}

/// 项目详情（列表项 + 附加统计信息）。
///
/// 字段与 `projects` 表一致，`file_count` / `last_scan_at` 直接读库（扫描时落库），
/// 避免实时统计的开销与不准确性。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetail {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub language: Option<String>,
    pub framework: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    /// 项目文件总数（扫描时统计，非实时）。
    pub file_count: i64,
    /// 最近一次扫描时间。
    pub last_scan_at: Option<String>,
    /// 项目归属的工作区根路径（扫描时写入）。
    pub workspace: Option<String>,
    /// 项目类型（v0.2）。
    pub kind: ProjectKind,
    /// 健康度评分（0-100）。
    pub health_score: i64,
    /// 父项目 id。
    pub parent_id: Option<i64>,
    /// 技术栈列表（V0.4 Recognition Model，PR1）。
    #[serde(default)]
    pub technologies: Vec<Technology>,
}

/// 扫描结果：一次 `scan_projects` 返回的载荷。
///
/// 既包含新发现/更新的项目列表，也包含被写入数据库的历史记录，
/// 供前端展示扫描反馈。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// 本次扫描识别出的项目（已 upsert 入库）。
    pub projects: Vec<Project>,
    /// 本次扫描写入的扫描历史记录。
    pub history: ScanHistory,
    /// 本次扫描识别出的项目数量（与 `projects.len()` 一致）。
    pub scanned_count: usize,
    /// 本次扫描忽略的目录数量（node_modules / .git / target 等）。
    pub ignored_count: usize,
}

/// 扫描历史记录（对齐 `scan_history` 表）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanHistory {
    pub id: i64,
    /// 被扫描的工作区路径。
    pub workspace: String,
    /// 扫描时间（RFC3339 / 或数据库原始字符串）。
    pub scan_time: String,
    /// 扫描状态，如 "success" / "partial" / "failed"。
    pub status: String,
}

/// 项目技术栈信息（scanner 识别出的原始元数据，尚未分配 id）。
///
/// 由 `core::scanner` 输出，`core::parser` 填充 `language` / `framework`，
/// 最终经 `core::database::upsert_projects` 落库。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedProject {
    pub name: String,
    pub path: String,
    pub language: Option<String>,
    pub framework: Option<String>,
    /// 项目归属的工作区根路径（扫描时由 `scan_projects` 填充）。
    pub workspace: Option<String>,
    /// 项目类型（v0.2）。
    pub kind: ProjectKind,
    /// 健康度评分（v0.2）。
    pub health_score: i64,
    /// 父项目 path（用于扫描后回填 parent_id；数据库落库前以 path 关联）。
    pub parent_path: Option<String>,
    /// 技术栈列表（V0.4 Recognition Model，PR1）。
    #[serde(default)]
    pub technologies: Vec<Technology>,
}

impl DetectedProject {
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
            workspace: None,
            kind: ProjectKind::Real,
            health_score: 0,
            parent_path: None,
            technologies: Vec::new(),
        }
    }

    /// 构造带工作区归属的项目（SPRINT5-05）。
    pub fn new_with_workspace(
        name: impl Into<String>,
        path: impl Into<String>,
        language: Option<String>,
        framework: Option<String>,
        workspace: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            language,
            framework,
            workspace,
            kind: ProjectKind::Real,
            health_score: 0,
            parent_path: None,
            technologies: Vec::new(),
        }
    }

    /// 构造带完整 v0.2 元数据的项目（scanner 识别用）。
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_kind(
        name: impl Into<String>,
        path: impl Into<String>,
        language: Option<String>,
        framework: Option<String>,
        workspace: Option<String>,
        kind: ProjectKind,
        health_score: i64,
        parent_path: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            language,
            framework,
            workspace,
            kind,
            health_score,
            parent_path,
            technologies: Vec::new(),
        }
    }
}
