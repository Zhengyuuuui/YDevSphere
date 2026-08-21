//! Database detector（Spec §5.7，PR2）。
//!
//! 依赖级数据库映射（**dependency-level detection**）：
//! 「manifest 中出现该依赖 → 暗示使用对应数据库」，**不是**实际运行环境检测
//! （有 redis 包 ≠ 部署了 Redis）。
//!
//! 映射表（Spec §5.7）：
//! ```text
//! better-sqlite3 / sqlite3 → sqlite
//! pg / postgres            → postgresql
//! mysql2                   → mysql
//! mongoose                 → mongodb
//! redis                    → redis
//! ```

use crate::core::models::TechnologyCategory as Cat;

use super::super::registry::{tech, DetectionResult, Detector, DetectorContext, SourceKind};

/// 依赖 → 数据库 canonical id 映射（Spec §5.7，P1-10）。
struct DbRule {
    dep: &'static str,
    id: &'static str,
    name: &'static str,
}

const DATABASES: &[DbRule] = &[
    DbRule { dep: "better-sqlite3", id: "sqlite", name: "SQLite" },
    DbRule { dep: "sqlite3", id: "sqlite", name: "SQLite" },
    DbRule { dep: "pg", id: "postgresql", name: "PostgreSQL" },
    DbRule { dep: "postgres", id: "postgresql", name: "PostgreSQL" },
    DbRule { dep: "mysql2", id: "mysql", name: "MySQL" },
    DbRule { dep: "mongoose", id: "mongodb", name: "MongoDB" },
    DbRule { dep: "redis", id: "redis", name: "Redis" },
];

/// 数据库 detector（依赖级，全部 ManifestDependency 来源）。
pub struct DatabaseDetector;

impl Detector for DatabaseDetector {
    fn name(&self) -> &'static str {
        "database"
    }

    fn detect(&self, ctx: &DetectorContext) -> Vec<DetectionResult> {
        let mut seen: Vec<&str> = Vec::new();
        let mut results = Vec::new();
        for rule in DATABASES {
            // 多个依赖命中同一数据库（如 better-sqlite3 + sqlite3）→ 只产出一次
            // （registry 也会兜底去重，这里提前避免重复结果）。
            if seen.contains(&rule.id) {
                continue;
            }
            if ctx.has_dependency(rule.dep) {
                seen.push(rule.id);
                results.push(DetectionResult::new(
                    // 数据库为跨生态技术，ecosystem 为 None（对齐 Spec §1.4 示例）。
                    tech(rule.id, rule.name, Cat::Database, None),
                    SourceKind::ManifestDependency,
                ));
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(deps: &[&str]) -> Vec<DetectionResult> {
        let ctx = DetectorContext {
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };
        DatabaseDetector.detect(&ctx)
    }

    fn ids(results: &[DetectionResult]) -> Vec<&str> {
        results.iter().map(|r| r.technology.id.as_str()).collect()
    }

    /// Spec §5.7 映射表逐条断言。
    #[test]
    fn maps_every_dependency_in_spec_table() {
        for (dep, id) in [
            ("better-sqlite3", "sqlite"),
            ("sqlite3", "sqlite"),
            ("pg", "postgresql"),
            ("postgres", "postgresql"),
            ("mysql2", "mysql"),
            ("mongoose", "mongodb"),
            ("redis", "redis"),
        ] {
            let out = detect(&[dep]);
            assert_eq!(ids(&out), vec![id], "依赖 {dep} 应映射为 {id}");
        }
    }

    /// 同一数据库多个依赖（better-sqlite3 + sqlite3）→ 去重为一个 sqlite。
    #[test]
    fn duplicate_database_deps_dedupe() {
        let out = detect(&["better-sqlite3", "sqlite3"]);
        assert_eq!(ids(&out), vec!["sqlite"]);
    }

    /// category = database，source = ManifestDependency。
    #[test]
    fn database_category_and_source() {
        let out = detect(&["mongoose"]);
        assert_eq!(out[0].technology.category, Cat::Database);
        assert_eq!(out[0].source_kind, SourceKind::ManifestDependency);
        assert_eq!(out[0].technology.ecosystem, None, "数据库跨生态，ecosystem 为 None");
    }

    /// 非数据库依赖不产出。
    #[test]
    fn unrelated_deps_produce_nothing() {
        let out = detect(&["express", "vue", "ioredis-proxy-not-real"]);
        assert!(out.is_empty());
    }

    /// 前缀不误报：`pgx`（Rust crate 名风格）不应命中 `pg`；精确名匹配。
    #[test]
    fn no_prefix_false_positive() {
        let out = detect(&["pgx", "mysql2-promisify-fake"]);
        assert!(out.is_empty(), "pgx 不应命中 pg");
    }
}
