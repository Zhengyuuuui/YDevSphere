//! 编辑器打开动作（白名单执行 + 打开方式分级）。
//!
//! ## 安全
//! - 仅执行白名单内、已解析出的编辑器可执行路径（`resolve_editor_by_id`）。
//! - 未知 `editor_id` 在解析阶段即被拒绝，不会 `spawn` 任何进程。
//! - `OpenA` 方式仅对声明 `public.folder` 的 app 执行 `open -a`；`Unsupported`
//!   不执行任何打开动作。
//!
//! ## 打开方式分级（V02-EDITOR-DISCOVER）
//! `cli`（PATH）→ `cli`（app 绝对路径）→ `open -a` → `unsupported`。

use std::path::Path;

use super::detect::{find_editor_by_id, resolve_editor_by_id, EditorError};
use crate::core::models::{AvailableEditor, OpenMethod};

/// 在指定编辑器内打开项目路径（白名单 id）。
///
/// 只启动白名单内已知编辑器，参数为解析出的可执行路径 + 项目路径。
pub fn open_in_editor(project_path: &Path, editor_id: &str) -> Result<(), EditorError> {
    let exec = resolve_editor_by_id(editor_id)?;

    let mut cmd = std::process::Command::new(&exec);
    cmd.arg(project_path);

    // 后台启动，不阻塞；失败视为不可用。
    cmd.spawn().map(|_| ()).map_err(EditorError::Launch)
}

/// 按编辑器 id 打开项目路径（动态发现优先 + 白名单兜底的统一入口）。
///
/// 流程：
/// 1. 在 `list_available_editors()`（白名单 + 动态发现）中找 `id == editor_id`。
/// 2. 找到 → 按 `open_method` 分级打开（`open_editor_via`）。
/// 3. 找不到 → `UnknownEditor`。
///
/// 相比旧的 `open_in_editor`（仅白名单），本入口支持动态发现的编辑器，
/// 使 `cursor` / `Trae` / ChatGPT 等都能正确打开。
pub fn open_editor_by_id(project_path: &Path, editor_id: &str) -> Result<(), EditorError> {
    let editor = find_editor_by_id(editor_id)
        .ok_or_else(|| EditorError::UnknownEditor(editor_id.to_string()))?;
    open_editor_via(&editor, project_path)
}

/// 按 `AvailableEditor` 的 `open_method` 打开项目路径。
///
/// 分级：
/// - `Cli`：用 `cli_command` 启动（可能是 PATH 命令名或 app 内绝对路径）。
/// - `OpenA`：`open -a <app_name>` 打开（仅 macOS；仅对支持 folder 的 app）。
/// - `Unsupported`：不执行任何动作，返回 `UnsupportedMethod` 错误。
pub fn open_editor_via(editor: &AvailableEditor, project_path: &Path) -> Result<(), EditorError> {
    match editor.open_method {
        OpenMethod::Cli => {
            let Some(cli) = &editor.cli_command else {
                return Err(EditorError::NotFound(editor.id.clone()));
            };

            // 解析要执行的命令，解决「VS Code 点开报错」：
            // 白名单项的 `cli_command` 是纯命令名（如 `code`），`app_path` 才是
            // `resolve_editor_command` 解析出的真实可执行路径。若 PATH 无 `code`
            //（VS Code 通过绝对路径候选命中、但 CLI 未装进 PATH），`Command::new("code")`
            // 会 spawn 失败。故优先用可执行文件路径启动，其次命令名。
            let exec = if Path::new(cli).is_absolute() {
                // cli_command 已是绝对路径（动态 Fork 回退 app 内 bin）
                cli.clone()
            } else if editor
                .app_path
                .as_deref()
                .map(|p| Path::new(p).is_file())
                .unwrap_or(false)
            {
                // app_path 是可执行文件（白名单项存 resolve_editor_command 结果）
                editor.app_path.as_deref().unwrap().to_string()
            } else {
                // 命令名，依赖 PATH（如动态 Fork PATH 命中）
                cli.clone()
            };

            let mut cmd = std::process::Command::new(&exec);
            cmd.arg(project_path);
            cmd.spawn().map(|_| ()).map_err(EditorError::Launch)
        }
        OpenMethod::OpenA => {
            // 提取 app 名（app_path 末尾的 .app 名，去掉后缀）。
            let app_name = editor
                .app_path
                .as_deref()
                .and_then(|p| Path::new(p).file_name())
                .map(|n| n.to_string_lossy().to_string())
                .map(|n| {
                    n.strip_suffix(".app").map(String::from).unwrap_or(n)
                })
                .ok_or_else(|| EditorError::NotFound(editor.id.clone()))?;

            // macOS：open -a <app> <path>
            let mut cmd = std::process::Command::new("open");
            cmd.arg("-a").arg(&app_name).arg(project_path);
            cmd.spawn().map(|_| ()).map_err(EditorError::Launch)
        }
        OpenMethod::Unsupported => {
            Err(EditorError::UnsupportedMethod(editor.id.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{EditorCategory, EditorSource};

    fn editor(
        id: &str,
        open_method: OpenMethod,
        cli_command: Option<&str>,
        app_path: Option<&str>,
    ) -> AvailableEditor {
        AvailableEditor {
            id: id.to_string(),
            name: id.to_string(),
            cli_command: cli_command.map(String::from),
            app_path: app_path.map(String::from),
            icon_base64: None,
            open_method,
            source: EditorSource::Discovered,
            category: EditorCategory::Native,
        }
    }

    /// Unsupported 不执行任何打开动作，返回明确错误。
    #[test]
    fn unsupported_method_returns_error() {
        let e = editor("chat", OpenMethod::Unsupported, None, None);
        let result = open_editor_via(&e, Path::new("/tmp/proj"));
        assert!(matches!(result, Err(EditorError::UnsupportedMethod(_))));
    }

    /// Cli 但缺 cli_command → NotFound（不 panic）。
    #[test]
    fn cli_without_command_returns_not_found() {
        let e = editor("broken", OpenMethod::Cli, None, None);
        let result = open_editor_via(&e, Path::new("/tmp/proj"));
        assert!(matches!(result, Err(EditorError::NotFound(_))));
    }

    /// `open_editor_by_id`：找不到 id → UnknownEditor（非「不支持打开」）。
    #[test]
    fn open_editor_by_id_unknown_returns_unknown_editor() {
        let result = open_editor_by_id(Path::new("/tmp/proj"), "__nonexistent_editor__");
        assert!(matches!(result, Err(EditorError::UnknownEditor(_))));
    }

    /// 错误区分：UnsupportedMethod 与 UnknownEditor 的 Display 文案不同。
    #[test]
    fn unsupported_and_unknown_have_distinct_messages() {
        let unsupported = EditorError::UnsupportedMethod("chat".to_string());
        let unknown = EditorError::UnknownEditor("chat".to_string());
        assert_ne!(unsupported.to_string(), unknown.to_string());
        // unsupported 文案应提示「手动选择目录」
        assert!(unsupported.to_string().contains("手动选择目录"));
        // unknown 文案应提示「未知编辑器」
        assert!(unknown.to_string().contains("未知编辑器"));
    }

    /// 根因修复（V03 vs code 打开报错）：白名单项 cli_command 是命令名（如
    /// `code`）但 PATH 无此命令时，应回退用 app_path（可执行文件路径）启动。
    ///
    /// 用临时可执行脚本作为 app_path，cli_command 设为一个不存在的命令名，
    /// 验证 open_editor_via 选择 app_path 启动（spawn 成功）而非 cli_command。
    #[cfg(unix)]
    #[test]
    fn cli_falls_back_to_app_path_when_command_missing_in_path() {
        use std::os::unix::fs::PermissionsExt;
        use std::sync::atomic::{AtomicUsize, Ordering};

        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!(
            "ydevsphere_open_test_{}_{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        // 临时可执行脚本（模拟 resolve_editor_command 解析出的可执行路径）。
        let script = tmp.join("fake-code");
        std::fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();

        // 白名单项：cli_command = 不存在的命令名，app_path = 可执行脚本。
        let e = editor(
            "vscode",
            OpenMethod::Cli,
            Some("nonexistent-xyz-command"),
            Some(script.to_str().unwrap()),
        );
        // 应通过 app_path（可执行文件）成功 spawn，而非 cli_command（失败）。
        let result = open_editor_via(&e, Path::new("/tmp"));
        assert!(result.is_ok(), "应回退用 app_path 启动，实际: {result:?}");

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
