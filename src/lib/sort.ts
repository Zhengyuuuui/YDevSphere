import type { Project } from "@/types";

/** 排序方式 */
export type SortMode = "updated" | "name";

/** 按指定方式排序项目（不修改原数组） */
export function sortProjects(list: Project[], mode: SortMode): Project[] {
  const sorted = [...list];
  if (mode === "name") {
    sorted.sort((a, b) => a.name.localeCompare(b.name, "zh-Hans-CN"));
  } else {
    // 默认按最近更新；无更新时间者排最后
    sorted.sort((a, b) => {
      const at = a.updated_at ? new Date(a.updated_at).getTime() : 0;
      const bt = b.updated_at ? new Date(b.updated_at).getTime() : 0;
      return bt - at;
    });
  }
  return sorted;
}
