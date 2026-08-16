// 与 src-tauri/src/core/models/git.rs 对齐
// NOTE: 前端类型必须与 Rust core/models 保持一致，修改时需同步两端。

/** 最近一次提交信息（对齐 core::models::CommitInfo） */
export interface CommitInfo {
  /** 短 hash（前 8 位） */
  hash: string;
  /** 提交 message（首行） */
  message: string;
  /** 提交作者（name，回退到 email） */
  author: string;
  /** 提交时间（RFC3339） */
  time: string;
}

/** 工作区状态（对齐 core::models::GitStatus 枚举） */
export type GitStatus =
  | { Clean: true }
  | { Dirty: { changed_files: number } };

/** Git 仓库分析结果（对齐 core::models::GitInfo） */
export interface GitInfo {
  /** 是否为 git 仓库（Repository::open 失败 / 非仓库时为 false） */
  is_git_repo: boolean;
  /** 当前分支名（HEAD detached 时为 null） */
  branch: string | null;
  /** 最近一次提交信息（无任何 commit 时为 null） */
  last_commit: CommitInfo | null;
  /** 工作区状态（Clean / Dirty） */
  status: GitStatus | null;
  /** 最近一次 commit 时间（ISO 8601 / RFC3339）；无 commit 时为 null */
  last_update: string | null;
}
