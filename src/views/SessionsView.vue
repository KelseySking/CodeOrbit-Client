<script setup lang="ts">
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
  emit("openSession", {
    id: s.sessionId,
    title: titleOf(s),
    subtitle: `${s.sourceDisplayName || s.source} · ${s.status || "会话"}`,
  });
}

function onDismiss(e: Event, id: string) {
  e.stopPropagation();
  emit("dismiss", id);
}
</script>

<template>
  <section class="pad stack" style="gap: 10px">
    <template v-if="connected">
      <div v-if="loading && !sessions.length" class="meta" style="padding: 12px 4px">加载中…</div>

      <div
        v-for="s in sessions"
        :key="s.sessionId"
        class="session-row"
        role="button"
        tabindex="0"
        @click="open(s)"
        @keydown.enter.prevent="open(s)"
        @keydown.space.prevent="open(s)"
      >
        <span class="dot" :class="{ idle: isIdle(s) }" aria-hidden="true" />
        <div>
          <div class="title">{{ titleOf(s) }}</div>
          <div class="sub">{{ subOf(s) }}</div>
        </div>
        <div class="right">
          <div class="num">{{ relativeTime(s.lastUpdatedAtUtc || s.createdAtUtc) }}</div>
          <div class="row" style="gap: 4px; justify-content: flex-end">
            <button
              type="button"
              class="icon-btn danger"
              aria-label="移除会话"
              @click="onDismiss($event, s.sessionId)"
            >
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M4 7h16" />
                <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
                <path d="M8 7l1 12a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1l1-12" />
                <path d="M10 11v6M14 11v6" />
              </svg>
            </button>
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
