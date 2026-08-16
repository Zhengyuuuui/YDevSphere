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

const style = computed<Style>(() => {
  switch (props.status) {
    case "clean":
      return { dot: "#16A34A", text: "#16A34A", label: t("git.clean") };
    case "dirty":
      return {
        dot: "#D97706",
        text: "#D97706",
        label: `${props.changes ?? 0} ${t("git.changed")}`,
      };
    case "detached":
      return { dot: "#DC2626", text: "#DC2626", label: t("git.detached") };
    default:
      return { dot: "#9CA3AF", text: "#9CA3AF", label: "—" };
  }
});
</script>

<template>
  <span
    v-if="status === 'none'"
    class="text-[12px] text-[#9CA3AF]"
  >
    —
  </span>
  <span
    v-else
    class="flex items-center gap-1.5 text-[12px]"
    :style="{ color: style.text }"
  >
    <span
      class="h-[6px] w-[6px] shrink-0 rounded-full"
      :style="{ backgroundColor: style.dot }"
    />
    {{ style.label }}
  </span>
</template>
