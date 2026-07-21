<script setup lang="ts">
import { ref } from "vue";
import { summarizeSession, type SessionDto } from "../runtimeApi";

defineProps<{
  connected: boolean;
  sessions: SessionDto[];
  loading: boolean;
}>();

const emit = defineEmits<{
  goConnect: [];
  openSession: [session: { id: string; title: string; subtitle: string }];
  dismiss: [sessionId: string];
}>();

const SWIPE_OPEN = 72;
const openedId = ref<string | null>(null);
const dragId = ref<string | null>(null);
const dragX = ref(0);
let startX = 0;
let startY = 0;
let tracking = false;
let decided = false;
let horizontal = false;
let baseX = 0;

function titleOf(s: SessionDto): string {
  return (
    s.projectName?.trim() ||
    s.workingDirectory?.trim()?.split(/[/\\]/).filter(Boolean).pop() ||
    s.sessionId
  );
}

function subOf(s: SessionDto): string {
  const source = s.sourceDisplayName || s.source;
  const summary = summarizeSession(s);
  return summary ? `${source} · ${summary}` : source;
}

function isIdle(s: SessionDto): boolean {
  const st = (s.status || "").toLowerCase();
  return st.includes("idle") || st.includes("complete") || st.includes("done") || st.includes("end");
}

function relativeTime(iso: string): string {
  const t = Date.parse(iso);
  if (Number.isNaN(t)) return "";
  const sec = Math.round((Date.now() - t) / 1000);
  if (sec < 60) return "刚刚";
  if (sec < 3600) return `${Math.floor(sec / 60)} 分钟前`;
  if (sec < 86400) return `${Math.floor(sec / 3600)} 小时前`;
  if (sec < 86400 * 2) return "昨天";
  return new Date(t).toLocaleDateString();
}

function open(s: SessionDto) {
  // 展开态点本行 → 先收起；点其他未展开行 → 收起后打开
  if (openedId.value === s.sessionId) {
    openedId.value = null;
    return;
  }
  openedId.value = null;
  emit("openSession", {
    id: s.sessionId,
    title: titleOf(s),
    subtitle: `${s.sourceDisplayName || s.source} · ${s.status || "会话"}`,
  });
}

function frontStyle(id: string): Record<string, string> | undefined {
  const x = offsetOf(id);
  // only set transform when needed so .session-row:active scale still works
  return x ? { transform: `translateX(${x}px)` } : undefined;
}

function onDismiss(e: Event, id: string) {
  e.stopPropagation();
  openedId.value = null;
  emit("dismiss", id);
}

function offsetOf(id: string): number {
  if (dragId.value === id) return dragX.value;
  return openedId.value === id ? -SWIPE_OPEN : 0;
}

function onTouchStart(e: TouchEvent, id: string) {
  if (e.touches.length !== 1) return;
  const t = e.touches[0];
  startX = t.clientX;
  startY = t.clientY;
  tracking = true;
  decided = false;
  horizontal = false;
  baseX = openedId.value === id ? -SWIPE_OPEN : 0;
  if (openedId.value && openedId.value !== id) {
    openedId.value = null;
    baseX = 0;
  }
  dragId.value = id;
  dragX.value = baseX;
}

function onTouchMove(e: TouchEvent, id: string) {
  if (!tracking || dragId.value !== id || e.touches.length !== 1) return;
  const t = e.touches[0];
  const dx = t.clientX - startX;
  const dy = t.clientY - startY;
  if (!decided) {
    if (Math.abs(dx) < 8 && Math.abs(dy) < 8) return;
    decided = true;
    horizontal = Math.abs(dx) > Math.abs(dy);
    if (!horizontal) {
      tracking = false;
      dragId.value = null;
      return;
    }
  }
  if (!horizontal) return;
  e.preventDefault();
  const next = Math.min(0, Math.max(-SWIPE_OPEN, baseX + dx));
  dragX.value = next;
}

function onTouchEnd(id: string) {
  if (dragId.value !== id) return;
  const x = dragX.value;
  dragId.value = null;
  tracking = false;
  if (!decided || !horizontal) {
    dragX.value = 0;
    return;
  }
  openedId.value = x <= -SWIPE_OPEN / 2 ? id : null;
  dragX.value = 0;
}

function onTouchCancel(id: string) {
  if (dragId.value !== id) return;
  dragId.value = null;
  tracking = false;
  dragX.value = 0;
}
</script>

<template>
  <section class="pad stack" style="gap: 10px">
    <template v-if="connected">
      <div v-if="loading && !sessions.length" class="meta" style="padding: 12px 4px">加载中…</div>

      <div v-for="s in sessions" :key="s.sessionId" class="session-swipe">
        <button
          type="button"
          class="session-swipe-delete"
          aria-label="移除会话"
          @click="onDismiss($event, s.sessionId)"
        >
          删除
        </button>
        <div
          class="session-row session-swipe-front"
          :class="{
            'is-swiping': dragId === s.sessionId,
            'is-open': openedId === s.sessionId && dragId !== s.sessionId,
          }"
          role="button"
          tabindex="0"
          :style="frontStyle(s.sessionId)"
          @click="open(s)"
          @keydown.enter.prevent="open(s)"
          @keydown.space.prevent="open(s)"
          @touchstart.passive="onTouchStart($event, s.sessionId)"
          @touchmove="onTouchMove($event, s.sessionId)"
          @touchend="onTouchEnd(s.sessionId)"
          @touchcancel="onTouchCancel(s.sessionId)"
        >
          <span class="dot" :class="{ idle: isIdle(s) }" aria-hidden="true" />
          <div class="body">
            <div class="head">
              <div class="title">{{ titleOf(s) }}</div>
              <div class="num time">{{ relativeTime(s.lastUpdatedAtUtc || s.createdAtUtc) }}</div>
            </div>
            <div class="sub">{{ subOf(s) }}</div>
          </div>
          <div class="actions">
            <svg class="chev" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M9 6l6 6-6 6" />
            </svg>
          </div>
        </div>
      </div>

      <div v-if="!loading && !sessions.length" class="empty">
        <div class="icon" aria-hidden="true">
          <svg viewBox="0 0 24 24">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
          </svg>
        </div>
        <h2 class="h2">暂无会话</h2>
        <p>Agent 会话会出现在这里。</p>
      </div>
    </template>
    <template v-else>
      <div class="empty">
        <div class="icon" aria-hidden="true">
          <svg viewBox="0 0 24 24">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
          </svg>
        </div>
        <h2 class="h2">暂无会话</h2>
        <p>连接 Runtime 后，Agent 会话会出现在这里。</p>
      </div>
      <button type="button" class="btn-primary" @click="$emit('goConnect')">去连接</button>
    </template>
  </section>
</template>
