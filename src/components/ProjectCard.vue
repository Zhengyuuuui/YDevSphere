<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { Project } from "@/types";
import TechnologyBadge from "./TechnologyBadge.vue";
import OpenActions from "./OpenActions.vue";
import { useEditorStore } from "@/stores/editor";
import { formatDateTime } from "@/lib/format";
import { splitTechs, stackTechnologies, techNameOf } from "@/lib/tech";

const props = defineProps<{
  project: Project;
  /** 是否已启用项目记忆（可选，用于卡片标识） */
  hasMemory?: boolean;
  /** 当前 git 分支（可选，避免卡片批量拉取开销，由上层按需提供） */
  branch?: string | null;
}>();

const { t } = useI18n();
const editorStore = useEditorStore();

/**
 * 卡片技术栈（Spec §7.1）：
 * - 架构级技术（Language/Runtime/Framework/Database/BuildTool/PackageManager/Platform）逐个展示。
 * - Library 折叠为 `+N libraries`（前端过滤，后端存储层不动）。
 * - technologies 为空 → 回退 language/framework（旧数据兼容）。
 */
const stack = computed(() => {
  const techs = stackTechnologies(props.project);
  const { architecture, libraries } = splitTechs(techs);
  return {
    architecture,
    librariesCount: libraries.length,
  };
});

/** 是否无任何技术栈（展示空态） */
const empty = computed(() => stack.value.architecture.length === 0);

/** 双击卡片：用默认编辑器打开项目 */
async function onCardDblclick() {
  await editorStore.openEditor(props.project.id, null);
}
</script>

<template>
  <RouterLink
    :to="`/project/${project.id}`"
    class="group block rounded-lg border border-line-3 bg-surface p-4 shadow-sm transition hover:border-primary hover:shadow-md"
    @dblclick.prevent="onCardDblclick"
  >
    <div class="flex items-start justify-between">
      <div class="min-w-0">
        <div class="flex items-center gap-2">
          <span class="truncate font-medium text-ink">{{ project.name }}</span>
          <span
            v-if="hasMemory"
            :title="t('card.memoryEnabled')"
            class="inline-flex h-5 w-5 items-center justify-center rounded-full bg-green-100 text-xs text-green-700"
          >
            ✓
          </span>
        </div>
        <div class="mt-1 flex items-center gap-2">
          <span class="truncate text-sm text-muted" :title="project.path">{{ project.path }}</span>
          <span
            v-if="branch"
            :title="t('card.gitBranch')"
            class="inline-flex shrink-0 items-center gap-0.5 rounded bg-surface-2 px-1.5 py-0.5 text-xs text-muted"
          >
            ⎇ {{ branch }}
          </span>
        </div>
      </div>
      <!-- 打开按钮 + 下拉菜单 -->
      <OpenActions :project-id="project.id" />
    </div>
    <div class="mt-3 flex flex-wrap items-center gap-2">
      <!-- 架构级技术逐个展示（Spec §7.1） -->
      <TechnologyBadge
        v-for="tech in stack.architecture"
        :key="tech.id"
        :label="techNameOf(tech)"
      />
      <!-- Library 折叠为 +N libraries -->
      <span
        v-if="stack.librariesCount > 0"
        class="inline-flex items-center rounded-[4px] bg-surface-2 px-[7px] py-[2px] text-[11px] font-medium leading-[18px] text-muted"
      >
        {{ t("card.librariesSuffix", { count: stack.librariesCount }) }}
      </span>
      <span
        v-if="empty"
        class="text-xs text-faint"
      >
        {{ t("card.noTech") }}
      </span>
    </div>
    <div v-if="project.updated_at" class="mt-3 text-xs text-faint">
      {{ t("card.updatedAt", { time: formatDateTime(project.updated_at) }) }}
    </div>
  </RouterLink>
</template>
