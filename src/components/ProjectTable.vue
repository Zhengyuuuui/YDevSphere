<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import type { ProjectView } from "@/types/view";
import type { ProjectKind } from "@/types";
import TechnologyBadge from "./TechnologyBadge.vue";
import GitStatusBadge from "./GitStatusBadge.vue";
import { useEditorStore } from "@/stores/editor";
import { useLayoutStore, type AppMode } from "@/stores/layout";

const { t } = useI18n();
const layout = useLayoutStore();

const props = defineProps<{
  projects: ProjectView[];
  /** true 显示「最近打开」时间列，否则显示「更新时间」列 */
  showLastOpened?: boolean;
  /** 树形模式（v0.2）：启用后可展开聚合根/分类目录的子项目 */
  tree?: boolean;
  /** 父项目 id → 已加载的子项目视图列表 */
  childrenOf?: (id: number) => ProjectView[];
  /** 判断项目是否已展开 */
  isExpanded?: (id: number) => boolean;
  /** 判断项目是否可展开 */
  canExpand?: (p: ProjectView) => boolean;
  /** 判断项目是否正在加载子项目 */
  isLoadingChildren?: (id: number) => boolean;
}>();

const emit = defineEmits<{
  /** 切换展开（树形模式下点击展开按钮触发） */
  (e: "toggle-expand", p: ProjectView): void;
}>();

/** 扁平化树形行（含缩进层级），供渲染 */
interface TreeRow {
  view: ProjectView;
  depth: number;
}

const rows = computed<TreeRow[]>(() => {
  if (!props.tree) {
    return props.projects.map((p) => ({ view: p, depth: 0 }));
  }
  const out: TreeRow[] = [];
  const walk = (list: ProjectView[], depth: number) => {
    for (const p of list) {
      out.push({ view: p, depth });
      const expandable = props.canExpand?.(p) ?? false;
      const expanded = props.isExpanded?.(p.id) ?? false;
      if (expandable && expanded) {
        const children = props.childrenOf?.(p.id) ?? [];
        walk(children, depth + 1);
      }
    }
  };
  walk(props.projects, 0);
  return out;
});

/** 类型标识文案与配色 */
const KIND_META = computed<Record<ProjectKind, { label: string; bg: string; text: string }>>(
  () => ({
    real: { label: "", bg: "", text: "" },
    aggregated_root: { label: t("table.aggregated"), bg: "#EEF2FF", text: "#4338CA" },
    category: { label: t("table.category"), bg: "#FEF9C3", text: "#854D0E" },
  })
);

/** 健康度颜色分级：≥60 绿、40-59 黄、<40 灰 */
function healthTone(score: number): { color: string; bg: string } {
  if (score >= 60) return { color: "#16A34A", bg: "#DCFCE7" };
  if (score >= 40) return { color: "#D97706", bg: "#FEF3C7" };
  return { color: "#9CA3AF", bg: "#F3F4F6" };
}

const router = useRouter();
const editorStore = useEditorStore();

const hoveredId = ref<number | null>(null);
const selectedId = ref<number | null>(null);
const openMenuId = ref<number | null>(null);
/** 行菜单 DOM 引用（按项目 id 记录，因每行菜单在 v-for 内，避免数组 ref 导致 .contains 报错） */
const menuEls = new Map<number, HTMLElement>();

const timeLabel = computed(() =>
  props.showLastOpened ? t("table.lastOpened") : t("table.updatedAt")
);

/**
 * 各布局模式的 gridTemplateColumns（Header 与 Row 共享，保证对齐）。
 * 各列有独立最小可用宽度，剩余空间允许时参与弹性分配（非简单 1fr 等比例）。
 */
const GRIDS: Record<AppMode, string> = {
  large: "minmax(240px,1fr) minmax(120px,1fr) minmax(110px,1fr) minmax(100px,1fr) minmax(110px,1fr) 76px",
  medium: "minmax(240px,1fr) minmax(120px,1fr) minmax(100px,1fr) 76px",
  small: "minmax(240px,1fr) minmax(120px,1fr) minmax(100px,1fr) 40px",
};

const GRID = computed(() => GRIDS[layout.appMode]);

/** 当前是否处于 small 模式（操作列改 More 按钮） */
const isSmall = computed(() => layout.appMode === "small");

/** 列显隐（Header + Row 同步）：medium/small 隐藏 Git 与时间列 */
const showGit = computed(() => layout.appMode === "large");
const showTime = computed(() => layout.appMode === "large");

/**
 * 技术栈可见数（Tech 单行固定展示前 N 个，超出显示 +N）：
 * large 3 / medium 2 / small 1
 */
const MAX_TECH = { large: 3, medium: 2, small: 1 } as const;
const maxTech = computed(() => MAX_TECH[layout.appMode]);

function timeValue(p: ProjectView): string {
  if (props.showLastOpened) return p.lastOpenedAt ?? "—";
  return p.updatedAt ?? "—";
}

/** 技术栈单行可见列表（前 maxTech 个） */
function visibleTechs(p: ProjectView): string[] {
  return p.technologies.slice(0, maxTech.value);
}

/** 隐藏的技术栈数量（用于 +N 徽标） */
function hiddenTechCount(p: ProjectView): number {
  return p.technologies.length - maxTech.value;
}

/** 平台相关「在文件管理器中显示」文案 */
const revealLabel = computed(() => {
  const ua = navigator.userAgent;
  const isMac = /Mac|iPhone|iPad/i.test(ua);
  const isWin = /Windows/i.test(ua);
  if (isMac) return t("table.revealInFinder");
  if (isWin) return t("table.revealInExplorer");
  return t("table.revealInFileManager");
});

/** 单击行：选中；双击行：打开默认编辑器（保留现有交互） */
function onRowClick(p: ProjectView) {
  selectedId.value = selectedId.value === p.id ? null : p.id;
}

function onRowDblclick(p: ProjectView) {
  editorStore.openEditor(p.id, null);
}

/** 行内「打开」按钮 → 默认编辑器 */
async function handleOpen(p: ProjectView) {
  await editorStore.openEditor(p.id, null);
}

function toggleMenu(id: number) {
  openMenuId.value = openMenuId.value === id ? null : id;
}

async function handleOpenWith(p: ProjectView, editorId: string) {
  openMenuId.value = null;
  await editorStore.openEditor(p.id, editorId);
}

async function handleFileManager(p: ProjectView) {
  openMenuId.value = null;
  await editorStore.openFileManager(p.id);
}

async function handleOpenTerminal(p: ProjectView) {
  openMenuId.value = null;
  // 无终端打开接口，降级为文件管理器打开项目目录
  await editorStore.openFileManager(p.id);
}

async function handleCopyPath(p: ProjectView) {
  openMenuId.value = null;
  try {
    await navigator.clipboard.writeText(p.path);
  } catch {
    // 剪贴板不可用时静默失败
  }
}

function goDetail(p: ProjectView) {
  router.push(`/project/${p.id}`);
}

// 点击外部关闭菜单（逐行判断当前打开菜单的元素是否包含点击目标）
function onDocClick(e: MouseEvent) {
  if (openMenuId.value == null) return;
  const el = menuEls.get(openMenuId.value);
  if (el && !el.contains(e.target as Node)) {
    openMenuId.value = null;
  }
}
onMounted(() => document.addEventListener("click", onDocClick));
onUnmounted(() => document.removeEventListener("click", onDocClick));

/** 可打开的编辑器（排除 open_method === "unsupported"） */
const openableEditors = computed(() =>
  editorStore.editors.filter((e) => e.open_method !== "unsupported")
);

/** 行内默认编辑器名（按钮 title） */
function defaultEditorName(): string | null {
  const id = editorStore.defaultEditorId;
  if (id) {
    const ed = editorStore.editors.find((e) => e.id === id);
    if (ed && ed.open_method !== "unsupported") return ed.name;
  }
  return openableEditors.value[0]?.name ?? null;
}
</script>

<template>
  <div class="w-full">
    <!-- 表头 -->
    <div
      class="grid items-center border-b border-divider px-4 py-2.5"
      :style="{ gridTemplateColumns: GRID }"
    >
      <span class="font-display text-[10px] font-semibold uppercase tracking-[0.09em] text-fainter">{{ t("table.project") }}</span>
      <span class="font-display text-[10px] font-semibold uppercase tracking-[0.09em] text-fainter">{{ t("table.techStack") }}</span>
      <span v-if="showGit" class="font-display text-[10px] font-semibold uppercase tracking-[0.09em] text-fainter">{{ t("table.git") }}</span>
      <span class="font-display text-[10px] font-semibold uppercase tracking-[0.09em] text-fainter">{{ t("table.health") }}</span>
      <span v-if="showTime" class="font-display text-right text-[10px] font-semibold uppercase tracking-[0.09em] text-fainter">{{ timeLabel }}</span>
      <span></span>
    </div>

    <!-- 行 -->
    <div
      v-for="(row, index) in rows"
      :key="row.view.id"
      class="relative grid cursor-pointer items-center border-b px-4 transition-colors duration-75"
      :class="index === rows.length - 1 ? 'border-transparent' : 'border-divider'"
      :style="{
        gridTemplateColumns: GRID,
        minHeight: '68px',
        backgroundColor:
          selectedId === row.view.id
            ? 'var(--color-primary-soft)'
            : hoveredId === row.view.id || openMenuId === row.view.id
            ? 'var(--color-surface-3)'
            : 'var(--color-surface)',
        borderRadius:
          index === 0 && index === rows.length - 1
            ? '7px'
            : index === 0
            ? '7px 7px 0 0'
            : index === rows.length - 1
            ? '0 0 7px 7px'
            : undefined,
      }"
      @mouseenter="hoveredId = row.view.id"
      @mouseleave="hoveredId = null"
      @click="onRowClick(row.view)"
      @dblclick="onRowDblclick(row.view)"
    >
      <!-- 名称 + 路径 -->
      <div class="flex min-w-0 items-center gap-3 pr-4">
        <!-- 树形缩进 + 展开按钮 -->
        <div
          v-if="tree"
          class="flex shrink-0 items-center"
          :style="{ paddingLeft: `${row.depth * 20}px` }"
        >
          <button
            v-if="canExpand?.(row.view)"
            class="flex h-[18px] w-[18px] shrink-0 items-center justify-center rounded-[4px] text-faint transition-colors hover:bg-surface-2 hover:text-muted"
            :title="isExpanded?.(row.view.id) ? t('table.collapse') : t('table.expand')"
            @click.stop="emit('toggle-expand', row.view)"
          >
            <svg
              v-if="isLoadingChildren?.(row.view.id)"
              width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              class="animate-spin"
            >
              <path d="M21 12a9 9 0 1 1-2.64-6.36" />
              <path d="M21 3v6h-6" />
            </svg>
            <svg
              v-else
              width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              :class="isExpanded?.(row.view.id) ? 'rotate-90 transition-transform' : 'transition-transform'"
            >
              <path d="M9 6l6 6-6 6" />
            </svg>
          </button>
          <span v-else class="h-[18px] w-[18px] shrink-0" />
        </div>

        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" class="shrink-0 text-fainter">
          <path
            d="M2 5C2 4.17 2.67 3.5 3.5 3.5H6L7.5 5H12.5C13.33 5 14 5.67 14 6.5V11.5C14 12.33 13.33 13 12.5 13H3.5C2.67 13 2 12.33 2 11.5V5Z"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linejoin="round"
          />
        </svg>
        <div class="min-w-0">
          <div
            class="flex items-center gap-1.5 text-[15px] font-medium leading-tight"
            :class="selectedId === row.view.id ? 'text-active' : 'text-ink'"
          >
            <span class="font-display truncate">{{ row.view.name }}</span>
            <span
              v-if="KIND_META[row.view.kind].label"
              class="inline-flex shrink-0 items-center rounded-[4px] px-[5px] py-[1px] text-[10px] font-medium leading-[14px]"
              :style="{ backgroundColor: KIND_META[row.view.kind].bg, color: KIND_META[row.view.kind].text }"
            >
              {{ KIND_META[row.view.kind].label }}
            </span>
          </div>
          <div class="mt-[3px] truncate text-[13px] text-faint">{{ row.view.path }}</div>
        </div>
      </div>

      <!-- 技术栈徽标（固定可见数，超出显示 +N；空间不足时换行避免徽标被切半） -->
      <div class="flex min-w-0 flex-wrap items-center gap-1 overflow-hidden pr-4">
        <span v-if="row.view.technologies.length === 0" class="text-[13px] text-faint">
          {{ t("table.unknown") }}
        </span>
        <template v-else>
          <TechnologyBadge
            v-for="tech in visibleTechs(row.view)"
            :key="tech"
            :label="tech"
            class="shrink-0"
          />
          <span
            v-if="hiddenTechCount(row.view) > 0"
            class="inline-flex shrink-0 items-center rounded-[4px] bg-surface-2 px-[7px] py-[2px] text-[11px] font-medium leading-[18px] text-faint"
          >
            {{ t("table.moreTech", { count: hiddenTechCount(row.view) }) }}
          </span>
        </template>
      </div>

      <!-- Git 状态（仅 large 显示） -->
      <div v-if="showGit" class="pr-4">
        <GitStatusBadge :status="row.view.gitType" :changes="row.view.gitChanges" />
      </div>

      <!-- 健康度 -->
      <div class="pr-4">
        <div class="flex items-center gap-1.5">
          <div class="h-[5px] w-[44px] overflow-hidden rounded-full bg-surface-2">
            <div
              class="h-full rounded-full"
              :style="{ width: `${Math.min(100, Math.max(0, row.view.healthScore))}%`, backgroundColor: healthTone(row.view.healthScore).color }"
            />
          </div>
          <span class="text-[13px] tabular-nums" :style="{ color: healthTone(row.view.healthScore).color }">
            {{ row.view.healthScore }}
          </span>
        </div>
      </div>

      <!-- 时间（仅 large 显示） -->
      <div v-if="showTime" class="text-right">
        <span class="text-[13px] text-faint">{{ timeValue(row.view) }}</span>
      </div>

      <!-- 行操作 -->
      <div
        :ref="(el) => (el ? menuEls.set(row.view.id, el as HTMLElement) : menuEls.delete(row.view.id))"
        class="relative flex items-center justify-end gap-1"
      >
        <!-- large/medium：hover 显示「打开」+「更多」两个按钮 -->
        <template v-if="!isSmall">
          <template v-if="hoveredId === row.view.id || openMenuId === row.view.id">
            <button
              class="rounded-[5px] p-1.5 text-faint transition-colors hover:bg-surface-2 hover:text-muted"
              :title="t('table.openWith', { name: defaultEditorName() ?? t('table.defaultEditor') })"
              @click.stop="handleOpen(row.view)"
            >
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
                <path d="M15 3h6v6" />
                <path d="M10 14 21 3" />
              </svg>
            </button>
            <button
              class="rounded-[5px] p-1.5 text-faint transition-colors hover:bg-surface-2 hover:text-muted"
              :class="openMenuId === row.view.id ? 'bg-surface-2 text-muted' : ''"
              :title="t('table.more')"
              @click.stop="toggleMenu(row.view.id)"
            >
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="1" />
                <circle cx="19" cy="12" r="1" />
                <circle cx="5" cy="12" r="1" />
              </svg>
            </button>

            <!-- 下拉菜单 -->
            <div
              v-if="openMenuId === row.view.id"
              class="absolute right-0 top-full z-50 mt-1 w-[196px] rounded-[8px] border border-line-3 bg-surface py-1"
              style="box-shadow: 0 4px 16px rgba(0,0,0,0.08), 0 1px 4px rgba(0,0,0,0.04)"
              @click.stop
            >
              <button
                class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
                @click="goDetail(row.view)"
              >
                {{ t("table.viewDetail") }}
              </button>
              <div class="my-1 border-t border-line-2" />
              <button
                v-for="ed in openableEditors"
                :key="ed.id"
                class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
                @click="handleOpenWith(row.view, ed.id)"
              >
                {{ t("table.openIn", { name: ed.name }) }}
              </button>
              <button
                class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
                @click="handleFileManager(row.view)"
              >
                {{ t("table.openFileManager") }}
              </button>
              <button
                class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
                @click="handleOpenTerminal(row.view)"
              >
                {{ t("table.openTerminal") }}
              </button>
              <div class="my-1 border-t border-line-2" />
              <button
                class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
                @click="handleCopyPath(row.view)"
              >
                {{ t("table.copyPath") }}
              </button>
            </div>
          </template>
        </template>

        <!-- small：始终可见 More(⋯) 按钮（菜单含 Open / Open in Editor / reveal / Copy Path / Details） -->
        <template v-else>
          <button
            class="rounded-[5px] p-1.5 text-faint transition-colors hover:bg-surface-2 hover:text-muted"
            :class="openMenuId === row.view.id ? 'bg-surface-2 text-muted' : ''"
            :title="t('table.more')"
            @click.stop="toggleMenu(row.view.id)"
          >
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="1" />
              <circle cx="19" cy="12" r="1" />
              <circle cx="5" cy="12" r="1" />
            </svg>
          </button>

          <div
            v-if="openMenuId === row.view.id"
            class="absolute right-0 top-full z-50 mt-1 w-[196px] rounded-[8px] border border-line bg-surface py-1"
            style="box-shadow: 0 4px 16px rgba(0,0,0,0.08), 0 1px 4px rgba(0,0,0,0.04)"
            @click.stop
          >
            <button
              class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
              @click="handleOpen(row.view)"
            >
              {{ t("table.open") }}
            </button>
            <button
              v-for="ed in openableEditors"
              :key="ed.id"
              class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
              @click="handleOpenWith(row.view, ed.id)"
            >
              {{ t("table.openIn", { name: ed.name }) }}
            </button>
            <div class="my-1 border-t border-line-2" />
            <button
              class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
              @click="handleFileManager(row.view)"
            >
              {{ revealLabel }}
            </button>
            <div class="my-1 border-t border-line-2" />
            <button
              class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
              @click="handleCopyPath(row.view)"
            >
              {{ t("table.copyPath") }}
            </button>
            <button
              class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left text-[14px] text-ink transition-colors duration-75 hover:bg-surface-3"
              @click="goDetail(row.view)"
            >
              {{ t("table.viewDetail") }}
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>
