import { invoke } from "@tauri-apps/api/core";
import type { ProjectMemory } from "@/types";
import { ApiError } from "./project";

/**
 * 项目记忆 API 封装（对齐后端 core::commands::memory 签名）。
 *
 * ⚠️ 安全红线：写操作（ensure/update）必须显式传 `authorized: true` 才允许执行；
 * 前端仅在用户点击「启用/更新」时才置 true。跳过则不调用任何写接口。
 */

async function call<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    throw new ApiError(typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
  }
}

/**
 * 为指定项目生成/刷新 `.ydevsphere/project.json`（写操作，需用户授权）。
 *
 * 后端签名：`ensure_project_memory(projectId, packageManager, authorized) -> ProjectMemory`
 */
export function ensureProjectMemory(
  projectId: number,
  packageManager: string | null,
  authorized: boolean
): Promise<ProjectMemory> {
  return call(() =>
    invoke<ProjectMemory>("ensure_project_memory", {
      projectId,
      packageManager,
      authorized,
    })
  );
}

/**
 * 读取项目记忆；未启用/不存在返回 `null`（只读）。
 *
 * 后端签名：`get_project_memory(projectId) -> Option<ProjectMemory>`
 */
export function getProjectMemory(projectId: number): Promise<ProjectMemory | null> {
  return call(() =>
    invoke<ProjectMemory | null>("get_project_memory", { projectId })
  );
}

/**
 * 更新项目记忆字段（写操作，需用户授权）。
 *
 * 后端签名：`update_project_memory(projectId, packageManager, stack, authorized) -> ProjectMemory`
 * `stack` 传 null 时保留既有值；传数组时整体替换。
 */
export function updateProjectMemory(
  projectId: number,
  packageManager: string | null,
  stack: string[] | null,
  authorized: boolean
): Promise<ProjectMemory> {
  return call(() =>
    invoke<ProjectMemory>("update_project_memory", {
      projectId,
      packageManager,
      stack,
      authorized,
    })
  );
}
