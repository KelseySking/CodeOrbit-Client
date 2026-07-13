<script setup lang="ts">
import StatusChip, { type ConnectionState } from "./StatusChip.vue";

const props = defineProps<{
  title: string;
  subtitle?: string;
  connectionState: ConnectionState;
  showBack?: boolean;
  showRefresh?: boolean;
}>();

defineEmits<{
  back: [];
  refresh: [];
}>();
</script>

<template>
  <header class="topbar" :style="showBack ? { paddingLeft: '10px' } : undefined">
    <div class="row" style="gap: 4px; min-width: 0; flex: 1">
      <button
        v-if="showBack"
        type="button"
        class="back-btn"
        aria-label="返回"
        @click="$emit('back')"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M15 6l-6 6 6 6" />
        </svg>
      </button>
      <div style="min-width: 0">
        <h1 class="title" :style="showBack ? { fontSize: '18px' } : undefined">
          {{ title }}
        </h1>
        <p v-if="subtitle" class="meta" style="margin: 2px 0 0">{{ subtitle }}</p>
      </div>
    </div>
    <div class="topbar-actions">
      <StatusChip :state="connectionState" />
      <button
        v-if="showRefresh !== false && !showBack"
        type="button"
        class="icon-btn"
        aria-label="刷新"
        :disabled="props.connectionState === 'connecting'"
        @click="$emit('refresh')"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M21 12a9 9 0 1 1-2.64-6.36" />
          <path d="M21 3v6h-6" />
        </svg>
      </button>
    </div>
  </header>
</template>
