<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  label: string | null;
}>();

/**
 * 技术栈徽标配色（对齐 Figma TechBadge 的低饱和配色）。
 * 按关键词匹配 category → 颜色；匹配不到用中性灰。
 */
interface Tone {
  bg: string;
  text: string;
}

function toneFor(name: string): Tone {
  const l = name.toLowerCase();
  if (["vue", "vue3", "nuxt"].some((k) => l.includes(k))) {
    return { bg: "#ECFDF5", text: "#059669" };
  }
  if (["react", "next", "nextjs", "svelte", "angular"].some((k) => l.includes(k))) {
    return { bg: "#EFF6FF", text: "#1D4ED8" };
  }
  if (["typescript", "ts"].some((k) => l.includes(k))) {
    return { bg: "#EEF2FF", text: "#4338CA" };
  }
  if (["javascript", "js", "node"].some((k) => l.includes(k))) {
    return { bg: "#F0FDF4", text: "#15803D" };
  }
  if (["rust", "cargo"].some((k) => l.includes(k))) {
    return { bg: "#FFF7ED", text: "#9A3412" };
  }
  if (["python", "py"].some((k) => l.includes(k))) {
    return { bg: "#FFFBEB", text: "#92400E" };
  }
  if (["go", "golang"].some((k) => l.includes(k))) {
    return { bg: "#ECFEFF", text: "#0E7490" };
  }
  if (["java", "spring", "kotlin"].some((k) => l.includes(k))) {
    return { bg: "#FEF9C3", text: "#854D0E" };
  }
  return { bg: "#F3F4F6", text: "#4B5563" };
}

const style = computed(() => {
  const tone = toneFor(props.label ?? "");
  return { backgroundColor: tone.bg, color: tone.text };
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
