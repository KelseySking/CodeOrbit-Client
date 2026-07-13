<script setup lang="ts">
export type ConnectionState =
  | "idle"
  | "connecting"
  | "connected"
  | "unauthorized"
  | "offline";

const props = defineProps<{
  state: ConnectionState;
}>();

const labels: Record<ConnectionState, string> = {
  idle: "未连接",
  connecting: "连接中…",
  connected: "已连接",
  unauthorized: "未授权",
  offline: "无法访问",
};

const chipClass = (state: ConnectionState) => {
  if (state === "connected" || state === "connecting") return "chip online";
  if (state === "offline") return "chip warn";
  if (state === "unauthorized") return "chip danger";
  return "chip";
};
</script>

<template>
  <span :class="chipClass(props.state)">
    <span class="dot" aria-hidden="true" />
    {{ labels[props.state] }}
  </span>
</template>
