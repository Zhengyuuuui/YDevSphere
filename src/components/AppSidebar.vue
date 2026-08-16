<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores/settings";
import { useProjectStore } from "@/stores/project";
import { useLayoutStore } from "@/stores/layout";

const route = useRoute();
const router = useRouter();
const settings = useSettingsStore();
const projectStore = useProjectStore();
const layout = useLayoutStore();
const { t } = useI18n();

/** 是否为 Icon-only（small 模式） */
const isCollapsed = computed(() => layout.appMode === "small");

/** 当前激活的导航 key（用于高亮） */
const activeKey = computed(() => {
  const name = route.name;
  if (name === "overview") return "overview";
  if (name === "projects" || name === "project-detail") return "projects";
  if (name === "recent") return "recent";
  if (name === "settings") return "settings";
  return "";
});

/** 工作区显示名（多工作区时显示数量，单个时显示路径最后一段） */
const workspaceName = computed(() => {
  const ws = settings.workspaces;
  if (ws.length === 0) return t("workspace.none");
  if (ws.length > 1) return t("workspace.multiple", { count: ws.length });
  const segs = ws[0].split(/[\\/]/).filter(Boolean);
  return segs[segs.length - 1] || ws[0];
});

const navItems = computed(() => [
  { key: "overview", label: t("nav.overview"), to: "/overview" },
  { key: "projects", label: t("nav.projects"), to: "/projects" },
  { key: "recent", label: t("nav.recent"), to: "/recent" },
]);

function navigate(to: string) {
  router.push(to);
}
</script>

<template>
  <aside
    class="flex h-full shrink-0 select-none flex-col border-r border-[#E5E7EB] bg-white transition-[width] duration-150 ease-out"
    :class="isCollapsed ? 'w-[72px]' : 'w-[220px]'"
  >
    <!-- Logo -->
    <div
      class="flex items-center gap-2.5 px-4 pb-3 pt-[18px]"
      :class="isCollapsed ? 'justify-center px-0' : ''"
    >
      <img
        src="/logo.png"
        alt="YDevSphere"
        class="h-5 w-5 rounded-[5px] object-contain"
        :title="isCollapsed ? 'YDevSphere' : undefined"
      />
      <span
        v-if="!isCollapsed"
        class="text-[13px] font-semibold tracking-tight text-[#17191C]"
      >
        YDevSphere
      </span>
    </div>

    <div class="mx-3 h-px bg-[#F0F1F3]" />

    <!-- Navigation -->
    <nav class="flex-1 overflow-y-auto px-2 pt-1">
      <div v-if="!isCollapsed" class="px-3 pb-1 pt-3">
        <span class="text-[10px] font-semibold uppercase tracking-[0.1em] text-[#C4C9D0]">
          {{ t("nav.main") }}
        </span>
      </div>

      <div :class="isCollapsed ? 'mt-2 space-y-1' : 'space-y-0.5'">
        <button
          v-for="item in navItems"
          :key="item.key"
          class="flex w-full items-center gap-2.5 rounded-[6px] text-left text-[13px] transition-colors duration-75"
          :class="[
            isCollapsed ? 'justify-center px-0 py-[9px]' : 'px-3 py-[7px]',
            activeKey === item.key
              ? 'bg-[#EEF2FF] font-medium text-[#2563EB]'
              : 'text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#374151]',
          ]"
          :title="isCollapsed ? item.label : undefined"
          @click="navigate(item.to)"
        >
          <!-- icon: overview -->
          <svg
            v-if="item.key === 'overview'"
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="shrink-0"
          >
            <rect x="3" y="3" width="7" height="9" rx="1" />
            <rect x="14" y="3" width="7" height="5" rx="1" />
            <rect x="14" y="12" width="7" height="9" rx="1" />
            <rect x="3" y="16" width="7" height="5" rx="1" />
          </svg>
          <!-- icon: projects -->
          <svg
            v-else-if="item.key === 'projects'"
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="shrink-0"
          >
            <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z" />
          </svg>
          <!-- icon: recent -->
          <svg
            v-else-if="item.key === 'recent'"
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="shrink-0"
          >
            <circle cx="12" cy="12" r="9" />
            <path d="M12 7v5l3 2" />
          </svg>
          <span
            v-if="!isCollapsed"
            :class="activeKey === item.key ? 'text-[#2563EB]' : 'text-[#9CA3AF]'"
          >
            {{ item.label }}
          </span>
        </button>
      </div>

      <div v-if="!isCollapsed" class="px-3 pb-1 pt-3">
        <span class="text-[10px] font-semibold uppercase tracking-[0.1em] text-[#C4C9D0]">
          {{ t("nav.workspace") }}
        </span>
      </div>

      <button
        class="mt-2 flex w-full items-center gap-2.5 rounded-[6px] text-left text-[13px] text-[#6B7280] transition-colors duration-75 hover:bg-[#F3F4F6] hover:text-[#374151]"
        :class="isCollapsed ? 'relative justify-center px-0 py-[9px]' : 'px-3 py-[7px]'"
        :title="isCollapsed ? `${t('nav.workspace')}: ${workspaceName}` : t('workspace.projectCount')"
        @click="navigate('/projects')"
      >
        <svg
          width="15"
          height="15"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="shrink-0 text-[#9CA3AF]"
        >
          <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z" />
        </svg>

        <!-- 展开态：名称 + 计数 -->
        <template v-if="!isCollapsed">
          <span class="flex-1 truncate">{{ workspaceName }}</span>
          <span class="shrink-0 text-[11px] text-[#C4C9D0]">
            {{ projectStore.projects.length }}
          </span>
        </template>
        <!-- 收起态：项目数角标 -->
        <span
          v-else
          class="absolute right-[9px] top-[3px] flex h-[14px] min-w-[14px] items-center justify-center rounded-full bg-[#EEF2FF] px-[3px] text-[9px] font-semibold leading-none text-[#2563EB]"
        >
          {{ projectStore.projects.length }}
        </span>
      </button>
    </nav>

    <!-- Settings -->
    <div class="border-t border-[#F0F1F3] px-2 pb-3 pt-2">
      <button
        class="flex w-full items-center gap-2.5 rounded-[6px] text-left text-[13px] transition-colors duration-75"
        :class="[
          isCollapsed ? 'justify-center px-0 py-[9px]' : 'px-3 py-[7px]',
          activeKey === 'settings'
            ? 'bg-[#EEF2FF] font-medium text-[#2563EB]'
            : 'text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#374151]',
        ]"
        :title="isCollapsed ? t('nav.settings') : undefined"
        @click="navigate('/settings')"
      >
        <svg
          width="15"
          height="15"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="shrink-0"
          :class="activeKey === 'settings' ? 'text-[#2563EB]' : 'text-[#9CA3AF]'"
        >
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
        <span
          v-if="!isCollapsed"
          :class="activeKey === 'settings' ? 'text-[#2563EB]' : 'text-[#9CA3AF]'"
        >
          {{ t("nav.settings") }}
        </span>
      </button>
    </div>
  </aside>
</template>
