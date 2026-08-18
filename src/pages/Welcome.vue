<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import type { AvailableEditor, SystemWorkspace } from "@/types";
import { getSystemWorkspaces } from "@/api/project";
import { useSettingsStore } from "@/stores/settings";
import { useScannerStore } from "@/stores/scanner";
import { useEditorStore } from "@/stores/editor";
import { toast } from "@/lib/toast";
import { iconSrc as editorIconSrc } from "@/lib/editorIcon";
import CloudField from "@/components/CloudField.vue";

const router = useRouter();
const { t } = useI18n();
const settings = useSettingsStore();
const scanner = useScannerStore();
const editorStore = useEditorStore();

/** 系统工作区入口（Documents / Desktop） */
const systemWorkspaces = ref<SystemWorkspace[]>([]);
/** 「其他选项」折叠状态 */
const moreOpen = ref(false);
/** 正在导入的入口 label（用于按钮 loading） */
const importingLabel = ref<string | null>(null);
/** 一键导入所有入口的 loading 标志 */
const importingAll = ref(false);
/** 编辑器引导是否显示（导入工作区后可选步骤） */
const editorGuideOpen = ref(false);
/** 引导中选中的候选 id */
const guideSelectedId = ref<string | null>(null);
/** 引导中待确认导入的自定义编辑器（非 vscode_fork） */
const guidePendingCustom = ref<AvailableEditor | null>(null);
/** 引导视图：新版图标滚动 / 旧版纵向列表 */
const guideView = ref<"icons" | "list">("icons");

/* ============ 新版图标栏 · Interactive Infinite Icon Track ============
 * 运动模型：Track 用 translate3d(offset,0,0) 位移；三份相同内容 [SET1][SET2][SET3]；
 * 初始显示 SET2（offset = -singleSetWidth）。rAF 驱动自动向左，offset 以 singleSetWidth
 * 为模归一化到 [-2*singleSetWidth, -singleSetWidth)，始终落在中间一份，视觉无缝无跳变。
 * Pointer Drag / Wheel / Hover 与自动统一改同一 offset，无边界冲突。
 * 不引第三方库，不用 setInterval / CSS marquee / @keyframes。 */

/** 可视视口 ref（overflow:hidden，无滚动条） */
const iconScroller = ref<HTMLElement | null>(null);
/** Track ref（translate3d 容器） */
const iconTrack = ref<HTMLElement | null>(null);
/** 三份 SET 的 ref（取 [0] 测第一份宽度） */
const iconSetEls = ref<HTMLElement[]>([]);

/** Track 位移（唯一运动状态，px） */
const offset = ref(0);
/** 单份 Icon 集合宽度（px），重测得到 */
const singleSetWidth = ref(0);

/* --- 交互状态机 --- */
const hovering = ref(false); // 悬停：暂停自动
const dragging = ref(false); // 拖动：暂停自动
/** 拖动/滚轮后恢复自动的停顿（rAF 仍跑，仅跳过自动步进） */
const resumePaused = ref(false);
let resumeTimer: ReturnType<typeof setTimeout> | null = null;
/** 拖动起始快照 */
let dragStartOffset = 0;
let dragStartX = 0;
/** 本次是否为拖动（位移超阈值），用于抑制点击选中 */
let dragMoved = false;
let suppressClick = false;

/* --- rAF 驱动 --- */
let rafId: number | null = null;
let lastTime = 0;
/** 自动滚动速度（px/s） */
const AUTO_SPEED = 40;

/** 新版仅展示 VS Code 系 Fork：过滤 vscode_fork 且带真实 app 图标项 */
const forkIcons = computed<AvailableEditor[]>(() =>
  editorStore.candidates.filter(
    (ed) => ed.category === "vscode_fork" && !!ed.icon_base64
  )
);

/** 图标是否溢出（单份内容宽 > 可视宽 → 启用自动滚动） */
const iconOverflow = computed(() => {
  const el = iconScroller.value;
  if (!el) return false;
  return singleSetWidth.value > el.clientWidth + 1;
});

function hasSingleOverflow(): boolean {
  const el = iconScroller.value;
  if (!el) return false;
  return singleSetWidth.value > el.clientWidth + 1;
}

/**
 * 归一化：把 offset 取模到 [-2*singleSetWidth, -singleSetWidth)（长度 = 单份宽）。
 * 以单份宽为周期循环，SET1/SET3 作双向拖动的冗余余量；每份开头内容相同，视觉无缝无跳变。
 */
function normalizeOffset(v: number): number {
  const span = singleSetWidth.value;
  if (span <= 0) return v;
  const m = ((v + 2 * span) % span + span) % span; // 标准化到 [0, span)
  return m - 2 * span; // [-2*span, -span)
}

/**
 * 应用位移到 Track。
 * @param doNormalize true（自动滚动/滚轮/拖动结束）：归一化到中间份，保证无缝；
 *   false（拖动中）：仅 clamp 到冗余范围 [-2*span, 0]，让用户可自由双向拖动而不露白、不触发归一化回弹。
 */
function applyTrack(doNormalize = true) {
  const span = singleSetWidth.value;
  if (doNormalize) {
    offset.value = normalizeOffset(offset.value);
  } else if (span > 0) {
    if (offset.value < -2 * span) offset.value = -2 * span;
    else if (offset.value > 0) offset.value = 0;
  }
  if (iconTrack.value) {
    iconTrack.value.style.transform = `translate3d(${offset.value}px,0,0)`;
  }
}

/** 收集三份 SET 的 DOM 引用（按 setIdx 1..3 → 数组索引 0..2） */
function setTrackRef(setIdx: number, el: HTMLElement | null) {
  if (!el) return;
  iconSetEls.value[setIdx - 1] = el;
}

/** 重测单份 SET 宽度（内容变化 / 容器 resize 后调用） */
function remeasure() {
  const first = iconSetEls.value[0];
  if (first) {
    singleSetWidth.value = first.offsetWidth;
  }
}

/** rAF 主循环：自动向左，暂停时不更新 offset */
function tick(now: number) {
  if (!lastTime) lastTime = now;
  const deltaTime = (now - lastTime) / 1000;
  lastTime = now;

  const autoPaused = hovering.value || dragging.value || resumePaused.value;
  if (!autoPaused && hasSingleOverflow()) {
    offset.value -= AUTO_SPEED * deltaTime;
  }
  applyTrack();
  rafId = requestAnimationFrame(tick);
}

/** 启动自动滚动（仅溢出时；否则静态展示） */
function startIconScroll() {
  stopIconScroll();
  // 下一帧等图标栏渲染完成后测量 + 定位
  requestAnimationFrame(() => {
    remeasure();
    if (!iconScroller.value) return;
    if (hasSingleOverflow()) {
      // 初始显示 SET2（中间一份），clamp 不归一化，精确落在 SET2 开头
      offset.value = -singleSetWidth.value;
      applyTrack(false);
      lastTime = 0;
      rafId = requestAnimationFrame(tick);
    } else {
      // 不溢出：静态展示第一项开头
      offset.value = 0;
      applyTrack(false);
    }
  });
}

/** 停止自动滚动 */
function stopIconScroll() {
  if (rafId) {
    cancelAnimationFrame(rafId);
    rafId = null;
  }
  if (resumeTimer) {
    clearTimeout(resumeTimer);
    resumeTimer = null;
  }
  resumePaused.value = false;
}

/** 拖动结束后停顿 700ms 再恢复自动 */
function scheduleResume() {
  if (resumeTimer) clearTimeout(resumeTimer);
  resumePaused.value = true;
  resumeTimer = setTimeout(() => {
    resumePaused.value = false;
    resumeTimer = null;
    lastTime = 0;
  }, 700);
}

/* --- Pointer Drag --- */
function onPointerDown(e: PointerEvent) {
  dragging.value = true;
  dragStartOffset = offset.value;
  dragStartX = e.clientX;
  dragMoved = false;
  // 跟随手指持续接收 move/up（即使移出按钮）
  (e.currentTarget as Element | null)?.setPointerCapture?.(e.pointerId);
  lastTime = 0;
}

function onPointerMove(e: PointerEvent) {
  if (!dragging.value) return;
  const dx = e.clientX - dragStartX;
  if (Math.abs(dx) > 5) dragMoved = true;
  offset.value = dragStartOffset + dx;
  // 拖动中不归一化（允许在 SET1/SET3 冗余区自由拖动），仅 clamp 边界防露白
  applyTrack(false);
}

function onPointerUp() {
  dragging.value = false;
  if (dragMoved) suppressClick = true;
  // 拖动结束：归一化回中间份循环区间，随后停顿恢复自动
  applyTrack();
  scheduleResume();
}

/* --- Wheel / Trackpad --- */
function onWheel(e: WheelEvent) {
  // 横向优先；纯垂直滚时用 deltaY 辅助
  const dx = e.deltaX !== 0 ? e.deltaX : e.deltaY;
  if (Math.abs(dx) > 0.1) {
    offset.value -= dx;
    applyTrack();
    scheduleResume();
  }
}

/* --- 图标点击（选中/取消） --- */
function onIconClick(ed: AvailableEditor) {
  if (suppressClick) {
    suppressClick = false;
    return;
  }
  pickForkIcon(ed);
}

/** 用 ResizeObserver 在视口尺寸变化时重测单份宽，避免归一化错误 */
let ro: ResizeObserver | null = null;

onBeforeUnmount(() => {
  stopIconScroll();
  ro?.disconnect();
  ro = null;
});

/**
 * 新版：点击图标 = 选中/取消切换（仅 Fork）。
 * - 未选中该图标 → 设为选中 + 设默认编辑器。
 * - 已选中该图标 → 取消选中（清 guideSelectedId）。
 *   清除默认编辑器偏好需后端支持（现有 set_editor_preference 仅接受非空已校验 id，
 *   传空会 InvalidEditor，无专用清除接口）→ 按红线不绕过后端，回传总负责人处理。
 */
function pickForkIcon(ed: AvailableEditor) {
  if (guideSelectedId.value === ed.id) {
    guideSelectedId.value = null;
    // TODO(V03): 清除默认编辑器偏好需后端新增清除能力后接入；当前不绕过后端。
  } else {
    guideSelectedId.value = ed.id;
    editorStore.setDefaultSilent(ed.id);
  }
}

/** 切换引导视图（只影响本引导，不持久化）；切到新版时重启自动滚动 */
function switchGuideView(view: "icons" | "list") {
  guideView.value = view;
  if (view === "icons") {
    // 等图标栏渲染后重测单份宽 + 挂载 resize 监听 + 启动自动滚动
    nextTick(() => {
      remeasure();
      startIconScroll();
    });
  } else {
    stopIconScroll();
  }
}

const documentsEntry = computed(() =>
  systemWorkspaces.value.find((w) => w.kind === "documents")
);
const desktopEntry = computed(() =>
  systemWorkspaces.value.find((w) => w.kind === "desktop")
);

onMounted(async () => {
  try {
    systemWorkspaces.value = await getSystemWorkspaces();
  } catch (e) {
    toast.error(t("workspace.loadSystemFailed", { msg: e instanceof Error ? e.message : String(e) }));
  }
});

/** 一键导入单个：扫描路径 → 加入工作区集合 → 持久化 → 跳 Overview */
async function importWorkspace(ws: SystemWorkspace | undefined) {
  if (!ws?.exists || !ws.path) {
    toast.error(t("workspace.notExistImport", { name: ws?.label ?? "" }));
    return;
  }
  if (scanner.status === "scanning") return;

  importingLabel.value = ws.label;
  await scanner.scan(ws.path);
  importingLabel.value = null;

  if (scanner.status === "error") {
    toast.error(t("scan.importFailed", { msg: scanner.error }));
    return;
  }

  // 加入工作区集合（去重，不覆盖已有工作区）+ 持久化到后端
  await settings.addWorkspace(ws.path);
  await openEditorGuide();
}

/** 一键导入所有可用系统工作区（Documents + Desktop 都加入集合，不覆盖） */
async function importAll() {
  // 1) 筛选出「存在且有路径」的入口（path 已确认为非空字符串）
  const entries: { label: string; path: string }[] = [];
  for (const w of [documentsEntry.value, desktopEntry.value]) {
    if (w?.exists && w.path) entries.push({ label: w.label, path: w.path });
  }
  // 2) 一个都不存在 → 明确提示，不进入扫描
  if (entries.length === 0) {
    toast.error(t("workspace.bothNotExist"));
    return;
  }
  // 3) 防重复：扫描中直接返回
  if (scanner.status === "scanning") return;

  importingAll.value = true;
  let scannedAny = false;
  try {
    // 4) 先把所有入口都加入工作区集合（不覆盖已有，addWorkspace 已持久化到后端）
    for (const entry of entries) {
      await settings.addWorkspace(entry.path);
    }
    // 5) 依次扫描所有已加入的工作区（汇总）
    for (const entry of entries) {
      await scanner.scan(entry.path);
      if (scanner.status === "error") {
        toast.error(
          t("scan.importFailedAt", {
            name: entry.label,
            msg: scanner.error ?? t("scan.unknownError"),
          })
        );
        continue; // 单个失败不阻断其余工作区
      }
      scannedAny = true;
    }
    // 6) 全部扫描成功则进入编辑器引导（可选步骤，不阻断）
    if (scannedAny) {
      await openEditorGuide();
    }
  } finally {
    importingAll.value = false;
  }
}

/** 打开「选择常用编辑器」引导（可选步骤）：加载候选列表，不阻断 */
async function openEditorGuide() {
  editorGuideOpen.value = true;
  guideSelectedId.value = null;
  guidePendingCustom.value = null;
  guideView.value = "icons";
  await editorStore.loadCandidates();
  // 等图标栏渲染后：重测单份宽 + 挂载 resize 监听 + 启动自动滚动
  nextTick(() => {
    remeasure();
    // 视口尺寸变化时重测单份宽，避免归一化错误
    if (!ro && iconScroller.value) {
      ro = new ResizeObserver(() => remeasure());
      ro.observe(iconScroller.value);
    }
    startIconScroll();
  });
}

/**
 * 引导：选择候选编辑器。
 * - 非 VS Code Fork（可能不是编辑器）→ 先弹确认框，确认后再确认导入 + 写偏好。
 * - VS Code Fork（已知编辑器）→ 直接确认导入 + 写偏好。
 */
function pickGuideCandidate(ed: AvailableEditor) {
  if (ed.category !== "vscode_fork") {
    guidePendingCustom.value = ed;
    return;
  }
  confirmGuideCustom(ed);
}

/** 引导：确认导入自定义编辑器 + 写默认偏好 */
async function confirmGuideCustom(ed: AvailableEditor) {
  guidePendingCustom.value = null;
  const ok = await editorStore.confirmCustom(ed.id);
  if (ok) {
    await editorStore.setDefaultSilent(ed.id);
    guideSelectedId.value = ed.id;
  } else {
    toast.error(t("editor.importFailed", { msg: editorStore.error }));
  }
}

/** 引导：完成（选择或跳过）→ 进入 Overview */
function finishEditorGuide() {
  editorGuideOpen.value = false;
  stopIconScroll();
  router.push("/overview");
}

/** 手动选择工作区（保留） */
async function chooseWorkspace() {
  const selected = await settings.selectWorkspacePath();
  if (selected) {
    router.push("/overview");
  }
}

function toggleMore() {
  moreOpen.value = !moreOpen.value;
}
</script>

<template>
  <!-- 点阵云团背景 -->
  <CloudField />

  <div class="relative z-10 flex min-h-screen flex-col items-center justify-center px-6">
    <div class="w-full max-w-2xl">
      <!-- Logo + 标题 -->
      <div class="text-center">
        <img
          src="/logo.png"
          alt="YDevSphere"
          class="mx-auto h-24 w-auto object-contain"
        />
        <h1 class="mt-6 text-4xl font-bold tracking-tight">YDevSphere</h1>
        <p class="mt-4 text-lg text-gray-600">
          {{ t("workspace.welcomeTagline") }}
        </p>
      </div>

      <!-- 一键导入所有系统工作区 -->
      <div class="mt-10">
        <button
          class="group w-full rounded-lg border border-gray-200 bg-white p-6 text-left shadow-sm transition hover:border-blue-200 hover:shadow-md disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="
            (!documentsEntry?.exists && !desktopEntry?.exists) ||
            scanner.status === 'scanning'
          "
          @click="importAll"
        >
          <div class="flex items-center gap-3">
            <span class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-50 text-blue-600">
              <svg
                class="h-5 w-5"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z" />
              </svg>
            </span>
            <div>
              <div class="font-medium text-gray-900">{{ t("workspace.importAll") }}</div>
              <div class="mt-0.5 text-xs text-gray-500">
                {{ t("workspace.importAllDesc") }}
              </div>
            </div>
          </div>
          <div v-if="importingAll" class="mt-3 text-xs text-blue-600">
            {{ t("workspace.importing") }}
          </div>
        </button>
      </div>

      <!-- 其他选项 -->
      <div class="mt-4 text-center">
        <button
          class="text-sm text-gray-500 hover:text-gray-700"
          @click="toggleMore"
        >
          {{ moreOpen ? t("workspace.collapseOptions") : t("workspace.moreOptions") }}
        </button>

        <div
          v-if="moreOpen"
          class="mx-auto mt-3 max-w-sm overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm"
        >
          <button
            class="flex w-full items-center justify-between px-4 py-3 text-left text-sm text-gray-700 hover:bg-gray-50"
            @click="chooseWorkspace"
          >
            <span>{{ t("workspace.select") }}</span>
            <span class="text-gray-400">{{ t("workspace.manualSelect") }}</span>
          </button>
          <button
            class="flex w-full items-center justify-between border-t border-gray-100 px-4 py-3 text-left text-sm text-gray-700 hover:bg-gray-50 disabled:opacity-50"
            :disabled="!documentsEntry?.exists || scanner.status === 'scanning'"
            @click="importWorkspace(documentsEntry)"
          >
            <span>{{ t("workspace.importDocuments") }}</span>
            <span class="text-gray-400">{{ documentsEntry?.exists ? t("workspace.available") : t("workspace.notExist") }}</span>
          </button>
          <button
            class="flex w-full items-center justify-between border-t border-gray-100 px-4 py-3 text-left text-sm text-gray-700 hover:bg-gray-50 disabled:opacity-50"
            :disabled="!desktopEntry?.exists || scanner.status === 'scanning'"
            @click="importWorkspace(desktopEntry)"
          >
            <span>{{ t("workspace.importDesktop") }}</span>
            <span class="text-gray-400">{{ desktopEntry?.exists ? t("workspace.available") : t("workspace.notExist") }}</span>
          </button>
        </div>
      </div>

      <p v-if="settings.error" class="mt-4 text-center text-sm text-red-600">
        {{ settings.error }}
      </p>

      <!-- 编辑器引导 · 旧版视图（纵向列表卡片，guideView === 'list' 时显示） -->
      <div
        v-if="editorGuideOpen && guideView === 'list'"
        class="mt-6 rounded-lg border border-gray-200 bg-white p-6 shadow-sm"
      >
        <div class="flex items-center justify-between">
          <h2 class="text-base font-semibold text-gray-900">
            {{ t("guide.title") }}
          </h2>
          <span class="text-xs text-gray-400">{{ t("guide.optional") }}</span>
        </div>
        <p class="mt-1 text-sm text-gray-500">{{ t("guide.subtitle") }}</p>

        <!-- 加载中 -->
        <div v-if="editorStore.candidatesLoading" class="mt-4 text-sm text-gray-400">
          {{ t("guide.loading") }}
        </div>

        <!-- 旧版：候选纵向列表（图标版独立展示在引导卡片下方） -->
        <div v-else-if="guideView === 'list'" class="mt-4 max-h-[240px] space-y-1 overflow-y-auto">
          <button
            v-for="ed in editorStore.candidates"
            :key="ed.id"
            class="flex w-full items-center justify-between gap-2 rounded-md border border-gray-100 px-3 py-2 text-left transition-colors hover:border-blue-200 hover:bg-blue-50"
            :class="guideSelectedId === ed.id ? 'border-blue-300 bg-blue-50' : ''"
            @click="pickGuideCandidate(ed)"
          >
            <span class="min-w-0 truncate text-sm text-gray-700">
              {{ ed.name }}
            </span>
            <span
              v-if="guideSelectedId === ed.id"
              class="shrink-0 text-[12px] font-medium text-[#2563EB]"
            >
              {{ t("guide.selected") }}
            </span>
            <span v-else class="shrink-0 text-[12px] text-[#2563EB]">{{ t("editor.select") }}</span>
          </button>
          <div v-if="editorStore.candidates.length === 0" class="py-4 text-center text-sm text-gray-400">
            {{ t("editor.noCandidates") }}
          </div>
        </div>

        <div class="mt-5 flex items-center justify-between">
          <button
            class="text-sm text-gray-500 transition-colors hover:text-gray-700"
            @click="finishEditorGuide"
          >
            {{ t("guide.skip") }}
          </button>
          <button
            class="rounded-lg bg-blue-600 px-5 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700"
            @click="finishEditorGuide"
          >
            {{ t("guide.done") }}
          </button>
        </div>

        <!-- 引导内确认框（非 Fork 候选） -->
        <div
          v-if="guidePendingCustom"
          class="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
          @click.self="guidePendingCustom = null"
        >
          <div class="w-[400px] rounded-[10px] border border-gray-200 bg-white p-5 shadow-lg">
            <h3 class="text-[15px] font-semibold text-gray-900">{{ t("editor.confirmTitle") }}</h3>
            <p class="mt-2 text-[13px] text-gray-600">
              {{ t("editor.confirmBody", { name: guidePendingCustom.name }) }}
            </p>
            <div class="mt-5 flex justify-end gap-2">
              <button
                class="rounded-[7px] border border-gray-200 px-4 py-2 text-[13px] text-gray-700 hover:bg-gray-50"
                @click="guidePendingCustom = null"
              >
                {{ t("editor.cancel") }}
              </button>
              <button
                class="rounded-[7px] bg-blue-600 px-4 py-2 text-[13px] font-medium text-white hover:bg-blue-700"
                @click="confirmGuideCustom(guidePendingCustom)"
              >
                {{ t("editor.confirm") }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- 新版图标滚动栏：独立于引导卡片展示（不受卡片 p-6 / border / bg 约束） -->
      <div
        v-if="editorGuideOpen && guideView === 'icons' && !editorStore.candidatesLoading"
        class="mt-4"
      >
        <!-- 溢出时提示可拖动 -->
        <p v-if="forkIcons.length > 0 && iconOverflow" class="text-xs text-gray-400">
          {{ t("guide.scrollHint") }}
        </p>

        <!-- Viewport：overflow-hidden（无滚动条）+ Pointer Drag / Wheel / Hover 交互 -->
        <div
          ref="iconScroller"
          class="mt-3 touch-pan-y select-none overflow-hidden border-y border-gray-200 py-3"
          @mouseenter="hovering = true"
          @mouseleave="hovering = false"
          @pointerdown="onPointerDown"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
          @pointercancel="onPointerUp"
          @wheel.prevent="onWheel"
        >
          <!-- Track：translate3d(offset) 位移；三份相同内容 [SET1][SET2][SET3] 无缝循环 -->
          <div ref="iconTrack" class="flex w-max will-change-transform" style="transform: translate3d(0,0,0)">
            <div
              v-for="setIdx in 3"
              :key="setIdx"
              :ref="(el) => (el ? setTrackRef(setIdx, el as HTMLElement) : null)"
              class="flex shrink-0 gap-3 pr-3"
            >
              <button
                v-for="(ed, index) in forkIcons"
                :key="ed.id + '-c' + setIdx + '-' + index"
                class="flex w-[112px] shrink-0 flex-col items-center justify-center rounded-[10px] p-2 transition-colors"
                :class="guideSelectedId === ed.id ? 'bg-blue-50' : 'hover:bg-gray-50'"
                :title="ed.name"
                @click="onIconClick(ed)"
              >
                <img
                  v-if="editorIconSrc(ed.icon_base64)"
                  :src="editorIconSrc(ed.icon_base64)"
                  :alt="ed.name"
                  class="h-[96px] w-[96px] rounded-[16px] object-contain"
                />
                <span
                  v-else
                  class="flex h-[96px] w-[96px] items-center justify-center rounded-[16px] bg-gray-100 text-[28px] font-medium text-gray-400"
                >
                  {{ ed.name.slice(0, 1).toUpperCase() }}
                </span>
                <span
                  v-if="guideSelectedId === ed.id"
                  class="mt-1 rounded-[3px] bg-blue-600 px-1.5 py-0.5 text-[10px] font-medium text-white"
                >
                  {{ t("guide.selected") }}
                </span>
              </button>
            </div>
          </div>
        </div>

        <!-- 无可用 Fork 图标 -->
        <div v-if="forkIcons.length === 0" class="py-4 text-center text-sm text-gray-400">
          {{ t("guide.noForkIcons") }}
        </div>
      </div>

      <!-- 新版 / 旧版 切换 + 完成（始终可见；新版模式下位于图标栏右下方） -->
      <div
        v-if="editorGuideOpen"
        class="mt-4 flex items-center justify-between"
      >
        <div class="flex items-center gap-2 text-[13px]">
          <span
            class="cursor-pointer font-medium transition-colors"
            :class="guideView === 'icons' ? 'text-[#2563EB]' : 'text-gray-400 hover:text-gray-600'"
            @click="switchGuideView('icons')"
          >
            {{ t("guide.viewNew") }}
          </span>
          <span class="text-gray-300">/</span>
          <span
            class="cursor-pointer font-medium transition-colors"
            :class="guideView === 'list' ? 'text-[#2563EB]' : 'text-gray-400 hover:text-gray-600'"
            @click="switchGuideView('list')"
          >
            {{ t("guide.viewOld") }}
          </span>
        </div>
        <!-- 新版（图标）模式隐藏了引导卡片，需要独立「完成」入口；旧版在卡片内已有 skip/done -->
        <button
          v-if="guideView === 'icons'"
          class="rounded-lg bg-blue-600 px-5 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700"
          @click="finishEditorGuide"
        >
          {{ t("guide.done") }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 图标滚动栏已改为原生 scrollLeft + JS 驱动无缝循环，移除原 CSS marquee（translateX 动画）。 */
</style>
