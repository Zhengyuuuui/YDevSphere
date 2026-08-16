import type { Project, ProjectKind } from "./project";

/**
 * 前端视图模型（供表格 / 总览使用）。
 *
 * 后端 Project（language/framework 单字段）与 Figma 设计稿的
 * Project（technologies[]、git{type}、updatedAt）结构不同，
 * 本类型是两者的适配层（详见 src/lib/view.ts 的 toProjectView）。
 */

/** Git 视图状态（映射自 git store 的 GitInfo.status，未获取为 "none"） */
export type GitStatusView = "clean" | "dirty" | "detached" | "none";

/** 技术栈徽标视图（name + 配色 variant，供表格渲染） */
export interface TechnologyView {
  name: string;
  /** 配色类别（供 TechnologyBadge 选色） */
  variant: string;
}

/** 项目视图模型 */
export interface ProjectView {
  id: number;
  name: string;
  path: string;
  /** 技术栈列表（[language, framework].filter(Boolean)） */
  technologies: string[];
  /** 更新时间展示（格式化后的字符串，空为 null） */
  updatedAt: string | null;
  /** 最近打开时间展示（Recent 页用；来自 localStorage，可能为 null） */
  lastOpenedAt: string | null;
  /** Git 状态（来自 git store 缓存；未获取为 "none"） */
  gitType: GitStatusView;
  /** Git 变更文件数（dirty 时有效） */
  gitChanges?: number;
  /** 健康度评分（v0.2 scanner 迭代后接入，0-100） */
  healthScore: number;
  /** 项目类型（v0.2：真项目 / 聚合根 / 分类目录） */
  kind: ProjectKind;
  /** 父项目 id（顶层为 null） */
  parentId: number | null;
  /** 原始后端 Project 引用（跳详情 / 打开用） */
  raw: Project;
}
