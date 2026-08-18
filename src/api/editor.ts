import { invoke } from "@tauri-apps/api/core";
import type { AvailableEditor, InstalledAppInfo } from "@/types";
import { ApiError } from "./project";

/**
 * 编辑器 API 封装（对齐后端 commands::editor）。
 *
 * ⚠️ 安全：后端仅执行白名单内已知编辑器；前端只能传后端返回的 editor id。
 */

async function call<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    throw new ApiError(typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
  }
}

/** 列出检测到的可用编辑器。后端：`list_editors() -> AvailableEditor[]` */
export function listEditors(): Promise<AvailableEditor[]> {
  return call(() => invoke<AvailableEditor[]>("list_editors"));
}

/** 重新扫描编辑器（清缓存 → 扫描 → 写缓存）。后端：`rescan_editors() -> AvailableEditor[]` */
export function rescanEditors(): Promise<AvailableEditor[]> {
  return call(() => invoke<AvailableEditor[]>("rescan_editors"));
}

/**
 * 获取「可选择常用编辑器」的干净候选列表（仅 `cli` + `open_a`，不含 unsupported，
 * 并合并已确认的自定义编辑器）。
 * 后端：`list_app_candidates() -> Vec<AvailableEditor>`
 */
export function listAppCandidates(): Promise<AvailableEditor[]> {
  return call(() => invoke<AvailableEditor[]>("list_app_candidates"));
}

/**
 * 确认导入一个自定义编辑器到 `custom_editors`（持久化用户确认）。
 * 后端：`confirm_custom_editor(editorId: String) -> Result<(), String>`
 * 未知 id 会抛错；Tauri 参数名映射：`editor_id` → `editorId`。
 */
export function confirmCustomEditor(editorId: string): Promise<void> {
  return call(() => invoke<void>("confirm_custom_editor", { editorId }));
}

/**
 * 列出 `/Applications` + `~/Applications` 下全部 `.app`（手动导入用，与识别逻辑解耦）。
 * 后端：`list_installed_apps() -> Vec<InstalledAppInfo>`
 */
export function listInstalledApps(): Promise<InstalledAppInfo[]> {
  return call(() => invoke<InstalledAppInfo[]>("list_installed_apps"));
}

/**
 * 手动导入一个自定义编辑器（写入 custom_editors，幂等）。
 * 后端：`import_custom_app(appPath: String) -> Result<AvailableEditor, String>`
 * - 路径不存在会抛错；已导入（同 id）幂等返回已有项。
 * Tauri 参数名映射：`app_path` → `appPath`。
 */
export function importCustomApp(appPath: string): Promise<AvailableEditor> {
  return call(() => invoke<AvailableEditor>("import_custom_app", { appPath }));
}

/**
 * 在指定编辑器内打开项目。
 * 后端：`open_in_editor(projectId, editorId) -> Result<(), String>`
 * 编辑器不可用 / 未知 id 会抛错，前端据此降级。
 */
export function openInEditor(projectId: number, editorId: string): Promise<void> {
  return call(() =>
    invoke<void>("open_in_editor", { projectId, editorId })
  );
}

/** 用系统文件管理器打开项目目录。后端：`open_in_file_manager(projectId)` */
export function openInFileManager(projectId: number): Promise<void> {
  return call(() =>
    invoke<void>("open_in_file_manager", { projectId })
  );
}

/** 读取默认编辑器偏好；未设置返回 null。后端：`get_editor_preference()` */
export function getEditorPreference(): Promise<string | null> {
  return call(() => invoke<string | null>("get_editor_preference"));
}

/** 设置默认编辑器偏好（后端白名单校验后持久化）。后端：`set_editor_preference(editorId)` */
export function setEditorPreference(editorId: string): Promise<void> {
  return call(() =>
    invoke<void>("set_editor_preference", { editorId })
  );
}
