<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

interface TechCount {
  name: string;
  count: number;
  color: string;
}

const props = defineProps<{
  /** 技术栈分布（name → count），由真实项目数据统计 */
  items: TechCount[];
}>();

const maxCount = computed(() => Math.max(...props.items.map((t) => t.count), 1));
</script>

<template>
  <div v-if="items.length === 0" class="py-6 text-center text-[14px] text-faint">
    {{ t("overview.noTech") }}
  </div>
  <div v-else class="space-y-3">
    <div v-for="t in items" :key="t.name" class="flex items-center justify-between gap-3">
      <div class="flex min-w-0 items-center gap-2">
        <span class="h-2 w-2 shrink-0 rounded-full" :style="{ background: t.color }" />
        <span class="truncate text-[14px] text-ink">{{ t.name }}</span>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <div class="h-[4px] w-[60px] overflow-hidden rounded-full bg-surface-2">
          <div
            class="h-full rounded-full opacity-70"
            :style="{
              width: `${(t.count / maxCount) * 100}%`,
              background: t.color,
            }"
          />
        </div>
        <span class="w-5 text-right text-[13px] tabular-nums text-faint">{{ t.count }}</span>
      </div>
    </div>
  </div>
</template>
