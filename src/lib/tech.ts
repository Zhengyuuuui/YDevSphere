import type { Project, Technology, TechnologyCategory } from "@/types";

/**
 * 技术栈展示工具（Spec §7.1/§7.2，PR4）。
 *
 * - 架构级 category（卡片展示）：Language / Runtime / Framework / Database /
 *   BuildTool / PackageManager / Platform。
 * - Library（非架构级）折叠为 `+N libraries`（**前端过滤**，后端存储层不动）。
 * - 旧数据 fallback：`technologies` 为空时回退 `language` / `framework`。
 */

/** 架构级 category 集合（Spec §7.1：Library 折叠） */
const ARCHITECTURE_CATEGORIES: ReadonlySet<TechnologyCategory> = new Set([
  "language",
  "runtime",
  "framework",
  "database",
  "build_tool",
  "package_manager",
  "platform",
]);

/** 判断是否为架构级技术（非 library 即架构级）。 */
export function isArchitectureTech(t: Technology): boolean {
  return ARCHITECTURE_CATEGORIES.has(t.category);
}

/** 拆分为「架构级 + libraries」两组（保持原有相对顺序）。 */
export function splitTechs(
  technologies: Technology[]
): { architecture: Technology[]; libraries: Technology[] } {
  const architecture: Technology[] = [];
  const libraries: Technology[] = [];
  for (const t of technologies) {
    if (isArchitectureTech(t)) architecture.push(t);
    else libraries.push(t);
  }
  return { architecture, libraries };
}

/** 技术展示名（name 优先，缺省回退 id）。 */
export function techNameOf(t: Technology): string {
  return t.name || t.id;
}

/**
 * 项目技术栈展示列表：优先 `technologies`，为空时回退旧字段
 * `language` / `framework`（Spec §7.2 旧数据 fallback）。
 */
export function stackTechnologies(p: Project): Technology[] {
  if (p.technologies && p.technologies.length > 0) return p.technologies;
  // 旧数据：由 language / framework 合成「伪 Technology」（category 未知 → library，
  // 仅供展示；不会落库）。
  const out: Technology[] = [];
  for (const label of [p.language, p.framework]) {
    if (label) out.push({ id: label, name: label, category: "library", ecosystem: null });
  }
  return out;
}
