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
    /// 用户手动导入（下拉框点选 / 导入自定义 .app）。
    #[serde(rename = "custom")]
    Custom,
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
    /// AI 编辑器（v0.3：无 product.json，但 Info.plist 声明了代码文件类型，
    /// 视为可编辑代码的 AI 编辑器，如 ChatGPT/Codex、Claude）。
    AiEditor,
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
    /// 编辑器图标（512px PNG 的 base64 编码），可空。
    ///
    /// 供 Welcome 编辑器选择器等大图标场景使用；`app_path` 指向真实 `.app`
    /// 时填充（V03-ICON-512），无 `.app`（如 vim/nvim 纯 CLI）为 `None`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_base64: Option<String>,
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

/// 已安装应用信息（V03-MANUAL-IMPORT-BACKEND：手动导入用，与识别逻辑解耦）。
///
/// `list_installed_apps` 返回，列出 `/Applications` + `~/Applications` 下
/// **全部** .app（不过滤识别逻辑），供前端展示、用户点选导入。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledAppInfo {
    /// 应用名（去掉 `.app` 后缀的包名）。
    pub name: String,
    /// .app 绝对路径，如 `/Applications/Cursor.app`。
    pub path: String,
    /// bundle id（读 Info.plist `CFBundleIdentifier`），可空。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    /// 是否存在 `product.json`（VS Code Fork 判断用）。
    pub has_product_json: bool,
    /// 应用图标（128px PNG 的 base64 编码），可空。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_base64: Option<String>,
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
