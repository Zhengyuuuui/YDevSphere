//! 应用本地设置（持久化到 `~/.ydevsphere/settings.json`）。
//!
//! 写的是应用自身的配置目录（`~/.ydevsphere/`），非用户项目文件；
//! 默认 Read Only 红线仅约束用户项目目录，此处允许写入应用配置。
//!
//! 管理两类偏好：
//! - 默认编辑器（`default_editor`）
//! - 最近工作区路径（`workspace_path`，解决「每次启动都要重新选工作区」）
//!
//! 所有写入都采用「读改写」（read-modify-save），保证两个字段**互不覆盖**。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::database::connection::app_data_dir;
use crate::core::models::{AvailableEditor, InstalledAppInfo};

use super::detect::is_available_editor;

/// 设置文件路径：`~/.ydevsphere/settings.json`。
///
/// 支持 `YDEVSPHERE_SETTINGS_PATH` 环境变量覆盖（供测试隔离，不污染真实配置）。
fn settings_path() -> PathBuf {
    if let Some(p) = std::env::var_os("YDEVSPHERE_SETTINGS_PATH") {
        return PathBuf::from(p);
    }
    app_data_dir().join("settings.json")
}

/// 当前编辑器识别逻辑版本（v0.3 缓存失效版本化）。
///
/// 每次修改识别逻辑（如收紧扩展名清单、强化判定信号）时应 `+1`：
/// `editor_cache` 记录的版本与它不匹配 → 缓存失效，清缓存重扫。
/// 本次从 0 → 1（收紧 CODE_FILE_EXTENSIONS + ≥2 扩展名 / Editor 角色判定）。
pub const EDITOR_LOGIC_VERSION: i32 = 1;

/// 应用本地设置结构。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSettings {
    /// 默认编辑器 id。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_editor: Option<String>,
    /// 最近一次选择/保存的工作区路径。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
    /// 用户自定义忽略的目录名列表（v0.2 Scanner 迭代）。
    ///
    /// 叠加在 scanner 预设忽略规则（node_modules / .git / target 等）之上；
    /// 空列表时序列化省略，向后兼容旧 settings.json。
    /// `#[serde(default)]`：旧 settings.json 无此字段时回退空列表（不报错）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignore_dirs: Vec<String>,
    /// 工作区路径集合（v0.2 多工作区模型的权威源）。
    ///
    /// 可同时容纳 Documents + Desktop + 手动选择的目录。旧 settings.json
    /// 无此字段时回退空列表（`#[serde(default)]`）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspaces: Vec<String>,
    /// 界面语言偏好（如 "zh-CN" / "en-US"）。
    ///
    /// `#[serde(skip_serializing_if = "Option::is_none")]`：旧 settings.json
    /// 无此字段时回退 `None`（向后兼容，不报错）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// 编辑器动态发现的缓存结果（v0.2 V02-EDITOR-DISCOVER）。
    ///
    /// 首次启动自动扫描后写入；`rescan_editors` 清空重扫。旧 settings.json
    /// 无此字段时回退 `None`（向后兼容）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor_cache: Option<Vec<AvailableEditor>>,
    /// 用户手动确认导入的自定义编辑器（v0.3 V02-EDITOR-FIX 误判治理）。
    ///
    /// 用户在「手动候选列表」中确认导入的非 Fork 编辑器写入此集合；
    /// `list_editors` / `list_app_candidates` 返回时与自动检测结果合并。
    /// 旧 settings.json 无此字段时回退空列表（`#[serde(default)]`）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_editors: Vec<AvailableEditor>,
    /// 应用目录快照（v0.3 启动快照重扫）。
    ///
    /// 只存 `/Applications` + `~/Applications` 下的 `.app` 包名（去重排序后字符串列表），
    /// 用于启动时比对磁盘变化、判断是否需重扫编辑器。不写数据库、不常驻内存。
    /// 旧 settings.json 无此字段时回退空列表（`#[serde(default)]`）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub app_snapshot: Vec<String>,
    /// 编辑器识别逻辑版本（v0.3 缓存失效版本化）。
    ///
    /// 记录生成 `editor_cache` 时的识别逻辑版本；与 `EDITOR_LOGIC_VERSION`
    /// 不匹配时缓存视为失效（旧逻辑产物，如误判 Safari），需清缓存重扫。
    /// 旧 settings.json 无此字段时回退 0（`#[serde(default)]`），与当前版本
    /// 不匹配即触发失效重扫。
    #[serde(default)]
    pub editor_cache_version: i32,
    /// 已安装应用列表缓存（V03-INSTALLED-APPS-CACHE）。
    ///
    /// 存 `list_installed_apps` 算好的结果（含 icon_base64）。`app_snapshot`
    /// 未变化（未装/卸 .app）时直接复用，避免每次遍历 + 取 icon 的昂贵开销。
    /// 旧 settings.json 无此字段时回退 `None`（触发首次计算）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installed_apps_cache: Option<Vec<InstalledAppInfo>>,
}

/// 设置操作错误。
#[derive(Debug)]
pub enum SettingsError {
    /// 非法 / 不在白名单的编辑器 id。
    InvalidEditor(String),
    /// 文件系统读写失败。
    Io(std::io::Error),
    /// 解析已有 settings.json 失败（当作无设置处理，不阻断）。
    Malformed,
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::InvalidEditor(id) => write!(f, "未知编辑器: {id}"),
            SettingsError::Io(e) => write!(f, "设置读写失败: {e}"),
            SettingsError::Malformed => write!(f, "settings.json 解析失败"),
        }
    }
}

impl std::error::Error for SettingsError {}

/// 读取完整设置；文件不存在 / 损坏时返回默认值（`None` 字段，不阻断）。
fn read_settings() -> Result<AppSettings, SettingsError> {
    let path = settings_path();
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(AppSettings::default()),
        Err(e) => return Err(SettingsError::Io(e)),
    };
    match serde_json::from_str(&raw) {
        Ok(s) => Ok(s),
        Err(_) => Ok(AppSettings::default()), // 损坏文件当作无设置
    }
}

/// 写回完整设置（确保目录存在）。
fn write_settings(settings: &AppSettings) -> Result<(), SettingsError> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(SettingsError::Io)?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| SettingsError::Io(std::io::Error::other(e)))?;
    std::fs::write(&path, json).map_err(SettingsError::Io)?;
    Ok(())
}

/// 一次性重置本地应用状态（V03-RESET-BACKEND）。
///
/// 将 `settings.json` 重置为 `AppSettings::default()` 并写回，清空全部字段：
/// `custom_editors` / `workspaces` / `workspace_path` / `default_editor` /
/// `editor_cache` / `editor_cache_version` / `installed_apps_cache` /
/// `ignore_dirs` / `language`（及 `app_snapshot`），回到全新状态。
///
/// ## 保留数据库（方案 A）
/// 只清 settings，**不删** projects / scan_history 表——项目索引保留，
/// 用户重新导入工作区后扫描自动 upsert + 清理 missing 对齐。
///
/// 写失败（目录不可写等）返回 `Err`，不部分写入。
pub fn reset_settings() -> Result<(), SettingsError> {
    write_settings(&AppSettings::default())
}

/// 读取默认编辑器 id；未设置 / 文件不存在返回 `Ok(None)`。
pub fn get_editor_preference() -> Result<Option<String>, SettingsError> {
    Ok(read_settings()?.default_editor)
}

/// 设置默认编辑器 id（白名单 + 动态发现校验通过才写入）。
///
/// v0.2：校验范围从「白名单」扩展到「白名单 + 动态发现」，使动态发现的
/// 编辑器（如 Trae / Qoder / ChatGPT 等）也能设为默认编辑器。
///
/// 采用读改写：保留已存在的 `workspace_path`，不互相覆盖。
pub fn set_editor_preference(editor_id: &str) -> Result<(), SettingsError> {
    if !is_available_editor(editor_id) {
        return Err(SettingsError::InvalidEditor(editor_id.to_string()));
    }

    let mut settings = read_settings()?;
    settings.default_editor = Some(editor_id.to_string());
    write_settings(&settings)
}

/// 读取最近保存的工作区路径；未设置 / 文件不存在返回 `Ok(None)`。
pub fn get_workspace_preference() -> Result<Option<String>, SettingsError> {
    Ok(read_settings()?.workspace_path)
}

/// 保存工作区路径（单值接口，向后兼容）。
///
/// - `path` 非空时写入；
/// - `path` 为空串 / 空白时**清除**工作区偏好（`None`），行为明确。
///
/// 采用读改写：保留 `default_editor` / `ignore_dirs`，不互相覆盖。
///
/// v0.2 集合同步：写入单值的同时同步 `workspaces` 集合——
/// - `path` 非空且不在集合中 → 加入集合（前端旧单值调用不丢集合）。
/// - `path` 为空 → 仅清 `workspace_path`，**不清空集合**（集合是权威源，单值字段只是冗余镜像）。
pub fn set_workspace_preference(path: &str) -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    if path.trim().is_empty() {
        settings.workspace_path = None;
    } else {
        let trimmed = path.trim().to_string();
        settings.workspace_path = Some(trimmed.clone());
        if !settings.workspaces.contains(&trimmed) {
            settings.workspaces.push(trimmed);
        }
    }
    write_settings(&settings)
}

/// 读取工作区路径集合（v0.2 多工作区模型的权威源）。
///
/// 兼容迁移：若集合为空但 `workspace_path`（单值）有值，返回 `[workspace_path]`，
/// 使旧数据（仅单值）能无缝升级。该函数**只读**，不做写入。
pub fn get_workspaces() -> Result<Vec<String>, SettingsError> {
    let settings = read_settings()?;
    if settings.workspaces.is_empty() {
        if let Some(single) = settings.workspace_path {
            return Ok(vec![single]);
        }
        return Ok(Vec::new());
    }
    Ok(settings.workspaces)
}

/// 设置工作区路径集合（整表替换，去重 + 去空白项）。
///
/// 采用读改写：保留 `default_editor` / `ignore_dirs`，不互相覆盖。
/// 同时将 `workspace_path`（单值）镜像为集合首项（`list[0]`），保持单值字段不脱节。
pub fn set_workspaces(dirs: &[String]) -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    let cleaned = clean_dirs(dirs);
    settings.workspaces = cleaned.clone();
    settings.workspace_path = cleaned.first().cloned();
    write_settings(&settings)
}

/// 读取用户自定义忽略目录列表；未设置 / 文件不存在返回空列表。
pub fn get_ignore_dirs() -> Result<Vec<String>, SettingsError> {
    Ok(read_settings()?.ignore_dirs)
}

/// 设置用户自定义忽略目录列表（整表替换，去重 + 去空白项）。
///
/// 采用读改写：保留 `default_editor` / `workspace_path` / `workspaces`，不互相覆盖。
pub fn set_ignore_dirs(dirs: &[String]) -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    settings.ignore_dirs = clean_dirs(dirs);
    write_settings(&settings)
}

/// 读取界面语言偏好；未设置 / 文件不存在返回 `Ok(None)`。
pub fn get_language_preference() -> Result<Option<String>, SettingsError> {
    Ok(read_settings()?.language)
}

/// 设置界面语言偏好。
///
/// - `lng` 非空（trim 后）→ 写入；
/// - `lng` 空串 / 空白 → 清除语言偏好（`None`）。
///
/// 采用读改写：保留 `default_editor` / `workspace_path` / `ignore_dirs` /
/// `workspaces`，不互相覆盖。
pub fn set_language_preference(lng: &str) -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    let trimmed = lng.trim();
    if trimmed.is_empty() {
        settings.language = None;
    } else {
        settings.language = Some(trimmed.to_string());
    }
    write_settings(&settings)
}

/// 读取编辑器发现缓存；未缓存返回 `Ok(None)`。
pub fn get_editor_cache() -> Result<Option<Vec<AvailableEditor>>, SettingsError> {
    Ok(read_settings()?.editor_cache)
}

/// 写入编辑器发现缓存（读改写，保留其他字段）。
pub fn set_editor_cache(cache: Vec<AvailableEditor>) -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    settings.editor_cache = Some(cache);
    write_settings(&settings)
}

/// 清空编辑器发现缓存（读改写，保留其他字段）。
pub fn clear_editor_cache() -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    settings.editor_cache = None;
    write_settings(&settings)
}

/// 读取缓存记录的识别逻辑版本；未记录回退 0（视为旧缓存，触发失效重扫）。
pub fn get_editor_cache_version() -> Result<i32, SettingsError> {
    Ok(read_settings()?.editor_cache_version)
}

/// 写入当前识别逻辑版本到缓存记录（读改写，保留其他字段）。
pub fn set_editor_cache_version(version: i32) -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    settings.editor_cache_version = version;
    write_settings(&settings)
}

/// 读取用户手动确认导入的自定义编辑器；未设置返回空列表。
pub fn get_custom_editors() -> Result<Vec<AvailableEditor>, SettingsError> {
    Ok(read_settings()?.custom_editors)
}

/// 设置自定义编辑器列表（整表替换；读改写保留其他字段）。
///
/// `custom_editors` 是用户手动确认导入的编辑器权威源。
pub fn set_custom_editors(editors: Vec<AvailableEditor>) -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    settings.custom_editors = editors;
    write_settings(&settings)
}

/// 判断某 id 是否为已确认导入的自定义编辑器。
pub fn is_custom_editor(editor_id: &str) -> Result<bool, SettingsError> {
    Ok(read_settings()?
        .custom_editors
        .iter()
        .any(|e| e.id == editor_id))
}

/// 读取上次启动的应用目录快照（`.app` 包名列表）；无快照返回空列表。
pub fn get_app_snapshot() -> Result<Vec<String>, SettingsError> {
    Ok(read_settings()?.app_snapshot)
}

/// 写入应用目录快照（读改写，保留其他字段）。
pub fn set_app_snapshot(snapshot: Vec<String>) -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    settings.app_snapshot = snapshot;
    write_settings(&settings)
}

/// 读取已安装应用列表缓存；无缓存返回 `Ok(None)`。
pub fn get_installed_apps_cache() -> Result<Option<Vec<InstalledAppInfo>>, SettingsError> {
    Ok(read_settings()?.installed_apps_cache)
}

/// 写入已安装应用列表缓存（读改写，保留其他字段）。
pub fn set_installed_apps_cache(
    cache: Vec<InstalledAppInfo>,
) -> Result<(), SettingsError> {
    let mut settings = read_settings()?;
    settings.installed_apps_cache = Some(cache);
    write_settings(&settings)
}

/// 清洗路径/目录列表：去空白项 + 去前后空格 + 去重（保持首次出现顺序）。
fn clean_dirs(dirs: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for d in dirs {
        let trimmed = d.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.clone()) {
            out.push(trimmed);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    /// 串行化：env var 是进程级状态，避免并行测试互相覆盖。
    /// 复用 editor 模块共享锁，与 commands 层测试互斥（防 env 竞态）。
    use super::super::TEST_ENV_LOCK as LOCK;

    /// 每个测试使用独立的临时设置文件路径，避免相互污染。
    fn test_settings_path() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "ydevsphere_settings_test_{}_{}.json",
            std::process::id(),
            n
        ))
    }

    /// 设置临时路径，返回「清理」闭包。
    fn setup() -> PathBuf {
        let path = test_settings_path();
        let _ = std::fs::remove_file(&path);
        std::env::set_var("YDEVSPHERE_SETTINGS_PATH", &path);
        path
    }

    fn teardown(path: &PathBuf) {
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn get_preference_returns_none_when_absent() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        assert_eq!(get_editor_preference().expect("读取应成功"), None);
        assert_eq!(get_workspace_preference().expect("读取应成功"), None);
        teardown(&path);
    }

    #[test]
    fn editor_preference_roundtrip() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        set_editor_preference("vscode").expect("写入应成功");
        assert_eq!(
            get_editor_preference().expect("读取应成功").as_deref(),
            Some("vscode")
        );
        teardown(&path);
    }

    #[test]
    fn set_preference_rejects_unknown_editor() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        let result = set_editor_preference("not-a-real-editor");
        assert!(matches!(result, Err(SettingsError::InvalidEditor(_))));
        teardown(&path);
    }

    #[test]
    fn workspace_preference_roundtrip() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        set_workspace_preference("/Users/me/Projects").expect("写入应成功");
        assert_eq!(
            get_workspace_preference().expect("读取应成功").as_deref(),
            Some("/Users/me/Projects")
        );
        teardown(&path);
    }

    #[test]
    fn set_workspace_clears_on_empty() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        set_workspace_preference("/tmp/ws").expect("写入应成功");
        assert_eq!(
            get_workspace_preference().expect("读取应成功").as_deref(),
            Some("/tmp/ws")
        );

        // 空串清除
        set_workspace_preference("").expect("空串应清除");
        assert_eq!(get_workspace_preference().expect("读取应成功"), None);

        // 空白清除
        set_workspace_preference("/tmp/ws").expect("写入应成功");
        set_workspace_preference("   ").expect("空白应清除");
        assert_eq!(get_workspace_preference().expect("读取应成功"), None);
        teardown(&path);
    }

    #[test]
    fn editor_and_workspace_coexist_without_overwrite() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        // 先设编辑器
        set_editor_preference("cursor").expect("设置编辑器应成功");
        // 再设工作区，不应覆盖编辑器
        set_workspace_preference("/tmp/ws").expect("设置工作区应成功");
        assert_eq!(
            get_editor_preference().expect("读取应成功").as_deref(),
            Some("cursor"),
            "设置工作区不应覆盖编辑器偏好"
        );
        assert_eq!(
            get_workspace_preference().expect("读取应成功").as_deref(),
            Some("/tmp/ws")
        );

        // 反向：改编辑器，工作区应保留
        set_editor_preference("vscode").expect("改编辑器应成功");
        assert_eq!(
            get_workspace_preference().expect("读取应成功").as_deref(),
            Some("/tmp/ws"),
            "改编辑器不应覆盖工作区偏好"
        );
        teardown(&path);
    }

    #[test]
    fn corrupt_file_degrades_gracefully() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        std::fs::write(&path, "{ not valid json ").expect("写损坏文件失败");

        // 损坏文件当作无设置，不报错
        assert_eq!(get_editor_preference().expect("读取应成功"), None);
        assert_eq!(get_workspace_preference().expect("读取应成功"), None);

        // 仍可正常写入（读改写会重建）
        set_workspace_preference("/tmp/ws").expect("写入应成功");
        assert_eq!(
            get_workspace_preference().expect("读取应成功").as_deref(),
            Some("/tmp/ws")
        );
        teardown(&path);
    }

    // ---- v0.2：忽略规则持久化 ----

    #[test]
    fn ignore_dirs_default_empty() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        assert_eq!(get_ignore_dirs().expect("读取应成功"), Vec::<String>::new());
        teardown(&path);
    }

    #[test]
    fn ignore_dirs_roundtrip_dedup_and_trim() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        // 写入：含重复 + 空白 + 前后空格
        set_ignore_dirs(&[
            "build".to_string(),
            "  dist  ".to_string(),
            "build".to_string(),
            "   ".to_string(),
            "".to_string(),
        ])
        .expect("写入应成功");

        // 读回：去重 + 去空白 + 排序
        assert_eq!(
            get_ignore_dirs().expect("读取应成功"),
            vec!["build".to_string(), "dist".to_string()]
        );

        // 覆盖写入（整表替换）
        set_ignore_dirs(&["vendor".to_string()]).expect("写入应成功");
        assert_eq!(
            get_ignore_dirs().expect("读取应成功"),
            vec!["vendor".to_string()]
        );
        teardown(&path);
    }

    #[test]
    fn ignore_dirs_does_not_overwrite_other_settings() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        // 先设编辑器与工作区
        set_editor_preference("vscode").expect("设置编辑器应成功");
        set_workspace_preference("/tmp/ws").expect("设置工作区应成功");

        // 再设忽略规则，不应覆盖前两者
        set_ignore_dirs(&["cache".to_string()]).expect("设置忽略规则应成功");
        assert_eq!(
            get_editor_preference().expect("读取应成功").as_deref(),
            Some("vscode")
        );
        assert_eq!(
            get_workspace_preference().expect("读取应成功").as_deref(),
            Some("/tmp/ws")
        );
        assert_eq!(
            get_ignore_dirs().expect("读取应成功"),
            vec!["cache".to_string()]
        );
        teardown(&path);
    }

    // ---- v0.2：工作区集合（V02-WS-BACKEND） ----

    #[test]
    fn workspaces_default_empty() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        assert_eq!(get_workspaces().expect("读取应成功"), Vec::<String>::new());
        teardown(&path);
    }

    #[test]
    fn workspaces_roundtrip_dedup_and_trim() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        // 写入：Documents + Desktop + 重复 + 空白
        set_workspaces(&[
            "/Users/me/Documents".to_string(),
            "/Users/me/Desktop".to_string(),
            "/Users/me/Documents".to_string(),
            "   ".to_string(),
            "".to_string(),
        ])
        .expect("写入应成功");

        let ws = get_workspaces().expect("读取应成功");
        assert_eq!(
            ws,
            vec!["/Users/me/Documents".to_string(), "/Users/me/Desktop".to_string()],
            "应去重 + 去空白，保持顺序"
        );

        // 单值 workspace_path 应镜像为集合首项
        assert_eq!(
            get_workspace_preference().expect("读取应成功").as_deref(),
            Some("/Users/me/Documents")
        );
        teardown(&path);
    }

    #[test]
    fn set_workspaces_clear_empties_collection_and_single() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_workspaces(&["/tmp/a".to_string(), "/tmp/b".to_string()]).expect("写入应成功");
        assert_eq!(get_workspaces().expect("读取应成功").len(), 2);

        // 清空集合（传空 / 空白项）→ 集合为空，单值也为 None
        set_workspaces(&[]).expect("清空应成功");
        assert_eq!(get_workspaces().expect("读取应成功"), Vec::<String>::new());
        assert_eq!(get_workspace_preference().expect("读取应成功"), None);
        teardown(&path);
    }

    /// 单值 → 集合双向同步：旧单值调用（set_workspace_preference）应同步加入集合。
    #[test]
    fn single_value_syncs_into_collection() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        // 用旧单值接口设置
        set_workspace_preference("/tmp/ws").expect("写入应成功");
        // 集合应包含该值（自动同步）
        assert_eq!(
            get_workspaces().expect("读取应成功"),
            vec!["/tmp/ws".to_string()]
        );

        // 再设置另一个单值 → 加入集合（不覆盖已有集合）
        set_workspace_preference("/tmp/ws2").expect("写入应成功");
        let ws = get_workspaces().expect("读取应成功");
        assert_eq!(ws, vec!["/tmp/ws".to_string(), "/tmp/ws2".to_string()]);

        // 清单值（空串）→ 仅清 workspace_path，不清空集合
        set_workspace_preference("").expect("清除应成功");
        assert_eq!(get_workspace_preference().expect("读取应成功"), None);
        assert_eq!(get_workspaces().expect("读取应成功").len(), 2, "清单值不应清空集合");
        teardown(&path);
    }

    /// 旧 settings.json（仅单值 workspace_path）升级兼容：集合读回该单值。
    #[test]
    fn legacy_single_workspace_migrates_to_collection_read() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        // 手工写一份"旧格式" settings.json（仅 default_editor + workspace_path）
        std::fs::write(
            &path,
            r#"{"default_editor":"vscode","workspace_path":"/Users/me/Projects"}"#,
        )
        .expect("写旧格式失败");

        // 集合读回：workspace_path 值作为单元素集合
        assert_eq!(
            get_workspaces().expect("读取应成功"),
            vec!["/Users/me/Projects".to_string()]
        );
        // 旧字段不丢
        assert_eq!(
            get_editor_preference().expect("读取应成功").as_deref(),
            Some("vscode")
        );
        assert_eq!(
            get_workspace_preference().expect("读取应成功").as_deref(),
            Some("/Users/me/Projects")
        );
        // 忽略规则默认空
        assert_eq!(get_ignore_dirs().expect("读取应成功"), Vec::<String>::new());
        teardown(&path);
    }

    /// 集合接口不覆盖 editor / ignore_dirs（读改写互不覆盖）。
    #[test]
    fn workspaces_does_not_overwrite_editor_and_ignore() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_editor_preference("vscode").expect("设置编辑器应成功");
        set_ignore_dirs(&["cache".to_string()]).expect("设置忽略规则应成功");

        // 设集合
        set_workspaces(&["/tmp/a".to_string(), "/tmp/b".to_string()]).expect("设置集合应成功");

        // editor / ignore_dirs 保留
        assert_eq!(
            get_editor_preference().expect("读取应成功").as_deref(),
            Some("vscode")
        );
        assert_eq!(
            get_ignore_dirs().expect("读取应成功"),
            vec!["cache".to_string()]
        );
        // 集合正确
        assert_eq!(get_workspaces().expect("读取应成功").len(), 2);
        teardown(&path);
    }

    // ---- v0.2：界面语言偏好（V02-I18N-BACKEND） ----

    #[test]
    fn language_preference_default_none() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        assert_eq!(get_language_preference().expect("读取应成功"), None);
        teardown(&path);
    }

    #[test]
    fn language_preference_roundtrip() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_language_preference("zh-CN").expect("写入应成功");
        assert_eq!(
            get_language_preference().expect("读取应成功").as_deref(),
            Some("zh-CN")
        );

        // 覆盖为另一语言
        set_language_preference("en-US").expect("写入应成功");
        assert_eq!(
            get_language_preference().expect("读取应成功").as_deref(),
            Some("en-US")
        );

        // trim 前后空格
        set_language_preference("  ja-JP  ").expect("写入应成功");
        assert_eq!(
            get_language_preference().expect("读取应成功").as_deref(),
            Some("ja-JP")
        );
        teardown(&path);
    }

    #[test]
    fn set_language_clears_on_empty() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_language_preference("zh-CN").expect("写入应成功");
        assert!(get_language_preference().expect("读取应成功").is_some());

        // 空串清除
        set_language_preference("").expect("空串应清除");
        assert_eq!(get_language_preference().expect("读取应成功"), None);

        // 空白清除
        set_language_preference("zh-CN").expect("写入应成功");
        set_language_preference("   ").expect("空白应清除");
        assert_eq!(get_language_preference().expect("读取应成功"), None);
        teardown(&path);
    }

    /// 语言偏好写入不覆盖 default_editor / workspaces / ignore_dirs（读改写互不覆盖）。
    #[test]
    fn language_does_not_overwrite_other_settings() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_editor_preference("vscode").expect("设置编辑器应成功");
        set_ignore_dirs(&["cache".to_string()]).expect("设置忽略规则应成功");
        set_workspaces(&["/tmp/a".to_string(), "/tmp/b".to_string()]).expect("设置集合应成功");

        // 设语言偏好
        set_language_preference("zh-CN").expect("设置语言应成功");

        // 其他字段保留
        assert_eq!(
            get_editor_preference().expect("读取应成功").as_deref(),
            Some("vscode")
        );
        assert_eq!(
            get_ignore_dirs().expect("读取应成功"),
            vec!["cache".to_string()]
        );
        assert_eq!(get_workspaces().expect("读取应成功").len(), 2);
        assert_eq!(
            get_workspace_preference().expect("读取应成功").as_deref(),
            Some("/tmp/a"),
            "workspace_path 应镜像集合首项"
        );
        // 语言正确
        assert_eq!(
            get_language_preference().expect("读取应成功").as_deref(),
            Some("zh-CN")
        );
        teardown(&path);
    }

    // ---- v0.3：自定义编辑器（V02-EDITOR-FIX 误判治理） ----

    fn custom_editor(id: &str, name: &str) -> AvailableEditor {
        AvailableEditor {
            id: id.to_string(),
            name: name.to_string(),
            cli_command: None,
            app_path: Some(format!("/Applications/{name}.app")),
            icon_base64: None,
            open_method: crate::core::models::OpenMethod::OpenA,
            source: crate::core::models::EditorSource::Discovered,
            category: crate::core::models::EditorCategory::AiEditor,
        }
    }

    #[test]
    fn custom_editors_default_empty() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        assert_eq!(get_custom_editors().expect("读取应成功"), Vec::<AvailableEditor>::new());
        assert!(!is_custom_editor("x").expect("读取应成功"));
        teardown(&path);
    }

    #[test]
    fn custom_editors_roundtrip_and_is_custom() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_custom_editors(vec![
            custom_editor("com.example.codex", "Codex"),
            custom_editor("com.example.claude", "Claude"),
        ])
        .expect("写入应成功");

        let list = get_custom_editors().expect("读取应成功");
        assert_eq!(list.len(), 2);
        assert!(is_custom_editor("com.example.codex").expect("读取应成功"));
        assert!(is_custom_editor("com.example.claude").expect("读取应成功"));
        assert!(!is_custom_editor("com.example.other").expect("读取应成功"));

        // 整表替换
        set_custom_editors(vec![custom_editor("com.example.new", "New")]).expect("替换应成功");
        let list2 = get_custom_editors().expect("读取应成功");
        assert_eq!(list2.len(), 1);
        assert_eq!(list2[0].id, "com.example.new");
        teardown(&path);
    }

    #[test]
    fn custom_editors_does_not_overwrite_other_settings() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_editor_preference("vscode").expect("设置编辑器应成功");
        set_language_preference("zh-CN").expect("设置语言应成功");

        // 写 custom_editors
        set_custom_editors(vec![custom_editor("com.example.codex", "Codex")]).expect("写入应成功");

        // 其他字段保留
        assert_eq!(
            get_editor_preference().expect("读取应成功").as_deref(),
            Some("vscode")
        );
        assert_eq!(
            get_language_preference().expect("读取应成功").as_deref(),
            Some("zh-CN")
        );
        assert_eq!(get_custom_editors().expect("读取应成功").len(), 1);
        teardown(&path);
    }

    // ---- v0.3：应用目录快照（启动快照重扫） ----

    #[test]
    fn app_snapshot_default_empty() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        assert_eq!(get_app_snapshot().expect("读取应成功"), Vec::<String>::new());
        teardown(&path);
    }

    #[test]
    fn app_snapshot_roundtrip() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_app_snapshot(vec![
            "Cursor".to_string(),
            "Visual Studio Code".to_string(),
            "Codex".to_string(),
        ])
        .expect("写入应成功");
        assert_eq!(
            get_app_snapshot().expect("读取应成功"),
            vec![
                "Cursor".to_string(),
                "Visual Studio Code".to_string(),
                "Codex".to_string()
            ]
        );

        // 覆盖（磁盘变化后重扫更新快照）
        set_app_snapshot(vec!["Cursor".to_string()]).expect("覆盖应成功");
        assert_eq!(get_app_snapshot().expect("读取应成功"), vec!["Cursor".to_string()]);
        teardown(&path);
    }

    #[test]
    fn app_snapshot_does_not_overwrite_other_settings() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_language_preference("en-US").expect("设置语言应成功");
        set_custom_editors(vec![custom_editor("com.example.codex", "Codex")]).expect("写入应成功");

        // 写快照，不应覆盖其他字段
        set_app_snapshot(vec!["Cursor".to_string()]).expect("写入快照应成功");

        assert_eq!(
            get_language_preference().expect("读取应成功").as_deref(),
            Some("en-US")
        );
        assert_eq!(get_custom_editors().expect("读取应成功").len(), 1);
        assert_eq!(get_app_snapshot().expect("读取应成功"), vec!["Cursor".to_string()]);
        teardown(&path);
    }

    // ---- v0.3 主修 B：custom_editors 作为已知编辑器独立权威源 ----

    /// 已确认的 custom 项在「系统卸载后」（自动检测不再返回）仍可设默认，
    /// 不会报「未知编辑器」。
    #[test]
    fn confirmed_custom_is_available_even_when_uninstalled() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        // 用户已确认导入一个虚构编辑器（其 app 已从 /Applications 卸载，
        // 自动检测不会返回它——用一个绝对不存在的 id 模拟）。
        let confirmed_id = "com.example.uninstalled-editor";
        set_custom_editors(vec![custom_editor(confirmed_id, "Uninstalled")])
            .expect("写入 custom_editors 应成功");

        // is_available_editor：custom 兜底生效 → 即使自动检测找不到也算可用。
        assert!(
            crate::core::editor::detect::is_available_editor(confirmed_id),
            "已确认 custom 项即使系统卸载也应可设默认"
        );

        // find_editor_by_id：同样能从 custom_editors 找回。
        assert!(
            crate::core::editor::detect::find_editor_by_id(confirmed_id).is_some(),
            "已确认 custom 项应能被按 id 找回"
        );
        teardown(&path);
    }

    /// 已确认项重复确认：幂等成功（不重复追加、不报错），
    /// 即使系统已卸载该 app（自动检测找不到）。
    #[test]
    fn confirm_custom_editor_is_idempotent_for_confirmed() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        let confirmed_id = "com.example.uninstalled-editor";
        set_custom_editors(vec![custom_editor(confirmed_id, "Uninstalled")])
            .expect("写入 custom_editors 应成功");

        // 重复确认 → Ok（幂等），不依赖 find_editor_by_id（该 id 不在自动检测）。
        crate::commands::editor::confirm_custom_editor(confirmed_id.to_string())
            .expect("已确认项重复确认应幂等成功");

        // custom_editors 不重复追加。
        assert_eq!(get_custom_editors().expect("读取应成功").len(), 1);
        teardown(&path);
    }

    // ---- v0.3：缓存失效版本化（识别逻辑版本） ----

    #[test]
    fn editor_cache_version_default_zero() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        assert_eq!(get_editor_cache_version().expect("读取应成功"), 0);
        teardown(&path);
    }

    #[test]
    fn editor_cache_version_roundtrip() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_editor_cache_version(EDITOR_LOGIC_VERSION).expect("写入应成功");
        assert_eq!(
            get_editor_cache_version().expect("读取应成功"),
            EDITOR_LOGIC_VERSION
        );

        // 覆盖（识别逻辑再更新时 +1）
        set_editor_cache_version(EDITOR_LOGIC_VERSION + 1).expect("写入应成功");
        assert_eq!(
            get_editor_cache_version().expect("读取应成功"),
            EDITOR_LOGIC_VERSION + 1
        );
        teardown(&path);
    }

    #[test]
    fn editor_cache_version_does_not_overwrite_other_settings() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        set_language_preference("en-US").expect("设置语言应成功");
        set_app_snapshot(vec!["Cursor".to_string()]).expect("写入快照应成功");

        set_editor_cache_version(EDITOR_LOGIC_VERSION).expect("写入版本应成功");

        assert_eq!(
            get_language_preference().expect("读取应成功").as_deref(),
            Some("en-US")
        );
        assert_eq!(get_app_snapshot().expect("读取应成功"), vec!["Cursor".to_string()]);
        assert_eq!(
            get_editor_cache_version().expect("读取应成功"),
            EDITOR_LOGIC_VERSION
        );
        teardown(&path);
    }

    // ---- V03-RESET-BACKEND：重置应用状态 ----

    /// reset_settings：清空全部字段回到默认态。
    #[test]
    fn reset_clears_all_fields() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        // 先写入各类非默认值。
        set_editor_preference("vscode").expect("写入编辑器应成功");
        set_workspace_preference("/tmp/ws").expect("写入工作区应成功");
        set_workspaces(&["/tmp/ws".to_string(), "/tmp/ws2".to_string()]).expect("写入集合应成功");
        set_ignore_dirs(&["cache".to_string()]).expect("写入忽略规则应成功");
        set_language_preference("zh-CN").expect("写入语言应成功");
        set_custom_editors(vec![custom_editor("com.example.codex", "Codex")])
            .expect("写入 custom 应成功");
        set_editor_cache_version(EDITOR_LOGIC_VERSION).expect("写入版本应成功");
        set_app_snapshot(vec!["Cursor".to_string()]).expect("写入快照应成功");

        // reset → 回到默认态。
        reset_settings().expect("reset 应成功");

        assert_eq!(get_editor_preference().expect("读取应成功"), None, "default_editor 清空");
        assert_eq!(get_workspace_preference().expect("读取应成功"), None, "workspace_path 清空");
        assert_eq!(get_workspaces().expect("读取应成功"), Vec::<String>::new(), "workspaces 清空");
        assert_eq!(get_ignore_dirs().expect("读取应成功"), Vec::<String>::new(), "ignore_dirs 清空");
        assert_eq!(get_language_preference().expect("读取应成功"), None, "language 清空");
        assert_eq!(get_custom_editors().expect("读取应成功"), Vec::<AvailableEditor>::new(), "custom_editors 清空");
        assert_eq!(get_editor_cache().expect("读取应成功"), None, "editor_cache 清空");
        assert_eq!(get_editor_cache_version().expect("读取应成功"), 0, "editor_cache_version 归零");
        assert_eq!(get_installed_apps_cache().expect("读取应成功"), None, "installed_apps_cache 清空");
        assert_eq!(get_app_snapshot().expect("读取应成功"), Vec::<String>::new(), "app_snapshot 清空");

        teardown(&path);
    }

    /// reset_settings：幂等——连续 reset 多次结果一致，不报错。
    #[test]
    fn reset_is_idempotent() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();

        // 写入非默认值后再 reset。
        set_editor_preference("cursor").expect("写入应成功");
        set_workspaces(&["/tmp/a".to_string()]).expect("写入应成功");
        set_language_preference("en-US").expect("写入应成功");

        // 连续两次 reset → 都成功，状态仍为默认。
        reset_settings().expect("第一次 reset 应成功");
        reset_settings().expect("第二次 reset 应成功");

        assert_eq!(get_editor_preference().expect("读取应成功"), None);
        assert_eq!(get_workspaces().expect("读取应成功"), Vec::<String>::new());
        assert_eq!(get_language_preference().expect("读取应成功"), None);

        teardown(&path);
    }

    /// reset_settings：文件不存在（全新安装）时也能安全 reset（写成默认态）。
    #[test]
    fn reset_works_when_file_missing() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        // setup 已清空文件 → 等价「文件不存在」。
        reset_settings().expect("reset 应成功");
        assert_eq!(get_editor_preference().expect("读取应成功"), None);
        assert_eq!(get_workspaces().expect("读取应成功"), Vec::<String>::new());
        teardown(&path);
    }

    // ---- v0.3：手动导入幂等写（V03-MANUAL-IMPORT-BACKEND） ----

    /// 构造临时 .app（含 product.json，可被识别为 Fork），返回路径。
    fn temp_fork_app() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!(
            "ydevsphere_import_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let app = tmp.join("MyFork.app");
        std::fs::create_dir_all(app.join("Contents/Resources/app")).unwrap();
        std::fs::write(
            app.join("Contents/Resources/app/product.json"),
            r#"{"nameShort":"MyFork","applicationName":"myfork","dataFolderName":".myfork"}"#,
        )
        .unwrap();
        app
    }

    /// import_and_persist_custom_app：幂等写——同 id 二次导入不重复追加。
    #[test]
    fn import_custom_app_is_idempotent_in_custom_editors() {
        let _guard = LOCK.lock().unwrap();
        let path = setup();
        let app = temp_fork_app();

        // 首次导入 → 写入 custom_editors（1 项）。
        let first = crate::core::editor::discover::import_and_persist_custom_app(&app)
            .expect("首次导入应成功");
        assert_eq!(first.id, "myfork");
        assert_eq!(get_custom_editors().expect("读取应成功").len(), 1);

        // 二次导入同 app → 幂等返回已有项，不重复追加。
        let second = crate::core::editor::discover::import_and_persist_custom_app(&app)
            .expect("二次导入应成功");
        assert_eq!(second.id, "myfork");
        assert_eq!(get_custom_editors().expect("读取应成功").len(), 1, "幂等，不重复追加");

        let _ = std::fs::remove_dir_all(app.parent().unwrap());
        teardown(&path);
    }
}
