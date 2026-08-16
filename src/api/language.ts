import { invoke } from "@tauri-apps/api/core";
import { ApiError } from "./project";

/**
 * 语言偏好 API 封装。
 *
 * 后端 `get_language_preference` / `set_language_preference` 已交付并注册
 * （见 src-tauri/src/commands/editor.rs，含 `set_language_preference` 空串/空白清除等单测）。
 *
 * 后端签名（参照 get_workspaces / set_workspaces 模式）：
 * - `get_language_preference() -> Result<Option<String>, String>`
 * - `set_language_preference(lng: String) -> Result<(), String>`
 *
 * 注：i18n store 仍保留 localStorage 降级（后端调用失败时兜底），属防御性设计，
 * 正常情况下以后端为准。
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

/** 读取语言偏好（未设置返回 null）。后端签名：`get_language_preference() -> Result<Option<String>, String>` */
export function getLanguagePreference(): Promise<string | null> {
  return call(() => invoke<string | null>("get_language_preference"));
}

/** 保存语言偏好。后端签名：`set_language_preference(lng) -> Result<(), String>` */
export function setLanguagePreference(lng: string): Promise<void> {
  return call(() => invoke<void>("set_language_preference", { lng }));
}
