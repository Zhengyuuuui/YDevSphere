import { defineStore } from "pinia";
import { ref, watch } from "vue";

/**
 * 主题模式（v0.3）：
 *   light  → 纯白/浅色背景
 *   dark   → 黑色背景（字体颜色同步适配）
 *   system → 跟随系统 prefers-color-scheme
 *
 * 解析后的实际主题写到 <html data-theme="light|dark">，样式层据此切换。
 * 纯前端实现；偏好持久化到 localStorage（不涉后端，不触碰 SQLite）。
 */

export type ThemeMode = "light" | "dark" | "system";

const STORAGE_KEY = "ydevsphere.theme";

/** 读取已保存的主题偏好；无则默认跟随系统 */
function loadSaved(): ThemeMode {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "light" || v === "dark" || v === "system") return v;
  } catch {
    /* localStorage 不可用时回退 system */
  }
  return "system";
}

const mq = typeof window !== "undefined"
  ? window.matchMedia("(prefers-color-scheme: dark)")
  : null;

/** 系统当前是否暗色 */
function systemIsDark(): boolean {
  return mq ? mq.matches : false;
}

export const useThemeStore = defineStore("theme", () => {
  const mode = ref<ThemeMode>(loadSaved());
  /** 解析后的实际主题（light | dark），供 <html data-theme> 与组件判断 */
  const resolved = ref<"light" | "dark">(mode.value === "system"
    ? (systemIsDark() ? "dark" : "light")
    : mode.value);

  /** 应用解析后主题到 <html data-theme> */
  function apply() {
    document.documentElement.dataset.theme = resolved.value;
  }

  /** 跟随系统暗色变化的监听（system 模式时） */
  let systemListener: ((e: MediaQueryListEvent) => void) | null = null;

  function bindSystem() {
    if (mq && !systemListener) {
      systemListener = (e: MediaQueryListEvent) => {
        if (mode.value === "system") {
          resolved.value = e.matches ? "dark" : "light";
          apply();
        }
      };
      mq.addEventListener("change", systemListener);
    }
  }

  function unbindSystem() {
    if (mq && systemListener) {
      mq.removeEventListener("change", systemListener);
      systemListener = null;
    }
  }

  /** 设置主题模式 */
  function setMode(m: ThemeMode) {
    mode.value = m;
    try {
      localStorage.setItem(STORAGE_KEY, m);
    } catch {
      /* 忽略 */
    }
    resolved.value = m === "system" ? (systemIsDark() ? "dark" : "light") : m;
    apply();
    if (m === "system") bindSystem();
    else unbindSystem();
  }

  /** 初始化：恢复偏好并监听系统变化 */
  function init() {
    resolved.value = mode.value === "system"
      ? (systemIsDark() ? "dark" : "light")
      : mode.value;
    apply();
    if (mode.value === "system") bindSystem();
  }

  // 组件内响应式切换时同步（setMode 已处理；此 watch 兜底直接改 mode 的场景）
  watch(resolved, () => apply());

  return { mode, resolved, init, setMode };
});
