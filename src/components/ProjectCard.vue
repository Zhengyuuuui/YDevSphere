<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { Project } from "@/types";
import TechnologyBadge from "./TechnologyBadge.vue";
import OpenActions from "./OpenActions.vue";
import { useEditorStore } from "@/stores/editor";
import { formatDateTime } from "@/lib/format";

const props = defineProps<{
  project: Project;
  /** 是否已启用项目记忆（可选，用于卡片标识） */
  hasMemory?: boolean;
  /** 当前 git 分支（可选，避免卡片批量拉取开销，由上层按需提供） */
  branch?: string | null;
}>();

const { t } = useI18n();
const editorStore = useEditorStore();

/** 双击卡片：用默认编辑器打开项目 */
async function onCardDblclick() {
  await editorStore.openEditor(props.project.id, null);
}
</script>

<template>
  <RouterLink
    :to="`/project/${project.id}`"
    class="group block rounded-lg border border-gray-200 bg-white p-4 shadow-sm transition hover:border-blue-200 hover:shadow-md"
    @dblclick.prevent="onCardDblclick"
  >
    <div class="flex items-start justify-between">
      <div class="min-w-0">
        <div class="flex items-center gap-2">
          <span class="truncate font-medium text-gray-900">{{ project.name }}</span>
          <span
            v-if="hasMemory"
            :title="t('card.memoryEnabled')"
            class="inline-flex h-5 w-5 items-center justify-center rounded-full bg-green-100 text-xs text-green-700"
          >
            ✓
          </span>
        </div>
        <div class="mt-1 flex items-center gap-2">
          <span class="truncate text-sm text-gray-500" :title="project.path">{{ project.path }}</span>
          <span
            v-if="branch"
            :title="t('card.gitBranch')"
            class="inline-flex shrink-0 items-center gap-0.5 rounded bg-gray-100 px-1.5 py-0.5 text-xs text-gray-600"
          >
            ⎇ {{ branch }}
          </span>
        </div>
      </div>
      <!-- 打开按钮 + 下拉菜单 -->
      <OpenActions :project-id="project.id" />
    </div>
    <div class="mt-3 flex flex-wrap items-center gap-2">
      <TechnologyBadge :label="project.language" />
      <TechnologyBadge :label="project.framework" />
      <span
        v-if="!project.language && !project.framework"
        class="text-xs text-gray-400"
      >
        {{ t("card.noTech") }}
      </span>
    </div>
    <div v-if="project.updated_at" class="mt-3 text-xs text-gray-400">
      {{ t("card.updatedAt", { time: formatDateTime(project.updated_at) }) }}
    </div>
  </RouterLink>
</template>
