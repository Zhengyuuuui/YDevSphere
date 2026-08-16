//! 项目记忆（`.ydevsphere/project.json`）读写模块。
//!
//! 职责：
//! - 生成 / 读取 / 更新项目记忆文件
//! - `packageManager` 检测（只读 lockfile 名称：pnpm-lock.yaml / package-lock.json
//!   / yarn.lock / bun.lockb，无则省略）
//! - `stack` 合并（language + framework，去重，language 优先）
//!
//! ## 安全红线（RESTRICTIONS.md 第 3 节）
//! - 默认只读；**仅** 允许写入 `<project>/.ydevsphere/project.json`。
//! - 创建 `.ydevsphere/` 目录（若不存在）；绝不触碰其他源码 / 配置 / 文件。
//! - 写入需由上层（commands 层）显式传入「用户已授权」标志，未经授权不得写。
//! - 写入使用原子写（临时文件 + rename），避免覆盖中断导致半成品文件。
//!
//! 硬性约束：本模块禁止 `use tauri`。

use std::path::{Path, PathBuf};

use crate::core::models::{ProjectMemory, ProjectRef};

/// 项目记忆目录名（仅允许写此目录内的文件）。
const MEMORY_DIR: &str = ".ydevsphere";
/// 记忆文件名。
const PROJECT_JSON: &str = "project.json";

/// 记忆操作错误。
#[derive(Debug)]
pub enum MemoryError {
    /// 未获用户授权（安全红线）。
    Unauthorized,
    /// 项目目录不存在或不可读。
    InvalidProject(String),
    /// 文件系统操作失败。
    Io(std::io::Error),
    /// 读取/解析已有 project.json 失败。
    Malformed(String),
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::Unauthorized => write!(f, "未获用户授权，禁止写入项目文件"),
            MemoryError::InvalidProject(p) => write!(f, "项目目录无效: {p}"),
            MemoryError::Io(e) => write!(f, "文件操作失败: {e}"),
            MemoryError::Malformed(m) => write!(f, "project.json 解析失败: {m}"),
        }
    }
}

impl std::error::Error for MemoryError {}

/// 为指定项目生成（或刷新）`.ydevsphere/project.json`。
///
/// - 幂等：若文件已存在，解析后刷新 `name` / `stack` / `package_manager`（`None` 不清空既有值）。
/// - `authorized` 必须为 `true`，否则返回 `MemoryError::Unauthorized` 且不写任何文件。
/// - `package_manager_override`：由命令层显式传入的包管理器；`None` 时自动检测。
pub fn ensure_project_memory(
    project: &ProjectRef,
    authorized: bool,
    package_manager_override: Option<&str>,
) -> Result<ProjectMemory, MemoryError> {
    if !authorized {
        return Err(MemoryError::Unauthorized);
    }

    let project_dir = validate_project_dir(&project.path)?;
    let mem_dir = project_dir.join(MEMORY_DIR);
    let json_path = mem_dir.join(PROJECT_JSON);

    // 读取既有值（若存在且合法），用于幂等刷新
    let existing = read_at(&json_path).ok().flatten();

    // package_manager 优先级：显式覆盖 > lockfile 检测 > 既有值（幂等保留）
    let package_manager = package_manager_override
        .map(|s| s.to_string())
        .or_else(|| detect_package_manager(&project_dir))
        .or_else(|| existing.as_ref().and_then(|m| m.package_manager.clone()));

    // stack：language + framework 合并去重（language 优先）
    let mut stack = Vec::new();
    if let Some(lang) = project.language.clone() {
        push_unique(&mut stack, lang);
    }
    if let Some(fw) = project.framework.clone() {
        push_unique(&mut stack, fw);
    }
    // 若外部已提供 package_manager 但 stack 为空，则补充到 stack（保证项目有技术信息）
    if stack.is_empty() {
        if let Some(lang) = project.language.clone() {
            push_unique(&mut stack, lang);
        }
    }

    let memory = ProjectMemory {
        name: project.name.clone(),
        stack,
        package_manager,
    };

    write_atomically(&json_path, &memory)?;
    Ok(memory)
}

/// 读取项目记忆；不存在返回 `Ok(None)`。
pub fn get_project_memory(
    project_dir: &Path,
) -> Result<Option<ProjectMemory>, MemoryError> {
    let mem_dir = project_dir.join(MEMORY_DIR);
    let json_path = mem_dir.join(PROJECT_JSON);
    read_at(&json_path)
}

/// 更新项目记忆字段（幂等刷新）。
///
/// - `authorized` 必须为 `true`。
/// - `package_manager_override`：`Some` 时更新；`None` 保留既有值（或检测）。
/// - `stack_override`：`Some` 时整体替换 stack；`None` 保留既有值。
pub fn update_project_memory(
    project: &ProjectRef,
    authorized: bool,
    package_manager_override: Option<&str>,
    stack_override: Option<Vec<String>>,
) -> Result<ProjectMemory, MemoryError> {
    if !authorized {
        return Err(MemoryError::Unauthorized);
    }

    let project_dir = validate_project_dir(&project.path)?;
    let json_path = project_dir.join(MEMORY_DIR).join(PROJECT_JSON);

    let existing = read_at(&json_path).ok().flatten().unwrap_or_else(|| {
        // 不存在则按当前项目信息构造
        let mut stack = Vec::new();
        if let Some(lang) = project.language.clone() {
            push_unique(&mut stack, lang);
        }
        if let Some(fw) = project.framework.clone() {
            push_unique(&mut stack, fw);
        }
        ProjectMemory {
            name: project.name.clone(),
            stack,
            package_manager: None,
        }
    });

    let stack = stack_override.unwrap_or(existing.stack);
    let package_manager = match package_manager_override {
        Some(pm) => Some(pm.to_string()),
        None => existing.package_manager,
    };

    let memory = ProjectMemory {
        name: project.name.clone(),
        stack,
        package_manager,
    };

    write_atomically(&json_path, &memory)?;
    Ok(memory)
}

/// 检测项目目录下的包管理器（只读文件名，不修改任何文件）。
fn detect_package_manager(project_dir: &Path) -> Option<String> {
    const LOCKFILES: &[(&str, &str)] = &[
        ("pnpm-lock.yaml", "pnpm"),
        ("package-lock.json", "npm"),
        ("yarn.lock", "yarn"),
        ("bun.lockb", "bun"),
        ("bun.lock", "bun"),
    ];
    LOCKFILES
        .iter()
        .find(|(file, _)| project_dir.join(file).is_file())
        .map(|(_, pm)| pm.to_string())
}

/// 将元素按顺序去重加入栈。
fn push_unique(stack: &mut Vec<String>, item: String) {
    if !stack.iter().any(|s| s == &item) {
        stack.push(item);
    }
}

/// 校验项目目录为存在且是目录，返回规范化路径。
fn validate_project_dir(path: &str) -> Result<PathBuf, MemoryError> {
    let p = PathBuf::from(path);
    if !p.is_dir() {
        return Err(MemoryError::InvalidProject(path.to_string()));
    }
    Ok(p)
}

/// 读取并解析 `project.json`（不存在返回 `Ok(None)`）。
fn read_at(path: &Path) -> Result<Option<ProjectMemory>, MemoryError> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path).map_err(MemoryError::Io)?;
    let memory: ProjectMemory = serde_json::from_str(&raw)
        .map_err(|e| MemoryError::Malformed(format!("{path:?}: {e}")))?;
    Ok(Some(memory))
}

/// 原子写入 `project.json`：
/// 1. 创建父目录 `.ydevsphere/`（若不存在）
/// 2. 写入临时文件 `project.json.tmp`
/// 3. rename 覆盖目标
///
/// 仅触碰 `.ydevsphere/project.json` 与其临时文件，绝不写其他文件。
fn write_atomically(path: &Path, memory: &ProjectMemory) -> Result<(), MemoryError> {
    let parent = path
        .parent()
        .ok_or_else(|| MemoryError::Io(std::io::Error::other("路径无父目录")))?;
    std::fs::create_dir_all(parent).map_err(MemoryError::Io)?;

    let tmp = parent.join(format!("{PROJECT_JSON}.tmp"));
    let json = serde_json::to_string_pretty(memory)
        .map_err(|e| MemoryError::Malformed(format!("序列化失败: {e}")))?;

    std::fs::write(&tmp, json).map_err(MemoryError::Io)?;
    std::fs::rename(&tmp, path).map_err(MemoryError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ydevsphere_memory_test_{}_{}",
            std::process::id(),
            name
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("创建临时目录失败");
        dir
    }

    fn proj_ref(name: &str, path: &str) -> ProjectRef {
        ProjectRef::new(name, path, Some("Rust".into()), Some("Vue".into()))
    }

    #[test]
    fn unauthorized_write_is_refused() {
        let dir = tmp_dir("unauthorized");
        let result = ensure_project_memory(
            &proj_ref("x", dir.to_str().unwrap()),
            false,
            None,
        );
        assert!(matches!(result, Err(MemoryError::Unauthorized)));
        // 不应创建任何文件
        assert!(!dir.join(MEMORY_DIR).exists());
    }

    #[test]
    fn ensure_creates_project_json() {
        let dir = tmp_dir("ensure");
        let mem = ensure_project_memory(
            &proj_ref("app", dir.to_str().unwrap()),
            true,
            None,
        )
        .expect("写入应成功");

        assert_eq!(mem.name, "app");
        // stack: language(Rust) + framework(Vue)
        assert_eq!(mem.stack, vec!["Rust".to_string(), "Vue".to_string()]);
        // 无 lockfile → 无 package_manager
        assert_eq!(mem.package_manager, None);

        // 文件存在且内容合法
        let path = dir.join(MEMORY_DIR).join(PROJECT_JSON);
        assert!(path.is_file());
        let parsed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(parsed["name"], "app");
        assert_eq!(parsed["stack"][0], "Rust");
    }

    #[test]
    fn ensure_omits_package_manager_when_no_lockfile() {
        let dir = tmp_dir("no_lock");
        let mem = ensure_project_memory(
            &proj_ref("x", dir.to_str().unwrap()),
            true,
            None,
        )
        .expect("写入应成功");
        assert_eq!(mem.package_manager, None);
        // 序列化后不应含 package_manager 字段
        let json = std::fs::read_to_string(dir.join(MEMORY_DIR).join(PROJECT_JSON)).unwrap();
        assert!(!json.contains("packageManager"), "无 lockfile 时应省略该字段");
    }

    #[test]
    fn detects_package_manager_from_lockfiles() {
        let dir = tmp_dir("locks");
        std::fs::write(dir.join("pnpm-lock.yaml"), "").unwrap();
        assert_eq!(detect_package_manager(&dir).as_deref(), Some("pnpm"));

        let dir2 = tmp_dir("locks2");
        std::fs::write(dir2.join("yarn.lock"), "").unwrap();
        assert_eq!(detect_package_manager(&dir2).as_deref(), Some("yarn"));

        let dir3 = tmp_dir("locks3");
        std::fs::write(dir3.join("package-lock.json"), "").unwrap();
        assert_eq!(detect_package_manager(&dir3).as_deref(), Some("npm"));

        let dir4 = tmp_dir("locks4");
        std::fs::write(dir4.join("bun.lockb"), "").unwrap();
        assert_eq!(detect_package_manager(&dir4).as_deref(), Some("bun"));

        // 优先级：pnpm 优先于 npm
        let dir5 = tmp_dir("locks5");
        std::fs::write(dir5.join("package-lock.json"), "").unwrap();
        std::fs::write(dir5.join("pnpm-lock.yaml"), "").unwrap();
        assert_eq!(detect_package_manager(&dir5).as_deref(), Some("pnpm"));
    }

    #[test]
    fn get_returns_none_when_absent() {
        let dir = tmp_dir("get_absent");
        let mem = get_project_memory(&dir).expect("读取应成功");
        assert!(mem.is_none());
    }

    #[test]
    fn get_reads_existing() {
        let dir = tmp_dir("get_present");
        let written = ensure_project_memory(
            &proj_ref("app", dir.to_str().unwrap()),
            true,
            Some("pnpm"),
        )
        .expect("写入应成功");
        assert_eq!(written.package_manager.as_deref(), Some("pnpm"));

        let read = get_project_memory(&dir)
            .expect("读取应成功")
            .expect("应存在");
        assert_eq!(read, written);
    }

    #[test]
    fn ensure_is_idempotent() {
        let dir = tmp_dir("idempotent");
        let first = ensure_project_memory(
            &proj_ref("app", dir.to_str().unwrap()),
            true,
            Some("pnpm"),
        )
        .expect("首次写入应成功");
        let second = ensure_project_memory(
            &proj_ref("app", dir.to_str().unwrap()),
            true,
            None,
        )
        .expect("再次写入应成功");
        // 再次写入未显式给 package_manager，应保留既有 pnpm
        assert_eq!(second.package_manager.as_deref(), Some("pnpm"));
        let _ = first;
    }

    #[test]
    fn update_refreshes_stack_and_pm() {
        let dir = tmp_dir("update");
        ensure_project_memory(
            &proj_ref("app", dir.to_str().unwrap()),
            true,
            Some("pnpm"),
        )
        .expect("写入应成功");

        let updated = update_project_memory(
            &proj_ref("app", dir.to_str().unwrap()),
            true,
            Some("yarn"),
            Some(vec!["React".to_string(), "TypeScript".to_string()]),
        )
        .expect("更新应成功");
        assert_eq!(updated.package_manager.as_deref(), Some("yarn"));
        assert_eq!(updated.stack, vec!["React".to_string(), "TypeScript".to_string()]);

        // 读回验证
        let read = get_project_memory(&dir).expect("读取应成功").expect("应存在");
        assert_eq!(read, updated);
    }

    #[test]
    fn stack_dedups_language_and_framework() {
        let dir = tmp_dir("dedup");
        let mem = ensure_project_memory(
            &ProjectRef::new(
                "x",
                dir.to_str().unwrap(),
                Some("Node".into()),
                Some("Node".into()),
            ),
            true,
            None,
        )
        .expect("写入应成功");
        assert_eq!(mem.stack, vec!["Node".to_string()]);
    }

    #[test]
    fn invalid_project_dir_errors() {
        let result = ensure_project_memory(
            &proj_ref("x", "/no/such/dir_xyz"),
            true,
            None,
        );
        assert!(matches!(result, Err(MemoryError::InvalidProject(_))));
    }
}
