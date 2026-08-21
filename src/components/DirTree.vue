<script setup lang="ts">
import { reactive, ref } from "vue";
import { useI18n } from "vue-i18n";
import { getDirChildren } from "@/api/project";
import type { DirNode } from "@/types";
import DirTreeItem from "./DirTreeItem.vue";

const { t } = useI18n();

const props = defineProps<{
  /** 根目录绝对路径（初始调 getDirChildren(rootPath) 展示直接子项） */
  rootPath: string;
}>();

/**
 * 懒加载目录树（v0.2）。
 * - 初始只展示根目录的直接子项（tree -L 1 效果）。
 * - 点击可展开的文件夹 → 实时调 getDirChildren(childPath) 加载下级插入。
 * - 仅 `is_dir && children_count > 0` 才可展开；`has_manifest` 标记真项目根。
 */

/**
 * 目录树响应式状态容器（v0.4 修复）。
 * 必须用 reactive 包裹 Set/Map：Vue 的 reactive collection 会对
 * add/delete/set 触发依赖（ref(new Set()) 不会触发），从而让
 * DirTreeItem 的 v-if="expanded.has(node.path)" 得以重求值。
 */
const state = reactive({
  /** 已加载的目录子项缓存：dirPath → DirNode[] */
  childrenCache: new Map<string, DirNode[]>(),
  /** 已展开的目录集合 */
  expanded: new Set<string>(),
  /** 正在加载的目录集合 */
  loading: new Set<string>(),
});

const rootChildren = ref<DirNode[] | null>(null);
const error = ref<string | null>(null);

/** 加载某目录的直接子项（带缓存） */
async function loadChildren(dirPath: string): Promise<DirNode[]> {
  const cached = state.childrenCache.get(dirPath);
  if (cached) return cached;
  const children = await getDirChildren(dirPath);
  state.childrenCache.set(dirPath, children);
  return children;
}

/** 切换展开：首次展开时按需加载下级 */
async function toggle(node: DirNode) {
  if (state.expanded.has(node.path)) {
    state.expanded.delete(node.path);
    return;
  }
  state.expanded.add(node.path);
  if (!state.childrenCache.has(node.path)) {
    state.loading.add(node.path);
    try {
      await loadChildren(node.path);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      state.expanded.delete(node.path);
    } finally {
      state.loading.delete(node.path);
    }
  }
}

/** 初始加载根目录 */
async function loadRoot() {
  error.value = null;
  try {
    rootChildren.value = await loadChildren(props.rootPath);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

loadRoot();
</script>

<template>
  <div class="text-sm">
    <!-- 加载中 -->
    <div v-if="rootChildren === null && !error" class="py-2 text-[14px] text-muted">
      {{ t("dirTree.loading") }}
    </div>

    <!-- 错误 -->
    <div v-else-if="error" class="py-2 text-[14px] text-red-600">
      {{ t("dirTree.failed", { msg: error }) }}
    </div>

    <!-- 空 -->
    <div
      v-else-if="rootChildren && rootChildren.length === 0"
      class="py-2 text-[14px] text-faint"
    >
      {{ t("dirTree.empty") }}
    </div>

    <!-- 树 -->
    <div v-else class="space-y-0.5">
      <DirTreeItem
        v-for="node in rootChildren ?? []"
        :key="node.path"
        :node="node"
        :expanded="state.expanded"
        :loading="state.loading"
        :cache="state.childrenCache"
        @toggle="toggle"
      />
    </div>
  </div>
</template>
