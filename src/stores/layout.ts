import { defineStore } from "pinia";
import { ref } from "vue";

/**
 * App Layout Mode（v0.3 定稿）：全局响应式布局状态。
 *
 * 架构原则：Layout Mode 只影响 UI，不影响数据模型——
 * 窗口变化不会触发重新扫描、不改任何 SQLite 查询字段。
 *
 * 判定（基于 window.innerWidth）：
 *   width >= 1072 → large
 *   width >= 852  → medium
 *   否则          → small
 */

export type AppMode = "large" | "medium" | "small";

/** 模式断点（px） */
export const LAYOUT_BREAKPOINTS = {
  large: 1072,
  medium: 852,
} as const;

export const useLayoutStore = defineStore("layout", () => {
  const appMode = ref<AppMode>(resolveMode(window.innerWidth));

  /** 由窗口宽度解析布局模式 */
  function setWidth(width: number) {
    appMode.value = resolveMode(width);
  }

  return {
    appMode,
    setWidth,
  };
});

/** 由窗口宽度判定布局模式 */
export function resolveMode(width: number): AppMode {
  if (width >= LAYOUT_BREAKPOINTS.large) return "large";
  if (width >= LAYOUT_BREAKPOINTS.medium) return "medium";
  return "small";
}
