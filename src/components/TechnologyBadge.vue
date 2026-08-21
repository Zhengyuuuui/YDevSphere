<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  label: string | null;
}>();

/**
 * 技术栈徽标配色（对齐 Figma TechBadge 的低饱和配色）。
 * 按关键词匹配 category → 颜色；匹配不到用中性灰。
 * 支持 light / dark 两套配色（dark 下深底浅字，保证可读性）。
 */
interface Tone {
  bg: string;
  text: string;
}
interface TonePair {
  light: Tone;
  dark: Tone;
}

function pair(light: Tone, dark: Tone): TonePair {
  return { light, dark };
}

const TONES: Record<string, TonePair> = {
  vue: pair({ bg: "#ECFDF5", text: "#059669" }, { bg: "#0f2e23", text: "#6ee7b7" }),
  react: pair({ bg: "#EFF6FF", text: "#1D4ED8" }, { bg: "#16233f", text: "#93c5fd" }),
  typescript: pair({ bg: "#EEF2FF", text: "#4338CA" }, { bg: "#1e2140", text: "#a5b4fc" }),
  javascript: pair({ bg: "#F0FDF4", text: "#15803D" }, { bg: "#0f2e1d", text: "#86efac" }),
  rust: pair({ bg: "#FFF7ED", text: "#9A3412" }, { bg: "#2e1c10", text: "#fdba74" }),
  python: pair({ bg: "#FFFBEB", text: "#92400E" }, { bg: "#2e2310", text: "#fcd34d" }),
  go: pair({ bg: "#ECFEFF", text: "#0E7490" }, { bg: "#0d2930", text: "#67e8f9" }),
  java: pair({ bg: "#FEF9C3", text: "#854D0E" }, { bg: "#2e260c", text: "#fde047" }),
};

function toneFor(name: string): TonePair {
  const l = name.toLowerCase();
  if (["vue", "vue3", "nuxt"].some((k) => l.includes(k))) return TONES.vue;
  if (["react", "next", "nextjs", "svelte", "angular"].some((k) => l.includes(k))) return TONES.react;
  if (["typescript", "ts"].some((k) => l.includes(k))) return TONES.typescript;
  if (["javascript", "js", "node"].some((k) => l.includes(k))) return TONES.javascript;
  if (["rust", "cargo"].some((k) => l.includes(k))) return TONES.rust;
  if (["python", "py"].some((k) => l.includes(k))) return TONES.python;
  if (["go", "golang"].some((k) => l.includes(k))) return TONES.go;
  if (["java", "spring", "kotlin"].some((k) => l.includes(k))) return TONES.java;
  return pair({ bg: "#F3F4F6", text: "#4B5563" }, { bg: "#2a2f35", text: "#c9d1d9" });
}

/** 当前是否深色主题（跟随 <html data-theme>） */
const isDark = computed(() =>
  typeof document !== "undefined" &&
  document.documentElement.dataset.theme === "dark"
);

const style = computed(() => {
  const tone = toneFor(props.label ?? "");
  const t = isDark.value ? tone.dark : tone.light;
  return { backgroundColor: t.bg, color: t.text };
});
</script>

<template>
  <span
    v-if="label"
    class="inline-flex items-center rounded-[4px] px-[7px] py-[2px] text-[11px] font-medium leading-[18px]"
    :style="style"
  >
    {{ label }}
  </span>
</template>
