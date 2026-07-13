<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { formatRuntimeError, type ChatMessageDto } from "../runtimeApi";

type SessionClient = {
  getSessionMessages: (sessionId: string) => Promise<ChatMessageDto[]>;
  getSession: (sessionId: string) => Promise<{ recentMessages: ChatMessageDto[] }>;
};

const props = defineProps<{
  sessionId: string;
  title: string;
  client: SessionClient | null;
}>();

const messages = ref<ChatMessageDto[]>([]);
const loading = ref(false);
const error = ref("");

async function load() {
  if (!props.client || !props.sessionId) {
    messages.value = [];
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    try {
      messages.value = await props.client.getSessionMessages(props.sessionId);
    } catch {
      const session = await props.client.getSession(props.sessionId);
      messages.value = session.recentMessages ?? [];
    }
  } catch (e) {
    error.value = formatRuntimeError(e);
    messages.value = [];
  } finally {
    loading.value = false;
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

onMounted(load);
watch(() => [props.sessionId, props.client] as const, load);
</script>

<template>
  <section class="msg-stream">
    <div v-if="loading" class="meta" style="padding: 12px 16px">加载消息…</div>
    <div v-else-if="error" class="banner" style="margin: 12px 16px">
      <span>{{ error }}</span>
      <button type="button" @click="load">重试</button>
    </div>
    <template v-else-if="messages.length">
      <div
        v-for="(m, i) in messages"
        :key="`${sessionId}-${i}-${m.timestampUtc}`"
        class="msg"
        :class="roleOf(m)"
      >
        {{ m.text }}
        <span class="time num">{{ timeOf(m.timestampUtc) }}</span>
      </div>
    </template>
    <div v-else class="empty" style="padding: 24px 16px">
      <h2 class="h2">暂无消息</h2>
      <p class="meta">该会话还没有可展示的消息。</p>
    </div>
  </section>
</template>
