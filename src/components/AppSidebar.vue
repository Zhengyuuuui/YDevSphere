<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores/settings";
import { useLayoutStore } from "@/stores/layout";

const route = useRoute();
const router = useRouter();
const settings = useSettingsStore();
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

/** 路径是否含指定目录段（如 .../Documents 或以 .../Documents/ 开头） */
function hasSegment(path: string, name: string): boolean {
  const segs = path.split(/[\\/]/).filter(Boolean);
  return segs.includes(name);
}

/** 侧边栏工作区项 */
interface WorkspaceItem {
  key: string;
  label: string;
  count: number;
}

/**
 * 工作区分组（v0.4）。
 * 将 settings.workspaces（路径列表）按路径分类为 Documents / Desktop 两组，
 * 其他手动路径本轮不展示（后续自定义工作区）。
 */
const workspaceGroups = computed<WorkspaceItem[]>(() => {
  const ws = settings.workspaces;
  const groups: WorkspaceItem[] = [];
  const docs = ws.filter((p) => hasSegment(p, "Documents"));
  const desktop = ws.filter((p) => hasSegment(p, "Desktop"));
  if (docs.length > 0) groups.push({ key: "documents", label: t("workspace.documents"), count: docs.length });
  if (desktop.length > 0) groups.push({ key: "desktop", label: t("workspace.desktop"), count: desktop.length });
  return groups;
});

/** 渲染用：无已识别工作区时回退为空态（避免破版） */
const workspaceItems = computed<WorkspaceItem[]>(() =>
  workspaceGroups.value.length > 0 ? workspaceGroups.value : [{ key: "none", label: t("workspace.none"), count: 0 }],
);

const navItems = computed(() => [
  { key: "overview", label: t("nav.overview"), to: "/overview" },
  { key: "projects", label: t("nav.projects"), to: "/projects" },
  { key: "recent", label: t("nav.recent"), to: "/recent" },
]);

function navigate(to: string) {
  router.push(to);
}

/**
 * 打开某工作区分组 → 进入 /projects 并应用对应分类筛选。
 * 无分类的占位项（none）仍进入聚合 /projects。
 */
function openWorkspace(item: WorkspaceItem) {
  if (item.key === "documents" || item.key === "desktop") {
    router.push({ path: "/projects", query: { workspace: item.key } });
  } else {
    router.push({ path: "/projects", query: {} });
  }
}
</script>

<template>
  <aside
    class="flex h-full shrink-0 select-none flex-col bg-surface transition-[width] duration-150 ease-out"
    :style="{
      boxShadow: 'inset -1px 0 0 var(--color-line-3)',
    }"
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
        class="font-display text-[15px] font-semibold tracking-tight text-ink"
      >
        YDevSphere
      </span>
    </div>

    <div class="mx-3 h-px bg-line-2" />

    <!-- Navigation -->
    <nav class="flex-1 overflow-y-auto px-2 pt-1">
      <div v-if="!isCollapsed" class="px-3 pb-1 pt-3">
        <span class="text-[10px] font-semibold uppercase tracking-[0.1em] text-fainter">
          {{ t("nav.main") }}
        </span>
      </div>

      <div :class="isCollapsed ? 'mt-2 space-y-1' : 'space-y-0.5'">
        <button
          v-for="item in navItems"
          :key="item.key"
          class="flex w-full items-center gap-2.5 rounded-[6px] text-left text-[14px] transition-colors duration-75"
          :class="[
            isCollapsed ? 'justify-center px-0 py-[9px]' : 'px-3 py-[7px]',
            activeKey === item.key
              ? 'bg-primary-soft font-medium text-primary'
              : 'text-muted hover:bg-surface-2 hover:text-ink',
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
            class="font-display"
            :class="activeKey === item.key ? 'text-primary' : 'text-faint'"
          >
            {{ item.label }}
          </span>
        </button>
      </div>

      <div v-if="!isCollapsed" class="px-3 pb-1 pt-3">
        <span class="text-[10px] font-semibold uppercase tracking-[0.1em] text-fainter">
          {{ t("nav.workspace") }}
        </span>
      </div>

      <!-- 工作区分组项：Documents / Desktop（v0.4 按分类展示） -->
      <button
        v-for="item in workspaceItems"
        :key="item.key"
        class="mt-2 flex w-full items-center gap-2.5 rounded-[6px] text-left text-[14px] text-muted transition-colors duration-75 hover:bg-surface-2 hover:text-ink"
        :class="isCollapsed ? 'relative justify-center px-0 py-[9px]' : 'px-3 py-[7px]'"
        :title="isCollapsed ? `${t('nav.workspace')}: ${item.label}` : t('workspace.projectCount')"
        @click="openWorkspace(item)"
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
          class="shrink-0 text-faint"
        >
          <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z" />
        </svg>

        <!-- 展开态：名称 + 计数 -->
        <template v-if="!isCollapsed">
          <span class="font-display flex-1 truncate">{{ item.label }}</span>
          <span v-if="item.count > 0" class="shrink-0 text-[11px] text-fainter">
            {{ item.count }}
          </span>
        </template>
        <!-- 收起态：项目数角标 -->
        <span
          v-else-if="item.count > 0"
          class="absolute right-[9px] top-[3px] flex h-[14px] min-w-[14px] items-center justify-center rounded-full bg-primary-soft px-[3px] text-[9px] font-semibold leading-none text-primary"
        >
          {{ item.count }}
        </span>
      </button>
    </nav>

    <!-- Settings -->
    <div class="border-t border-divider px-2 pb-3 pt-2">
      <button
        class="flex w-full items-center gap-2.5 rounded-[6px] text-left text-[14px] transition-colors duration-75"
        :class="[
          isCollapsed ? 'justify-center px-0 py-[9px]' : 'px-3 py-[7px]',
          activeKey === 'settings'
            ? 'bg-primary-soft font-medium text-primary'
            : 'text-muted hover:bg-surface-2 hover:text-ink',
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
          :class="activeKey === 'settings' ? 'text-primary' : 'text-faint'"
        >
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
        <span
          v-if="!isCollapsed"
          class="font-display"
          :class="activeKey === 'settings' ? 'text-primary' : 'text-faint'"
        >
          {{ t("nav.settings") }}
        </span>
      </button>
    </div>
  </aside>
</template>
