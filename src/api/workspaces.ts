import { invoke } from "@tauri-apps/api/core";
import { ApiError } from "./project";

/**
 * 工作区集合 API 封装（对齐后端 commands::editor 的 get_workspaces / set_workspaces）。
 *
 * 后端语义（见 src-tauri/src/core/editor/settings.rs）：
 * - `get_workspaces`：读取工作区集合（权威源）；兼容迁移——集合空但有单值偏好时返回 `[单值]`。
 * - `set_workspaces(dirs)`：整表替换（去重 + 去空白），同时镜像 `workspace_path` 为集合首项。
 *
 * Tauri 参数名映射：`dirs` → `dirs`。
 */

async function call<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    throw new ApiError(
      typeof e === "string" ? e : e instanceof Error ? e.message : String(e)
    );
  }
}

/** 读取工作区集合（后端权威源）。后端签名：`get_workspaces() -> Result<Vec<String>, String>` */
export function getWorkspaces(): Promise<string[]> {
  return call(() => invoke<string[]>("get_workspaces"));
}

/** 设置工作区集合（整表替换）。后端签名：`set_workspaces(dirs: Vec<String>) -> Result<(), String>` */
export function setWorkspaces(dirs: string[]): Promise<void> {
  return call(() => invoke<void>("set_workspaces", { dirs }));
}
