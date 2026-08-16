import { invoke } from "@tauri-apps/api/core";
import type {
  DirNode,
  Project,
  ProjectDetail,
  ProjectKind,
  ScanResult,
  SystemWorkspace,
} from "@/types";

/** 统一错误：将后端返回的错误转成可读信息 + 可选错误码 */
export class ApiError extends Error {
  /** 后端结构化错误码（如 INVALID_DIRECTORY / IO_ERROR / DB_ERROR）；无则 undefined */
  code?: string;
  constructor(message: string, code?: string) {
    super(message);
    this.name = "ApiError";
    if (code !== undefined) this.code = code;
  }
}

/** 从各种错误形态中提取 message + 可选 code */
function parseError(e: unknown): { message: string; code?: string } {
  // 1) 字符串：可能是 { code, message } / { success:false, message } 的 JSON 字符串化，或纯文本
  if (typeof e === "string") {
    try {
      const parsed = JSON.parse(e) as {
        code?: string;
        message?: string;
        success?: boolean;
      };
      if (parsed && typeof parsed === "object") {
        const msg = parsed.message ?? e;
        return { message: msg, code: parsed.code };
      }
    } catch {
      // 非 JSON，回退纯文本
    }
    return { message: e };
  }
  // 2) Error 实例：可能携带 code 属性（Tauri 反序列化自定义错误）
  if (e instanceof Error) {
    const maybeCode = (e as Error & { code?: unknown }).code;
    return {
      message: e.message,
      code: typeof maybeCode === "string" ? maybeCode : undefined,
    };
  }
  // 3) 对象：直接含 code / message 字段
  if (e && typeof e === "object") {
    const obj = e as { code?: unknown; message?: unknown };
    return {
      message: typeof obj.message === "string" ? obj.message : String(e),
      code: typeof obj.code === "string" ? obj.code : undefined,
    };
  }
  return { message: String(e) };
}

async function call<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    const { message, code } = parseError(e);
    throw new ApiError(message, code);
  }
}

/**
 * 打开原生目录选择器。
 *
 * 返回用户选择的绝对路径；若用户取消选择返回 `null`。
 * 后端签名：`Result<Option<String>, String>`
 */
export function selectWorkspace(): Promise<string | null> {
  return call(() => invoke<string | null>("select_workspace"));
}

/**
 * 扫描指定目录，识别项目、解析技术栈并写入数据库。
 *
 * 后端签名：`Result<ScanResult, String>`
 */
export function scanProjects(workspacePath: string): Promise<ScanResult> {
  return call(() => invoke<ScanResult>("scan_projects", { workspacePath }));
}

/** 工作区筛选值（对齐后端 workspace_filter） */
export type WorkspaceFilter = "all" | "documents" | "desktop";

/** 项目排序值（对齐后端 sort_by；"updated_at" 为默认） */
export type ProjectSortBy = "name" | "updated_at";

/**
 * 获取项目列表。
 *
 * 后端签名：`get_projects(sort_by?, workspace_filter?, kind_filter?, parent_id_filter?) -> Vec<Project>`
 * - `sortBy`："name"（名称升序）或 "updated_at"（默认，最近扫描倒序）；不传/非法回退默认。
 * - `workspaceFilter`："all"（默认）/ "documents" / "desktop"；不传回退 all（向后兼容）。
 * - `kindFilter`（v0.2）：`"real"` / `"aggregated_root"` / `"category"`；不传不过滤。
 * - `parentIdFilter`（v0.2）：默认（`undefined`/`null`）只返回顶层项目（`parent_id IS NULL`）；
 *   传父项目 id 返回其直接子项目（前端展开聚合根/分类目录用）。
 * Tauri 参数名映射：`sort_by`→`sortBy`，`workspace_filter`→`workspaceFilter`，
 * `kind_filter`→`kindFilter`，`parent_id_filter`→`parentIdFilter`。
 */
export function getProjects(
  sortBy?: ProjectSortBy,
  workspaceFilter?: WorkspaceFilter,
  kindFilter?: ProjectKind,
  parentIdFilter?: number | null
): Promise<Project[]> {
  return call(() =>
    invoke<Project[]>("get_projects", {
      sortBy,
      workspaceFilter,
      kindFilter,
      parentIdFilter,
    })
  );
}

/**
 * 按需返回指定目录的直接子项（`DirNode[]`），供前端懒加载目录树。
 *
 * 后端签名：`get_dir_children(path: String) -> Vec<DirNode>`（不递归，仅直接子项；
 * 隐藏项与预设忽略目录跳过；目录不存在/不可读返回空列表）。
 * Tauri 参数名映射：`path` → `path`。
 */
export function getDirChildren(path: string): Promise<DirNode[]> {
  return call(() => invoke<DirNode[]>("get_dir_children", { path }));
}

/**
 * 获取项目详情。
 *
 * 后端签名：`Result<Option<ProjectDetail>, String>`，项目不存在时返回 `null`。
 */
export function getProjectDetail(projectId: number): Promise<ProjectDetail | null> {
  return call(() =>
    invoke<ProjectDetail | null>("get_project_detail", { projectId })
  );
}

/**
 * 获取 Documents / Desktop 两个系统工作区入口。
 * 仅返回路径与存在性；实际扫描由前端调 `scan_projects(path)`。
 * 后端签名：`get_system_workspaces() -> SystemWorkspace[]`
 */
export function getSystemWorkspaces(): Promise<SystemWorkspace[]> {
  return call(() => invoke<SystemWorkspace[]>("get_system_workspaces"));
}
