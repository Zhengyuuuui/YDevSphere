// 与 src-tauri/src/core/models/editor.rs 对齐
// NOTE: 前端类型必须与 Rust core/models 保持一致，修改时需同步两端。

/** 编辑器打开方式（对齐 core::models::OpenMethod，snake_case 序列化） */
export type OpenMethod = "cli" | "open_a" | "unsupported";

/** 编辑器来源（对齐 core::models::EditorSource） */
export type EditorSource = "whitelist" | "discovered" | "custom";

/** 编辑器分类（对齐 core::models::EditorCategory，snake_case 序列化） */
export type EditorCategory =
  | "vscode_fork"
  | "native"
  | "ai_chat"
  | "ai_editor";

/** 一个可用的编辑器（检测结果，对齐 core::models::AvailableEditor） */
export interface AvailableEditor {
  /** 稳定的编辑器标识（白名单 key 或 product.json applicationName），如 "vscode" / "cursor" */
  id: string;
  /** 展示名，如 "Visual Studio Code" / "Cursor" */
  name: string;
  /** CLI 命令（PATH 中的命令名或 app 内绝对路径）；可能不存在 */
  cli_command?: string | null;
  /** .app 包路径（macOS），如 /Applications/Cursor.app；可能不存在 */
  app_path?: string | null;
  /** 应用图标（256px PNG 的 base64 编码），app_path 指向真实 .app 时填充；可能为空 */
  icon_base64?: string | null;
  /** 打开方式 */
  open_method: OpenMethod;
  /** 来源 */
  source: EditorSource;
  /** 分类 */
  category: EditorCategory;
}

/**
 * 已安装应用信息（V03-MANUAL-IMPORT：手动导入用，与识别逻辑解耦）。
 * 对齐 core::models::InstalledAppInfo（camelCase 序列化）。
 */
export interface InstalledAppInfo {
  /** 应用名（去掉 `.app` 后缀的包名） */
  name: string;
  /** .app 绝对路径，如 /Applications/Cursor.app */
  path: string;
  /** bundle id（读 Info.plist CFBundleIdentifier），可空 */
  bundle_id?: string | null;
  /** 是否存在 product.json（VS Code Fork 判断用） */
  has_product_json: boolean;
  /** 应用图标（128px PNG 的 base64 编码），可空 */
  icon_base64?: string | null;
}
