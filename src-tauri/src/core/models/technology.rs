//! 技术栈（Technology）数据结构 —— V0.4 识别引擎 / Recognition Model（PR1）。
//!
//! 依据 `docs/V04-RECOGNITION-SPEC.md` §1。
//!
//! - `TechnologyCategory`：技术类别（Language / Runtime / Framework / ...）。
//! - `Technology`：单个技术（含 canonical id，跨 detector 稳定）。
//! - `TechnologiesJson`：落库到 `projects.technologies_json` 列的 JSON 封装，
//!   含 `schema_version` 便于未来迁移旧 JSON（Spec §6.3）。
//!
//! 本模块仅依赖 serde，不依赖 tauri / 数据库驱动，便于 Desktop / CLI / MCP 共享。
//!
//! ## 序列化契约（前端 PR4 对齐）
//!
//! 沿用项目现有 snake_case JSON 字段命名（与 `Project` / `ProjectDetail` 一致）：
//!
//! ```json
//! {
//!   "id": "vue",
//!   "name": "Vue",
//!   "category": "framework",
//!   "ecosystem": "javascript"
//! }
//! ```
//!
//! `category` 采用 snake_case（`build_tool` / `package_manager` / `language` / ...，
//! 见 Spec §1.4）。

use serde::{Deserialize, Serialize};

/// 技术类别（Spec §1.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnologyCategory {
    /// 编程语言：TypeScript / JavaScript / Python / Rust ...
    Language,
    /// 运行时：Node.js / Bun / Deno ...
    Runtime,
    /// 框架：Vue / React / Express / NestJS / Fastify ...
    Framework,
    /// 库：Pinia / JWT / Axios ...
    Library,
    /// 数据库：SQLite / PostgreSQL / MySQL / MongoDB / Redis ...
    Database,
    /// 构建工具：Vite / Webpack / Rspack ...
    BuildTool,
    /// 包管理器：pnpm / npm / yarn / bun ...
    PackageManager,
    /// 平台/框架：uni-app / Tauri / Electron ...
    Platform,
}

/// 单个技术（Spec §1.1），`id` 为 canonical id（稳定，跨 detector 统一）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Technology {
    /// canonical id（稳定，跨 detector 统一；dedupe / aggregation / filter 基于此）。
    pub id: String,
    /// 展示名（人类可读）。
    pub name: String,
    /// 技术类别。
    pub category: TechnologyCategory,
    /// 技术生态，如 `"javascript"` / `"python"` / `"rust"`；未知为 `None`。
    pub ecosystem: Option<String>,
}

impl Technology {
    /// 便捷构造（供 detector / 测试使用）。
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: TechnologyCategory,
        ecosystem: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category,
            ecosystem,
        }
    }
}

/// `projects.technologies_json` 列的 JSON 封装（Spec §6.3）。
///
/// 序列化示例：
/// ```json
/// {
///   "schema_version": 1,
///   "technologies": [ ... ]
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechnologiesJson {
    /// JSON 结构版本，未来 Technology 结构变化时据此迁移旧数据。
    pub schema_version: i32,
    /// 技术列表。
    pub technologies: Vec<Technology>,
}

impl TechnologiesJson {
    /// 当前 schema 版本。
    pub const SCHEMA_VERSION: i32 = 1;

    /// 构造当前版本的技术列表封装。
    pub fn new(technologies: Vec<Technology>) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            technologies,
        }
    }

    /// 序列化为 JSON 字符串（落库 `technologies_json` 列）。
    pub fn encode(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            // 理论上不会失败（结构均可序列化）；兜底返回空封装，不阻断写库。
            serde_json::to_string(&Self::empty()).unwrap_or_else(|_| "{}".to_string())
        })
    }

    /// 从数据库 `technologies_json` 文本解析；`None`（旧数据无列值）或
    /// 解析失败（异常 JSON）均回退为空封装，保证兼容旧数据与极端脏数据。
    pub fn decode(raw: Option<&str>) -> Self {
        match raw {
            None => Self::empty(),
            Some(s) if s.trim().is_empty() => Self::empty(),
            Some(s) => serde_json::from_str::<Self>(s).unwrap_or_else(|_| Self::empty()),
        }
    }

    /// 空封装（无技术，schema_version 仍为当前版本）。
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// 取技术列表引用。
    pub fn technologies(&self) -> &[Technology] {
        &self.technologies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::ProjectKind;
    use serde_json::json;

    fn vue() -> Technology {
        Technology::new(
            "vue",
            "Vue",
            TechnologyCategory::Framework,
            Some("javascript".to_string()),
        )
    }

    /// Technology 序列化：category 用 snake_case，字段名 camelCase 无关（本结构无复合字段）。
    #[test]
    fn technology_serializes_to_spec_shape() {
        let json = serde_json::to_value(vue()).expect("序列化应成功");
        assert_eq!(
            json,
            json!({
                "id": "vue",
                "name": "Vue",
                "category": "framework",
                "ecosystem": "javascript"
            })
        );
    }

    /// Technology roundtrip：反序列化后与源值相等。
    #[test]
    fn technology_roundtrip() {
        let t = vue();
        let encoded = serde_json::to_string(&t).expect("序列化应成功");
        let decoded: Technology = serde_json::from_str(&encoded).expect("反序列化应成功");
        assert_eq!(decoded, t);
    }

    /// TechnologyCategory 各变体 snake_case 序列化断言。
    #[test]
    fn category_snake_case_values() {
        let cases = [
            (TechnologyCategory::Language, "language"),
            (TechnologyCategory::Runtime, "runtime"),
            (TechnologyCategory::Framework, "framework"),
            (TechnologyCategory::Library, "library"),
            (TechnologyCategory::Database, "database"),
            (TechnologyCategory::BuildTool, "build_tool"),
            (TechnologyCategory::PackageManager, "package_manager"),
            (TechnologyCategory::Platform, "platform"),
        ];
        for (cat, expected) in cases {
            let s = serde_json::to_value(cat).expect("序列化应成功");
            assert_eq!(s, serde_json::Value::String(expected.to_string()));
        }
    }

    /// TechnologiesJson 序列化：含 schema_version 与 technologies。
    #[test]
    fn technologies_json_contains_schema_version() {
        let tj = TechnologiesJson::new(vec![vue()]);
        let json = serde_json::to_value(&tj).expect("序列化应成功");
        assert_eq!(
            json,
            json!({
                "schema_version": 1,
                "technologies": [
                    { "id": "vue", "name": "Vue", "category": "framework", "ecosystem": "javascript" }
                ]
            })
        );
    }

    /// TechnologiesJson encode/decode roundtrip。
    #[test]
    fn technologies_json_roundtrip() {
        let tj = TechnologiesJson::new(vec![vue()]);
        let encoded = tj.encode();
        let decoded = TechnologiesJson::decode(Some(&encoded));
        assert_eq!(decoded, tj);
        assert_eq!(decoded.schema_version, 1);
    }

    /// decode 对 NULL / 空 / 脏 JSON 回退为空封装（旧数据兼容）。
    #[test]
    fn decode_handles_missing_and_malformed() {
        assert_eq!(TechnologiesJson::decode(None), TechnologiesJson::empty());
        assert_eq!(TechnologiesJson::decode(Some("")), TechnologiesJson::empty());
        assert_eq!(TechnologiesJson::decode(Some("not json")), TechnologiesJson::empty());
        assert_eq!(
            TechnologiesJson::decode(Some(r#"{"schema_version":1}"#)),
            TechnologiesJson::empty()
        );
    }

    /// ProjectKind 序列化（已在 project.rs 定义，此处交叉验证其 serde 契约）。
    #[test]
    fn project_kind_serde() {
        let vals = [
            (ProjectKind::Real, "real"),
            (ProjectKind::AggregatedRoot, "aggregated_root"),
            (ProjectKind::Category, "category"),
        ];
        for (kind, expected) in vals {
            let s = serde_json::to_value(kind).expect("序列化应成功");
            assert_eq!(s, serde_json::Value::String(expected.to_string()));
            let back: ProjectKind = serde_json::from_value(s).expect("反序列化应成功");
            assert_eq!(back, kind);
        }
    }
}
