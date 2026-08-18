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
    AvailableEditor, EditorCategory, EditorSource, InstalledAppInfo, OpenMethod,
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

/// 高度专属的编程语言/构建配置扩展名清单（v0.3 L2 判定，回归修复）。
///
/// 用于 `CFBundleTypeExtensions` 命中判定：声明了这些扩展名的 app 视为
/// 可编辑代码的 AI 编辑器（ChatGPT/Codex、Claude 等），自动进列表。
///
/// ## 取舍说明（回归修复：Safari/Chrome/视频播放器等被误判）
/// 上一版清单过宽，含 `html` / `css` / `js` / `txt` / `json` / `md` /
/// `xml` / `yaml` / `yml` 等**通用文档类型**——浏览器 / 文本查看器 /
/// 视频播放器（Safari、Chrome、IINA 等）也声明这些类型，导致被误判为编辑器。
///
/// 因此收紧为**高度专属的编程语言 / 构建配置扩展名**（浏览器等不会声明它们）：
/// - 保留：`py` / `rs` / `go` / `swift` / `c` / `cpp` / `h` / `hpp` / `java` /
///   `kt` / `kts` / `rb` / `php` / `lua` / `zig` / `nim` / `dart` / `sql` 等。
/// - 移除：`html` / `css` / `js` / `txt` / `json` / `md` / `xml` / `yaml` /
///   `yml` 等浏览器/通用文档也声明的类型。
///
/// 判定再叠加「命中 ≥2 个不同专属扩展名」或「Editor 角色」信号（见
/// `declares_code_types`），避免单个通用扩展名误判。
///
/// 扩展名不带点号，与 macOS `CFBundleTypeExtensions` 的书写方式一致。
const CODE_FILE_EXTENSIONS: &[&str] = &[
    // 高度专属的编程语言（浏览器/文本查看器通常不声明）
    "py", "pyw", "rs", "go", "swift", "c", "h", "cpp", "cc", "cxx",
    "hpp", "hh", "java", "kt", "kts", "rb", "php", "lua", "zig", "nim",
    "dart", "sql", "ex", "exs", "hs", "erl", "clj", "cljs", "scala",
    "f90", "f95", "pro", "r", "m", "mm",
    // 构建 / 配置（专属，非通用文档）
    "gradle", "dockerfile", "pbxproj", "xcworkspace", "xcodeproj",
];

/// `Info.plist` 解析出的兜底信息。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InfoPlistInfo {
    /// `CFBundleIdentifier`。
    pub bundle_identifier: Option<String>,
    /// `CFBundleName`。
    pub bundle_name: Option<String>,
    /// `CFBundleDocumentTypes` 是否声明 `public.folder`（决定能否 `open -a`）。
    pub supports_folder: bool,
    /// `CFBundleTypeExtensions` 是否声明了足够强的代码类型信号（v0.3 L2 判定，
    /// 回归修复：≥2 个专属扩展名 或 1 个专属扩展名 + Editor 角色）。
    pub declares_code: bool,
}

impl InfoPlistInfo {
    /// 是否声明了足够强的代码类型信号（L2 判定）。
    pub fn declares_code_types(&self) -> bool {
        self.declares_code
    }
}

/// 解析 `Info.plist`（XML 字符串）提取关键字段。
///
/// 用轻量字符串扫描（不引入 plist 依赖），提取：
/// - `<key>CFBundleIdentifier</key><string>...</string>`
/// - `<key>CFBundleName</key><string>...</string>`
/// - `<key>CFBundleDocumentTypes</key>` 块内是否含 `public.folder`
/// - `<key>CFBundleTypeExtensions</key>` 内是否声明代码扩展名（`declares_code`）
pub fn parse_info_plist(raw: &str) -> InfoPlistInfo {
    let mut info = InfoPlistInfo::default();

    info.bundle_identifier = extract_plist_string(raw, "CFBundleIdentifier");
    info.bundle_name = extract_plist_string(raw, "CFBundleName");

    // 判定 public.folder：CFBundleDocumentTypes 块内是否出现 public.folder
    // 简化处理：整个 plist 中若 CFBundleDocumentTypes 与 public.folder 共存即视为支持。
    // （更精确的做法是解析数组结构，但字符串扫描足够覆盖绝大多数情况。）
    info.supports_folder = raw.contains("CFBundleDocumentTypes")
        && raw.contains("public.folder");

    // 判定代码类型：扫描 CFBundleTypeExtensions 块内是否命中代码扩展名。
    // 用局部小写化避免大小写差异；`declares_code_types` 命中任一即 true。
    info.declares_code = declares_code_types(raw);

    info
}

/// 判定 `CFBundleTypeExtensions` 是否声明了足够强的「代码类型」信号（v0.3 回归修复）。
///
/// 为避免单个通用扩展名误判（Safari/Chrome 声明 css/html/js 但非编辑器），
/// 采用**双重信号**：
/// - 命中 **≥2 个不同**专属编程扩展名 → 视为编辑器（强信号）。
/// - 或命中 ≥1 个专属扩展名 且 文档类型角色为 `CFBundleTypeRole == Editor`
///   （而非 Viewer）→ 视为编辑器（更强信号）。
///
/// 这样：
/// - ChatGPT(Codex)/Claude：Editor 角色 + 大量代码类型 → 命中。
/// - Safari/Chrome/IINA：Viewer 角色 + 通用文档类型（css/html/js 已从清单移除，
///   且不声明专属编程扩展名）→ 不命中 → L3 排除。
///
/// 简化为对整个 plist 小写化后逐一查找扩展名（全局查找足够覆盖且代码扩展名
/// 拼写足够特殊，不易误命中无关区域）。
fn declares_code_types(raw: &str) -> bool {
    let lower = raw.to_lowercase();

    // 命中的不同专属扩展名数量。
    let matched_count = CODE_FILE_EXTENSIONS
        .iter()
        .filter(|ext| {
            lower.contains(&format!("<string>{}</string>", ext.to_lowercase()))
        })
        .count();

    // 强信号：文档类型角色为 Editor（而非 Viewer）。
    let role_is_editor = lower.contains("<key>cfbundletyperole</key>")
        && lower.contains("<string>editor</string>");

    // ≥2 个不同专属扩展名 → 编辑器。
    if matched_count >= 2 {
        return true;
    }
    // 1 个专属扩展名 + Editor 角色 → 编辑器。
    role_is_editor && matched_count >= 1
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

/// 派生当前应用目录快照（`.app` 包名列表，去重排序）。
///
/// 复用 `discover_app_candidates()` 已收集的 `.app` 路径，只取包名
/// （去掉 `.app` 后缀），去重 + 排序，供启动比对磁盘变化。只读目录名，
/// 不读 Info.plist / product.json。
pub fn derive_app_snapshot() -> Vec<String> {
    let mut names: Vec<String> = discover_app_candidates()
        .into_iter()
        .map(|p| app_name_from_path(&p))
        .collect();
    names.sort();
    names.dedup();
    names
}

/// 判断是否需要重扫编辑器：当前快照与上次快照不同。
///
/// - 上次快照为空（首次启动）→ 需要扫描（`true`）。
/// - 快照不同（新增/卸载 .app）→ 需要扫描（`true`）。
/// - 快照相同 → 不需要（`false`），走缓存。
pub fn should_rescan_from_snapshot(prev: &[String], current: &[String]) -> bool {
    if prev.is_empty() {
        return true; // 首次（无快照）
    }
    prev != current
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
// 手动导入（V03-MANUAL-IMPORT-BACKEND）
// ---------------------------------------------------------------------------

/// 列出 `/Applications` + `~/Applications` 下**全部** `.app`（与识别逻辑解耦）。
///
/// 复用 `discover_app_candidates()` 收集路径（不过滤识别逻辑、不去重编辑器），
/// 每个返回 `InstalledAppInfo`（name / path / bundle_id / has_product_json /
/// icon_base64）。按名称排序，方便前端展示。
///
/// ## 缓存（V03-INSTALLED-APPS-CACHE）
/// 遍历 + 逐个 NSWorkspace 取 icon + 转 PNG 成本高，故缓存优先：
/// - 读 `installed_apps_cache` + 当前 `app_snapshot`（`.app` 包名快照）比对：
///   - 缓存存在 **且** 快照一致（未装/卸 .app）→ 直接返回缓存（不遍历、不取 icon）。
///   - 缓存缺失 **或** 快照变化（装/卸 .app）→ 重新遍历 + 取 icon + 更新缓存 + 更新快照。
///
/// ## icon 尺寸
/// icon 为单尺寸 PNG（约 128px，够列表缩略图），经 `installed_apps_cache`
/// 持久化，避免每次重复计算，控制缓存体积。
pub fn list_installed_apps() -> Vec<InstalledAppInfo> {
    // 读缓存 + 当前快照。
    let cached = super::settings::get_installed_apps_cache()
        .ok()
        .flatten();
    let prev_snapshot = super::settings::get_app_snapshot().unwrap_or_default();
    let current_snapshot = derive_app_snapshot();

    // 缓存存在 且 快照一致 → 直接返回缓存（秒回，不遍历、不取 icon）。
    if cached.is_some() && !should_rescan_from_snapshot(&prev_snapshot, &current_snapshot) {
        return cached.expect("已判 Some");
    }

    // 缓存缺失 或 快照变化 → 重新计算 + 更新缓存 + 更新快照。
    let mut apps: Vec<InstalledAppInfo> = discover_app_candidates()
        .into_iter()
        .map(|path| installed_app_info(&path))
        .collect();

    // 按名称排序（不区分大小写），方便前端展示。
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let _ = super::settings::set_installed_apps_cache(apps.clone());
    let _ = super::settings::set_app_snapshot(current_snapshot);
    apps
}

/// 构造单个 `.app` 的 `InstalledAppInfo`。
fn installed_app_info(app_path: &Path) -> InstalledAppInfo {
    let name = app_name_from_path(app_path);

    // bundle_id：读 Info.plist CFBundleIdentifier（可空）。
    let bundle_id = std::fs::read_to_string(app_path.join("Contents/Info.plist"))
        .ok()
        .map(|raw| parse_info_plist(&raw).bundle_identifier)
        .flatten();

    // has_product_json：是否存在 product.json（VS Code Fork 判断用）。
    let has_product_json = product_json_path(app_path).is_file();

    InstalledAppInfo {
        name,
        path: app_path.display().to_string(),
        bundle_id,
        has_product_json,
        icon_base64: extract_app_icon_base64(app_path),
    }
}

/// 手动导入一个自定义编辑器（校验 + 复用识别逻辑 + 构造 `AvailableEditor`）。
///
/// - 路径非 `.app` 目录 → `Err`。
/// - 优先复用 `detect_app`（product.json 指纹 / Info.plist 兜底），若识别到
///   则用其构造；若被 L3 排除（非编辑器）则用 bundleId 或 app 名构造一个
///   Native + OpenA 兜底项（用户手动导入，不因识别逻辑而拒绝）。
/// - 返回构造的 `AvailableEditor`（由 command 层写入 custom_editors）。
///
/// ## source 统一（V03-CUSTOM-SOURCE）
/// 无论命中哪条路径（Fork / AI 编辑器 / 兜底），构造结果一律
/// `source = EditorSource::Custom`，标记「用户手动导入」，与自动检测项
/// （`Discovered`）区分。前端据此持久区分手动项，重启不丢失。
pub fn import_custom_app(app_path: &Path) -> Result<AvailableEditor, String> {
    let app_path = app_path.canonicalize().map_err(|_| {
        format!("应用不存在: {}", app_path.display())
    })?;

    if !app_path.is_dir() {
        return Err(format!("应用不存在: {}", app_path.display()));
    }

    // 优先复用识别逻辑（L1 Fork / L2 AI 编辑器）。
    if let Some(det) = detect_app(&app_path) {
        let mut editor = to_available_editor(det);
        // 用户手动导入 → source 固定为 Custom（覆盖识别路径的 Discovered）。
        editor.source = EditorSource::Custom;
        return Ok(editor);
    }

    // 识别逻辑排除（如 OpenCode/Manus/WorkBuddy 等 Info.plist 不被识别为编辑器）：
    // 手动导入兜底——用 bundleId 或 app 名构造 Native + OpenA 项。
    let app_name = app_name_from_path(&app_path);
    let bundle_id = std::fs::read_to_string(app_path.join("Contents/Info.plist"))
        .ok()
        .map(|raw| parse_info_plist(&raw).bundle_identifier)
        .flatten();

    let id = bundle_id.clone().unwrap_or_else(|| app_name.clone());

    let mut editor = AvailableEditor {
        id,
        name: app_name,
        cli_command: None,
        app_path: Some(app_path.display().to_string()),
        icon_base64: None,
        open_method: OpenMethod::OpenA,
        source: EditorSource::Custom,
        category: EditorCategory::Native,
    };
    // 手动导入兜底项：app_path 是真实 .app → 填充 512px 大图标。
    fill_editor_icon(&mut editor);
    Ok(editor)
}

/// 手动导入并持久化一个自定义编辑器（构造 + 幂等写入 custom_editors）。
///
/// - 构造：复用 `import_custom_app`（校验路径 + 识别逻辑 + 兜底）。
/// - 幂等：若 `custom_editors` 已有同 id 项 → 返回已有项，不重复追加、不报错。
/// - 否则写入 `custom_editors`（读改写保留其他字段）。
pub fn import_and_persist_custom_app(app_path: &Path) -> Result<AvailableEditor, String> {
    let editor = import_custom_app(app_path)?;

    let mut custom = super::settings::get_custom_editors().map_err(|e| e.to_string())?;
    if let Some(existing) = custom.iter().find(|e| e.id == editor.id) {
        return Ok(existing.clone());
    }
    custom.push(editor.clone());
    super::settings::set_custom_editors(custom).map_err(|e| e.to_string())?;
    Ok(editor)
}

/// 提取 `.app` 图标为指定尺寸 PNG 的 base64（可空）。
///
/// ## 尺寸参数化（V03-EDITOR-ICON-LARGE）
/// - `size`：目标图标边长像素（128px 供 Settings 手动导入缩略图，256px 供
///   Welcome 编辑器选择器大图标）。
/// - 提取后先把 NSImage 缩放到 `size × size`，再编码为 PNG。
///
/// ## 技术路径（macOS，NSWorkspace 原生）
/// 用 `NSWorkspace.sharedWorkspace().iconForFile(app_path)` 获取系统图标
/// （覆盖所有 app，不读具体 icns 文件），`setSize:` 缩放到目标尺寸，
/// 经 `NSBitmapImageRep` 转 PNG data，再 base64 编码。任何一步失败返回 `None`
/// （优雅降级，前端显示占位图），不 panic、不影响整个列表。
///
/// 不读 `.icns` 文件、不调用子进程（修复 sips 在 wpsoffice / Chrome Apps 等
/// `.icns` 转换失败的问题）。
///
/// ## 跨平台
/// `#[cfg(target_os = "macos")]` 隔离：仅 macOS 编译（依赖 objc2 系，已在
/// Cargo.toml target-specific 声明，版本与 tauri 传递依赖一致）。
/// Windows / Linux 回退 `None`（与既有 `is_executable` 的 unix/windows 分支一致）。
pub fn extract_icon_base64(app_path: &Path, size: u32) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use std::os::raw::c_void;

        use objc2::rc::Retained;
        use objc2::runtime::AnyObject;
        use objc2::{class, msg_send};
        use objc2_foundation::{NSSize, NSString};

        let path_str = NSString::from_str(&app_path.display().to_string());

        unsafe {
            // NSWorkspace.sharedWorkspace → 系统图标服务
            let workspace: Option<Retained<AnyObject>> =
                msg_send![class!(NSWorkspace), sharedWorkspace];
            let workspace = workspace?;
            // iconForFile: → NSImage
            let icon: Option<Retained<AnyObject>> =
                msg_send![&workspace, iconForFile: &*path_str];
            let icon = icon?;
            // 缩放到目标尺寸（setSize:）
            let size = NSSize {
                width: f64::from(size),
                height: f64::from(size),
            };
            let _: () = msg_send![&icon, setSize: size];
            // TIFFRepresentation → NSData
            let tiff: Option<Retained<AnyObject>> = msg_send![&icon, TIFFRepresentation];
            let tiff = tiff?;
            // NSBitmapImageRep.imageRepWithData: → 位图
            let bitmap: Option<Retained<AnyObject>> =
                msg_send![class!(NSBitmapImageRep), imageRepWithData: &*tiff];
            let bitmap = bitmap?;
            // representationUsingType:properties:（PNG = 4）
            let png: Option<Retained<AnyObject>> = msg_send![
                &bitmap,
                representationUsingType: 4,
                properties: std::ptr::null::<AnyObject>()
            ];
            let png = png?;
            let bytes: *const c_void = msg_send![&png, bytes];
            let len: usize = msg_send![&png, length];
            if bytes.is_null() || len == 0 {
                return None;
            }
            let slice = std::slice::from_raw_parts(bytes as *const u8, len);
            Some(base64_encode(slice))
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app_path, size);
        None
    }
}

/// 提取 `.app` 图标为 128px PNG 的 base64（可空）。
///
/// 兼容包装：Settings 手动导入缩略图用 128px，等价
/// `extract_icon_base64(app_path, 128)`。
pub fn extract_app_icon_base64(app_path: &Path) -> Option<String> {
    extract_icon_base64(app_path, 128)
}

/// 从「可执行文件路径 / .app 目录」向上定位 `.app` 包目录。
///
/// - 白名单编辑器（如 vscode）的 `app_path` 指向 `.app/Contents/Resources/app/bin/code`，
///   需向上找到 `.app` 包才能取图标；
/// - vim/nvim 等纯 CLI 的 `app_path` 是 PATH 中的二进制（`/usr/bin/vim`），
///   不在任何 `.app` 内 → 返回 `None`（无图标）。
pub fn locate_app_dir(path: &Path) -> Option<PathBuf> {
    let mut cur = path.to_path_buf();
    loop {
        if cur.extension().and_then(|e| e.to_str()) == Some("app") && cur.is_dir() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// 为编辑器填充 512px 大图标（仅当其 `app_path` 指向真实 `.app`）。
///
/// - `app_path` 指向 `.app`（直接或经由 `.app` 内 bin 路径）→ 提取 512px PNG base64；
/// - 无 `app_path` 或非 `.app`（如 vim/nvim 纯 CLI）→ `icon_base64 = None`。
pub fn fill_editor_icon(editor: &mut AvailableEditor) {
    editor.icon_base64 = editor
        .app_path
        .as_ref()
        .and_then(|p| locate_app_dir(Path::new(p)))
        .and_then(|app| extract_icon_base64(&app, 512));
}

/// base64 编码（无额外 crate 的轻量实现，避免引入 base64 依赖）。
fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
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

/// 构建 Info.plist 兜底检测结果（v0.3 L2/L3 口径）。
///
/// - **L2 代码类型自动**：`declares_code`（Info.plist 声明了代码文件类型）→
///   `OpenA`（open -a 打开）+ `category = AiEditor`，自动进列表。
/// - **L3 排除**：仅 public.folder 或仅 bundleId、**无代码文件类型** →
///   一律返回 `None`，不产生 Unsupported，不进列表、不进手动候选。
///   （IINA / 浏览器 / 办公软件等被排除。）
///
/// `public.folder` 不再单独作为编辑器判定依据；`bundleId` 仅用于生成稳定 id。
fn build_infoplist_fallback(
    app_path: &Path,
    app_name: &str,
    info: &InfoPlistInfo,
) -> Option<AppDetection> {
    // L3 排除：无代码文件类型 → 不是编辑器，一律排除。
    if !info.declares_code_types() {
        return None;
    }

    let display_name = info.bundle_name.clone().unwrap_or_else(|| app_name.to_string());
    let id = info
        .bundle_identifier
        .clone()
        .unwrap_or_else(|| app_name.to_string());

    Some(AppDetection {
        id,
        name: display_name,
        cli_command: None,
        app_path: app_path.display().to_string(),
        open_method: OpenMethod::OpenA,
        category: EditorCategory::AiEditor,
    })
}

/// 将 `AppDetection` 转为 `AvailableEditor`。
///
/// 动态发现的 `app_path` 都是真实 `.app`，故填充 256px 大图标
/// （V03-EDITOR-ICON-LARGE）。
fn to_available_editor(det: AppDetection) -> AvailableEditor {
    let mut editor = AvailableEditor {
        id: det.id,
        name: det.name,
        cli_command: det.cli_command,
        app_path: Some(det.app_path),
        icon_base64: None,
        open_method: det.open_method,
        source: EditorSource::Discovered,
        category: det.category,
    };
    fill_editor_icon(&mut editor);
    editor
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
            // 防御性过滤：Unsupported 一律不进列表（v0.3 误判治理）。
            // 正常流程下 detect_app 已不产生 Unsupported（L3 排除），此行为保险。
            if det.open_method == OpenMethod::Unsupported {
                continue;
            }
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

    /// L2：Info.plist 兜底——声明 ≥2 个专属代码扩展名（py + rs + swift）→
    /// OpenA + AiEditor，自动进列表。
    #[test]
    fn detect_app_l2_code_type_open_a_ai_editor() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>com.openai.codex</string>
            <key>CFBundleName</key><string>Codex</string>
            <key>CFBundleDocumentTypes</key><array><dict>
                <key>CFBundleTypeRole</key><string>Editor</string>
                <key>CFBundleTypeExtensions</key><array><string>py</string><string>rs</string><string>swift</string></array>
            </dict></array>
        </dict></plist>"#;
        let app = fake_app("Codex", None, Some(plist));
        let det = detect_app(&app).expect("应识别为 L2 AiEditor + open-a");
        assert_eq!(det.id, "com.openai.codex");
        assert_eq!(det.name, "Codex");
        assert_eq!(det.open_method, OpenMethod::OpenA);
        assert_eq!(det.category, EditorCategory::AiEditor);
        assert!(det.cli_command.is_none());
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// L3 回归修复：Safari/Chrome 类——Viewer 角色 + 通用文档类型
    /// （css/html/js，已从清单移除）→ 被排除（None），不再误判为编辑器。
    #[test]
    fn detect_app_l3_safari_like_viewer_excluded() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>com.apple.Safari</string>
            <key>CFBundleName</key><string>Safari</string>
            <key>CFBundleDocumentTypes</key><array><dict>
                <key>CFBundleTypeRole</key><string>Viewer</string>
                <key>CFBundleTypeExtensions</key><array><string>html</string><string>css</string><string>js</string><string>txt</string></array>
            </dict></array>
        </dict></plist>"#;
        let app = fake_app("Safari", None, Some(plist));
        assert!(
            detect_app(&app).is_none(),
            "Safari 类（Viewer + 通用文档类型）应被 L3 排除"
        );
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// L3：仅声明 public.folder（无代码类型）→ 排除（None），不再产生 OpenA。
    #[test]
    fn detect_app_l3_folder_only_excluded() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>com.example.folderapp</string>
            <key>CFBundleName</key><string>FolderApp</string>
            <key>CFBundleDocumentTypes</key><array><dict>
                <key>LSItemContentTypes</key><array><string>public.folder</string></array>
            </dict></array>
        </dict></plist>"#;
        let app = fake_app("FolderApp", None, Some(plist));
        assert!(detect_app(&app).is_none(), "仅 public.folder 应被 L3 排除");
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// L3：仅 bundleId、无代码类型 → 排除（None），不产生 Unsupported。
    #[test]
    fn detect_app_l3_bundle_id_only_excluded() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>com.example.chat</string>
            <key>CFBundleName</key><string>ChatApp</string>
        </dict></plist>"#;
        let app = fake_app("ChatApp", None, Some(plist));
        assert!(detect_app(&app).is_none(), "仅 bundleId 应被 L3 排除，不再返回 Unsupported");
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// 代码类型判定（v0.3 回归修复）：
    /// - 单专属扩展名（无 Editor 角色）→ false
    /// - ≥2 个不同专属扩展名 → true
    /// - 1 个专属扩展名 + Editor 角色 → true
    /// - 通用文档类型（css/html/js/vue）→ false（已从清单移除）
    /// - 大小写不敏感
    /// - 非代码扩展名 / 无 CFBundleTypeExtensions → false
    #[test]
    fn declares_code_types_matches_code_extensions() {
        // ≥2 个专属扩展名 → true
        assert!(parse_info_plist(r#"<plist><dict><key>CFBundleTypeExtensions</key><array><string>rs</string><string>go</string></array></dict></plist>"#).declares_code_types(), "≥2 专属扩展名应判定");
        // 大小写不敏感（PY + RS）
        assert!(parse_info_plist(r#"<plist><dict><key>CFBundleTypeExtensions</key><array><string>PY</string><string>RS</string></array></dict></plist>"#).declares_code_types(), "应大小写不敏感");
        // 1 个专属扩展名 + Editor 角色 → true
        assert!(parse_info_plist(r#"<plist><dict><key>CFBundleDocumentTypes</key><array><dict><key>CFBundleTypeRole</key><string>Editor</string><key>CFBundleTypeExtensions</key><array><string>rs</string></array></dict></array></dict></plist>"#).declares_code_types(), "Editor 角色 + 单专属扩展名应判定");

        // 单专属扩展名（无 Editor 角色）→ false
        assert!(!parse_info_plist(r#"<plist><dict><key>CFBundleTypeExtensions</key><array><string>rs</string></array></dict></plist>"#).declares_code_types(), "单扩展名不应判定");
        // 通用文档类型（css/html/js/vue 已移除）→ false
        assert!(!parse_info_plist(r#"<plist><dict><key>CFBundleTypeExtensions</key><array><string>html</string></array></dict></plist>"#).declares_code_types(), "html 不应判定");
        assert!(!parse_info_plist(r#"<plist><dict><key>CFBundleTypeExtensions</key><array><string>css</string><string>js</string></array></dict></plist>"#).declares_code_types(), "css/js 不应判定");
        assert!(!parse_info_plist(r#"<plist><dict><key>CFBundleTypeExtensions</key><array><string>vue</string></array></dict></plist>"#).declares_code_types(), "vue 已移除不应判定");
        // 非代码扩展名 → false
        assert!(!parse_info_plist(r#"<plist><dict><key>CFBundleTypeExtensions</key><array><string>mp4</string></array></dict></plist>"#).declares_code_types());
        // 无 CFBundleTypeExtensions → false
        assert!(!parse_info_plist(r#"<plist><dict><key>CFBundleIdentifier</key><string>com.x</string></dict></plist>"#).declares_code_types());
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

    /// 快照比对：相同 → 不重扫；不同 → 重扫；首次（无快照）→ 重扫。
    #[test]
    fn should_rescan_snapshot_logic() {
        let prev = vec!["A".to_string(), "B".to_string()];
        let same = vec!["A".to_string(), "B".to_string()];
        assert!(!should_rescan_from_snapshot(&prev, &same), "快照相同不重扫");

        // 顺序无关（两者都已排序，但逻辑上相等即可）
        let diff_add = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        assert!(should_rescan_from_snapshot(&prev, &diff_add), "新增 .app 触发重扫");

        let diff_remove = vec!["A".to_string()];
        assert!(should_rescan_from_snapshot(&prev, &diff_remove), "卸载 .app 触发重扫");

        // 首次：上次快照为空 → 重扫
        assert!(should_rescan_from_snapshot(&[], &same), "首次无快照应重扫");
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

    // ---- V03-MANUAL-IMPORT-BACKEND ----

    /// import_custom_app：路径不存在 → Err("应用不存在")。
    #[test]
    fn import_custom_app_missing_path_errors() {
        let r = import_custom_app(std::path::Path::new("/no/such/app_xyz"));
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("应用不存在"));
    }

    /// import_custom_app：路径不是目录 → Err。
    #[test]
    fn import_custom_app_non_dir_errors() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let file = std::env::temp_dir().join(format!(
            "ydevsphere_notdir_{}_{}",
            std::process::id(),
            n
        ));
        std::fs::write(&file, "x").unwrap();
        let r = import_custom_app(&file);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("应用不存在"));
        let _ = std::fs::remove_file(&file);
    }

    /// import_custom_app：product.json Fork → 复用识别逻辑，Cli + VscodeFork + Custom source。
    #[test]
    fn import_custom_app_vscode_fork() {
        let app = fake_app(
            "MyFork",
            Some(r#"{"nameShort":"MyFork","applicationName":"myfork","dataFolderName":".myfork"}"#),
            None,
        );
        let editor = import_custom_app(&app).expect("应导入成功");
        assert_eq!(editor.id, "myfork");
        assert_eq!(editor.open_method, OpenMethod::Cli);
        assert_eq!(editor.category, EditorCategory::VscodeFork);
        assert_eq!(editor.source, EditorSource::Custom, "手动导入 Fork 路径 source 应为 custom");
        assert!(editor.app_path.as_deref().unwrap().ends_with(".app"));
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// import_custom_app：L2 AI 编辑器（detect_app 识别为 AiEditor）→ 复用 + source = Custom。
    #[test]
    fn import_custom_app_ai_editor_custom_source() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>com.example.aieditor</string>
            <key>CFBundleName</key><string>AICode</string>
            <key>CFBundleDocumentTypes</key><array><dict>
                <key>CFBundleTypeRole</key><string>Editor</string>
                <key>CFBundleTypeExtensions</key><array><string>py</string><string>rs</string></array>
            </dict></array>
        </dict></plist>"#;
        let app = fake_app("AICode", None, Some(plist));
        let editor = import_custom_app(&app).expect("应识别为 AI 编辑器并导入");
        assert_eq!(editor.open_method, OpenMethod::OpenA);
        assert_eq!(editor.category, EditorCategory::AiEditor);
        assert_eq!(editor.source, EditorSource::Custom, "手动导入 AI 路径 source 应为 custom");
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// import_custom_app：识别逻辑排除（无代码类型）→ 兜底 Native + OpenA（用 bundleId 作 id）。
    #[test]
    fn import_custom_app_fallback_native_open_a() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>com.example.workbuddy</string>
            <key>CFBundleName</key><string>WorkBuddy</string>
        </dict></plist>"#;
        let app = fake_app("WorkBuddy", None, Some(plist));
        // 无代码类型 → detect_app 排除 → 兜底 Native + OpenA
        let editor = import_custom_app(&app).expect("应兜底导入成功");
        assert_eq!(editor.id, "com.example.workbuddy", "用 bundleId 作 id");
        assert_eq!(editor.open_method, OpenMethod::OpenA);
        assert_eq!(editor.category, EditorCategory::Native);
        assert_eq!(editor.source, EditorSource::Custom, "手动导入兜底路径 source 应为 custom");
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// installed_app_info：字段构造（name / bundle_id / has_product_json）。
    #[test]
    fn installed_app_info_fields() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>com.example.demo</string>
            <key>CFBundleName</key><string>Demo</string>
        </dict></plist>"#;
        let app = fake_app("Demo", Some(r#"{"applicationName":"demo"}"#), Some(plist));
        let info = installed_app_info(&app);
        assert_eq!(info.name, "Demo");
        assert_eq!(info.bundle_id.as_deref(), Some("com.example.demo"));
        assert!(info.has_product_json, "含 product.json 应 true");
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// base64_encode：可逆、含 padding、正确性。
    #[test]
    fn base64_encode_correctness() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    // ---- V03-MANUAL-IMPORT-FIX ----

    /// icon 提取优雅降级：不存在/无效路径不 panic，返回 None 或 Some 均可，
    /// 不影响调用方。验证「取图标失败不 panic、不影响列表」。
    #[test]
    fn extract_icon_fails_gracefully_for_missing_app() {
        // 不存在的路径：应优雅降级（不 panic）。
        let r = extract_app_icon_base64(std::path::Path::new("/no/such/app_xyz"));
        let _ = r; // 返回值是 Option，可为 None（失败降级）或 Some（兜底图标）
    }

    /// installed_app_info：无图标资源时 icon_base64 为可空 Option，不 panic。
    #[test]
    fn installed_app_info_icon_nullable() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!(
            "ydevsphere_icon_null_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        // 构造一个不含图标的空 .app（无 Resources 图标）。
        let app = tmp.join("NoIcon.app");
        std::fs::create_dir_all(app.join("Contents/Resources")).unwrap();
        std::fs::write(app.join("Contents/Info.plist"), "<plist><dict></dict></plist>").unwrap();

        // list_installed_apps 对该 app 不应 panic，icon_base64 为 Option。
        let info = installed_app_info(&app);
        assert_eq!(info.name, "NoIcon");
        let _ = info.icon_base64; // 可空

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ---- V03-INSTALLED-APPS-CACHE：list_installed_apps 缓存优先 ----

    fn temp_settings_path() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let p = std::env::temp_dir().join(format!(
            "ydevsphere_instapps_{}_{}.json",
            std::process::id(),
            n
        ));
        std::env::set_var("YDEVSPHERE_SETTINGS_PATH", &p);
        let _ = std::fs::remove_file(&p);
        p
    }

    /// 构造带标记的缓存项（用唯一 id 标记，验证是否走了缓存分支）。
    fn marker_app() -> InstalledAppInfo {
        InstalledAppInfo {
            name: "__CACHE_MARKER__".to_string(),
            path: "/nonexistent/marker.app".to_string(),
            bundle_id: None,
            has_product_json: false,
            icon_base64: None,
        }
    }

    /// 缓存缺失 → 首次计算并写缓存（返回真实遍历结果，且缓存被写入）。
    #[test]
    fn installed_apps_cache_miss_computes_and_writes() {
        let _guard = super::super::TEST_ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // 清空缓存（默认无）。
        let apps = list_installed_apps();
        // 应返回真实遍历结果（可能为空或非空，但不含 marker）。
        assert!(!apps.iter().any(|a| a.name == "__CACHE_MARKER__"));

        // 缓存应已被写入（get_installed_apps_cache 返回 Some）。
        assert!(
            crate::core::editor::settings::get_installed_apps_cache()
                .expect("读缓存应成功")
                .is_some(),
            "首次计算后应写缓存"
        );

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }

    /// 缓存命中 且 快照一致 → 直接返回缓存（含 marker，不重算）。
    #[test]
    fn installed_apps_cache_hit_returns_cache() {
        let _guard = super::super::TEST_ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // 构造与真实快照一致的 app_snapshot，并写入 marker 缓存。
        let real_snapshot = derive_app_snapshot();
        crate::core::editor::settings::set_app_snapshot(real_snapshot.clone()).unwrap();
        crate::core::editor::settings::set_installed_apps_cache(vec![marker_app()]).unwrap();

        // 快照一致 → 应直接返回缓存（含 marker）。
        let apps = list_installed_apps();
        assert!(
            apps.iter().any(|a| a.name == "__CACHE_MARKER__"),
            "快照一致应命中缓存返回 marker"
        );

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }

    /// 快照变化（装/卸 .app）→ 重新计算（返回真实遍历，不含 marker）。
    #[test]
    fn installed_apps_cache_snapshot_change_rescans() {
        let _guard = super::super::TEST_ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // 写入一个与真实不同的假快照（含假包名），并写 marker 缓存。
        let mut fake_snapshot = derive_app_snapshot();
        fake_snapshot.push("__FAKE_APP__".to_string());
        crate::core::editor::settings::set_app_snapshot(fake_snapshot).unwrap();
        crate::core::editor::settings::set_installed_apps_cache(vec![marker_app()]).unwrap();

        // 快照变化 → 重算，返回真实遍历（不含 marker），且快照被更新。
        let apps = list_installed_apps();
        assert!(
            !apps.iter().any(|a| a.name == "__CACHE_MARKER__"),
            "快照变化应重算，不含 marker"
        );

        // 快照应更新为真实快照（不再含假包名）。
        let updated = crate::core::editor::settings::get_app_snapshot().unwrap();
        assert!(
            !updated.iter().any(|s| s == "__FAKE_APP__"),
            "快照变化重算后应更新快照，移除假包名"
        );

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }

    // ---- V03-EDITOR-ICON-LARGE：256px 大图标 ----

    /// locate_app_dir：从 `.app` 目录路径本身可直接定位；从 `.app` 内 bin 路径
    /// 可向上定位到 `.app`；纯 CLI 路径（/usr/bin/vim）无 `.app` → None。
    #[test]
    fn locate_app_dir_finds_app_and_rejects_cli() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!(
            "ydevsphere_iconloc_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let app = tmp.join("MyApp.app");
        std::fs::create_dir_all(app.join("Contents")).unwrap();

        // .app 目录本身。
        assert_eq!(locate_app_dir(&app), Some(app.clone()));
        // .app 内 bin 路径 → 向上定位。
        let bin = app.join("Contents/Resources/app/bin/mycmd");
        assert_eq!(locate_app_dir(&bin), Some(app.clone()), "应向上定位到 .app");

        // 纯 CLI（/usr/bin/vim）→ None。
        assert_eq!(locate_app_dir(Path::new("/usr/bin/vim")), None);
        // 不存在的路径 → None。
        assert_eq!(locate_app_dir(Path::new("/no/such/thing.app")), None);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// fill_editor_icon：无 app_path（vim/nvim 纯 CLI）→ icon_base64 = None。
    #[test]
    fn fill_editor_icon_none_without_app_path() {
        let mut e = to_available_editor(AppDetection {
            id: "vim".to_string(),
            name: "Vim".to_string(),
            cli_command: Some("vim".to_string()),
            app_path: "/usr/bin/vim".to_string(),
            open_method: OpenMethod::Cli,
            category: EditorCategory::Native,
        });
        fill_editor_icon(&mut e);
        assert_eq!(e.icon_base64, None, "无 .app 路径（纯 CLI）应无图标");
    }

    /// Fork 编辑器带真实 `.app` app_path → icon 填充（macOS 提取到 512px PNG）。
    ///
    /// macOS 上 `NSWorkspace.iconForFile` 对任意路径返回图标（含通用占位图），
    /// 故 `to_available_editor`（内部已 `fill_editor_icon` → 512px）后 icon_base64
    /// 应为 Some；非 macOS 回退 None。
    #[test]
    fn fork_editor_with_app_path_gets_icon() {
        let app = fake_app(
            "IconFork",
            Some(r#"{"nameShort":"IconFork","applicationName":"iconfork","dataFolderName":".iconfork"}"#),
            None,
        );
        let det = detect_app(&app).expect("应识别为 Fork");
        let editor = to_available_editor(det);
        assert!(editor.app_path.is_some(), "Fork 编辑器应有 app_path");
        #[cfg(target_os = "macos")]
        assert!(
            editor.icon_base64.is_some(),
            "macOS 下 Fork 编辑器带 .app 应填充 512px 图标"
        );
        #[cfg(not(target_os = "macos"))]
        assert_eq!(editor.icon_base64, None, "非 macOS 无图标提取");
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// icon 提取失败降级：参数化版本对不存在路径不 panic、优雅降级（macOS
    /// 上 iconForFile 对任意路径返回占位图标 → 可能 Some；失败路径返回 None）。
    /// 用候选编辑器尺寸（512px）验证。
    #[test]
    fn extract_icon_fails_gracefully_for_missing_path() {
        // 重点是调用不 panic；返回值 Option（None 或 macOS 占位图标）。
        let r = extract_icon_base64(Path::new("/no/such/app_xyz.app"), 512);
        let _ = r;
    }

    /// 候选编辑器大图标走 512px：`fill_editor_icon` 以 512px 提取。
    ///
    /// 间接验证——`fill_editor_icon` 内对 .app 调 `extract_icon_base64(app, 512)`
    /// （512px）。macOS 下结果应为 Some（可编码 PNG）。
    #[test]
    fn candidate_editor_icon_uses_512px() {
        let app = fake_app(
            "Icon512",
            Some(r#"{"nameShort":"Icon512","applicationName":"icon512","dataFolderName":".icon512"}"#),
            None,
        );
        let det = detect_app(&app).expect("应识别为 Fork");
        let mut editor = to_available_editor(det);
        // 再次 fill（幂等），确认候选编辑器路径走 512px。
        fill_editor_icon(&mut editor);
        #[cfg(target_os = "macos")]
        assert!(editor.icon_base64.is_some(), "macOS 下候选编辑器应提取到 512px 图标");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(editor.icon_base64, None);
        let _ = std::fs::remove_dir_all(app.parent().unwrap());
    }

    /// 手动导入面板仍 128px：`extract_app_icon_base64` 等价 `extract_icon_base64(path, 128)`。
    ///
    /// 覆盖 InstalledAppInfo（Settings 手动导入缩略图）不被 512px 改动影响。
    #[test]
    fn manual_import_panel_keeps_128px() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!(
            "ydevsphere_icon128_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let app = tmp.join("PanelApp.app");
        std::fs::create_dir_all(app.join("Contents/Resources")).unwrap();
        std::fs::write(app.join("Contents/Info.plist"), "<plist><dict></dict></plist>").unwrap();

        // 手动导入面板（128px）与参数化 128px 提取一致。
        let panel = extract_app_icon_base64(&app);
        let explicit128 = extract_icon_base64(&app, 128);
        assert_eq!(panel, explicit128, "extract_app_icon_base64 应为 128px");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
