<script setup lang="ts">
defineProps<{
  connected: boolean;
}>();

const emit = defineEmits<{
  goConnect: [];
  openSession: [session: { id: string; title: string; subtitle: string }];
}>();

const sessions = [
  {
    id: "s1",
    title: "CodeOrbit-Client",
    sub: "Claude Code · 正在编辑 App.vue…",
    time: "2 分钟前",
    idle: false,
  },
  {
    id: "s2",
    title: "MyApp · auth-flow",
    sub: "Claude Code · 等待权限：Write",
    time: "8 分钟前",
    idle: false,
  },
  {
    id: "s3",
    title: "docs-site",
    sub: "Claude Code · 已完成 README 修订",
    time: "1 小时前",
    idle: true,
  },
  {
    id: "s4",
    title: "runtime-bridge",
    sub: "Claude Code · 修复 Token 刷新竞态",
    time: "昨天",
    idle: true,
  },
];

function open(s: (typeof sessions)[number]) {
  emit("openSession", {
    id: s.id,
    title: s.title,
    subtitle: s.idle ? "Claude Code · 已完成" : "Claude Code · 进行中",
  });
}
</script>

<template>
  <section class="pad stack" style="gap: 10px">
    <template v-if="connected">
      <button
        v-for="s in sessions"
        :key="s.id"
        type="button"
        class="session-row"
        @click="open(s)"
      >
        <span class="dot" :class="{ idle: s.idle }" aria-hidden="true" />
        <div>
          <div class="title">{{ s.title }}</div>
          <div class="sub">{{ s.sub }}</div>
        </div>
        <div class="right">
          <div class="num">{{ s.time }}</div>
          <svg class="chev" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M9 6l6 6-6 6" />
          </svg>
        </div>
      </button>
    </template>
    <template v-else>
      <div class="empty">
        <div class="icon" aria-hidden="true">
          <svg viewBox="0 0 24 24">
            <path
              d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"
            />
          </svg>
        </div>
        <h2 class="h2">暂无会话</h2>
        <p>连接 Runtime 后，Agent 会话会出现在这里。</p>
      </div>
      <button type="button" class="btn-primary" @click="$emit('goConnect')">去连接</button>
    </template>
  </section>
</template>
