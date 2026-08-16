// 与 src-tauri/src/core/models/workspace.rs 对齐
// NOTE: 前端类型必须与 Rust core/models 保持一致，修改时需同步两端。

/** 系统工作区种类（serde lowercase） */
export type SystemWorkspaceKind = "documents" | "desktop";

/** 一个系统工作区快捷入口（Documents / Desktop） */
export interface SystemWorkspace {
  kind: SystemWorkspaceKind;
  /** 展示名：`"Documents"` / `"Desktop"`（英文，不本地化） */
  label: string;
  /** 解析出的绝对路径；目录不存在时为 `null` */
  path: string | null;
  /** 目录是否存在（`false` 时快捷入口应禁用） */
  exists: boolean;
}
