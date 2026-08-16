//! 目录扫描模块（v0.2 重写：智能项目边界识别）。
//!
//! 职责：
//! - 递归遍历工作区目录，按「项目边界」语义识别项目/聚合根/分类目录
//! - 项目识别依据：`package.json`(Node) / `Cargo.toml`(Rust) / `go.mod`(Go)
//!   / `requirements.txt`、`pyproject.toml`(Python)
//! - 忽略目录：预设（node_modules / .git / target / dist / build / vendor /
//!   .cache / 隐藏目录）+ 用户自定义（经 `set_ignore_rules` 持久化）
//! - 健康度评分（文件数适中 + 结构规范 + 清单 + README + Git + CI）
//! - 只读，不修改源码、不执行命令、无网络请求。
//!
//! ## 识别语义（`docs/v0.2-scanner-plan.md` §2.1）
//!
//! 对工作区根 R 的每个直接子目录 D：
//! - ① 真项目：含清单文件 → 卡片（健康度高），内部不拆
//! - ② 聚合根：无清单，但直接子目录含 ≥2 个真项目/聚合根 → 卡片
//! - ③ 分类目录：无清单/子项目，但含子目录 → 可折叠卡片，递归识别内部
//! - ④ 普通目录：无项目特征 → 不生成卡片
//!
//! 关键规则（父项目边界优先）：一旦 D 判定为真项目/聚合根，其后代一律
//! 不生成卡片，只作 D 的 tree 子结构。
//!
//! 硬性约束：本模块禁止 `use tauri`。

use std::path::{Path, PathBuf};

use crate::core::models::{DetectedProject, DirNode, ProjectKind};
use crate::core::parser::{self, ProjectMeta};

/// 扫描错误。
#[derive(Debug)]
pub enum ScanError {
    /// 工作区路径不存在或不可读。
    NotADirectory(String),
    /// 遍历目录失败。
    Io(std::io::Error),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::NotADirectory(p) => write!(f, "不是有效目录: {p}"),
            ScanError::Io(e) => write!(f, "目录遍历失败: {e}"),
        }
    }
}

impl std::error::Error for ScanError {}

impl ScanError {
    /// 结构化错误码（前端按码判断，不依赖中文字符串）。
    pub fn code(&self) -> &'static str {
        match self {
            ScanError::NotADirectory(_) => "INVALID_DIRECTORY",
            ScanError::Io(_) => "IO_ERROR",
        }
    }
}

/// 目录深度上限（默认 6 层，可配置）。
const DEFAULT_MAX_DEPTH: usize = 6;

/// 默认忽略的目录名（不含隐藏目录，隐藏目录统一跳过）。
const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    "vendor",
    ".cache",
];

/// 项目清单文件名。
const MANIFEST_FILES: &[&str] = &[
    "package.json",
    "Cargo.toml",
    "go.mod",
    "pyproject.toml",
    "requirements.txt",
];

/// 扫描配置（v0.2）。
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// 递归深度上限（默认 6）。
    pub max_depth: usize,
    /// 用户自定义忽略的目录名（叠加在预设之上）。
    pub extra_ignored_dirs: Vec<String>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            extra_ignored_dirs: Vec::new(),
        }
    }
}

/// 扫描结果：识别出的项目 + 统计信息。
#[derive(Debug)]
pub struct ScanOutput {
    pub projects: Vec<DetectedProject>,
    /// 忽略的目录数量。
    pub ignored_count: usize,
}

/// 扫描指定工作区路径，返回其中识别出的所有项目（含聚合根/分类目录）。
///
/// 使用默认配置（深度 6 层、无自定义忽略规则）。
/// 该函数为向后兼容入口，内部转 `scan_workspace_with_options`。
pub fn scan_workspace(workspace_path: &Path) -> Result<ScanOutput, ScanError> {
    scan_workspace_with_options(workspace_path, &ScanOptions::default())
}

/// 扫描指定工作区路径（带配置），返回识别出的项目。
///
/// 语义（`docs/v0.2-scanner-plan.md` §2.1）：对工作区根 R 的**每个直接子目录 D**
/// 逐一做 4 类判定，根 R 本身不生成卡片。
///
/// 特例兼容：若 R 本身含清单（用户直接选了一个项目目录作工作区），
/// 则只识别 R 为真项目，不再展开其内部。
pub fn scan_workspace_with_options(
    workspace_path: &Path,
    options: &ScanOptions,
) -> Result<ScanOutput, ScanError> {
    if !workspace_path.is_dir() {
        return Err(ScanError::NotADirectory(workspace_path.display().to_string()));
    }

    let mut projects = Vec::new();
    let mut ignored_count = 0usize;

    // 特例：根本身是真项目 → 只识别它，不拆内部。
    if has_manifest(workspace_path) {
        let meta = project_meta_if_manifest(workspace_path)?.unwrap_or_else(|| ProjectMeta::new(None, None));
        let health = health_score(workspace_path, ProjectKind::Real);
        let name = project_name(workspace_path, &meta);
        projects.push(DetectedProject::new_with_kind(
            name,
            dir_path_string(workspace_path),
            meta.language,
            meta.framework,
            None,
            ProjectKind::Real,
            health,
            None,
        ));
        return Ok(ScanOutput {
            projects,
            ignored_count,
        });
    }

    // 遍历根的直接子目录 D，逐一判定（parent = None，即顶层卡片）。
    let child_dirs = list_child_dirs(workspace_path, options, &mut ignored_count)?;
    for child in &child_dirs {
        classify_dir(child, 0, options, None, &mut projects, &mut ignored_count)?;
    }

    Ok(ScanOutput {
        projects,
        ignored_count,
    })
}

/// 自顶向下递归识别目录 D，并决定其卡片类型。
///
/// - `dir`：当前判定的目录绝对路径
/// - `depth`：当前深度（相对工作区根）
/// - `parent`：父项目 path（用于树形归属；顶层为 `None`）
fn classify_dir(
    dir: &Path,
    depth: usize,
    options: &ScanOptions,
    parent: Option<&str>,
    projects: &mut Vec<DetectedProject>,
    ignored_count: &mut usize,
) -> Result<(), ScanError> {
    // 深度保护
    if depth >= options.max_depth {
        return Ok(());
    }

    let name = dir_name(dir);
    if should_ignore(&name, options) {
        // 根目录本身一般不会被 ignore；此处防御性处理。
        return Ok(());
    }

    // ① 真项目：含清单文件 → 生成卡片，内部不再拆。
    if let Some(meta) = project_meta_if_manifest(dir)? {
        let health = health_score(dir, ProjectKind::Real);
        let pname = project_name(dir, &meta);
        projects.push(DetectedProject::new_with_kind(
            pname,
            dir_path_string(dir),
            meta.language,
            meta.framework,
            None,
            ProjectKind::Real,
            health,
            parent.map(String::from),
        ));
        return Ok(());
    }

    // 读取直接子目录（仅目录，且排除忽略项）。
    let child_dirs = list_child_dirs(dir, options, ignored_count)?;

    // ② 聚合根：无清单，但直接子目录含 ≥2 个真项目/聚合根。
    // 判定方式：对每个直接子目录，判断其是否为「真项目」或「聚合根」。
    // 为判定「聚合根」需要递归，但为避免指数级重复遍历，这里用一次轻量探测：
    // 统计「含清单的直接子目录」数量 + 「直接子目录自身是聚合根」的数量。
    let mut project_like_children = 0usize;
    for child in &child_dirs {
        if is_real_project(child) || is_aggregated_root(child, options) {
            project_like_children += 1;
        }
    }

    if project_like_children >= 2 {
        // 聚合根：生成一张卡片，子项目作为其 tree 子结构。
        //
        // 「父项目边界优先」落地：子项目**不并列生成卡片**——它们仍入库
        // （带 `parent_path` 归属，落库后回填为 `parent_id`），但 `get_projects`
        // 默认只返回顶层项目（`parent_id IS NULL`），子项目经
        // `parent_id_filter = Some(父id)` 或 `get_dir_children` 按需获取。
        let health = health_score(dir, ProjectKind::AggregatedRoot);
        let parent_path = dir_path_string(dir);
        projects.push(DetectedProject::new_with_kind(
            name.clone(),
            parent_path.clone(),
            None,
            None,
            None,
            ProjectKind::AggregatedRoot,
            health,
            parent.map(String::from),
        ));

        // 子项目作为聚合根的 tree 子结构：入库（带 parent 归属），后代不再展开。
        for child in &child_dirs {
            classify_dir(child, depth + 1, options, Some(&parent_path), projects, ignored_count)?;
        }
        return Ok(());
    }

    // ③ 分类目录：无清单/子项目，但含子目录 → 可折叠卡片，递归识别内部。
    // ④ 普通目录：无任何子目录 → 不生成卡片。
    if child_dirs.is_empty() {
        // 普通目录（无子目录、无清单）→ 不生成卡片。
        return Ok(());
    }

    // 分类目录：先递归识别内部，判断内部是否有项目。
    // 为支持「内部无项目 → 低分卡片」与「内部有项目 → 正常卡片」，
    // 先递归收集内部项目，再决定是否生成分类目录卡片。
    let parent_path = dir_path_string(dir);
    let before = projects.len();
    for child in &child_dirs {
        classify_dir(child, depth + 1, options, Some(&parent_path), projects, ignored_count)?;
    }
    let found_inner = projects.len() > before;

    // 分类目录卡片：内部有项目 → 健康度按分类目录规则；内部无项目 → 低分。
    let health = if found_inner {
        health_score(dir, ProjectKind::Category)
    } else {
        // 低分容器：仅目录名与存在性，给个很低的分数。
        low_category_score()
    };

    projects.push(DetectedProject::new_with_kind(
        name,
        parent_path,
        None,
        None,
        None,
        ProjectKind::Category,
        health,
        parent.map(String::from),
    ));

    Ok(())
}

/// 判断目录是否含可识别清单文件。
fn has_manifest(dir: &Path) -> bool {
    MANIFEST_FILES.iter().any(|m| dir.join(m).is_file())
}

/// 判断目录是否为「真项目」（含清单）。
fn is_real_project(dir: &Path) -> bool {
    has_manifest(dir)
}

/// 判断目录是否为「聚合根」（无清单，但直接子目录含 ≥2 个真项目/聚合根）。
fn is_aggregated_root(dir: &Path, options: &ScanOptions) -> bool {
    if has_manifest(dir) {
        return false;
    }
    let mut ignored = 0usize;
    let children = match list_child_dirs(dir, options, &mut ignored) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let count = children
        .iter()
        .filter(|c| is_real_project(c) || is_aggregated_root(c, options))
        .count();
    count >= 2
}

/// 读取目录的直接子目录列表（排除忽略项；忽略的目录计入 `ignored_count`）。
fn list_child_dirs(
    dir: &Path,
    options: &ScanOptions,
    ignored_count: &mut usize,
) -> Result<Vec<PathBuf>, ScanError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()), // 权限不足等，静默跳过
    };

    let mut dirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if should_ignore(&file_name, options) {
            *ignored_count += 1;
            continue;
        }
        dirs.push(path);
    }
    Ok(dirs)
}

/// 判断目录是否含可识别清单文件，若有则返回技术栈元数据。
fn project_meta_if_manifest(dir: &Path) -> Result<Option<ProjectMeta>, ScanError> {
    if !has_manifest(dir) {
        return Ok(None);
    }
    // 用 parser 解析出技术栈；解析失败不影响识别（仍视为项目）。
    match parser::detect_stack(dir) {
        Ok(Some(meta)) => Ok(Some(meta)),
        Ok(None) => Ok(Some(ProjectMeta::new(None, None))),
        Err(_) => Ok(Some(ProjectMeta::new(None, None))),
    }
}

/// 忽略规则：隐藏目录 + 预定义忽略目录 + 用户自定义。
fn should_ignore(file_name: &str, options: &ScanOptions) -> bool {
    // 隐藏目录（以 . 开头，排除 . 和 ..）
    if file_name.starts_with('.') {
        return true;
    }
    if IGNORED_DIRS.contains(&file_name) {
        return true;
    }
    options.extra_ignored_dirs.iter().any(|d| d == file_name)
}

/// 计算项目名：优先取清单中的 name，否则取目录名。
fn project_name(dir: &Path, meta: &ProjectMeta) -> String {
    let dir_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.display().to_string());

    for manifest in MANIFEST_FILES {
        let path = dir.join(manifest);
        if !path.is_file() {
            continue;
        }
        if *manifest == "package.json" {
            if let Some(name) = read_json_name(&path) {
                return name;
            }
        } else if *manifest == "pyproject.toml" {
            if let Some(name) = read_toml_name(&path) {
                return name;
            }
        }
    }

    let _ = meta;
    dir_name
}

/// 计算目录名（用于分类目录/聚合根的 name）。
fn dir_name(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.display().to_string())
}

/// 将目录路径转为字符串。
fn dir_path_string(dir: &Path) -> String {
    dir.display().to_string()
}

/// 从 JSON 读取顶层 `name` 字符串字段。
fn read_json_name(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    value.get("name").and_then(|v| v.as_str()).map(String::from)
}

/// 从 TOML 读取 `[project].name`。
fn read_toml_name(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let value: toml::Value = toml::from_str(&raw).ok()?;
    value
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(String::from)
}

// ---------------------------------------------------------------------------
// 健康度评分（`docs/v0.2-scanner-plan.md` §2.3）
// ---------------------------------------------------------------------------

/// 健康度评分：基础分 + 加权分，返回 0-100。
///
/// 加分维度（`docs/v0.2-scanner-plan.md` §2.3，逐维度独立、与 kind 解耦）：
/// - 含清单文件 +20
/// - 有 README +5
/// - 是 Git 仓库（`.git` 存在）+15
/// - 有 CI 配置（`.github/workflows` 等）+5
/// - 文件数适中（约 10-500）+20（过少/过多降权）
/// - 结构规范（src/、cmd/、internal/ 等典型目录）+10
///
/// 聚合根/分类目录另有保底分（见 `kind` 分支），使其作为容器也有可读分数。
pub fn health_score(dir: &Path, kind: ProjectKind) -> i64 {
    let mut score = 0i64;

    // 含清单文件 +20：按「实际清单存在」加分，不依赖 kind（避免隐式耦合）。
    if has_manifest(dir) {
        score += 20;
    }

    // README
    if dir.join("README.md").is_file() || dir.join("README").is_file() {
        score += 5;
    }

    // Git 仓库
    if dir.join(".git").exists() {
        score += 15;
    }

    // CI 配置
    if dir.join(".github").join("workflows").is_dir()
        || dir.join(".gitlab-ci.yml").is_file()
        || dir.join(".travis.yml").is_file()
    {
        score += 5;
    }

    // 文件数适中（约 10-500）
    let file_count = count_files_light(dir);
    score += match file_count {
        0..=4 => 0,     // 极空
        5..=9 => 5,
        10..=500 => 20, // 适中
        _ => 10,        // 过大，降权
    };

    // 结构规范
    if has_standard_structure(dir) {
        score += 10;
    }

    // 聚合根/分类目录额外保底分（作容器，通常 0-40）
    match kind {
        ProjectKind::Real => {}
        ProjectKind::AggregatedRoot => {
            // 聚合根含 ≥2 子项目，给中等基础分
            score += 20;
        }
        ProjectKind::Category => {
            // 分类目录作为容器，基础分低
            score += 5;
        }
    }

    score.clamp(0, 100)
}

/// 分类目录（内部无项目）的低分：仅给一个很小的保底分。
fn low_category_score() -> i64 {
    2
}

/// 判断目录是否含规范结构（src/、cmd/、internal/、lib/、pkg/ 等典型目录）。
fn has_standard_structure(dir: &Path) -> bool {
    const STD_DIRS: &[&str] = &[
        "src", "cmd", "internal", "lib", "pkg", "app", "components", "pages",
        "server", "client", "test", "tests", "examples",
    ];
    STD_DIRS.iter().any(|d| dir.join(d).is_dir())
}

/// 轻量文件计数（只读，跳过隐藏目录与忽略目录，限制深度避免慢）。
fn count_files_light(root: &Path) -> i64 {
    const IGNORED: &[&str] = &[
        "node_modules",
        ".git",
        "target",
        "dist",
        "build",
        "vendor",
        ".cache",
    ];

    fn walk(dir: &Path, ignored: &[&str], count: &mut i64, depth: usize) {
        if depth > 8 {
            return;
        }
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                if ignored.contains(&name.as_str()) {
                    continue;
                }
                walk(&path, ignored, count, depth + 1);
            } else if path.is_file() {
                *count += 1;
            }
        }
    }

    let mut count = 0i64;
    walk(root, IGNORED, &mut count, 0);
    count
}

// ---------------------------------------------------------------------------
// 目录树按需加载（`docs/v0.2-scanner-plan.md` §3.4 `get_dir_children`）
// ---------------------------------------------------------------------------

/// 返回指定目录的直接子项（`DirNode[]`），供前端懒加载目录树。
///
/// - 只返回**直接**子项（不递归），性能友好。
/// - 目录/文件均返回；隐藏项与预设忽略目录默认跳过（与扫描一致）。
/// - `has_manifest` 标记该子目录是否为「真项目根」（含清单文件）。
/// - `children_count` 仅目录有效，为直接子项数量（前端据此判断能否展开）。
///
/// 目录不可读 / 不存在返回 `Ok(空)`（静默降级，不报错）。
pub fn list_dir_children(dir: &Path) -> Vec<DirNode> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut nodes: Vec<DirNode> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        // 跳过隐藏项（. 开头）
        if name.starts_with('.') {
            continue;
        }
        let is_dir = path.is_dir();
        if is_dir {
            // 跳过预设忽略目录（node_modules 等），避免目录树里塞满产物
            if IGNORED_DIRS.contains(&name.as_str()) {
                continue;
            }
        }

        let has_manifest = is_dir && has_manifest(&path);
        let children_count = if is_dir {
            count_direct_children(&path)
        } else {
            0
        };

        nodes.push(DirNode {
            name,
            path: path.display().to_string(),
            is_dir,
            has_manifest,
            children_count,
        });
    }

    // 目录在前、文件在后，各自按名称排序（稳定、可预期）。
    nodes.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    nodes
}

/// 统计目录的直接子项数量（不含隐藏项与预设忽略目录）。
fn count_direct_children(dir: &Path) -> usize {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    entries
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            !name.starts_with('.') && !IGNORED_DIRS.contains(&name.as_str())
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// 构造临时工作区树（v0.2 语义）。
    fn build_fixture() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ydevsphere_scan_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("创建根目录失败");

        let write = |rel: &str, content: &str| {
            let path = root.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("创建父目录失败");
            }
            std::fs::write(path, content).expect("写入失败");
        };

        write("app/package.json", r#"{"name":"my-vue-app","dependencies":{"vue":"^3.4.0"}}"#);
        write("server/Cargo.toml", "[package]\nname = \"server\"\n");
        write("python-app/pyproject.toml", "[project]\nname = \"py-app\"\n");
        write("nested/lib/go.mod", "module example.com/lib\n");
        write("ignored-projects/node_modules/dep/package.json", "{}");
        write("ignored-projects/target/app/package.json", "{}");
        write("ignored-projects/.hidden/app/package.json", "{}");
        root
    }

    /// 构造「聚合根」场景：sub2api 下 frontend/backend（两个真项目）。
    fn build_aggregate_fixture() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ydevsphere_agg_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("创建根目录失败");

        let write = |rel: &str, content: &str| {
            let path = root.join(rel);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("创建父目录失败");
            }
            std::fs::write(path, content).expect("写入失败");
        };

        write("sub2api/frontend/package.json", r#"{"name":"frontend","dependencies":{"vue":"^3.0.0"}}"#);
        write("sub2api/backend/Cargo.toml", "[package]\nname = \"backend\"\n");
        write("sub2api/bridge/go.mod", "module example.com/bridge\n");
        root
    }

    #[test]
    fn detects_projects_and_ignores_dirs() {
        let root = build_fixture();
        let out = scan_workspace(&root).expect("扫描应成功");

        let mut paths: Vec<String> = out
            .projects
            .iter()
            .map(|p| {
                p.path
                    .trim_start_matches(root.to_str().unwrap())
                    .trim_start_matches('/')
                    .to_string()
            })
            .collect();
        paths.sort();

        // v0.2 语义：app / server / python-app 是真项目（顶层）；
        // nested 是分类目录（含子项目 lib）；nested/lib 是真项目（parent=nested）。
        assert_eq!(
            paths,
            vec!["app", "nested", "nested/lib", "python-app", "server"]
        );
        assert!(out.ignored_count >= 3);
    }

    #[test]
    fn project_names_resolved() {
        let root = build_fixture();
        let out = scan_workspace(&root).expect("扫描应成功");

        let app = out
            .projects
            .iter()
            .find(|p| p.path.ends_with("/app"))
            .or_else(|| out.projects.iter().find(|p| p.path.ends_with("app") && !p.path.contains("nested")))
            .expect("应找到 app 项目");
        assert_eq!(app.name, "my-vue-app");

        let server = out
            .projects
            .iter()
            .find(|p| p.path.ends_with("server"))
            .expect("应找到 server 项目");
        assert_eq!(server.name, "server");
    }

    #[test]
    fn errors_on_non_directory() {
        let path = std::path::Path::new("/this/path/does/not/exist_xyz");
        assert!(matches!(
            scan_workspace(path),
            Err(ScanError::NotADirectory(_))
        ));
    }

    /// ScanError::code()：返回结构化错误码（前端按码判断）。
    #[test]
    fn scan_error_codes() {
        let not_dir = ScanError::NotADirectory("/bad".to_string());
        assert_eq!(not_dir.code(), "INVALID_DIRECTORY");

        let io = ScanError::Io(std::io::Error::new(std::io::ErrorKind::Other, "boom"));
        assert_eq!(io.code(), "IO_ERROR");
    }

    #[test]
    fn ignored_dirs_list_is_complete() {
        let expected: HashSet<&str> =
            ["node_modules", ".git", "target", "dist", "build", "vendor", ".cache"]
                .into_iter()
                .collect();
        for d in IGNORED_DIRS {
            assert!(expected.contains(d), "忽略规则缺少 {d}");
        }
    }

    // ---- v0.2 新增单测 ----

    /// 聚合根识别：sub2api 无清单，但含 ≥2 个真项目，应识别为聚合根，
    /// 且 frontend/backend/bridge 作为其子结构（parent 归属 sub2api）。
    #[test]
    fn detects_aggregated_root_with_children() {
        let root = build_aggregate_fixture();
        let out = scan_workspace(&root).expect("扫描应成功");

        // 应识别出 sub2api 聚合根
        let agg = out
            .projects
            .iter()
            .find(|p| p.path.ends_with("sub2api"))
            .expect("应识别 sub2api 聚合根");
        assert_eq!(agg.kind, ProjectKind::AggregatedRoot);

        // frontend / backend / bridge 应作为聚合根的子结构（parent 指向 sub2api）
        let children: Vec<_> = out
            .projects
            .iter()
            .filter(|p| p.parent_path.as_deref() == Some(agg.path.as_str()))
            .collect();
        assert_eq!(children.len(), 3, "聚合根应有 3 个子项目");
    }

    /// 真项目内部不拆：含清单的项目内部不再识别子项目。
    #[test]
    fn real_project_not_split_internally() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ydevsphere_realsplit_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("创建根目录失败");

        // 根项目含清单 + 内部子目录也含清单（不应被拆出）
        std::fs::write(root.join("package.json"), r#"{"name":"root-app"}"#).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/package.json"), r#"{"name":"nested-in-src"}"#).unwrap();

        let out = scan_workspace(&root).expect("扫描应成功");
        // 仅 1 个项目（root-app），src 内的 package.json 不应被识别
        assert_eq!(out.projects.len(), 1);
        assert_eq!(out.projects[0].name, "root-app");
        assert_eq!(out.projects[0].kind, ProjectKind::Real);
    }

    /// 分类目录识别：含子目录但无清单/无子项目时，仍生成低分可折叠卡片。
    #[test]
    fn detects_category_dir() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ydevsphere_cat_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("创建根目录失败");

        // 学习/ 分类目录，内部有子目录但无项目特征
        std::fs::create_dir_all(root.join("学习/笔记")).unwrap();
        std::fs::create_dir_all(root.join("学习/教程")).unwrap();

        let out = scan_workspace(&root).expect("扫描应成功");
        // 应识别出「学习」分类目录（低分）
        let cat = out
            .projects
            .iter()
            .find(|p| p.path.ends_with("学习"))
            .expect("应识别「学习」分类目录");
        assert_eq!(cat.kind, ProjectKind::Category);
        // 内部无项目 → 低分
        assert!(cat.health_score < 20, "内部无项目的分类目录应为低分");
    }

    /// 健康度评分：真项目（含清单 + README + Git）分数应高于空目录。
    #[test]
    fn health_score_real_project_higher_than_empty() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ydevsphere_health_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("创建根目录失败");

        // 真项目：清单 + README + .git + src + 10 个源文件（文件数适中）
        std::fs::write(root.join("package.json"), r#"{"name":"healthy"}"#).unwrap();
        std::fs::write(root.join("README.md"), "# Healthy").unwrap();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        for i in 0..10 {
            std::fs::write(root.join(format!("src/file{i}.rs")), "// x").unwrap();
        }

        let out = scan_workspace(&root).expect("扫描应成功");
        let proj = &out.projects[0];
        assert_eq!(proj.kind, ProjectKind::Real);
        // 清单20 + README5 + git15 + 文件数适中20 + 结构规范10 = 70
        assert!(proj.health_score >= 60, "规范真项目健康度应 ≥60，实际 {}", proj.health_score);
    }

    /// 健康度评分逐维度加权断言（`docs/v0.2-scanner-plan.md` §2.3）。
    ///
    /// 直接调用 `health_score`，逐步叠加各维度，验证每个维度的精确加分：
    /// 清单 +20、README +5、Git +15、CI +5、文件数适中 +20、结构规范 +10。
    #[test]
    fn health_score_dimension_weights() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "ydevsphere_health_dims_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("创建根目录失败");

        // 基线：空目录，Real kind（无清单），0 分。
        // 注意：Real kind 无额外保底分，空目录各维度均 0 → 0 分。
        assert_eq!(health_score(&dir, ProjectKind::Real), 0, "空目录基线应为 0");

        // + 清单（package.json）→ +20
        std::fs::write(dir.join("package.json"), r#"{"name":"x"}"#).unwrap();
        assert_eq!(health_score(&dir, ProjectKind::Real), 20, "清单 +20");

        // + README → +5
        std::fs::write(dir.join("README.md"), "# x").unwrap();
        assert_eq!(health_score(&dir, ProjectKind::Real), 25, "README +5");

        // + Git 仓库（.git 目录）→ +15
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        assert_eq!(health_score(&dir, ProjectKind::Real), 40, "Git +15");

        // + CI 配置（.github/workflows）→ +5
        std::fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        assert_eq!(health_score(&dir, ProjectKind::Real), 45, "CI +5");

        // + 文件数适中（10-500）→ +20
        for i in 0..10 {
            std::fs::write(dir.join(format!("f{i}.txt")), "x").unwrap();
        }
        assert_eq!(health_score(&dir, ProjectKind::Real), 65, "文件数适中 +20");

        // + 结构规范（src/）→ +10
        std::fs::create_dir_all(dir.join("src")).unwrap();
        assert_eq!(health_score(&dir, ProjectKind::Real), 75, "结构规范 +10");

        // 聚合根保底分 +20（单独用无清单目录验证，避免与清单加分混淆）。
        let agg_dir = dir.join("agg");
        std::fs::create_dir_all(&agg_dir).unwrap();
        // 空聚合根：无清单/README/Git/CI/文件/结构 → 仅保底 +20
        assert_eq!(
            health_score(&agg_dir, ProjectKind::AggregatedRoot),
            20,
            "空聚合根仅保底 +20"
        );

        // 分类目录保底分 +5
        assert_eq!(
            health_score(&agg_dir, ProjectKind::Category),
            5,
            "空分类目录仅保底 +5"
        );
    }

    /// 健康度评分：纯空目录（无清单）不应被识别为项目（普通目录）。
    #[test]
    fn empty_dir_not_detected() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ydevsphere_empty_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("创建根目录失败");
        // 空目录（无任何子目录/清单）
        let out = scan_workspace(&root).expect("扫描应成功");
        assert!(out.projects.is_empty(), "空目录不应识别任何项目");
    }

    /// `list_dir_children`：返回直接子项，标记 has_manifest 与 children_count。
    #[test]
    fn list_dir_children_returns_direct_children() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ydevsphere_dirchildren_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("创建根目录失败");

        // 结构：app(含清单) / docs(空目录) / readme.md / node_modules(应跳过)
        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::write(root.join("app/package.json"), r#"{"name":"app"}"#).unwrap();
        std::fs::create_dir_all(root.join("docs")).unwrap();
        std::fs::write(root.join("readme.md"), "# readme").unwrap();
        std::fs::create_dir_all(root.join("node_modules/xxx")).unwrap();

        let nodes = list_dir_children(&root);

        // node_modules 应被跳过
        assert!(nodes.iter().all(|n| n.name != "node_modules"));

        // app 目录：has_manifest = true，children_count >= 1（含 package.json）
        let app = nodes.iter().find(|n| n.name == "app").expect("应有 app 目录");
        assert!(app.is_dir);
        assert!(app.has_manifest, "app 含清单，应标记 has_manifest");
        assert_eq!(app.children_count, 1, "app 直接子项为 package.json");

        // docs 目录：has_manifest = false，children_count = 0
        let docs = nodes.iter().find(|n| n.name == "docs").expect("应有 docs 目录");
        assert!(docs.is_dir);
        assert!(!docs.has_manifest);
        assert_eq!(docs.children_count, 0);

        // readme.md 文件
        let readme = nodes.iter().find(|n| n.name == "readme.md").expect("应有 readme.md");
        assert!(!readme.is_dir);
        assert!(!readme.has_manifest);
        assert_eq!(readme.children_count, 0);
    }

    /// `list_dir_children`：不存在的目录返回空列表（不 panic）。
    #[test]
    fn list_dir_children_missing_dir_returns_empty() {
        let nodes = list_dir_children(std::path::Path::new("/no/such/dir_xyz"));
        assert!(nodes.is_empty());
    }

    /// 自定义忽略规则：`extra_ignored_dirs` 应生效。
    #[test]
    fn custom_ignore_rules_take_effect() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "ydevsphere_custom_ignore_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("创建根目录失败");

        std::fs::create_dir_all(root.join("app")).unwrap();
        std::fs::create_dir_all(root.join("vendored")).unwrap();
        std::fs::write(root.join("app/package.json"), r#"{"name":"app"}"#).unwrap();
        std::fs::write(root.join("vendored/package.json"), r#"{"name":"vendored"}"#).unwrap();

        // 默认扫描应识别 app + vendored
        let out_default = scan_workspace(&root).expect("扫描应成功");
        assert_eq!(out_default.projects.len(), 2);

        // 自定义忽略 vendored 后，只剩 app
        let options = ScanOptions {
            max_depth: DEFAULT_MAX_DEPTH,
            extra_ignored_dirs: vec!["vendored".to_string()],
        };
        let out_custom = scan_workspace_with_options(&root, &options).expect("扫描应成功");
        assert_eq!(out_custom.projects.len(), 1);
        assert!(out_custom.projects[0].path.ends_with("/app"));
    }
}
