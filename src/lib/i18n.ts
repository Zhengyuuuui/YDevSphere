import { createI18n } from "vue-i18n";
import { messages, defaultLng, supportedLngs } from "@/locales";

/**
 * 全局 vue-i18n 实例。
 *
 * - 组件内用 `useI18n()` 的 `$t`；非组件（store / lib / 纯函数）用 `i18n.global.t`。
 * - locale 状态由 `stores/i18n.ts` 管理并联动本实例。
 */
export const i18n = createI18n({
  legacy: false,
  locale: defaultLng,
  fallbackLocale: defaultLng,
  messages,
  availableLocales: [...supportedLngs],
});

/** 非组件环境下取翻译（纯函数 / store 内使用） */
export function t(key: string, params?: Record<string, unknown>): string {
  return i18n.global.t(key, params as never);
}
