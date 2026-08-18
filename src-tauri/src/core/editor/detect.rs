//! 编辑器检测与解析（白名单）。
//!
//! 维护一份**固定白名单**的编辑器定义，每项含：
//! - `id`：稳定标识（对外 key）
//! - `name`：展示名
//! - `cli`：可执行命令名（在 PATH 中检测）
//! - `paths`：各平台安装的候选绝对路径
//!
//! ## 安全
//! 只有白名单内已知编辑器才可能被执行；解析未知 `editor_id` 一律返回
//! `EditorError::UnknownEditor`，绝不执行任何进程。

use std::path::PathBuf;

use crate::core::models::{AvailableEditor, EditorCategory, EditorSource, OpenMethod};

use super::discover;

/// 检测错误。
#[derive(Debug)]
pub enum EditorError {
    /// 未知 / 不在白名单的编辑器 id。
    UnknownEditor(String),
    /// 编辑器检测到但可执行文件不存在 / 不可执行。
    NotFound(String),
    /// 打开失败（启动进程失败）。
    Launch(std::io::Error),
    /// 该编辑器不支持打开（`open_method = Unsupported`）。
    UnsupportedMethod(String),
}

impl std::fmt::Display for EditorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorError::UnknownEditor(id) => write!(f, "未知编辑器: {id}"),
            EditorError::NotFound(id) => write!(f, "编辑器不可用: {id}"),
            EditorError::Launch(e) => write!(f, "启动编辑器失败: {e}"),
            EditorError::UnsupportedMethod(id) => {
                write!(f, "编辑器不支持一键打开，请手动选择目录: {id}")
            }
        }
    }
}

impl std::error::Error for EditorError {}

/// 单个编辑器的静态定义。
pub struct EditorDefinition {
    pub id: &'static str,
    pub name: &'static str,
    /// PATH 中检测的 CLI 命令名。
    pub cli: &'static [&'static str],
    /// 各平台安装的候选绝对路径。
    pub paths: &'static [&'static str],
}

/// 支持的编辑器白名单（Q2 主流：VS Code / Cursor / VSCodium / JetBrains(WebStorm等) /
/// Sublime Text / Atom(可选) / Vim / Neovim）。
pub const SUPPORTED_EDITORS: &[EditorDefinition] = &[
    EditorDefinition {
        id: "vscode",
        name: "Visual Studio Code",
        cli: &["code", "code-insiders"],
        paths: &[
            // macOS
            "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
            // Windows
            "C:/Program Files/Microsoft VS Code/bin/code.cmd",
            "C:/Program Files/Microsoft VS Code/bin/code",
            // Linux (常见安装路径)
            "/usr/bin/code",
            "/usr/local/bin/code",
            "/snap/bin/code",
        ],
    },
    EditorDefinition {
        id: "cursor",
        name: "Cursor",
        cli: &["cursor"],
        paths: &[
            // macOS
            "/Applications/Cursor.app/Contents/Resources/app/bin/cursor",
            // Windows
            "C:/Users/%USERNAME%/AppData/Local/Programs/cursor/resources/app/bin/cursor.cmd",
            // Linux
            "/usr/bin/cursor",
        ],
    },
    EditorDefinition {
        id: "vscodium",
        name: "VSCodium",
        cli: &["codium"],
        paths: &[
            "/Applications/VSCodium.app/Contents/Resources/app/bin/codium",
            "C:/Program Files/VSCodium/bin/codium.cmd",
            "/usr/bin/codium",
        ],
    },
    EditorDefinition {
        id: "webstorm",
        name: "WebStorm",
        cli: &["webstorm"],
        paths: &[
            // macOS JetBrains Toolbox
            "/Applications/WebStorm.app/Contents/MacOS/webstorm",
            // Windows
            "C:/Program Files/JetBrains/WebStorm/bin/webstorm64.exe",
            // Linux
            "/opt/WebStorm/bin/webstorm.sh",
        ],
    },
    EditorDefinition {
        id: "intellij",
        name: "IntelliJ IDEA",
        cli: &["idea"],
        paths: &[
            "/Applications/IntelliJ IDEA.app/Contents/MacOS/idea",
            "C:/Program Files/JetBrains/IntelliJ IDEA/bin/idea64.exe",
            "/opt/idea/bin/idea.sh",
        ],
    },
    EditorDefinition {
        id: "goland",
        name: "GoLand",
        cli: &["goland"],
        paths: &[
            "/Applications/GoLand.app/Contents/MacOS/goland",
            "C:/Program Files/JetBrains/GoLand/bin/goland64.exe",
            "/opt/GoLand/bin/goland.sh",
        ],
    },
    EditorDefinition {
        id: "sublime",
        name: "Sublime Text",
        cli: &["subl"],
        paths: &[
            "/Applications/Sublime Text.app/Contents/SharedSupport/bin/subl",
            "C:/Program Files/Sublime Text/subl.exe",
            "/usr/bin/subl",
        ],
    },
    EditorDefinition {
        id: "atom",
        name: "Atom",
        cli: &["atom"],
        paths: &[
            "/Applications/Atom.app/Contents/Resources/app/atom.sh",
            "C:/Users/%USERNAME%/AppData/Local/atom/bin/atom.cmd",
            "/usr/bin/atom",
        ],
    },
    EditorDefinition {
        id: "vim",
        name: "Vim",
        cli: &["vim", "gvim"],
        paths: &[],
    },
    EditorDefinition {
        id: "nvim",
        name: "Neovim",
        cli: &["nvim"],
        paths: &[],
    },
];

/// 检测所有可用编辑器（白名单 + 动态发现合并）。
///
/// 流程：
/// 1. 白名单：命中（`resolve_editor_command` 可用）→ 用可靠配置，填充完整字段。
/// 2. 动态发现：遍历 `/Applications` + `~/Applications`，补充白名单外的编辑器
///    （product.json 指纹 + Info.plist 兜底）。
/// 3. 去重：动态发现结果若与白名单已覆盖的编辑器重复，跳过。
pub fn list_available_editors() -> Vec<AvailableEditor> {
    let mut out: Vec<AvailableEditor> = Vec::new();

    // 1. 白名单
    let mut whitelist_ids: Vec<String> = Vec::new();
    for def in SUPPORTED_EDITORS {
        if let Some(cmd) = resolve_editor_command(def) {
            whitelist_ids.push(def.id.to_string());
            out.push(whitelist_to_editor(def, &cmd));
        }
    }

    // 2. 动态发现（仅补充白名单外）
    for discovered in discover::discover_editors() {
        // 去重：动态发现的 id 或 CLI 命令与白名单重叠则跳过
        if is_covered_by_whitelist(&discovered, &whitelist_ids) {
            continue;
        }
        // 防御性过滤：Unsupported 一律不进列表（v0.3 误判治理，保险）。
        if discovered.open_method == OpenMethod::Unsupported {
            continue;
        }
        out.push(discovered);
    }

    // 统一填充 256px 大图标（V03-EDITOR-ICON-LARGE）：app_path 指向真实 .app
    // 的编辑器携带图标，纯 CLI（如 vim/nvim）为 None。动态发现项已在
    // to_available_editor 填充，白名单项在此定位 .app 后填充；幂等无害。
    for e in &mut out {
        discover::fill_editor_icon(e);
    }

    out
}

/// 将白名单定义 + 解析出的命令转为 `AvailableEditor`（填充完整字段）。
fn whitelist_to_editor(def: &EditorDefinition, cmd: &PathBuf) -> AvailableEditor {
    // 分类：VS Code 系列 → VscodeFork；其余 → Native。
    let category = if matches!(def.id, "vscode" | "cursor" | "vscodium") {
        EditorCategory::VscodeFork
    } else {
        EditorCategory::Native
    };

    // CLI 命令：取 def.cli 首个（用户可见命令名）；app_path 取 cmd 的绝对路径（作为可用路径）。
    AvailableEditor {
        id: def.id.to_string(),
        name: def.name.to_string(),
        cli_command: def.cli.first().map(|s| s.to_string()),
        app_path: Some(cmd.display().to_string()),
        // icon 由 list_available_editors 统一按 app_path 定位 .app 后填充。
        icon_base64: None,
        open_method: OpenMethod::Cli,
        source: EditorSource::Whitelist,
        category,
    }
}

/// 判断动态发现的编辑器是否已被白名单覆盖（按 id 或 CLI 命令名）。
fn is_covered_by_whitelist(discovered: &AvailableEditor, whitelist_ids: &[String]) -> bool {
    // 按 id 覆盖
    if whitelist_ids.contains(&discovered.id) {
        return true;
    }
    // 按 CLI 命令名覆盖（例如动态发现的 "code" 与白名单 vscode 的 "code" 冲突）
    if let Some(cli) = &discovered.cli_command {
        if SUPPORTED_EDITORS
            .iter()
            .filter(|d| whitelist_ids.iter().any(|id| id == d.id))
            .any(|d| d.cli.contains(&cli.as_str()))
        {
            return true;
        }
    }
    false
}

/// 编辑器 id 是否在白名单内（校验用，不执行任何进程）。
pub fn is_known_editor(id: &str) -> bool {
    SUPPORTED_EDITORS.iter().any(|def| def.id == id)
}

/// 编辑器 id 是否「可用」（白名单 + 动态发现 + 已确认 custom_editors 均纳入）。
///
/// 供 `set_editor_preference` 校验使用：动态发现的编辑器（如 Cursor 之外的
/// Trae/Qoder/ChatGPT 等）以及用户已确认导入的自定义编辑器均可设为默认编辑器。
///
/// **主修 B（V03-EDITOR-FIX）**：纳入 `custom_editors` 作为独立权威源——
/// 即使编辑器已从 /Applications 卸载、自动检测不再返回它，只要它是用户
/// 已确认过的 custom 项，仍视为「已知」并可设为默认（不会报「未知编辑器」）。
///
/// 注意：本函数会触发一次动态发现（遍历 /Applications），成本较高，
/// 仅用于低频的「设默认编辑器」校验场景。
pub fn is_available_editor(id: &str) -> bool {
    if list_available_editors().iter().any(|e| e.id == id) {
        return true;
    }
    // 兜底：查用户已确认导入的 custom_editors。
    super::settings::get_custom_editors()
        .map(|custom| custom.iter().any(|e| e.id == id))
        .unwrap_or(false)
}

/// 按 id 在「白名单 + 动态发现 + 已确认 custom_editors」中查找编辑器；未找到返回 `None`。
///
/// 先查自动检测列表，未命中再查 `custom_editors`（主修 B：卸载后仍可识别）。
pub fn find_editor_by_id(id: &str) -> Option<AvailableEditor> {
    if let Some(editor) = list_available_editors().into_iter().find(|e| e.id == id) {
        return Some(editor);
    }
    // 兜底：查已确认导入的 custom_editors（即使系统已卸载该 app）。
    super::settings::get_custom_editors()
        .ok()?
        .into_iter()
        .find(|e| e.id == id)
}

/// 解析某个编辑器为可执行路径；不可用返回 `None`。
pub fn resolve_editor_command(def: &EditorDefinition) -> Option<PathBuf> {
    // 1) 绝对路径候选
    for path in def.paths {
        let candidate = expand_env(path);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    // 2) PATH 中的 CLI 命令
    for cmd in def.cli {
        if let Some(found) = find_in_path(cmd) {
            return Some(found);
        }
    }
    None
}

/// 解析某个编辑器 id 为可执行路径（白名单内才解析）。
pub fn resolve_editor_by_id(id: &str) -> Result<PathBuf, EditorError> {
    let def = SUPPORTED_EDITORS
        .iter()
        .find(|def| def.id == id)
        .ok_or_else(|| EditorError::UnknownEditor(id.to_string()))?;
    resolve_editor_command(def).ok_or_else(|| EditorError::NotFound(id.to_string()))
}

/// 在 PATH 中查找可执行命令。
fn find_in_path(cmd: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(cmd);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        // Windows：`.exe` / `.cmd` 后缀
        #[cfg(windows)]
        {
            let with_exe = dir.join(format!("{cmd}.exe"));
            if is_executable(&with_exe) {
                return Some(with_exe);
            }
            let with_cmd = dir.join(format!("{cmd}.cmd"));
            if is_executable(&with_cmd) {
                return Some(with_cmd);
            }
        }
    }
    None
}

/// 是否是可执行文件（存在 + 是文件；Windows 下忽略可执行位判断）。
fn is_executable(path: &PathBuf) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(path) {
            Ok(m) => m.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        }
    }
    #[cfg(windows)]
    {
        true
    }
}

/// 展开路径中的 `%USERNAME%`（Windows）与环境变量（`$HOME`）。
fn expand_env(path: &str) -> PathBuf {
    // 处理 %USERNAME% （Windows 风格）
    if path.contains("%USERNAME%") {
        if let Some(user) = std::env::var_os("USERNAME").or_else(|| std::env::var_os("USER")) {
            let user = user.to_string_lossy().to_string();
            return PathBuf::from(path.replace("%USERNAME%", &user));
        }
    }
    // 处理 $HOME / ~ （POSIX 风格）
    if let Some(stripped) = path.strip_prefix('~') {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(stripped.trim_start_matches('/'));
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// 创建临时目录并放入一个可执行文件（模拟 PATH 中的 CLI），返回目录。
    /// 通过临时 PATH 隔离，避免影响真实系统。
    fn fake_bin(name: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "ydevsphere_editor_test_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("创建目录失败");

        let exe = dir.join(name);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::write(&exe, "#!/bin/sh\nexit 0\n").expect("写文件失败");
            std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755))
                .expect("设置权限失败");
        }
        #[cfg(windows)]
        {
            std::fs::write(&exe, "").expect("写文件失败");
        }
        (dir, exe)
    }

    #[test]
    fn whitelist_contains_supported_editors() {
        assert!(is_known_editor("vscode"));
        assert!(is_known_editor("cursor"));
        assert!(is_known_editor("vscodium"));
        assert!(is_known_editor("webstorm"));
        assert!(is_known_editor("intellij"));
        assert!(is_known_editor("goland"));
        assert!(is_known_editor("sublime"));
        assert!(is_known_editor("atom"));
        assert!(is_known_editor("vim"));
        assert!(is_known_editor("nvim"));
    }

    #[test]
    fn rejects_unknown_editor_id() {
        assert!(!is_known_editor("notepad"));
        assert!(!is_known_editor(""));
        assert!(!is_known_editor("rm -rf /")); // 绝不进入白名单
    }

    #[test]
    fn resolve_unknown_editor_errors() {
        assert!(matches!(
            resolve_editor_by_id("bogus"),
            Err(EditorError::UnknownEditor(_))
        ));
    }

    #[test]
    fn detects_editor_from_path() {
        let (dir, _exe) = fake_bin("code");
        let prev = std::env::var_os("PATH");
        std::env::set_var("PATH", &dir);

        // 找到 vscode 的 code 命令
        let found = SUPPORTED_EDITORS
            .iter()
            .find(|d| d.id == "vscode")
            .and_then(resolve_editor_command);
        assert!(found.is_some(), "应在临时 PATH 中找到 code");

        // list_available_editors 应包含 vscode
        let available = list_available_editors();
        assert!(
            available.iter().any(|e| e.id == "vscode"),
            "应检测到 vscode"
        );

        match prev {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_editors_ids_are_unique() {
        let mut ids: Vec<&str> = SUPPORTED_EDITORS.iter().map(|d| d.id).collect();
        ids.sort();
        let mut dedup = ids.clone();
        dedup.dedup();
        assert_eq!(ids, dedup, "编辑器 id 应唯一");
    }

    /// `find_editor_by_id`：白名单内已检测到的编辑器可被找到。
    #[test]
    fn find_editor_by_id_finds_whitelisted_editor() {
        let (dir, _exe) = fake_bin("code");
        let prev = std::env::var_os("PATH");
        std::env::set_var("PATH", &dir);

        // 临时 PATH 含 code → vscode 出现在 list_available_editors
        let found = find_editor_by_id("vscode");
        assert!(found.is_some(), "应在白名单检测中找到 vscode");
        let e = found.unwrap();
        assert_eq!(e.id, "vscode");
        assert_eq!(e.open_method, OpenMethod::Cli);
        assert_eq!(e.source, EditorSource::Whitelist);

        match prev {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `find_editor_by_id`：不存在的 id 返回 None（含潜在注入串）。
    #[test]
    fn find_editor_by_id_unknown_returns_none() {
        assert!(find_editor_by_id("__nonexistent__").is_none());
        assert!(find_editor_by_id("").is_none());
        assert!(find_editor_by_id("rm -rf /").is_none());
    }
}
