<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import ProjectTable from "@/components/ProjectTable.vue";
import EnableMemoryDialog from "@/components/EnableMemoryDialog.vue";
import { useProjectStore } from "@/stores/project";
import { useSettingsStore } from "@/stores/settings";
import { useScannerStore } from "@/stores/scanner";
import { useGitStore } from "@/stores/git";
import { useLayoutStore } from "@/stores/layout";
import { toProjectViews } from "@/lib/view";
import { formatDuration } from "@/lib/format";
import { toast } from "@/lib/toast";
import { getProjects, getSystemWorkspaces } from "@/api/project";
import type { Project } from "@/types";
import type { ProjectView } from "@/types/view";
import type { WorkspaceFilter, ProjectSortBy } from "@/api/project";
import type { SystemWorkspaceKind } from "@/types";

const router = useRouter();
const { t } = useI18n();
const projectStore = useProjectStore();
const settings = useSettingsStore();
const scanner = useScannerStore();
const gitStore = useGitStore();
const layout = useLayoutStore();

const keyword = ref("");
const workspaceFilter = ref<WorkspaceFilter>("all");
const sortMode = ref<ProjectSortBy>("updated_at");

const workspaceOpen = ref(false);
const sortOpen = ref(false);
const memoryDialogOpen = ref(false);
/** 扫描下拉展开状态（与 workspaceOpen / sortOpen 互斥） */
const scanMenuOpen = ref(false);

/**
 * 树形展开状态（v0.2）：
 * - `childrenOf`：父项目 id → 已加载的直接子项目列表。
 * - `expanded`：父项目 id → 是否展开。
 * - `loadingChildren`：正在加载子项目的父项目 id 集合。
 */
const childrenOf = reactive(new Map<number, Project[]>());
const expanded = reactive(new Set<number>());
const loadingChildren = reactive(new Set<number>());

/** 工作区筛选项 */
const filterOptions = computed<{ value: WorkspaceFilter; label: string }[]>(() => [
  { value: "all", label: t("projects.filterAll") },
  { value: "documents", label: t("workspace.documents") },
  { value: "desktop", label: t("workspace.desktop") },
]);

const sortOptions = computed<{ value: ProjectSortBy; label: string }[]>(() => [
  { value: "updated_at", label: t("projects.sortUpdated") },
  { value: "name", label: t("projects.sortName") },
]);

const currentFilterLabel = computed(
  () => filterOptions.value.find((o) => o.value === workspaceFilter.value)?.label ?? t("projects.filterAll")
);

/** 客户端搜索过滤（顶层项目） */
const filtered = computed(() => {
  const kw = keyword.value.trim().toLowerCase();
  if (!kw) return projectStore.projects;
  return projectStore.projects.filter(
    (p) => p.name.toLowerCase().includes(kw) || p.path.toLowerCase().includes(kw)
  );
});

/** 视图模型（搜索 + 排序后） */
const views = computed(() =>
  toProjectViews(filtered.value, (id) => gitStore.infoOf(id))
);

/** 某项目的直接子项目视图（树形展开用） */
function childViewsOf(parentId: number) {
  const children = childrenOf.get(parentId) ?? [];
  return toProjectViews(children, (id) => gitStore.infoOf(id));
}

/** 项目是否可展开（聚合根 / 分类目录，且可能有子项目） */
function canExpand(p: ProjectView): boolean {
  return p.kind === "aggregated_root" || p.kind === "category";
}

/** 切换展开：首次展开时按需加载直接子项目 */
async function toggleExpand(p: ProjectView) {
  if (expanded.has(p.id)) {
    expanded.delete(p.id);
    return;
  }
  expanded.add(p.id);
  if (!childrenOf.has(p.id)) {
    await loadChildren(p.id);
  }
}

/** 按需加载父项目的直接子项目 */
async function loadChildren(parentId: number) {
  loadingChildren.add(parentId);
  try {
    const children = await getProjects(
      sortMode.value,
      workspaceFilter.value,
      undefined,
      parentId
    );
    childrenOf.set(parentId, children);
  } catch (e) {
    toast.error(
      t("projects.loadChildrenFailed", { msg: e instanceof Error ? e.message : String(e) })
    );
    expanded.delete(parentId);
  } finally {
    loadingChildren.delete(parentId);
  }
}

const sectionLabel = computed(() => {
  if (keyword.value) {
    return t("projects.results", { count: views.value.length });
  }
  return t("projects.allProjects");
});

function toggleWorkspace() {
  workspaceOpen.value = !workspaceOpen.value;
  sortOpen.value = false;
  scanMenuOpen.value = false;
}

function toggleSort() {
  sortOpen.value = !sortOpen.value;
  workspaceOpen.value = false;
  scanMenuOpen.value = false;
}

function toggleScanMenu() {
  scanMenuOpen.value = !scanMenuOpen.value;
  workspaceOpen.value = false;
  sortOpen.value = false;
}

/** 解析系统工作区目录路径（Documents / Desktop），不存在返回 null */
async function resolveSystemDir(kind: SystemWorkspaceKind): Promise<string | null> {
  const entries = await getSystemWorkspaces();
  const found = entries.find((e) => e.kind === kind);
  if (!found || !found.exists || !found.path) return null;
  return found.path;
}

/** 清空树形展开缓存（筛选/排序/扫描变化时重置，避免父项目 id 失效） */
function resetTree() {
  expanded.clear();
  childrenOf.clear();
  loadingChildren.clear();
}

async function selectWorkspace(filter: WorkspaceFilter) {
  workspaceOpen.value = false;
  if (workspaceFilter.value === filter) return;
  workspaceFilter.value = filter;
  resetTree();
  await projectStore.fetchProjects(sortMode.value, filter);
}

async function selectSort(sort: ProjectSortBy) {
  sortOpen.value = false;
  if (sortMode.value === sort) return;
  sortMode.value = sort;
  resetTree();
  await projectStore.fetchProjects(sort, workspaceFilter.value);
}

/** 扫描按钮文案 */
const scanButtonLabel = computed(() =>
  scanner.status === "scanning" ? t("scan.scanning") : t("scan.scan")
);

const scanSummary = computed(() => {
  if (scanner.status !== "done" || !scanner.lastResult) return null;
  const { scannedCount, ignoredCount, durationMs } = scanner.lastResult;
  return t("scan.summary", {
    scanned: scannedCount,
    ignored: ignoredCount,
    duration: formatDuration(durationMs),
  });
});

async function handleScan(target: "all" | "documents" | "desktop" = "all") {
  // 触发扫描即关闭下拉菜单
  scanMenuOpen.value = false;

  // 计算本次扫描目标列表
  let targets: string[];
  if (target === "all") {
    // 默认扫全部（所有已添加的工作区）
    targets = settings.workspaces.length > 0 ? settings.workspaces : ["/"];
  } else {
    // 只扫 Documents / Desktop（用系统工作区解析的路径）
    const dir = await resolveSystemDir(target);
    if (!dir) {
      toast.error(
        target === "documents" ? t("scan.notFoundDocuments") : t("scan.notFoundDesktop")
      );
      return;
    }
    targets = [dir];
  }

  let totalScanned = 0;
  let totalIgnored = 0;
  let totalMs = 0;
  let anyError = false;
  let anySuccess = false;

  for (const ws of targets) {
    await scanner.scan(ws);
    if (scanner.status === "error") {
      anyError = true;
      if (scanner.errorCode === "INVALID_DIRECTORY") {
        await settings.invalidateWorkspace();
        toast.error(t("workspace.invalidated"));
        router.replace({ name: "welcome" });
        return;
      }
      continue;
    }
    if (scanner.status === "done" && scanner.lastResult) {
      anySuccess = true;
      totalScanned += scanner.lastResult.scannedCount;
      totalIgnored += scanner.lastResult.ignoredCount;
      totalMs += scanner.lastResult.durationMs;
    }
  }

  if (anyError) {
    toast.error(t("scan.partialFailed"));
  }
  if (anySuccess) {
    toast.success(
      t("scan.summary", {
        scanned: totalScanned,
        ignored: totalIgnored,
        duration: formatDuration(totalMs),
      })
    );
  } else if (!anyError) {
    toast.success(t("scan.done"));
  }

  resetTree();
  await projectStore.fetchProjects(sortMode.value, workspaceFilter.value);
}

onMounted(() => {
  projectStore.fetchProjects(sortMode.value, workspaceFilter.value);
});

/** 首次扫描成功且有项目时，弹出「启用项目记忆」询问 */
watch(
  () => scanner.memoryPromptTriggered,
  (triggered) => {
    if (triggered && !memoryDialogOpen.value) {
      memoryDialogOpen.value = true;
    }
  }
);
</script>

<template>
  <div class="min-h-full bg-[#F7F8FA]">
    <div class="mx-auto max-w-[1140px] px-8 py-7">
      <!-- 页头（small 下扫描按钮允许换行） -->
      <div
        class="mb-5 flex items-start justify-between"
        :class="layout.appMode === 'small' ? 'flex-wrap gap-3' : ''"
      >
        <div>
          <h1 class="text-[22px] font-semibold leading-tight tracking-tight text-[#17191C]">
            {{ t("projects.title") }}
          </h1>
          <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
            <!-- 工作区下拉 -->
            <div class="relative">
              <button
                class="flex items-center gap-1 text-[13px] text-[#6B7280] transition-colors hover:text-[#374151]"
                @click="toggleWorkspace"
              >
                {{ currentFilterLabel }}
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-[#B0B7C3]">
                  <path d="M6 9l6 6 6-6" />
                </svg>
              </button>
              <div
                v-if="workspaceOpen"
                class="absolute left-0 top-full z-50 mt-2 w-[220px] rounded-[8px] border border-[#E5E7EB] bg-white py-1"
                style="box-shadow: 0 4px 16px rgba(0,0,0,0.08), 0 1px 4px rgba(0,0,0,0.04)"
              >
                <div class="px-3 py-1.5">
                  <span class="text-[10px] font-semibold uppercase tracking-[0.09em] text-[#B0B7C3]">
                    {{ t("nav.workspace") }}
                  </span>
                </div>
                <button
                  v-for="opt in filterOptions"
                  :key="opt.value"
                  class="flex w-full items-center justify-between px-3 py-2 text-left transition-colors hover:bg-[#F9FAFB]"
                  @click="selectWorkspace(opt.value)"
                >
                  <span class="text-[13px] text-[#17191C]">{{ opt.label }}</span>
                  <svg
                    v-if="workspaceFilter === opt.value"
                    width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="#2563EB" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"
                    class="ml-2 shrink-0"
                  >
                    <path d="M20 6 9 17l-5-5" />
                  </svg>
                </button>
              </div>
            </div>
            <span class="text-[13px] leading-none text-[#D1D5DB]">·</span>
            <span class="text-[13px] text-[#9CA3AF]">{{ t("projects.count", { count: projectStore.projects.length }) }}</span>
          </div>
        </div>

        <!-- 扫描按钮：主按钮扫全部 + 下箭头下拉（只扫 Documents/Desktop） -->
        <div class="relative mt-1 shrink-0">
          <div
            class="flex overflow-hidden rounded-[7px] border border-[#E5E7EB] bg-white shadow-sm transition-colors hover:bg-[#F9FAFB] disabled:cursor-not-allowed disabled:opacity-50"
            :class="scanner.status === 'scanning' ? 'pointer-events-none opacity-50' : ''"
          >
            <!-- 主按钮：扫全部 -->
            <button
              class="flex items-center gap-1.5 py-[7px] pl-3 pr-2.5 text-[13px] text-[#374151] transition-colors hover:bg-[#F9FAFB]"
              :disabled="scanner.status === 'scanning'"
              @click="handleScan('all')"
            >
              <svg
                width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
                class="text-[#9CA3AF]"
                :class="scanner.status === 'scanning' ? 'animate-spin' : ''"
              >
                <path d="M21 12a9 9 0 1 1-2.64-6.36" />
                <path d="M21 3v6h-6" />
              </svg>
              {{ scanButtonLabel }}
            </button>
            <!-- 分隔线 -->
            <span class="my-[7px] w-px bg-[#E5E7EB]" />
            <!-- 下箭头：展开下拉 -->
            <button
              class="flex items-center px-2 text-[#9CA3AF] transition-colors hover:bg-[#F3F4F6] hover:text-[#6B7280]"
              :class="scanMenuOpen ? 'bg-[#F3F4F6] text-[#6B7280]' : ''"
              :disabled="scanner.status === 'scanning'"
              :title="t('scan.range')"
              @click.stop="toggleScanMenu"
            >
              <svg
                width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <path d="m6 9 6 6 6-6" />
              </svg>
            </button>
          </div>

          <!-- 下拉菜单 -->
          <div
            v-if="scanMenuOpen"
            class="absolute right-0 top-full z-50 mt-1.5 w-[180px] rounded-[8px] border border-[#E5E7EB] bg-white py-1"
            style="box-shadow: 0 4px 16px rgba(0,0,0,0.08), 0 1px 4px rgba(0,0,0,0.04)"
            @click.stop
          >
            <button
              class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[#374151] transition-colors duration-75 hover:bg-[#F9FAFB]"
              @click="handleScan('all')"
            >
              {{ t("scan.scanAll") }}
            </button>
            <div class="my-1 border-t border-[#F3F4F6]" />
            <button
              class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[#374151] transition-colors duration-75 hover:bg-[#F9FAFB]"
              @click="handleScan('documents')"
            >
              {{ t("scan.scanDocuments") }}
            </button>
            <button
              class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[13px] text-[#374151] transition-colors duration-75 hover:bg-[#F9FAFB]"
              @click="handleScan('desktop')"
            >
              {{ t("scan.scanDesktop") }}
            </button>
          </div>
        </div>
      </div>

      <!-- 扫描状态条 -->
      <div v-if="scanner.status === 'scanning'" class="mb-4">
        <div class="flex items-center gap-3 rounded-[8px] border border-[#E5E7EB] bg-white px-4 py-2.5 text-[13px]">
          <span class="h-[7px] w-[7px] shrink-0 animate-pulse rounded-full bg-[#2563EB]" />
          <span class="font-medium text-[#17191C]">{{ t("scan.scanningStatus") }}</span>
          <span class="text-[#9CA3AF]">{{ t("scan.pleaseWait") }}</span>
        </div>
      </div>
      <div v-else-if="scanner.status === 'done' && scanSummary" class="mb-4">
        <div class="flex items-center gap-3 rounded-[8px] border border-[#BBF7D0] bg-[#F0FDF4] px-4 py-2.5 text-[13px]">
          <span class="h-[7px] w-[7px] shrink-0 rounded-full bg-[#16A34A]" />
          <span class="font-medium text-[#15803D]">{{ scanSummary }}</span>
        </div>
      </div>
      <div v-else-if="scanner.status === 'error'" class="mb-4">
        <div class="flex items-center gap-3 rounded-[8px] border border-[#FECACA] bg-[#FEF2F2] px-4 py-2.5 text-[13px]">
          <span class="h-[7px] w-[7px] shrink-0 rounded-full bg-[#DC2626]" />
          <span class="font-medium text-[#B91C1C]">{{ scanner.error }}</span>
        </div>
      </div>

      <!-- 工具栏（small 下允许换行，不强制一行） -->
      <div
        class="mb-5 flex items-center justify-between gap-3"
        :class="layout.appMode === 'small' ? 'flex-wrap' : ''"
      >
        <!-- 搜索框 -->
        <div class="relative" :class="layout.appMode === 'small' ? 'w-full max-w-[340px]' : ''">
          <svg
            width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
            class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[#B0B7C3]"
          >
            <circle cx="11" cy="11" r="8" />
            <path d="M21 21l-4.35-4.35" />
          </svg>
          <input
            v-model="keyword"
            type="text"
            :placeholder="t('projects.searchPlaceholder')"
            class="h-[36px] w-full rounded-[8px] border border-[#E5E7EB] bg-white pl-8 pr-8 text-[13px] text-[#17191C] placeholder:text-[#B0B7C3] focus:border-[#2563EB] focus:outline-none"
          />
          <button
            v-if="keyword"
            class="absolute right-2.5 top-1/2 -translate-y-1/2 text-[#B0B7C3] transition-colors hover:text-[#6B7280]"
            @click="keyword = ''"
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M18 6 6 18" />
              <path d="M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- 排序下拉 -->
        <div class="relative shrink-0">
          <button
            class="flex h-[32px] items-center gap-1.5 rounded-[6px] px-3 text-[13px] transition-colors"
            :class="
              sortOpen
                ? 'border border-[#E5E7EB] bg-white text-[#17191C] shadow-[0_1px_2px_rgba(0,0,0,0.04)]'
                : 'text-[#6B7280] hover:bg-[#F3F4F6] hover:text-[#374151]'
            "
            @click="toggleSort"
          >
            {{ t("projects.sortBy") }}
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-[#B0B7C3]">
              <path d="M6 9l6 6 6-6" />
            </svg>
          </button>
          <div
            v-if="sortOpen"
            class="absolute right-0 top-full z-50 mt-1 w-[180px] rounded-[8px] border border-[#E5E7EB] bg-white py-1"
            style="box-shadow: 0 4px 16px rgba(0,0,0,0.08), 0 1px 4px rgba(0,0,0,0.04)"
          >
            <button
              v-for="opt in sortOptions"
              :key="opt.value"
              class="flex w-full items-center justify-between px-3 py-1.5 text-left text-[13px] transition-colors hover:bg-[#F9FAFB]"
              @click="selectSort(opt.value)"
            >
              <span :class="sortMode === opt.value ? 'font-medium text-[#2563EB]' : 'text-[#374151]'">
                {{ opt.label }}
              </span>
              <svg
                v-if="sortMode === opt.value"
                width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#2563EB" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"
              >
                <path d="M20 6 9 17l-5-5" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- 区块标签 -->
      <div v-if="views.length > 0" class="mb-2 px-1">
        <span class="text-[10px] font-semibold uppercase tracking-[0.09em] text-[#B0B7C3]">
          {{ sectionLabel }}
        </span>
      </div>

      <!-- 表格 -->
      <div class="rounded-[8px] border border-[#E5E7EB] bg-white">
        <!-- 加载中 -->
        <div v-if="projectStore.listLoading" class="py-20 text-center text-[13px] text-[#9CA3AF]">
          {{ t("projects.loading") }}
        </div>
        <!-- 错误 -->
        <div
          v-else-if="projectStore.error && projectStore.projects.length === 0"
          class="py-20 text-center text-[13px] text-[#DC2626]"
        >
          {{ projectStore.error }}
        </div>
        <!-- 无项目 -->
        <div v-else-if="projectStore.projects.length === 0" class="py-20 text-center">
          <p class="mb-1.5 text-[15px] font-semibold text-[#17191C]">{{ t("projects.empty") }}</p>
          <p class="text-[13px] text-[#9CA3AF]">{{ t("projects.emptyHint") }}</p>
        </div>
        <!-- 搜索无结果 -->
        <div v-else-if="views.length === 0" class="py-20 text-center">
          <p class="mb-1.5 text-[15px] font-semibold text-[#17191C]">{{ t("projects.noMatch") }}</p>
          <p class="mb-5 text-[13px] text-[#9CA3AF]">{{ t("projects.noMatchHint") }}</p>
          <button
            class="rounded-[6px] border border-[#E5E7EB] px-3 py-1.5 text-[13px] text-[#6B7280] transition-colors hover:bg-[#F3F4F6]"
            @click="keyword = ''"
          >
            {{ t("projects.clearSearch") }}
          </button>
        </div>
        <!-- 表格（树形：可展开聚合根/分类目录） -->
        <ProjectTable
          v-else
          :projects="views"
          tree
          :children-of="childViewsOf"
          :is-expanded="(id) => expanded.has(id)"
          :can-expand="canExpand"
          :is-loading-children="(id) => loadingChildren.has(id)"
          @toggle-expand="toggleExpand"
        />
      </div>
    </div>

    <!-- 首次扫描后的「启用项目记忆」询问 -->
    <EnableMemoryDialog
      :open="memoryDialogOpen"
      :projects="projectStore.projects"
      @close="memoryDialogOpen = false"
    />
  </div>
</template>
