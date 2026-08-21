<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { DirNode } from "@/types";

/**
 * 目录树递归节点（v0.2 懒加载）。
 * 每个节点仅渲染自身 + 展开后的直接子项（子项继续递归本组件），
 * 由父级 DirTree 统一维护 expanded/loading/childrenCache 状态。
 */

const { t } = useI18n();

defineProps<{
  node: DirNode;
  /** 已展开的目录路径集合 */
  expanded: Set<string>;
  /** 正在加载子项的目录路径集合 */
  loading: Set<string>;
  /** 目录路径 → 直接子项缓存 */
  cache: Map<string, DirNode[]>;
}>();

const emit = defineEmits<{
  (e: "toggle", node: DirNode): void;
}>();

function isExpandable(node: DirNode): boolean {
  return node.is_dir && node.children_count > 0;
}
</script>

<template>
  <div>
    <!-- 当前节点行 -->
    <div
      class="flex items-center gap-1.5 rounded-[5px] px-1.5 py-1 transition-colors hover:bg-surface-2"
      :class="isExpandable(node) ? 'cursor-pointer' : ''"
      @click="isExpandable(node) && emit('toggle', node)"
    >
      <button
        v-if="isExpandable(node)"
        class="flex h-[16px] w-[16px] shrink-0 items-center justify-center rounded-[3px] text-faint hover:bg-surface-2 hover:text-muted"
        @click.stop="emit('toggle', node)"
      >
        <svg
          v-if="loading.has(node.path)"
          width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
          class="animate-spin"
        >
          <path d="M21 12a9 9 0 1 1-2.64-6.36" />
          <path d="M21 3v6h-6" />
        </svg>
        <svg
          v-else
          width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
          :class="expanded.has(node.path) ? 'rotate-90 transition-transform' : 'transition-transform'"
        >
          <path d="M9 6l6 6-6 6" />
        </svg>
      </button>
      <span v-else class="h-[16px] w-[16px] shrink-0" />

      <!-- 目录 / 文件图标 -->
      <svg
        v-if="node.is_dir"
        width="14" height="14" viewBox="0 0 16 16" fill="none"
        class="shrink-0 text-[#C4C9D0]"
      >
        <path
          d="M2 5C2 4.17 2.67 3.5 3.5 3.5H6L7.5 5H12.5C13.33 5 14 5.67 14 6.5V11.5C14 12.33 13.33 13 12.5 13H3.5C2.67 13 2 12.33 2 11.5V5Z"
          stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"
        />
      </svg>
      <svg
        v-else
        width="14" height="14" viewBox="0 0 16 16" fill="none"
        class="shrink-0 text-[#C4C9D0]"
      >
        <path
          d="M3.5 2.5H8L9.5 4H12.5C13.33 4 14 4.67 14 5.5V12.5C14 12.33 13.33 13 12.5 13H3.5C2.67 13 2 13.33 2 12.5V3.5C2 2.67 2.67 2 3.5 2.5Z"
          stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"
        />
      </svg>

      <span class="min-w-0 truncate text-[14px] text-ink">{{ node.name }}</span>

      <!-- 真项目根标记 -->
      <span
        v-if="node.has_manifest"
        class="inline-flex shrink-0 items-center rounded-[3px] bg-[#ECFDF5] px-[5px] py-[1px] text-[10px] font-medium leading-[14px] text-[#059669]"
        :title="t('dirTree.projectRootTitle')"
      >
        {{ t("dirTree.projectRoot") }}
      </span>
    </div>

    <!-- 子节点（递归，懒加载） -->
    <div v-if="expanded.has(node.path)" class="ml-4 border-l border-[#EAECEF] pl-1">
      <DirTreeItem
        v-for="child in cache.get(node.path) ?? []"
        :key="child.path"
        :node="child"
        :expanded="expanded"
        :loading="loading"
        :cache="cache"
        @toggle="emit('toggle', $event)"
      />
    </div>
  </div>
</template>

<script lang="ts">
// 递归组件自引用（Vue SFC 默认支持通过文件名自引用）
export default { name: "DirTreeItem" };
</script>
