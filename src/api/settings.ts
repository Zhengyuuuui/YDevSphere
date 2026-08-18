import { invoke } from "@tauri-apps/api/core";
import { ApiError } from "./project";

/**
 * 设置 API 封装（对齐后端 commands::editor 中的 reset_app_state）。
 */

async function call<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    throw new ApiError(typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
  }
}

/**
 * 重置应用本地状态（登出）：清空 settings.json 中的工作区 / 编辑器 / 偏好 / 缓存，保留数据库。
 * 后端：`reset_app_state() -> Result<(), String>`
 */
export function resetAppState(): Promise<void> {
  return call(() => invoke<void>("reset_app_state"));
}
