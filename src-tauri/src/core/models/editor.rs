//! 编辑器相关数据结构（SPRINT5-02 + V02-EDITOR-DISCOVER）。

use serde::{Deserialize, Serialize};

/// 编辑器打开方式。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpenMethod {
    /// CLI 打开（PATH 命中或 app 内绝对路径）。
    Cli,
    /// `open -a <App>` 打开（仅限声明了 public.folder 等文档类型的 app）。
    OpenA,
    /// 不支持打开（纯 AI 聊天 / 无 CLI 无文档类型）。
    Unsupported,
}

/// 编辑器来源。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EditorSource {
    /// 白名单预定义。
    Whitelist,
    /// 动态发现（扫描 /Applications）。
    Discovered,
}

/// 编辑器分类。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EditorCategory {
    /// VS Code Fork（有 product.json 指纹）。
    VscodeFork,
    /// 原生编辑器 / IDE（JetBrains / Sublime / Vim 等）。
    Native,
    /// AI 聊天工具（不含编辑器功能，如纯 CLI AI）。
    AiChat,
}

/// 一个可用的编辑器（检测结果）。
///
/// 向后兼容：`id` + `name` 保持原样；新字段均有 `serde(default)` /
/// `skip_serializing_if` 保证旧数据可反序列化。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AvailableEditor {
    /// 稳定的编辑器标识（白名单 key 或 product.json applicationName），
    /// 如 `"vscode"` / `"cursor"` / `"windsurf"`。
    pub id: String,
    /// 展示名，如 `"Visual Studio Code"` / `"Cursor"`。
    pub name: String,
    /// CLI 命令（PATH 中的命令名或 app 内绝对路径）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli_command: Option<String>,
    /// .app 包路径（macOS），如 `/Applications/Cursor.app`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_path: Option<String>,
    /// 打开方式。
    #[serde(default = "default_open_method")]
    pub open_method: OpenMethod,
    /// 来源。
    #[serde(default = "default_source")]
    pub source: EditorSource,
    /// 分类。
    #[serde(default = "default_category")]
    pub category: EditorCategory,
}

fn default_open_method() -> OpenMethod {
    OpenMethod::Cli
}

fn default_source() -> EditorSource {
    EditorSource::Whitelist
}

fn default_category() -> EditorCategory {
    EditorCategory::Native
}
