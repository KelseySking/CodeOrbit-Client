<script setup lang="ts">
defineProps<{
  sessionId: string;
  title: string;
}>();

const messages = [
  {
    role: "assistant" as const,
    text: "我先检查 App.vue 里的会话列表渲染，确认空态和加载态是否齐全。",
    time: "14:02",
  },
  {
    role: "user" as const,
    text: "好，重点看列表行截断和相对时间。",
    time: "14:03",
  },
  {
    role: "assistant" as const,
    text: "已定位：摘要超过一行时没有 ellipsis。准备改 session-row 样式。",
    time: "14:04",
  },
  {
    role: "user" as const,
    text: "顺便把权限审批卡片的三按钮也对齐一下。",
    time: "14:05",
  },
  {
    role: "assistant" as const,
    text: "收到。正在编辑 App.vue…",
    time: "14:06",
  },
];
</script>

<template>
  <section class="msg-stream">
    <div
      v-for="(m, i) in messages"
      :key="`${sessionId}-${i}`"
      class="msg"
      :class="m.role"
    >
      {{ m.text }}
      <span class="time num">{{ m.time }}</span>
    </div>
  </section>
</template>
