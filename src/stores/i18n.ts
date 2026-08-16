import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { i18n } from "@/lib/i18n";
import { supportedLngs, defaultLng, type SupportedLng } from "@/locales";
import {
  getLanguagePreference,
  setLanguagePreference,
} from "@/api/language";

/** 本地持久化键（后端语言接口未就绪时的降级存储） */
const LEGACY_LANG_KEY = "ydevsphere.language";

function isSupported(lng: unknown): lng is SupportedLng {
  return typeof lng === "string" && (supportedLngs as readonly string[]).includes(lng);
}

function readLocalLang(): SupportedLng | null {
  try {
    const raw = localStorage.getItem(LEGACY_LANG_KEY);
    return isSupported(raw) ? raw : null;
  } catch {
    return null;
  }
}

function writeLocalLang(lng: SupportedLng) {
  try {
    localStorage.setItem(LEGACY_LANG_KEY, lng);
  } catch {
    // localStorage 不可用时静默失败
  }
}

/**
 * i18n 语言状态。
 *
 * 持久化优先级：
 * 1. 后端 `get_language_preference`（权威源，接口就绪后生效）。
 * 2. localStorage 降级（后端接口未交付时的临时方案）。
 * 3. 默认 `zh-CN`。
 */
export const useI18nStore = defineStore("i18n", () => {
  const locale = ref<SupportedLng>(defaultLng);
  /** 是否已初始化（供 App 启动时等待语言就绪） */
  const ready = ref(false);

  const isZh = computed(() => locale.value === "zh-CN");

  /** 应用语言到 vue-i18n 实例 */
  function apply(lng: SupportedLng) {
    locale.value = lng;
    i18n.global.locale.value = lng;
    document.documentElement.lang = lng;
  }

  /** 切换语言：更新状态 + 持久化（后端优先，失败降级 localStorage） */
  async function setLocale(lng: SupportedLng) {
    apply(lng);
    // 后端接口就绪则持久化到后端；失败（接口未交付）则降级 localStorage
    try {
      await setLanguagePreference(lng);
      writeLocalLang(lng);
    } catch {
      writeLocalLang(lng);
    }
  }

  /** 启动时初始化：读后端偏好，无则读 localStorage，再无则默认 */
  async function init() {
    try {
      const backend = await getLanguagePreference();
      if (isSupported(backend)) {
        apply(backend);
        ready.value = true;
        return;
      }
    } catch {
      // 后端接口未就绪，忽略
    }
    const local = readLocalLang();
    apply(local ?? defaultLng);
    ready.value = true;
  }

  return {
    locale,
    isZh,
    ready,
    setLocale,
    init,
  };
});
