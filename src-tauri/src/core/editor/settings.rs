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
use crate::core::models::AvailableEditor;

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
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
}
