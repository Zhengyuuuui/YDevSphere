import { invoke } from "@tauri-apps/api/core";
import type { GitInfo } from "@/types";
import { ApiError } from "./project";

/**
 * Git 分析 API 封装（对齐后端 commands::git::get_project_git_info）。
 *
 * 后端签名：`get_project_git_info(projectId) -> Result<Option<GitInfo>, String>`
 * - 项目不存在 → Err
 * - 非 git 仓库 → null（优雅降级，GitInfo.is_git_repo 为 false）
 * - git 仓库 → GitInfo
 *
 * ⚠️ 只读：后端仅用 git2 只读 API，绝不修改 git 状态。
 */

async function call<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    throw new ApiError(typeof e === "string" ? e : e instanceof Error ? e.message : String(e));
  }
}

/** 获取指定项目的 git 信息（只读）；非 git 仓库返回 null */
export function getProjectGitInfo(projectId: number): Promise<GitInfo | null> {
  return call(() =>
    invoke<GitInfo | null>("get_project_git_info", { projectId })
  );
}
