//! 编辑器相关命令（薄壳层）。
//!
//! 职责：参数解析 + 调用 `core::editor` 转发。
//! - 编辑器检测 / 打开走 `core::editor`（白名单）。
//! - 文件管理器打开走 `tauri-plugin-opener`（系统默认文件管理器）。
//!
//! ## 安全
//! - 仅执行白名单内已知编辑器（`core::editor` 强制）。
//! - 非法 `editor_id` 直接拒绝，不执行任何进程。

use std::sync::Mutex;

use tauri::State;

use crate::core::database::Database;
use crate::core::editor::discover;
use crate::core::editor::{self, EditorError, SettingsError};
use crate::core::models::{AvailableEditor, EditorSource, InstalledAppInfo, OpenMethod};

/// 列出检测到的可用编辑器（自动检测 + 用户手动导入合并）。
///
/// - 自动检测（`scan_and_cache`）：优先读缓存，无缓存则扫描并写缓存。
/// - **custom_editors 合并**（V03-MANUAL-IMPORT-FIX）：用户手动导入的编辑器
///   实时从 settings.json 读取并合并进返回，按 id 去重（custom 优先）。
///   不依赖 editor_cache——导入成功即出现在下拉框，无需重扫/清缓存。
#[tauri::command]
pub fn list_editors() -> Vec<AvailableEditor> {
    merge_custom(scan_and_cache(false))
}

/// 合并 custom_editors 到自动检测结果（按 id 去重，**custom 优先**）。
///
/// 供 `list_editors` / `list_app_candidates` 复用。custom_editors 实时读
/// settings.json，不受 editor_cache 影响。
///
/// ## custom 优先（V03-MERGE-CUSTOM-PRIORITY）
/// 当某 id 同时存在于自动检测与 custom_editors 时，**用 custom 项覆盖自动检测项**：
/// - 自动检测项若 id 在 custom 中 → 跳过（被 custom 覆盖，不保留自动检测版）。
/// - custom 项全部加入（custom 是权威源）。
///
/// 这保证「用户手动导入过的编辑器 source 稳定为 custom（手动）」，不再随
/// 自动检测能否识别而同 id 跳变（上次手动选入、这次自动导入）。
fn merge_custom(automatic: Vec<AvailableEditor>) -> Vec<AvailableEditor> {
    // custom_editors 实时读；读失败忽略（不影响自动检测结果）。
    let custom = match editor::get_custom_editors() {
        Ok(c) => c,
        Err(_) => return dedup_by_cli(automatic),
    };

    let mut out: Vec<AvailableEditor> = Vec::with_capacity(automatic.len() + custom.len());

    // 1. 自动检测项：仅当 id 不在 custom 中才保留（custom 优先，跳过被覆盖的）。
    for e in automatic {
        if !custom.iter().any(|c| c.id == e.id) {
            out.push(e);
        }
    }

    // 2. custom 项：全部加入（custom 权威源；内部按 id 去重防御）。
    for c in custom {
        if !out.iter().any(|x| x.id == c.id) {
            out.push(c);
        }
    }

    dedup_by_cli(out)
}

/// 同 CLI 命令去重（V03-EDITOR-DEDUP）。
///
/// 按 `cli_command` 的 basename（最后一段）分组，同组只保留一条：
/// - 优先保留 `Whitelist` 来源（更可靠，如 VS Code 白名单 id=vscode），
///   丢弃动态发现重复项（如 applicationName=code 的 VS Code.app）。
/// - 无 CLI 命令的项不去重（直接保留）。
///
/// 解决：设置页下拉框同时出现白名单 vscode（CLI=code）与动态 code 两条。
fn dedup_by_cli(editors: Vec<AvailableEditor>) -> Vec<AvailableEditor> {
    use std::collections::HashMap;
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out: Vec<AvailableEditor> = Vec::with_capacity(editors.len());

    for e in editors {
        // 提取 CLI basename（兼容绝对路径与纯命令名）。
        let key = e
            .cli_command
            .as_deref()
            .map(|c| {
                c.rsplit('/').next().map(String::from).unwrap_or_else(|| c.to_string())
            })
            .filter(|k| !k.is_empty());

        match key {
            Some(k) => match seen.get(&k) {
                Some(&idx) => {
                    // 同 CLI 已存在：若当前是 Whitelist 且已保留的非 Whitelist，
                    // 用白名单覆盖（白名单更可靠）；否则保留现有。
                    if e.source == EditorSource::Whitelist
                        && out[idx].source != EditorSource::Whitelist
                    {
                        out[idx] = e;
                    }
                }
                None => {
                    seen.insert(k, out.len());
                    out.push(e);
                }
            },
            None => out.push(e),
        }
    }
    out
}

/// 重新扫描编辑器（清缓存 → 扫描 → 写缓存）。
#[tauri::command]
pub fn rescan_editors() -> Vec<AvailableEditor> {
    scan_and_cache(true)
}

/// 扫描编辑器并（可选）先清缓存。
///
/// - `force`：是否强制清缓存重扫。
/// - 读缓存优先；命中则直接返回，不触发扫描。
/// 缓存识别逻辑版本是否与当前一致（v0.3 缓存失效版本化判定）。
///
/// 版本不匹配（识别逻辑已更新，旧缓存是旧逻辑产物）→ 缓存失效，需重扫。
/// 读版本失败（未记录）视为失效（`false`）。
fn cache_version_matches() -> bool {
    editor::get_editor_cache_version()
        .map(|v| v == editor::EDITOR_LOGIC_VERSION)
        .unwrap_or(false)
}

fn scan_and_cache(force: bool) -> Vec<AvailableEditor> {
    // 非强制：读缓存
    if !force {
        // v0.3 缓存失效版本化：仅当缓存存在且识别逻辑版本匹配时才复用。
        // 版本不匹配（识别逻辑已更新）→ 缓存视为旧逻辑产物，走重扫。
        let version_ok = cache_version_matches();
        if version_ok {
            if let Ok(Some(cache)) = editor::get_editor_cache() {
                // v0.3 缓存清洗：过滤 unsupported（清旧缓存污染）。
                return filter_usable(cache);
            }
        }
    } else {
        // 强制重扫：清缓存
        let _ = editor::clear_editor_cache();
    }

    // 扫描（白名单 + 动态发现），过滤 unsupported。
    let editors = filter_usable(editor::list_available_editors());
    // 写缓存 + 记录当前识别逻辑版本（失败不阻断返回）。
    let _ = editor::set_editor_cache(editors.clone());
    let _ = editor::set_editor_cache_version(editor::EDITOR_LOGIC_VERSION);
    editors
}

/// 过滤掉 `Unsupported` 的编辑器（缓存清洗 / 结果兜底，v0.3 误判治理）。
///
/// 保证返回列表只含「可打开」的编辑器（`cli` + `open_a`）。
fn filter_usable(editors: Vec<AvailableEditor>) -> Vec<AvailableEditor> {
    editors
        .into_iter()
        .filter(|e| e.open_method != OpenMethod::Unsupported)
        .collect()
}

/// 手动候选列表：自动检测（L1 + L2）+ 用户已确认导入的自定义编辑器，去重合并。
///
/// 返回所有「可打开」的编辑器（`cli` + `open_a`），不含 unsupported。
/// 产品哲学（v0.3）：不替用户猜测，把选择权交给用户——此列表供用户在
/// Welcome / 设置中"选择常用编辑器"或"手动导入"。
#[tauri::command]
pub fn list_app_candidates() -> Vec<AvailableEditor> {
    merge_custom(scan_and_cache(false))
}

/// 确认导入一个自定义编辑器到 `custom_editors`（读改写，去重，幂等）。
///
/// - 该 id **已是** `custom_editors` 成员（用户已确认过）→ 直接成功（幂等，
///   不重复追加、不报错），即使系统已卸载该 app。
/// - 否则在自动检测结果（L1+L2）中查找 → 找到则写入 `custom_editors`。
/// - 自动检测与 custom_editors 均无 → 返回错误「未知编辑器」（不执行打开动作）。
#[tauri::command]
pub fn confirm_custom_editor(editor_id: String) -> Result<(), String> {
    // 已是已确认的 custom 项 → 幂等成功。
    let mut custom = editor::get_custom_editors().map_err(map_settings_err)?;
    if custom.iter().any(|e| e.id == editor_id) {
        return Ok(());
    }

    // 否则从自动检测（白名单 + 动态发现）查找，复用已知编辑器校验。
    let editor = editor::find_editor_by_id(&editor_id)
        .ok_or_else(|| format!("未知编辑器: {editor_id}"))?;

    custom.push(editor);
    editor::set_custom_editors(custom).map_err(map_settings_err)
}

/// 列出 `/Applications` + `~/Applications` 下全部 `.app`（手动导入用，与识别逻辑解耦）。
///
/// 返回 `InstalledAppInfo[]`（name / path / bundle_id / has_product_json / icon_base64），
/// 按名称排序。含 icon_base64（macOS 提取 128px PNG；非 macOS / 失败为 `null`）。
#[tauri::command]
pub fn list_installed_apps() -> Vec<InstalledAppInfo> {
    editor::list_installed_apps()
}

/// 手动导入一个自定义编辑器（写入 custom_editors，幂等）。
///
/// - `app_path` 非存在的 `.app` 目录 → `Err("应用不存在: {app_path}")`。
/// - 优先复用识别逻辑构造（product.json Fork / AI 编辑器）；被识别逻辑排除的
///   常用编辑器（OpenCode/Manus/WorkBuddy 等）用 bundleId 或 app 名兜底为
///   Native + OpenA 项。
/// - 已导入（同 id）→ 幂等返回已有项，不重复追加。
#[tauri::command]
pub fn import_custom_app(app_path: String) -> Result<AvailableEditor, String> {
    editor::import_and_persist_custom_app(std::path::Path::new(&app_path))
}

/// 首次启动自动扫描一次（后台写缓存，不阻塞启动）。
///
/// 由 `lib.rs` 的 setup 钩子调用。v0.3 启动快照重扫：
/// 1. 读取 settings.json 上次 `app_snapshot`。
/// 2. 现场轻量遍历一次 `.app` 包名（只读目录名）得到当前快照。
/// 3. 比对：
///    - 首次（无上次快照）→ 直接扫描 + 写快照。
///    - 快照相同 → 走缓存，不重扫。
///    - 快照变化（新增/卸载 .app）→ 重扫编辑器 + 更新快照。
pub fn scan_editors_once() {
    // v0.3 缓存失效版本化：识别逻辑版本不匹配 → 视为缓存失效，强制重扫。
    // （旧缓存可能是旧识别逻辑产物，如误判 Safari，重启后不应继续使用。）
    let version_ok = cache_version_matches();

    // 读取上次快照（读失败当作无快照，触发首次扫描）。
    let prev = editor::get_app_snapshot().unwrap_or_default();
    // 现场轻量遍历当前快照（只读目录名）。
    let current = discover::derive_app_snapshot();

    // 快照相同 且 版本匹配 → 走缓存，不重扫。
    if version_ok && !discover::should_rescan_from_snapshot(&prev, &current) {
        return;
    }

    // 需要重扫（首次 / 磁盘变化 / 版本失效）：扫描并更新缓存、版本与快照。
    let editors = editor::list_available_editors();
    let _ = editor::set_editor_cache(editors);
    let _ = editor::set_editor_cache_version(editor::EDITOR_LOGIC_VERSION);
    let _ = editor::set_app_snapshot(current);
}

/// 在指定编辑器内打开项目。
///
/// v0.2：改为「动态发现优先 + 白名单兜底」——按 id 在「白名单 + 动态发现」
/// 中查找编辑器，命中后按 `open_method` 分级打开（cli / open-a / unsupported）。
///
/// - 项目不存在 → 返回明确错误。
/// - `editor_id` 未知（列表找不到）→ `UnknownEditor`（前端提示「编辑器不存在」）。
/// - `open_method = Unsupported` → `UnsupportedMethod`（前端提示「请手动选择目录」）。
#[tauri::command]
pub fn open_in_editor(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    editor_id: String,
) -> Result<(), String> {
    let project_path = fetch_project_path(&db, project_id)?;
    editor::open_editor_by_id(std::path::Path::new(&project_path), &editor_id)
        .map_err(map_editor_err)
}

/// 用系统文件管理器打开项目目录。
#[tauri::command]
pub fn open_in_file_manager(
    db: State<'_, Mutex<Database>>,
    project_id: i64,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let project_path = fetch_project_path(&db, project_id)?;
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(project_path, None::<&str>)
        .map_err(|e| format!("打开文件管理器失败: {e}"))
}

/// 读取默认编辑器偏好（未设置返回 `null`）。
#[tauri::command]
pub fn get_editor_preference() -> Result<Option<String>, String> {
    editor::get_editor_preference().map_err(map_settings_err)
}

/// 设置默认编辑器偏好（白名单校验通过才写入 `~/.ydevsphere/settings.json`）。
#[tauri::command]
pub fn set_editor_preference(editor_id: String) -> Result<(), String> {
    editor::set_editor_preference(&editor_id).map_err(map_settings_err)
}

/// 从库读取项目路径。
/// 读取最近保存的工作区路径（未设置返回 `null`）。
#[tauri::command]
pub fn get_workspace_preference() -> Result<Option<String>, String> {
    editor::get_workspace_preference().map_err(map_settings_err)
}

/// 保存工作区路径。
///
/// - `path` 非空：写入（启动时自动恢复直达 Dashboard）。
/// - `path` 空串 / 空白：清除工作区偏好。
#[tauri::command]
pub fn set_workspace_preference(path: String) -> Result<(), String> {
    editor::set_workspace_preference(&path).map_err(map_settings_err)
}

/// 读取用户自定义忽略目录列表（未设置返回空数组）。
#[tauri::command]
pub fn get_ignore_rules() -> Result<Vec<String>, String> {
    editor::get_ignore_dirs().map_err(map_settings_err)
}

/// 设置用户自定义忽略目录列表（整表替换，去重 + 去空白项）。
#[tauri::command]
pub fn set_ignore_rules(dirs: Vec<String>) -> Result<(), String> {
    editor::set_ignore_dirs(&dirs).map_err(map_settings_err)
}

/// 读取工作区路径集合（v0.2 多工作区模型权威源）。
///
/// 兼容迁移：集合为空但旧单值 `workspace_path` 有值时，返回 `[workspace_path]`。
#[tauri::command]
pub fn get_workspaces() -> Result<Vec<String>, String> {
    editor::get_workspaces().map_err(map_settings_err)
}

/// 设置工作区路径集合（整表替换，去重 + 去空白项）。
#[tauri::command]
pub fn set_workspaces(dirs: Vec<String>) -> Result<(), String> {
    editor::set_workspaces(&dirs).map_err(map_settings_err)
}

/// 一次性重置本地应用状态（V03-RESET-BACKEND）。
///
/// 调 `core::editor::reset_settings()`：将 `settings.json` 重置为默认态，
/// 清空全部字段（custom_editors / workspaces / workspace_path /
/// default_editor / editor_cache / editor_cache_version / installed_apps_cache /
/// ignore_dirs / language）。**不删数据库项目**（方案 A：索引保留，重新导入
/// 工作区后扫描自动 upsert + 清理 missing 对齐）。
///
/// 仅转发；失败返回错误字符串。
#[tauri::command]
pub fn reset_app_state() -> Result<(), String> {
    editor::reset_settings().map_err(map_settings_err)
}

/// 读取界面语言偏好（未设置返回 `null`）。
#[tauri::command]
pub fn get_language_preference() -> Result<Option<String>, String> {
    editor::get_language_preference().map_err(map_settings_err)
}

/// 设置界面语言偏好（空串/空白清除偏好）。
#[tauri::command]
pub fn set_language_preference(lng: String) -> Result<(), String> {
    editor::set_language_preference(&lng).map_err(map_settings_err)
}

fn fetch_project_path(db: &Mutex<Database>, project_id: i64) -> Result<String, String> {
    let db = db.lock().map_err(|e| e.to_string())?;
    let detail = db
        .get_project_detail(project_id)
        .map_err(|e| format!("查询项目失败: {e}"))?
        .ok_or_else(|| format!("项目不存在: {project_id}"))?;
    Ok(detail.path)
}

fn map_editor_err(e: EditorError) -> String {
    e.to_string()
}

fn map_settings_err(e: SettingsError) -> String {
    e.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::EditorCategory;

    /// 共享锁：串行化所有使用 `YDEVSPHERE_SETTINGS_PATH` env 的测试，避免 env 竞态。
    /// 复用 editor 模块共享锁，与 settings 层测试互斥。
    use crate::core::editor::TEST_ENV_LOCK as ENV_LOCK;

    fn editor(id: &str, method: OpenMethod) -> AvailableEditor {
        AvailableEditor {
            id: id.to_string(),
            name: id.to_string(),
            cli_command: None,
            app_path: None,
            icon_base64: None,
            open_method: method,
            source: crate::core::models::EditorSource::Discovered,
            category: EditorCategory::AiEditor,
        }
    }

    /// 缓存清洗：读缓存时过滤掉 unsupported（旧缓存污染清除，升级即净）。
    #[test]
    fn filter_usable_removes_unsupported() {
        let list = vec![
            editor("cli-editor", OpenMethod::Cli),
            editor("open-a-editor", OpenMethod::OpenA),
            editor("unsupported-editor", OpenMethod::Unsupported),
        ];
        let usable = filter_usable(list);
        assert_eq!(usable.len(), 2, "应过滤掉 unsupported");
        assert!(usable.iter().all(|e| e.open_method != OpenMethod::Unsupported));
        assert!(usable.iter().any(|e| e.id == "cli-editor"));
        assert!(usable.iter().any(|e| e.id == "open-a-editor"));
    }

    /// 缓存清洗：全 unsupported 列表 → 空结果（不会残留非编辑器）。
    #[test]
    fn filter_usable_all_unsupported_empty() {
        let list = vec![
            editor("a", OpenMethod::Unsupported),
            editor("b", OpenMethod::Unsupported),
        ];
        assert!(filter_usable(list).is_empty());
    }

    // ---- v0.3：缓存失效版本化 ----

    /// 设置临时 settings 路径（env 隔离），并返回路径。
    fn temp_settings_path() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let p = std::env::temp_dir().join(format!(
            "ydevsphere_cachever_{}_{}.json",
            std::process::id(),
            n
        ));
        std::env::set_var("YDEVSPHERE_SETTINGS_PATH", &p);
        let _ = std::fs::remove_file(&p);
        p
    }

    /// 版本匹配 → 缓存有效；版本不匹配 / 未设置 → 缓存失效（需重扫）。
    #[test]
    fn cache_version_mismatch_invalidates() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // 未设置（默认 0）→ 版本不匹配（EDITOR_LOGIC_VERSION 通常 > 0）→ 失效
        assert!(
            !cache_version_matches(),
            "未记录版本应视为失效（触发重扫）"
        );

        // 写入当前版本 → 匹配 → 有效
        editor::set_editor_cache_version(editor::EDITOR_LOGIC_VERSION).unwrap();
        assert!(cache_version_matches(), "版本匹配应视为缓存有效");

        // 写入旧版本（+1 模拟识别逻辑更新）→ 不匹配 → 失效
        editor::set_editor_cache_version(editor::EDITOR_LOGIC_VERSION + 1).unwrap();
        assert!(!cache_version_matches(), "版本不匹配应视为缓存失效");

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }

    // ---- v0.3：list_editors 合并 custom_editors（V03-MANUAL-IMPORT-FIX） ----

    /// merge_custom：custom_editors 实时合并进自动检测结果（按 id 去重）。
    #[test]
    fn merge_custom_includes_custom_editors() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // 无 custom_editors → 合并后 = 自动检测。
        let auto = vec![editor("code", OpenMethod::Cli)];
        let merged = merge_custom(auto.clone());
        assert_eq!(merged.len(), 1, "无 custom 时合并 = 自动检测");

        // 写入一个手动导入的 custom 编辑器。
        let custom = editor("com.example.codex", OpenMethod::OpenA);
        editor::set_custom_editors(vec![custom]).unwrap();

        // 合并后应包含 custom 项。
        let merged2 = merge_custom(auto.clone());
        assert!(merged2.iter().any(|e| e.id == "com.example.codex"), "custom 项应合并进返回");
        assert_eq!(merged2.len(), 2);

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }

    /// merge_custom：custom 与自动检测 id 重复 → 去重，保留 **custom 项**（custom 优先）。
    #[test]
    fn merge_custom_dedups_by_id_keeps_custom() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // 自动检测与 custom 有同 id "code"，但用不同标记区分。
        let mut auto = vec![editor("code", OpenMethod::Cli)];
        auto[0].category = EditorCategory::Native;
        auto[0].cli_command = Some("auto-code".to_string());

        let mut custom = editor("code", OpenMethod::Cli);
        custom.category = EditorCategory::AiEditor;
        custom.cli_command = Some("custom-code".to_string());
        editor::set_custom_editors(vec![custom]).unwrap();

        let merged = merge_custom(auto);
        // 同 id 去重为 1 条，且保留 custom 项（category=AiEditor, cli="custom-code"）。
        assert_eq!(merged.len(), 1, "同 id 应去重");
        assert_eq!(merged[0].id, "code");
        assert_eq!(
            merged[0].cli_command.as_deref(),
            Some("custom-code"),
            "同 id 应保留 custom 项（custom 优先）"
        );
        assert_eq!(merged[0].category, EditorCategory::AiEditor);

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }

    /// 同 id 自动检测 + custom → source 稳定为 custom（手动），不随自动检测跳变。
    ///
    /// V03-CUSTOM-SOURCE：手动导入项 source = Custom。merge_custom 合并后
    /// 该 source 保持稳定（Custom），不会被同 id 的自动检测项覆盖，也不会被
    /// dedup_by_cli 误改。
    #[test]
    fn merge_custom_stable_source_for_manual_import() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // custom 项（手动导入的编辑器，source=Custom）。
        let mut custom = editor("com.example.editor", OpenMethod::OpenA);
        custom.source = EditorSource::Custom;
        editor::set_custom_editors(vec![custom]).unwrap();

        // 场景 A：自动检测能识别该 id（discovered 版，且带相同 cli 触发 dedup 路径）。
        let mut auto_a = vec![editor("com.example.editor", OpenMethod::Cli)];
        auto_a[0].source = EditorSource::Discovered;
        auto_a[0].cli_command = Some("shared-cli".to_string());
        let merged_a = merge_custom(auto_a);
        let a = merged_a.iter().find(|e| e.id == "com.example.editor").unwrap();
        assert_eq!(a.open_method, OpenMethod::OpenA, "自动检测版应被 custom 覆盖");
        assert_eq!(a.source, EditorSource::Custom, "source 稳定为 custom（手动导入）");

        // 场景 B：自动检测无法识别该 id（列表无此项）→ 仍返回 custom。
        let merged_b = merge_custom(Vec::new());
        let b = merged_b.iter().find(|e| e.id == "com.example.editor").unwrap();
        assert_eq!(b.open_method, OpenMethod::OpenA);
        assert_eq!(b.source, EditorSource::Custom, "source 不变");

        // 两种场景下 source 一致（稳定），不会一个 discovered 一个 custom 跳变。
        assert_eq!(a.source, b.source);

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }

    /// merge_custom：返回的 custom 项 source 稳定为 custom（手动导入）。
    ///
    /// 手动导入项（source=Custom）经 merge_custom 合并后 source 保持 Custom，
    /// 不被 dedup_by_cli（仅同 CLI 时用 Whitelist 覆盖）改动。
    #[test]
    fn merge_custom_returns_custom_source_for_manual_import() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // 一个带 cli_command 的手动导入 custom 项（source=Custom），
        // 同时存在自动检测项同 id（source=Discovered，同 CLI 触发 dedup 分支）。
        let mut custom = editor("com.example.fork", OpenMethod::Cli);
        custom.source = EditorSource::Custom;
        custom.cli_command = Some("myfork-cli".to_string());
        editor::set_custom_editors(vec![custom]).unwrap();

        let mut auto = vec![editor("com.example.fork", OpenMethod::Cli)];
        auto[0].source = EditorSource::Discovered;
        auto[0].cli_command = Some("myfork-cli".to_string());

        let merged = merge_custom(auto);
        let item = merged.iter().find(|e| e.id == "com.example.fork").unwrap();
        assert_eq!(item.source, EditorSource::Custom, "merge 后手动项 source 稳定为 custom");
        assert_eq!(item.cli_command.as_deref(), Some("myfork-cli"));

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }

    /// 闭环确认：导入 custom 后，list_editors（经 merge_custom）返回含该编辑器。
    #[test]
    fn import_then_list_contains_custom_editor() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // 模拟用户点选导入一个被识别逻辑排除的 app（OpenA 方式）。
        let editor = editor("com.example.manus", OpenMethod::OpenA);
        editor::set_custom_editors(vec![editor]).unwrap();

        // list_editors 走 scan_and_cache（真实扫描，可能返回真实编辑器）+ merge_custom。
        // 核心断言：返回列表必含刚导入的 custom 项（不受缓存影响）。
        let list = list_editors();
        assert!(
            list.iter().any(|e| e.id == "com.example.manus"),
            "导入后 list_editors 必须包含刚导入的编辑器"
        );

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }

    // ---- v0.3：同 CLI 命令去重（V03-EDITOR-DEDUP） ----

    /// 构造带 source / cli_command 的编辑器。
    fn editor_with(id: &str, cli: Option<&str>, source: EditorSource) -> AvailableEditor {
        AvailableEditor {
            id: id.to_string(),
            name: id.to_string(),
            cli_command: cli.map(String::from),
            app_path: None,
            icon_base64: None,
            open_method: OpenMethod::Cli,
            source,
            category: EditorCategory::Native,
        }
    }

    /// 同 CLI 命令（code）两条：保留白名单 vscode，丢弃动态 code。
    #[test]
    fn dedup_by_cli_keeps_whitelist() {
        let list = vec![
            editor_with("vscode", Some("code"), EditorSource::Whitelist),
            editor_with("code", Some("code"), EditorSource::Discovered),
        ];
        let deduped = dedup_by_cli(list);
        assert_eq!(deduped.len(), 1, "同 CLI 命令应合并为一条");
        assert_eq!(deduped[0].id, "vscode", "应保留白名单 vscode");
    }

    /// 同 CLI 命令且动态项在前：仍保留白名单（覆盖动态项）。
    #[test]
    fn dedup_by_cli_whitelist_wins_regardless_of_order() {
        let list = vec![
            editor_with("code", Some("code"), EditorSource::Discovered),
            editor_with("vscode", Some("code"), EditorSource::Whitelist),
        ];
        let deduped = dedup_by_cli(list);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].id, "vscode", "白名单应覆盖动态项");
    }

    /// CLI 为绝对路径（VS Code.app 的 bin）时，按 basename 也能识别同 CLI。
    #[test]
    fn dedup_by_cli_matches_absolute_path_basename() {
        let abs = "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code";
        let list = vec![
            editor_with("vscode", Some("code"), EditorSource::Whitelist),
            editor_with("code", Some(abs), EditorSource::Discovered),
        ];
        let deduped = dedup_by_cli(list);
        assert_eq!(deduped.len(), 1, "绝对路径 CLI basename 相同应去重");
        assert_eq!(deduped[0].id, "vscode");
    }

    /// 不同 CLI 命令不去重；无 CLI 项不去重。
    #[test]
    fn dedup_by_cli_keeps_distinct_cli() {
        let list = vec![
            editor_with("vscode", Some("code"), EditorSource::Whitelist),
            editor_with("cursor", Some("cursor"), EditorSource::Whitelist),
            editor_with("vim", None, EditorSource::Whitelist),
        ];
        let deduped = dedup_by_cli(list);
        assert_eq!(deduped.len(), 3, "不同 CLI / 无 CLI 不应去重");
    }

    // ---- V03-RESET-BACKEND：reset_app_state ----

    /// reset_app_state：清空 settings 且**不碰数据库项目**（方案 A）。
    ///
    /// 用内存数据库模拟已导入的项目，调用 reset 后：
    /// - settings 字段被清空；
    /// - 数据库项目记录仍保留（reset 只清 settings，不删 projects）。
    #[test]
    fn reset_app_state_clears_settings_keeps_db_projects() {
        use crate::core::database::connection::Database;
        use crate::core::models::DetectedProject;

        let _guard = ENV_LOCK.lock().unwrap();
        let path = temp_settings_path();

        // 1. 写入非默认 settings（模拟已使用状态）。
        editor::set_custom_editors(vec![editor("com.example.codex", OpenMethod::OpenA)])
            .unwrap();
        editor::set_language_preference("zh-CN").unwrap();
        editor::set_workspaces(&["/tmp/ws".to_string()]).unwrap();
        assert_eq!(editor::get_custom_editors().unwrap().len(), 1, "前置：settings 非空");

        // 2. 构造内存数据库并插入一个项目（模拟已导入的项目索引）。
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::core::database::migrations::run(&conn).unwrap();
        let db = Database::from_conn(conn);
        let p = DetectedProject::new("DemoProject", "/tmp/ws", Some("rust".to_string()), None);
        let inserted = db.upsert_projects(&[p]).expect("upsert 应成功");
        assert_eq!(inserted.len(), 1, "前置：已插入 1 个项目");
        assert_eq!(db.get_projects(None, Some("all"), None, Some(i64::MIN)).unwrap().len(), 1);

        // 3. 调用 reset_app_state。
        reset_app_state().expect("reset_app_state 应成功");

        // 4. settings 清空。
        assert_eq!(editor::get_custom_editors().unwrap(), Vec::new(), "custom_editors 清空");
        assert_eq!(editor::get_language_preference().unwrap(), None, "language 清空");
        assert_eq!(editor::get_workspaces().unwrap(), Vec::<String>::new(), "workspaces 清空");

        // 5. 数据库项目保留（reset 不碰数据库）。
        assert_eq!(
            db.get_projects(None, Some("all"), None, Some(i64::MIN)).unwrap().len(),
            1,
            "reset 后数据库项目应保留（方案 A）"
        );

        let _ = std::fs::remove_file(&path);
        std::env::remove_var("YDEVSPHERE_SETTINGS_PATH");
    }
}
