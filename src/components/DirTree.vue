<script setup lang="ts">
import { ref } from "vue";
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

/** 已加载的目录子项缓存：dirPath → DirNode[] */
const childrenCache = new Map<string, DirNode[]>();
/** 已展开的目录集合 */
const expanded = new Set<string>();
/** 正在加载的目录集合 */
const loading = new Set<string>();

const rootChildren = ref<DirNode[] | null>(null);
const error = ref<string | null>(null);

/** 加载某目录的直接子项（带缓存） */
async function loadChildren(dirPath: string): Promise<DirNode[]> {
  const cached = childrenCache.get(dirPath);
  if (cached) return cached;
  const children = await getDirChildren(dirPath);
  childrenCache.set(dirPath, children);
  return children;
}

/** 切换展开：首次展开时按需加载下级 */
async function toggle(node: DirNode) {
  if (expanded.has(node.path)) {
    expanded.delete(node.path);
    return;
  }
  expanded.add(node.path);
  if (!childrenCache.has(node.path)) {
    loading.add(node.path);
    try {
      await loadChildren(node.path);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      expanded.delete(node.path);
    } finally {
      loading.delete(node.path);
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
    <div v-if="rootChildren === null && !error" class="py-2 text-[13px] text-gray-500">
      {{ t("dirTree.loading") }}
    </div>

    <!-- 错误 -->
    <div v-else-if="error" class="py-2 text-[13px] text-red-600">
      {{ t("dirTree.failed", { msg: error }) }}
    </div>

    <!-- 空 -->
    <div
      v-else-if="rootChildren && rootChildren.length === 0"
      class="py-2 text-[13px] text-gray-400"
    >
      {{ t("dirTree.empty") }}
    </div>

    <!-- 树 -->
    <div v-else class="space-y-0.5">
      <DirTreeItem
        v-for="node in rootChildren ?? []"
        :key="node.path"
        :node="node"
        :expanded="expanded"
        :loading="loading"
        :cache="childrenCache"
        @toggle="toggle"
      />
    </div>
  </div>
</template>
