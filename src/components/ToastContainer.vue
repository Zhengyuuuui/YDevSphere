<script setup lang="ts">
import { toastState, remove } from "@/lib/toast";

const tone = (type: string) => {
  if (type === "success") return "border-green-200 bg-green-50 text-green-800";
  if (type === "error") return "border-red-200 bg-red-50 text-red-800";
  return "border-gray-200 bg-white text-gray-800";
};
</script>

<template>
  <div class="pointer-events-none fixed inset-x-0 top-4 z-50 flex flex-col items-center gap-2">
    <transition-group name="toast">
      <div
        v-for="t in toastState.toasts"
        :key="t.id"
        class="pointer-events-auto flex items-center gap-2 rounded-lg border px-4 py-2 text-sm shadow-sm"
        :class="tone(t.type)"
      >
        <span>{{ t.message }}</span>
        <button class="ml-1 text-xs opacity-60 hover:opacity-100" @click="remove(t.id)">✕</button>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
