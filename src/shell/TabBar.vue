<script setup lang="ts">
export type TabId = "pending" | "sessions" | "connect";

defineProps<{
  active: TabId;
  pendingCount?: number;
  showBadge?: boolean;
}>();

defineEmits<{
  change: [tab: TabId];
}>();

function badgeText(count: number): string {
  if (count > 99) return "99+";
  return String(count);
}
</script>

<template>
  <nav class="tabbar" aria-label="主导航">
    <button
      type="button"
      class="tab"
      :class="{ active: active === 'pending' }"
      :aria-current="active === 'pending' ? 'page' : undefined"
      @click="$emit('change', 'pending')"
    >
      <span
        v-if="showBadge && (pendingCount ?? 0) > 0"
        class="badge"
        :aria-label="`${pendingCount} 条待处理`"
      >
        {{ badgeText(pendingCount ?? 0) }}
      </span>
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M8 6h13" />
        <path d="M8 12h13" />
        <path d="M8 18h13" />
        <path d="M3 6h.01" />
        <path d="M3 12h.01" />
        <path d="M3 18h.01" />
      </svg>
      待处理
    </button>

    <button
      type="button"
      class="tab"
      :class="{ active: active === 'sessions' }"
      :aria-current="active === 'sessions' ? 'page' : undefined"
      @click="$emit('change', 'sessions')"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
      </svg>
      会话
    </button>

    <button
      type="button"
      class="tab"
      :class="{ active: active === 'connect' }"
      :aria-current="active === 'connect' ? 'page' : undefined"
      @click="$emit('change', 'connect')"
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M5 12.55a11 11 0 0 1 14.08 0" />
        <path d="M1.42 9a16 16 0 0 1 21.16 0" />
        <path d="M8.53 16.11a6 6 0 0 1 6.95 0" />
        <circle cx="12" cy="20" r="1" />
      </svg>
      连接
    </button>
  </nav>
</template>
