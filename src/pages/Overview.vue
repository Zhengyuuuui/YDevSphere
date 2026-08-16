<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import StatCard from "@/components/StatCard.vue";
import ActivityChart from "@/components/ActivityChart.vue";
import TechStackList from "@/components/TechStackList.vue";
import { useProjectStore } from "@/stores/project";
import { useGitStore } from "@/stores/git";
import { toGitStatusView } from "@/lib/view";

const router = useRouter();
const { t } = useI18n();
const projectStore = useProjectStore();
const gitStore = useGitStore();

onMounted(() => {
  projectStore.fetchProjects();
});

/** 统计卡（来自真实项目列表） */
const totalProjects = computed(() => projectStore.projects.length);

/** Git 仓库数（已缓存的项目里 is_git_repo 为 true 的数量） */
const repoCount = computed(() => {
  let n = 0;
  for (const p of projectStore.projects) {
    const info = gitStore.infoOf(p.id);
    if (info?.is_git_repo) n++;
  }
  return n;
});

/** Git 干净项目数（已缓存里 Clean 的数量） */
const cleanCount = computed(() => {
  let n = 0;
  for (const p of projectStore.projects) {
    const info = gitStore.infoOf(p.id);
    if (info && toGitStatusView(info) === "clean") n++;
  }
  return n;
});

/** 技术栈分布（由真实项目的 language/framework 统计，非 mock） */
const TECH_COLORS = [
  "#3178C6",
  "#F7DF1E",
  "#CE422B",
  "#3776AB",
  "#059669",
  "#4338CA",
  "#D97706",
  "#0E7490",
];

const techStack = computed(() => {
  const map = new Map<string, number>();
  for (const p of projectStore.projects) {
    for (const t of [p.language, p.framework]) {
      if (!t) continue;
      map.set(t, (map.get(t) ?? 0) + 1);
    }
  }
  const sorted = [...map.entries()].sort((a, b) => b[1] - a[1]);
  return sorted.slice(0, 8).map(([name, count], i) => ({
    name,
    count,
    color: TECH_COLORS[i % TECH_COLORS.length],
  }));
});

/** 最近项目（复用 recentProjects） */
const recentProjects = computed(() =>
  projectStore.recentProjects.slice(0, 4).map((p) => {
    const tech = [p.language, p.framework].filter(Boolean).join(" · ");
    return { id: p.id, name: p.name, tech, timeAgo: "" };
  })
);

function greeting(): string {
  const h = new Date().getHours();
  if (h < 12) return t("overview.morning");
  if (h < 17) return t("overview.afternoon");
  return t("overview.evening");
}

function goProjects() {
  router.push("/projects");
}

function goDetail(id: number) {
  router.push(`/project/${id}`);
}
</script>

<template>
  <div class="min-h-full bg-[#F7F8FA]">
    <div class="mx-auto max-w-[1060px] px-8 py-8">
      <!-- Greeting -->
      <div class="mb-7">
        <h1 class="text-[22px] font-semibold leading-tight tracking-tight text-[#17191C]">
          {{ greeting() }}
        </h1>
        <p class="mt-1 text-[13px] text-[#9CA3AF]">
          {{ t("overview.subtitle") }}
        </p>
      </div>

      <!-- 统计行 -->
      <div class="mb-7 flex items-center gap-8 border-b border-[#EAEDF0] pb-7">
        <StatCard :value="totalProjects" :label="t('overview.statProjects')" />
        <div class="h-8 w-px bg-[#EAEDF0]" />
        <StatCard :value="repoCount" :label="t('overview.statRepos')" />
        <div class="h-8 w-px bg-[#EAEDF0]" />
        <StatCard :value="cleanCount" :label="t('overview.statClean')" />
      </div>

      <!-- 两列：活动图 + 技术栈分布 -->
      <div class="mb-5 grid grid-cols-[1fr_260px] gap-5">
        <!-- 活动图（mock） -->
        <div class="rounded-[10px] border border-[#EAEDF0] bg-white p-5">
          <div class="mb-4 flex items-center justify-between">
            <span class="text-[13px] font-semibold text-[#17191C]">{{ t("overview.activity") }}</span>
            <span class="text-[12px] text-[#9CA3AF]">{{ t("overview.activitySub") }}</span>
          </div>
          <ActivityChart />
        </div>

        <!-- 技术栈分布（真实数据） -->
        <div class="rounded-[10px] border border-[#EAEDF0] bg-white p-5">
          <span class="mb-4 block text-[13px] font-semibold text-[#17191C]">{{ t("overview.techStack") }}</span>
          <TechStackList :items="techStack" />
          <p class="mt-3 text-[11px] text-[#B0B7C3]">
            {{ t("overview.techStackNote") }}
          </p>
        </div>
      </div>

      <!-- 最近项目 -->
      <div class="rounded-[10px] border border-[#EAEDF0] bg-white">
        <div class="flex items-center justify-between border-b border-[#F3F4F6] px-5 py-4">
          <span class="text-[13px] font-semibold text-[#17191C]">{{ t("overview.recentProjects") }}</span>
          <button
            class="flex items-center gap-1 text-[12px] text-[#6B7280] transition-colors hover:text-[#2563EB]"
            @click="goProjects"
          >
            {{ t("overview.viewAll") }}
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M5 12h14" />
              <path d="M12 5l7 7-7 7" />
            </svg>
          </button>
        </div>

        <div class="divide-y divide-[#F3F4F6]">
          <div
            v-for="r in recentProjects"
            :key="r.id"
            class="flex cursor-pointer items-center justify-between px-5 py-3 transition-colors hover:bg-[#FAFAFA]"
            @click="goDetail(r.id)"
          >
            <div class="flex flex-col gap-0.5">
              <span class="text-[13px] font-medium text-[#17191C]">{{ r.name }}</span>
              <span class="text-[11px] text-[#9CA3AF]">{{ r.tech || t("card.noTech") }}</span>
            </div>
            <span class="shrink-0 text-[12px] tabular-nums text-[#B0B7C3]">{{ r.timeAgo }}</span>
          </div>
          <div
            v-if="recentProjects.length === 0"
            class="px-5 py-8 text-center text-[13px] text-[#9CA3AF]"
          >
            {{ t("overview.noRecent") }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
