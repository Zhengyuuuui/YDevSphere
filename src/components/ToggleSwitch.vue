<script setup lang="ts">
defineProps<{
  enabled: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle"): void;
}>();
</script>

<template>
  <button
    class="relative h-5 w-9 shrink-0 rounded-full transition-colors duration-150"
    :class="enabled ? 'bg-[#2563EB]' : 'bg-[#D1D5DB]'"
    @click="emit('toggle')"
  >
    <span
      class="absolute top-0.5 h-4 w-4 rounded-full bg-white transition-transform duration-150"
      :class="enabled ? 'translate-x-4' : 'translate-x-0.5'"
      style="box-shadow: 0 1px 3px rgba(0,0,0,0.15)"
    />
  </button>
</template>
