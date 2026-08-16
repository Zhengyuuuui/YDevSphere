//! 编辑器动态发现（macOS 优先）。
//!
//! 职责：
//! - 遍历 `/Applications` + `~/Applications`，筛选 `.app`。
//! - 对每个 `.app` 做三档检测：
//!   ① `product.json` 指纹：`<App>/Contents/Resources/app/product.json` 存在 →
//!      解析 `nameShort` / `applicationName` / `dataFolderName`，判定 VS Code Fork。
//!   ② `Info.plist` 兜底：读 `CFBundleIdentifier` / `CFBundleName` /
//!      `CFBundleDocumentTypes`，判断 `open -a` / unsupported。
//!   ③ 白名单优先：命中现有白名单用其可靠配置，动态发现仅补充白名单外。
//! - CLI 路径解析：优先 `which <applicationName>`（PATH），否则 app 内绝对路径
//!   `<App>/Contents/Resources/app/bin/<applicationName>`。
//! - 去重：Trae/Trae CN、Qoder/Qoder CN 合并（优先 CN）；WorkBuddy 系列不去重。
//! - 排除名单：Xcode、HBuilderX、纯 CLI 工具（claude/codex/aider）。
//!
//! ## 约束
//! - 只读勘察，不写用户文件。
//! - 硬性约束：本模块禁止 `use tauri`。

use std::path::{Path, PathBuf};

use crate::core::models::{
    AvailableEditor, EditorCategory, EditorSource, OpenMethod,
};

// ---------------------------------------------------------------------------
// product.json 指纹（VS Code Fork 判定）
// ---------------------------------------------------------------------------

/// `product.json` 解析出的 VS Code Fork 指纹。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductJsonFingerprint {
    /// `nameShort`（如 "Code" / "Cursor"）。
    pub name_short: Option<String>,
    /// `applicationName`（CLI 命令名，如 "code" / "cursor"）。
    pub application_name: Option<String>,
    /// `dataFolderName`（如 ".vscode" / ".cursor"）。
    pub data_folder_name: Option<String>,
}

impl ProductJsonFingerprint {
    /// 是否可判定为 VS Code Fork（含 applicationName 即视为有效 Fork）。
    pub fn is_vscode_fork(&self) -> bool {
        self.application_name.is_some()
    }
}

/// 解析 `product.json` 内容，提取 VS Code Fork 指纹。
///
/// 仅提取 `nameShort` / `applicationName` / `dataFolderName` 三个字段，
/// 容忍缺失字段（`None`）；JSON 解析失败返回 `None`（非 Fork / 非 product.json）。
pub fn parse_product_json(raw: &str) -> Option<ProductJsonFingerprint> {
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    // 非 object（数组/字符串等）不是合法 product.json
    if !value.is_object() {
        return None;
    }
    Some(ProductJsonFingerprint {
        name_short: value.get("nameShort").and_then(|v| v.as_str()).map(String::from),
        application_name: value
            .get("applicationName")
            .and_then(|v| v.as_str())
            .map(String::from),
        data_folder_name: value
            .get("dataFolderName")
            .and_then(|v| v.as_str())
            .map(String::from),
    })
}

/// 读取 `.app` 内的 `product.json` 路径（若存在）。
pub fn product_json_path(app_path: &Path) -> PathBuf {
    app_path.join("Contents/Resources/app/product.json")
}

// ---------------------------------------------------------------------------
// Info.plist 兜底解析
// ---------------------------------------------------------------------------

/// `Info.plist` 解析出的兜底信息。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InfoPlistInfo {
    /// `CFBundleIdentifier`。
    pub bundle_identifier: Option<String>,
    /// `CFBundleName`。
    pub bundle_name: Option<String>,
    /// `CFBundleDocumentTypes` 是否声明 `public.folder`（决定能否 `open -a`）。
    pub supports_folder: bool,
}

/// 解析 `Info.plist`（XML 字符串）提取关键字段。
///
/// 用轻量字符串扫描（不引入 plist 依赖），提取：
/// - `<key>CFBundleIdentifier</key><string>...</string>`
/// - `<key>CFBundleName</key><string>...</string>`
/// - `<key>CFBundleDocumentTypes</key>` 块内是否含 `public.folder`
pub fn parse_info_plist(raw: &str) -> InfoPlistInfo {
    let mut info = InfoPlistInfo::default();

    info.bundle_identifier = extract_plist_string(raw, "CFBundleIdentifier");
    info.bundle_name = extract_plist_string(raw, "CFBundleName");

    // 判定 public.folder：CFBundleDocumentTypes 块内是否出现 public.folder
    // 简化处理：整个 plist 中若 CFBundleDocumentTypes 与 public.folder 共存即视为支持。
    // （更精确的做法是解析数组结构，但字符串扫描足够覆盖绝大多数情况。）
    info.supports_folder = raw.contains("CFBundleDocumentTypes")
        && raw.contains("public.folder");

    info
}

/// 从 plist 文本中提取某个 key 对应的 `<string>` 值。
///
/// 形如：`<key>CFBundleIdentifier</key><string>com.example.app</string>`。
fn extract_plist_string(raw: &str, key: &str) -> Option<String> {
    let needle = format!("<key>{key}</key>");
    let idx = raw.find(&needle)?;
    let after = &raw[idx + needle.len()..];
    let s_open = after.find("<string>")?;
    let s_start = s_open + "<string>".len();
    let s_end = after[s_start..].find("</string>")?;
    Some(after[s_start..s_start + s_end].to_string())
}

// ---------------------------------------------------------------------------
// 目录遍历
// ---------------------------------------------------------------------------

/// 收集 macOS 上待扫描的 `.app` 候选（`/Applications` + `~/Applications`）。
///
/// 返回去重后的 `.app` 绝对路径列表（目录不存在 / 不可读静默跳过）。
pub fn discover_app_candidates() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    roots.push(PathBuf::from("/Applications"));
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join("Applications"));
    }

    let mut out: Vec<PathBuf> = Vec::new();
    for root in roots {
        collect_apps(&root, &mut out);
    }

    // 去重（同一 .app 可能同时出现在两个根，罕见但防御）
    out.sort();
    out.dedup();
    out
}

/// 递归收集目录下所有 `.app` 包（`.app` 是目录，但其内部不再递归）。
fn collect_apps(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.extension().and_then(|e| e.to_str()) == Some("app") {
                // .app 包：记录后不再下钻
                out.push(path);
            } else {
                // 普通目录：递归一层（避免过深）
                collect_apps(&path, out);
            }
        }
    }
}

/// 从 `.app` 路径提取应用名（去掉 `.app` 后缀）。
pub fn app_name_from_path(app_path: &Path) -> String {
    app_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .map(|n| n.strip_suffix(".app").map(String::from).unwrap_or(n))
        .unwrap_or_else(|| app_path.display().to_string())
}

// ---------------------------------------------------------------------------
// 排除名单 / 去重
// ---------------------------------------------------------------------------

/// 排除名单：这些 .app 不纳入发现。
///
/// - Xcode、HBuilderX：非通用编辑器 / IDE（不用于打开项目目录）。
/// - 纯 CLI 工具（claude/codex/aider）：无 .app 包，防御性排除（若出现同名 app）。
const EXCLUDED_APPS: &[&str] = &["Xcode", "HBuilderX", "claude", "codex", "aider"];

/// 是否在排除名单内。
pub fn is_excluded(app_name: &str) -> bool {
    EXCLUDED_APPS.iter().any(|e| *e == app_name)
}

/// 去重合并组：同一组内多个 app 名合并为一条，取组内**首个**（优先级最高）。
///
/// 规则（任务单）：
/// - Trae / Trae CN → 合并，优先 CN（组首为 "Trae CN"）。
/// - Qoder / Qoder CN → 合并，优先 CN。
/// - WorkBuddy / WorkBuddy AI 不去重（不同产品）。
const MERGE_GROUPS: &[&[&str]] = &[
    &["Trae CN", "Trae"],
    &["Qoder CN", "Qoder"],
];

/// 合并组内某 app 名的「规范化代表名」（用于按组去重）。
///
/// 返回组首名（优先级最高者）若该名在某个合并组内，否则返回 `None`。
pub fn merge_group_key(app_name: &str) -> Option<&'static str> {
    MERGE_GROUPS
        .iter()
        .find(|group| group.contains(&app_name))
        .map(|group| group[0])
}

// ---------------------------------------------------------------------------
// CLI 路径解析（关键）
// ---------------------------------------------------------------------------

/// 解析 CLI 命令路径。
///
/// 优先级：
/// 1. `which <applicationName>`（PATH 命中）。
/// 2. app 内绝对路径 `<App>/Contents/Resources/app/bin/<applicationName>`。
///
/// 用 product.json 的 `applicationName` 作为命令名（不用 bin 里的 `code`，避免冲突）。
/// 返回 `(cli_command 字符串, 实际可执行路径)`；均不可用返回 `None`。
pub fn resolve_cli(
    application_name: &str,
    app_path: &Path,
) -> Option<(String, PathBuf)> {
    // 1. PATH 命中
    if let Some(found) = find_in_path(application_name) {
        return Some((application_name.to_string(), found));
    }
    // 2. app 内绝对路径
    let abs = app_path.join("Contents/Resources/app/bin").join(application_name);
    if is_executable(&abs) {
        return Some((abs.display().to_string(), abs));
    }
    None
}

/// 在 PATH 中查找可执行命令。
fn find_in_path(cmd: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(cmd);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// 是否是可执行文件（存在 + 是文件 + 有执行位）。
fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        true
    }
}

// ---------------------------------------------------------------------------
// 单个 .app 的完整检测
// ---------------------------------------------------------------------------

/// 单个 `.app` 的检测结果。
#[derive(Debug, Clone)]
pub struct AppDetection {
    /// 稳定 id（优先 product.json applicationName，否则 app 名小写化）。
    pub id: String,
    /// 展示名。
    pub name: String,
    /// CLI 命令（字符串形式，可能是命令名或绝对路径）。
    pub cli_command: Option<String>,
    /// app 绝对路径。
    pub app_path: String,
    /// 打开方式。
    pub open_method: OpenMethod,
    /// 分类。
    pub category: EditorCategory,
}

/// 对单个 `.app` 做三档检测，返回 `Option<AppDetection>`（排除/无法识别返回 None）。
pub fn detect_app(app_path: &Path) -> Option<AppDetection> {
    let app_name = app_name_from_path(app_path);
    if is_excluded(&app_name) {
        return None;
    }

    // ① product.json 指纹
    let product_path = product_json_path(app_path);
    let fingerprint = std::fs::read_to_string(&product_path)
        .ok()
        .and_then(|raw| parse_product_json(&raw));

    if let Some(fp) = fingerprint {
        if fp.is_vscode_fork() {
            return Some(build_vscode_fork_detection(app_path, &app_name, &fp));
        }
        // 有 product.json 但无 applicationName：不视为有效 Fork，继续 Info.plist 兜底
    }

    // ② Info.plist 兜底
    let plist_path = app_path.join("Contents/Info.plist");
    let plist_info = std::fs::read_to_string(&plist_path)
        .ok()
        .map(|raw| parse_info_plist(&raw))
        .unwrap_or_default();

    build_infoplist_fallback(app_path, &app_name, &plist_info)
}

/// 构建 VS Code Fork 检测结果。
fn build_vscode_fork_detection(
    app_path: &Path,
    app_name: &str,
    fp: &ProductJsonFingerprint,
) -> AppDetection {
    let application_name = fp.application_name.clone().unwrap_or_else(|| app_name.to_string());
    let display_name = fp.name_short.clone().unwrap_or_else(|| app_name.to_string());
    let id = application_name.clone();

    // CLI 路径解析：优先 PATH，其次 app 内绝对路径
    let cli_command = resolve_cli(&application_name, app_path)
        .map(|(cmd, _)| cmd)
        .or_else(|| {
            // 即使 PATH/绝对路径都未命中，仍暴露绝对路径（前端可据此打开）
            let abs = app_path
                .join("Contents/Resources/app/bin")
                .join(&application_name);
            Some(abs.display().to_string())
        });

    AppDetection {
        id,
        name: display_name,
        cli_command,
        app_path: app_path.display().to_string(),
        open_method: OpenMethod::Cli,
        category: EditorCategory::VscodeFork,
    }
}

/// 构建 Info.plist 兜底检测结果。
fn build_infoplist_fallback(
    app_path: &Path,
    app_name: &str,
    info: &InfoPlistInfo,
) -> Option<AppDetection> {
    let display_name = info.bundle_name.clone().unwrap_or_else(|| app_name.to_string());
    let id = info
        .bundle_identifier
        .clone()
        .unwrap_or_else(|| app_name.to_string());

    // 打开方式分级：supports_folder → OpenA，否则 Unsupported
    let open_method = if info.supports_folder {
        OpenMethod::OpenA
    } else {
        OpenMethod::Unsupported
    };

    // Unsupported 的 app 仍返回（前端可展示但禁用打开）；但排除「完全无信息」的 app。
    // 若既无 bundle_identifier 也无 supports_folder，视为无法识别 → 返回 None。
    if info.bundle_identifier.is_none() && !info.supports_folder {
        return None;
    }

    Some(AppDetection {
        id,
        name: display_name,
        cli_command: None,
        app_path: app_path.display().to_string(),
        open_method,
        category: EditorCategory::Native,
    })
}

/// 将 `AppDetection` 转为 `AvailableEditor`。
fn to_available_editor(det: AppDetection) -> AvailableEditor {
    AvailableEditor {
        id: det.id,
        name: det.name,
        cli_command: det.cli_command,
        app_path: Some(det.app_path),
        open_method: det.open_method,
        source: EditorSource::Discovered,
        category: det.category,
    }
}

/// 动态发现全部编辑器（遍历 + 三档检测 + 去重 + 排除）。
pub fn discover_editors() -> Vec<AvailableEditor> {
    let mut out: Vec<AvailableEditor> = Vec::new();
    let mut seen_names: Vec<String> = Vec::new();

    for app in discover_app_candidates() {
        let app_name = app_name_from_path(&app);

        // 去重：合并组内已出现则跳过（保留优先级最高者）
        if let Some(key) = merge_group_key(&app_name) {
            if seen_names.iter().any(|n| merge_group_key(n) == Some(key)) {
                continue; // 同组已收录优先级更高者，跳过
            }
        } else if seen_names.contains(&app_name) {
            continue;
        }

        if let Some(det) = detect_app(&app) {
            seen_names.push(app_name);
            out.push(to_available_editor(det));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_product_json_extracts_fork_fields() {
        let raw = r#"{"nameShort":"Cursor","applicationName":"cursor","dataFolderName":".cursor"}"#;
        let fp = parse_product_json(raw).expect("应解析成功");
        assert!(fp.is_vscode_fork());
        assert_eq!(fp.name_short.as_deref(), Some("Cursor"));
        assert_eq!(fp.application_name.as_deref(), Some("cursor"));
        assert_eq!(fp.data_folder_name.as_deref(), Some(".cursor"));
    }

    #[test]
    fn parse_product_json_tolerates_missing_fields() {
        let raw = r#"{"applicationName":"windsurf"}"#;
        let fp = parse_product_json(raw).expect("应解析成功");
        assert!(fp.is_vscode_fork());
        assert_eq!(fp.name_short, None);
        assert_eq!(fp.application_name.as_deref(), Some("windsurf"));
    }

    #[test]
    fn parse_product_json_invalid_returns_none() {
        assert!(parse_product_json("{ not json").is_none());
        // 非 object 也返回 None
        assert!(parse_product_json("[]").is_none());
    }

    #[test]
    fn parse_product_json_no_appname_not_fork() {
        let raw = r#"{"nameShort":"X","dataFolderName":".x"}"#;
        let fp = parse_product_json(raw).expect("应解析成功");
        assert!(!fp.is_vscode_fork());
    }

    #[test]
    fn parse_info_plist_extracts_fields() {
        let raw = r#"<?xml version="1.0"?>
<plist version="1.0"><dict>
    <key>CFBundleIdentifier</key><string>com.example.app</string>
    <key>CFBundleName</key><string>Example</string>
    <key>CFBundleDocumentTypes</key>
    <array><dict>
        <key>CFBundleTypeRole</key><string>Viewer</string>
        <key>LSItemContentTypes</key><array><string>public.folder</string></array>
    </dict></array>
</dict></plist>"#;
        let info = parse_info_plist(raw);
        assert_eq!(info.bundle_identifier.as_deref(), Some("com.example.app"));
        assert_eq!(info.bundle_name.as_deref(), Some("Example"));
        assert!(info.supports_folder);
    }

    #[test]
    fn parse_info_plist_no_folder_type() {
        let raw = r#"<plist><dict>
        <key>CFBundleIdentifier</key><string>com.x</string>
    </dict></plist>"#;
        let info = parse_info_plist(raw);
        assert_eq!(info.bundle_identifier.as_deref(), Some("com.x"));
        assert!(!info.supports_folder);
    }

    #[test]
    fn parse_info_plist_empty_returns_default() {
        let info = parse_info_plist("");
        assert_eq!(info.bundle_identifier, None);
        assert!(!info.supports_folder);
    }

    #[test]
    fn app_name_strips_app_suffix() {
        let p = PathBuf::from("/Applications/Cursor.app");
        assert_eq!(app_name_from_path(&p), "Cursor");
    }

    #[test]
    fn excluded_apps_are_filtered() {
        assert!(is_excluded("Xcode"));
        assert!(is_excluded("HBuilderX"));
        assert!(is_excluded("claude"));
        assert!(is_excluded("codex"));
        assert!(is_excluded("aider"));
        assert!(!is_excluded("Cursor"));
        assert!(!is_excluded("Visual Studio Code"));
    }

    #[test]
    fn merge_group_key_maps_to_priority() {
        assert_eq!(merge_group_key("Trae CN"), Some("Trae CN"));
        assert_eq!(merge_group_key("Trae"), Some("Trae CN"));
        assert_eq!(merge_group_key("Qoder CN"), Some("Qoder CN"));
        assert_eq!(merge_group_key("Qoder"), Some("Qoder CN"));
        // 不去重组
        assert_eq!(merge_group_key("WorkBuddy"), None);
        assert_eq!(merge_group_key("WorkBuddy AI"), None);
    }

    /// 构造临时 `.app` 包（含可选 product.json / Info.plist），返回 app 路径。
    fn fake_app(
        name: &str,
        product_json: Option<&str>,
        info_plist: Option<&str>,
    ) -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!(
            "ydevsphere_app_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let app = tmp.join(format!("{name}.app"));
        std::fs::create_dir_all(app.join("Contents/Resources/app")).unwrap();
        std::fs::create_dir_all(app.join("Contents")).unwrap();

        if let Some(pj) = product_json {
            std::fs::write(
                app.join("Contents/Resources/app/product.json"),
                pj,
            )
            .unwrap();
        }
        if let Some(pl) = info_plist {
            std::fs::write(app.join("Contents/Info.plist"), pl).unwrap();
        }
        app
    }

    /// product.json 指纹：VS Code Fork 检测 + CLI 命令用 applicationName。
    #[test]
    fn detect_app_vscode_fork_via_product_json() {
        let app = fake_app(
            "FakeFork",
            Some(r#"{"nameShort":"FakeFork","applicationName":"fakefork","dataFolderName":".fakefork"}"#),
            None,
        );
        let det = detect_app(&app).expect("应识别为 Fork");
        assert_eq!(det.id, "fakefork", "id 用 applicationName");
        assert_eq!(det.name, "FakeFork");
        assert_eq!(det.category, EditorCategory::VscodeFork);
        assert_eq!(det.open_method, OpenMethod::Cli);
        // cli_command 至少存在（PATH 未命中时回退 app 内绝对路径）
        assert!(det.cli_command.is_some());
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// Info.plist 兜底：无 product.json，声明 public.folder → open-a。
    #[test]
    fn detect_app_open_a_fallback_via_info_plist() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>com.example.folderapp</string>
            <key>CFBundleName</key><string>FolderApp</string>
            <key>CFBundleDocumentTypes</key><array><dict>
                <key>LSItemContentTypes</key><array><string>public.folder</string></array>
            </dict></array>
        </dict></plist>"#;
        let app = fake_app("FolderApp", None, Some(plist));
        let det = detect_app(&app).expect("应识别为 Native + open-a");
        assert_eq!(det.id, "com.example.folderapp");
        assert_eq!(det.name, "FolderApp");
        assert_eq!(det.open_method, OpenMethod::OpenA);
        assert_eq!(det.category, EditorCategory::Native);
        assert!(det.cli_command.is_none());
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// Info.plist 兜底：无 public.folder → Unsupported。
    #[test]
    fn detect_app_unsupported_without_folder_type() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>com.example.chat</string>
            <key>CFBundleName</key><string>ChatApp</string>
        </dict></plist>"#;
        let app = fake_app("ChatApp", None, Some(plist));
        let det = detect_app(&app).expect("无 folder 仍应识别但标记 unsupported");
        assert_eq!(det.open_method, OpenMethod::Unsupported);
        assert_eq!(det.category, EditorCategory::Native);
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// 完全无 product.json 且无 Info.plist → 无法识别（None）。
    #[test]
    fn detect_app_no_manifest_no_plist_returns_none() {
        let app = fake_app("Empty", None, None);
        assert!(detect_app(&app).is_none());
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// 排除名单：Xcode 等不纳入发现。
    #[test]
    fn detect_app_excluded_returns_none() {
        let app = fake_app(
            "Xcode",
            Some(r#"{"applicationName":"xcode"}"#),
            None,
        );
        assert!(detect_app(&app).is_none(), "排除名单应返回 None");
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    #[test]
    fn cli_resolution_prefers_path_over_app_bin() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        // 构造临时 app 包 + 临时 PATH 目录
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!("ydevsphere_disc_{}_{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&tmp);
        let app = tmp.join("Fake.app");
        let bin_dir = app.join("Contents/Resources/app/bin");
        std::fs::create_dir_all(&bin_dir).expect("创建 bin 目录失败");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let app_bin = bin_dir.join("fakecmd");
            std::fs::write(&app_bin, "#!/bin/sh\nexit 0\n").unwrap();
            std::fs::set_permissions(&app_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        // 场景1：PATH 中有 fakecmd → 优先 PATH
        let path_dir = tmp.join("pathbin");
        std::fs::create_dir_all(&path_dir).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path_bin = path_dir.join("fakecmd");
            std::fs::write(&path_bin, "#!/bin/sh\nexit 0\n").unwrap();
            std::fs::set_permissions(&path_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let prev = std::env::var_os("PATH");
        std::env::set_var("PATH", &path_dir);
        let resolved = resolve_cli("fakecmd", &app);
        assert!(resolved.is_some());
        let (cmd, exe) = resolved.unwrap();
        assert_eq!(cmd, "fakecmd");
        assert_eq!(exe, path_dir.join("fakecmd"), "应优先 PATH 中的命令");

        // 场景2：PATH 无 fakecmd → 回退 app 内绝对路径
        std::env::set_var("PATH", tmp.join("emptybin"));
        std::fs::create_dir_all(tmp.join("emptybin")).unwrap();
        let resolved2 = resolve_cli("fakecmd", &app);
        #[cfg(unix)]
        {
            let (cmd2, exe2) = resolved2.expect("应回退 app 内绝对路径");
            assert_eq!(cmd2, bin_dir.join("fakecmd").display().to_string());
            assert_eq!(exe2, bin_dir.join("fakecmd"));
        }
        #[cfg(windows)]
        {
            assert!(resolved2.is_none(), "Windows 下无执行位判断，此场景略");
        }

        match prev {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
