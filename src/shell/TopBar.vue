<script setup lang="ts">
import StatusChip, { type ConnectionState } from "./StatusChip.vue";

const props = withDefaults(
  defineProps<{
    title: string;
    subtitle?: string;
    connectionState: ConnectionState;
    showBack?: boolean;
    /** default true — matches Open Design tab screens */
    showRefresh?: boolean;
  }>(),
  {
    showBack: false,
    showRefresh: true,
  },
);

defineEmits<{
  back: [];
  refresh: [];
}>();
</script>

<template>
  <header class="topbar" :style="showBack ? { paddingLeft: '10px' } : undefined">
    <div class="topbar-lead">
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
      <div class="topbar-titles">
        <h1 class="title" :style="showBack ? { fontSize: '18px' } : undefined">
          {{ title }}
        </h1>
        <p v-if="subtitle" class="meta" style="margin: 2px 0 0">{{ subtitle }}</p>
      </div>
    </div>
    <div class="topbar-actions">
      <StatusChip :state="connectionState" />
      <button
        v-if="showRefresh"
        type="button"
        class="icon-btn icon-btn--sm"
        aria-label="刷新"
        :disabled="props.connectionState === 'connecting'"
        @click="$emit('refresh')"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path
            d="M21 12a9 9 0 1 1-2.64-6.36"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path d="M21 3v6h-6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </div>
  </header>
</template>
