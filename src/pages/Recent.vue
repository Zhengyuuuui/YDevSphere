<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import ProjectTable from "@/components/ProjectTable.vue";
import { useProjectStore } from "@/stores/project";
import { useGitStore } from "@/stores/git";
import { toProjectView } from "@/lib/view";
import { getRecentOpenedAt } from "@/lib/recent";

const { t } = useI18n();
const projectStore = useProjectStore();
const gitStore = useGitStore();

onMounted(() => {
  if (projectStore.projects.length === 0) {
    projectStore.fetchProjects();
  }
});

/** 最近打开项目（复用 recentProjects，最多 8 条） */
const recent = computed(() => {
  const list = projectStore.recentProjects.slice(0, 8);
  return list.map((p) => {
    const ts = getRecentOpenedAt(p.id);
    const info = gitStore.infoOf(p.id);
    return {
      ...toProjectView(p, info),
      lastOpenedAt: ts ? new Date(ts).toLocaleString() : null,
    };
  });
});
</script>

<template>
  <div class="min-h-full bg-[#F7F8FA]">
    <div class="mx-auto max-w-[1140px] px-8 py-7">
      <div class="mb-5">
        <h1 class="text-[22px] font-semibold leading-tight tracking-tight text-[#17191C]">
          {{ t("recent.title") }}
        </h1>
        <p class="mt-1 text-[13px] text-[#9CA3AF]">{{ t("recent.subtitle") }}</p>
      </div>

      <div v-if="recent.length > 0" class="mb-2 px-1">
        <span class="text-[10px] font-semibold uppercase tracking-[0.09em] text-[#B0B7C3]">
          {{ t("recent.recentlyOpened") }}
        </span>
      </div>

      <div class="rounded-[8px] border border-[#E5E7EB] bg-white">
        <div v-if="recent.length === 0" class="py-20 text-center">
          <p class="mb-1.5 text-[15px] font-semibold text-[#17191C]">{{ t("recent.empty") }}</p>
          <p class="text-[13px] text-[#9CA3AF]">{{ t("recent.emptyHint") }}</p>
        </div>
        <ProjectTable v-else :projects="recent" :show-last-opened="true" />
      </div>
    </div>
  </div>
</template>
