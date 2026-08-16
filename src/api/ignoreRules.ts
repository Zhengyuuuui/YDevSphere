import { invoke } from "@tauri-apps/api/core";
import { ApiError } from "./project";

/**
 * 忽略规则 API 封装（对齐后端 commands::editor 的 get_ignore_rules / set_ignore_rules）。
 *
 * 语义：返回的是「目录名」（非完整路径），仅匹配扫描时的目录名；
 * 修改后下次 `scan_projects` 生效（预设规则 node_modules / .git / target / dist /
 * build / vendor / .cache / 隐藏目录始终生效，此处仅管理用户自定义追加项）。
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

/**
 * 读取用户自定义忽略目录列表（未设置返回空数组）。
 * 后端签名：`get_ignore_rules() -> Result<Vec<String>, String>`
 */
export function getIgnoreRules(): Promise<string[]> {
  return call(() => invoke<string[]>("get_ignore_rules"));
}

/**
 * 设置用户自定义忽略目录列表（整表替换，后端去重 + 去空白项）。
 * 后端签名：`set_ignore_rules(dirs: Vec<String>) -> Result<(), String>`
 */
export function setIgnoreRules(dirs: string[]): Promise<void> {
  return call(() => invoke<void>("set_ignore_rules", { dirs }));
}
