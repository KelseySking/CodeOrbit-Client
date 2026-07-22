<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from "vue";
import { formatRuntimeError, type ChatMessageDto } from "../runtimeApi";
import { renderMarkdown } from "../utils/markdown";
import { animateMessageIn } from "../utils/motion";

type SessionClient = {
  getSessionMessages: (sessionId: string) => Promise<ChatMessageDto[]>;
  getSession: (sessionId: string) => Promise<{ recentMessages: ChatMessageDto[] }>;
};

const props = defineProps<{
  sessionId: string;
  title: string;
  client: SessionClient | null;
  /** parent bumps on WS session refresh / manual probe */
  reloadToken?: number;
  /** pending actions linked to this session */
  pendingCount?: number;
}>();

const emit = defineEmits<{
  goPending: [];
}>();

const messages = ref<ChatMessageDto[]>([]);
const loading = ref(false);
const error = ref("");
const stickBottom = ref(true);
const streamEl = ref<HTMLElement | null>(null);
/** keys of bubbles that should play enter animation once */
const animateKeys = ref<Set<string>>(new Set());

function msgKey(m: ChatMessageDto, i: number): string {
  return `${props.sessionId}-${i}-${m.timestampUtc}-${m.text.slice(0, 24)}`;
}

function sameMsg(a: ChatMessageDto, b: ChatMessageDto): boolean {
  return a.isUser === b.isUser && a.text === b.text && a.timestampUtc === b.timestampUtc;
}

function isPrefixAppend(prev: ChatMessageDto[], next: ChatMessageDto[]): boolean {
  if (prev.length === 0 || next.length <= prev.length) return false;
  for (let i = 0; i < prev.length; i++) {
    if (!sameMsg(prev[i], next[i])) return false;
  }
  return true;
}

function scrollToBottom() {
  const el = streamEl.value;
  if (!el) return;
  el.scrollTop = el.scrollHeight;
}

async function scrollToBottomWhenStuck() {
  if (!stickBottom.value) return;
  await nextTick();
  requestAnimationFrame(() => {
    scrollToBottom();
  });
}

async function playEnterAnimations() {
  const keys = animateKeys.value;
  if (!keys.size) return;
  await nextTick();
  requestAnimationFrame(() => {
    const root = streamEl.value;
    if (!root) {
      animateKeys.value = new Set();
      return;
    }
    const nodes = root.querySelectorAll<HTMLElement>("[data-msg-key]");
    for (const el of nodes) {
      const key = el.dataset.msgKey;
      if (key && keys.has(key)) void animateMessageIn(el);
    }
    animateKeys.value = new Set();
  });
}

async function load(opts: { silent?: boolean } = {}) {
  if (!props.client || !props.sessionId) {
    messages.value = [];
    animateKeys.value = new Set();
    return;
  }
  const silent = opts.silent && messages.value.length > 0;
  if (!silent) {
    loading.value = true;
    error.value = "";
  }
  try {
    let next: ChatMessageDto[];
    try {
      next = await props.client.getSessionMessages(props.sessionId);
    } catch {
      const session = await props.client.getSession(props.sessionId);
      next = session.recentMessages ?? [];
    }

    const prev = messages.value;
    const keys = new Set<string>();
    // first screen / empty prev → no animation; pure append → animate tail only
    if (isPrefixAppend(prev, next)) {
      for (let i = prev.length; i < next.length; i++) {
        keys.add(msgKey(next[i], i));
      }
    }
    animateKeys.value = keys;
    messages.value = next;
    error.value = "";
    await scrollToBottomWhenStuck();
    if (keys.size) await playEnterAnimations();
  } catch (e) {
    if (!silent) {
      error.value = formatRuntimeError(e);
      messages.value = [];
      animateKeys.value = new Set();
    }
  } finally {
    loading.value = false;
    // layout may settle after loading flag flips off
    await scrollToBottomWhenStuck();
  }
}

function roleOf(m: ChatMessageDto): "user" | "assistant" {
  return m.isUser ? "user" : "assistant";
}

function timeOf(iso: string): string {
  const t = Date.parse(iso);
  if (Number.isNaN(t)) return "";
  return new Date(t).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

function onScroll() {
  const el = streamEl.value;
  if (!el) return;
  stickBottom.value = el.scrollHeight - el.scrollTop - el.clientHeight < 48;
}

onMounted(() => {
  void load();
});

watch(
  () => props.sessionId,
  () => {
    stickBottom.value = true;
    animateKeys.value = new Set();
    void load();
  },
);

// WS / topbar refresh bumps sessionsEpoch → silent re-fetch messages
watch(
  () => props.reloadToken,
  (token, prev) => {
    if (token === undefined || token === prev) return;
    void load({ silent: true });
  },
);
</script>

<template>
  <div class="detail-root">
    <div v-if="(pendingCount ?? 0) > 0" class="pending-banner pad">
      <div class="pending-banner-inner">
        <div>
          <p class="h3" style="margin: 0 0 2px">
            本会话有 {{ pendingCount }} 条待处理
          </p>
          <p class="meta" style="margin: 0">权限审批或提问等待确认</p>
        </div>
        <button type="button" class="btn-text" @click="emit('goPending')">去处理</button>
      </div>
    </div>
    <section ref="streamEl" class="msg-stream" @scroll.passive="onScroll">
    <div v-if="loading && !messages.length" class="meta" style="padding: 12px 16px">
      加载消息…
    </div>
    <div v-else-if="error && !messages.length" class="banner" style="margin: 12px 16px">
      <span>{{ error }}</span>
      <button type="button" @click="load()">重试</button>
    </div>
    <template v-else-if="messages.length">
      <div
        v-for="(m, i) in messages"
        :key="msgKey(m, i)"
        class="msg"
        :class="roleOf(m)"
        :data-msg-key="msgKey(m, i)"
      >
        <div class="msg-body md" v-html="renderMarkdown(m.text)" />
        <span class="time num">{{ timeOf(m.timestampUtc) }}</span>
      </div>
    </template>
    <div v-else class="empty" style="padding: 24px 16px">
      <h2 class="h2">暂无消息</h2>
      <p class="meta">该会话还没有可展示的消息。</p>
    </div>
    </section>
  </div>
</template>

<style scoped>
.detail-root {
  flex: 1 1 auto;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
}
.pending-banner {
  flex: 0 0 auto;
  padding-top: 8px;
  padding-bottom: 4px;
}
.pending-banner-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 14px;
  border-radius: var(--radius-card);
  border: 1px solid color-mix(in srgb, var(--warn) 28%, var(--border));
  background: color-mix(in srgb, var(--warn) 10%, var(--surface));
}
.msg-stream {
  /* flex child owns scroll; avoid height:100% so pending banner does not force overflow */
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  -webkit-overflow-scrolling: touch;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 8px 18px 20px;
}

.msg-stream::-webkit-scrollbar {
  display: none;
}
</style>
