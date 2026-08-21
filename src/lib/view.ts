import type { Project } from "@/types";
import type { ProjectView, GitStatusView } from "@/types/view";
import { formatDateTime } from "./format";
import type { GitInfo } from "@/types";

/**
 * 数据适配层：把后端真实 `Project` 转成前端视图模型 `ProjectView`。
 *
 * Figma 设计稿的 Project 结构与后端不同（technologies[] / git{type} /
 * updatedAt 字符串 vs language/framework 单字段 / git 需单独拉取），
 * 所有页面统一经此适配，避免直接搬运 Figma mock 结构。
 */

/** 由后端技术栈合成展示列表（Figma 的 technologies[] 语义，V0.4 PR4）。
 *
 * 优先 `Project.technologies`（V0.4 识别引擎产出，含架构级 + library）；
 * 旧数据（technologies 为空）回退 `language` / `framework`（旧字段兼容）。
 */
export function toTechnologies(p: Project): string[] {
  if (p.technologies && p.technologies.length > 0) {
    return p.technologies.map((t) => t.name || t.id);
  }
  return [p.language, p.framework].filter((s): s is string => Boolean(s));
}

/** 由 GitInfo.status（Rust 枚举）映射到视图 Git 状态 */
export function toGitStatusView(info: GitInfo | null | undefined): GitStatusView {
  if (!info || !info.is_git_repo) return "none";
  if (info.status && "Dirty" in info.status && info.status.Dirty) return "dirty";
  if (info.status && "Clean" in info.status) return "clean";
  return "none";
}

/** Git dirty 变更文件数（仅 dirty 时有效） */
export function gitChangeCount(info: GitInfo | null | undefined): number | undefined {
  if (info?.status && "Dirty" in info.status && info.status.Dirty) {
    return info.status.Dirty.changed_files;
  }
  return undefined;
}

/**
 * 项目 → 视图模型。
 *
 * @param project 后端 Project
 * @param gitInfo 该项目已缓存的 git 信息（来自 git store，按需拉取）；未获取传 undefined
 * @param lastOpenedAt 最近打开时间（来自 recent store 的本地记录，可选）
 */
export function toProjectView(
  project: Project,
  gitInfo?: GitInfo | null,
  lastOpenedAt?: string | null
): ProjectView {
  return {
    id: project.id,
    name: project.name,
    path: project.path,
    technologies: toTechnologies(project),
    updatedAt: formatDateTime(project.updated_at),
    lastOpenedAt: lastOpenedAt ?? null,
    gitType: toGitStatusView(gitInfo),
    gitChanges: gitChangeCount(gitInfo),
    healthScore: project.health_score,
    kind: project.kind,
    parentId: project.parent_id,
    raw: project,
  };
}

/** 批量适配（列表场景） */
export function toProjectViews(
  projects: Project[],
  gitOf: (id: number) => GitInfo | null | undefined,
  lastOpenedOf?: (id: number) => string | null | undefined
): ProjectView[] {
  return projects.map((p) =>
    toProjectView(p, gitOf(p.id), lastOpenedOf ? lastOpenedOf(p.id) : null)
  );
}
