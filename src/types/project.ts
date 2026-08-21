// 与 src-tauri/src/core/models/project.rs 对齐
// NOTE: 前端类型必须与 Rust core/models 保持一致，修改时需同步两端。

/** 项目类型（v0.2 Scanner 迭代，对齐 core::models::ProjectKind） */
export type ProjectKind = "real" | "aggregated_root" | "category";

/**
 * 技术类别（对齐 core::models::TechnologyCategory，serde snake_case）。
 *
 * 架构级（用于前端展示过滤，Spec §7.1）：Language / Runtime / Framework /
 * Database / BuildTool / PackageManager / Platform。
 * Library 为「非架构级」，展示时折叠为 `+N libraries`（前端过滤，后端存储层不动）。
 */
export type TechnologyCategory =
  | "language"
  | "runtime"
  | "framework"
  | "library"
  | "database"
  | "build_tool"
  | "package_manager"
  | "platform";

/**
 * 单个技术（对齐 core::models::Technology，serde snake_case）。
 *
 * - `id`：canonical id（稳定，去重/聚合基于此）。
 * - `name`：展示名。
 * - `category`：技术类别。
 * - `ecosystem`：技术生态，未知为 null。
 */
export interface Technology {
  id: string;
  name: string;
  category: TechnologyCategory;
  ecosystem: string | null;
}

/**
 * `projects.technologies_json` 列的 JSON 封装（对齐 core::models::TechnologiesJson）。
 *
 * 后端落库：`{ "schema_version": 1, "technologies": [...] }`。
 * 旧数据（technologies_json 为 NULL）→ 前端 `Project.technologies` 为空数组，
 * 展示时回退 `language` / `framework`（旧字段兼容）。
 */
export interface TechnologiesJson {
  /** JSON 结构版本（当前 = 1） */
  schema_version: number;
  technologies: Technology[];
}

/** 目录树节点（按需返回，供前端懒加载目录树，对齐 core::models::DirNode） */
export interface DirNode {
  /** 子项名称（目录 / 文件名） */
  name: string;
  /** 子项绝对路径 */
  path: string;
  /** 是否为目录 */
  is_dir: boolean;
  /** 是否为「真项目根」（该目录含清单文件） */
  has_manifest: boolean;
  /** 直接子项数量（仅目录时有效；文件为 0） */
  children_count: number;
}

/** 项目基础信息（对齐 projects 表 / core::models::Project） */
export interface Project {
  id: number;
  name: string;
  path: string;
  language: string | null;
  framework: string | null;
  created_at: string | null;
  updated_at: string | null;
  /** 项目文件总数（扫描时统计落库，非实时） */
  file_count: number;
  /** 最近一次扫描时间 */
  last_scan_at: string | null;
  /** 项目归属的工作区根路径（扫描时写入）；手动目录为 null（归「全部」） */
  workspace: string | null;
  /** 项目类型（v0.2：真项目 / 聚合根 / 分类目录） */
  kind: ProjectKind;
  /** 健康度评分（0-100） */
  health_score: number;
  /** 父项目 id（聚合根 / 分类目录下的树形归属；顶层为 null） */
  parent_id: number | null;
  /**
   * 技术栈列表（V0.4 PR1，对齐后端 Project.technologies）。
   *
   * 后端由 `technologies_json` 列解析；旧数据为空数组时前端回退
   * `language` / `framework`。聚合根为子项目 derived 并集（含 library）。
   */
  technologies: Technology[];
}

/** 项目详情（对齐 core::models::ProjectDetail） */
export interface ProjectDetail {
  id: number;
  name: string;
  path: string;
  language: string | null;
  framework: string | null;
  created_at: string | null;
  updated_at: string | null;
  /** 项目文件总数（由扫描时统计，非实时） */
  file_count: number;
  /** 最近一次扫描时间 */
  last_scan_at: string | null;
  /** 项目归属的工作区根路径 */
  workspace: string | null;
  /** 项目类型（v0.2） */
  kind: ProjectKind;
  /** 健康度评分（0-100） */
  health_score: number;
  /** 父项目 id */
  parent_id: number | null;
  /**
   * 技术栈列表（V0.4 PR1，对齐后端 ProjectDetail.technologies）。
   * 聚合根为 derived 并集；空数组时回退 language / framework（旧数据）。
   */
  technologies: Technology[];
}

/** 扫描历史记录（对齐 scan_history 表 / core::models::ScanHistory） */
export interface ScanHistory {
  id: number;
  /** 被扫描的工作区路径 */
  workspace: string;
  /** 扫描时间 */
  scan_time: string;
  /** 扫描状态：success / partial / failed 等 */
  status: string;
}

/** 扫描结果（对齐 core::models::ScanResult） */
export interface ScanResult {
  /** 本次扫描识别出的项目（已 upsert 入库） */
  projects: Project[];
  /** 本次扫描写入的扫描历史记录 */
  history: ScanHistory;
  /** 本次扫描识别出的项目数量 */
  scanned_count: number;
  /** 本次扫描忽略的目录数量 */
  ignored_count: number;
}

/** 项目记忆（对齐 core::models::ProjectMemory，`.ydevsphere/project.json`） */
export interface ProjectMemory {
  name: string;
  /** 技术栈列表：language + framework 合并去重（language 优先） */
  stack: string[];
  /** 包管理器（由 lockfile 检测；无 lockfile 时可能为 null） */
  packageManager: string | null;
}
