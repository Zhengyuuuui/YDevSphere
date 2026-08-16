import { invoke } from "@tauri-apps/api/core";
import type { AvailableEditor } from "@/types";
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
