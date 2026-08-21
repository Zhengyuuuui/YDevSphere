<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { GitStatusView } from "@/types/view";

const props = defineProps<{
  status: GitStatusView;
  /** dirty 时的变更文件数（可选） */
  changes?: number;
}>();

const { t } = useI18n();

interface Style {
  dot: string;
  text: string;
  label: string;
}

/** 深色主题判断（dark 下状态色提亮，保证可读） */
const isDark = () =>
  typeof document !== "undefined" && document.documentElement.dataset.theme === "dark";

const style = computed<Style>(() => {
  const dark = isDark();
  switch (props.status) {
    case "clean":
      return { dot: dark ? "#4ade80" : "#16A34A", text: dark ? "#4ade80" : "#16A34A", label: t("git.clean") };
    case "dirty":
      return {
        dot: dark ? "#fbbf24" : "#D97706",
        text: dark ? "#fbbf24" : "#D97706",
        label: `${props.changes ?? 0} ${t("git.changed")}`,
      };
    case "detached":
      return { dot: dark ? "#f87171" : "#DC2626", text: dark ? "#f87171" : "#DC2626", label: t("git.detached") };
    default:
      return { dot: dark ? "#6b7280" : "#9CA3AF", text: dark ? "#6b7280" : "#9CA3AF", label: "—" };
  }
});
</script>

<template>
  <span
    v-if="status === 'none'"
    class="text-[13px] text-faint"
  >
    —
  </span>
  <span
    v-else
    class="flex items-center gap-1.5 text-[13px]"
    :style="{ color: style.text }"
  >
    <span
      class="h-[6px] w-[6px] shrink-0 rounded-full"
      :style="{ backgroundColor: style.dot }"
    />
    {{ style.label }}
  </span>
</template>
