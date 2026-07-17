<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from "vue";
import { formatRuntimeError, type ChatMessageDto } from "../runtimeApi";
import { renderMarkdown } from "../utils/markdown";

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
}>();

const messages = ref<ChatMessageDto[]>([]);
const loading = ref(false);
const error = ref("");
const stickBottom = ref(true);
const streamEl = ref<HTMLElement | null>(null);

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

async function load(opts: { silent?: boolean } = {}) {
  if (!props.client || !props.sessionId) {
    messages.value = [];
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
    messages.value = next;
    error.value = "";
    await scrollToBottomWhenStuck();
  } catch (e) {
    if (!silent) {
      error.value = formatRuntimeError(e);
      messages.value = [];
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
        :key="`${sessionId}-${i}-${m.timestampUtc}-${m.text.slice(0, 24)}`"
        class="msg"
        :class="roleOf(m)"
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
</template>

<style scoped>
.msg-stream {
  flex: 1 1 auto;
  height: 100%;
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
