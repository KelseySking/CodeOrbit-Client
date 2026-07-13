<script setup lang="ts">
import { ref } from "vue";
import type { ConnectionState } from "../shell/StatusChip.vue";

defineProps<{
  connectionState: ConnectionState;
}>();

const emit = defineEmits<{
  goConnect: [];
  toast: [message: string];
  retry: [];
}>();

const resolved = ref<Record<string, boolean>>({});
const selectedEnv = ref("Staging");
const freeAnswer = ref("");
const answerLoading = ref(false);

function perm(action: "allow" | "always" | "deny", id: string) {
  const map = {
    allow: "已允许",
    always: "已设为总是允许",
    deny: "已拒绝",
  } as const;
  resolved.value = { ...resolved.value, [id]: true };
  emit("toast", map[action]);
}

function answer() {
  const text = freeAnswer.value.trim() || selectedEnv.value;
  if (!text) {
    emit("toast", "请选择或输入回答");
    return;
  }
  answerLoading.value = true;
  window.setTimeout(() => {
    answerLoading.value = false;
    resolved.value = { ...resolved.value, q1: true };
    emit("toast", `已回答：${text}`);
  }, 450);
}
</script>

<template>
  <section class="pad stack" style="gap: 12px">
    <div v-if="connectionState === 'offline'" class="banner">
      <span>无法访问 Runtime</span>
      <button type="button" @click="$emit('retry')">重试</button>
    </div>

    <template v-if="connectionState === 'connected'">
      <div
        class="card soft stack"
        style="gap: 12px"
        :style="resolved.bash ? { opacity: 0.45 } : undefined"
      >
        <div class="row-between">
          <span class="chip tag">权限</span>
          <span class="meta">刚刚</span>
        </div>
        <div>
          <p class="meta" style="margin: 0 0 4px">Claude Code · MyApp</p>
          <h2 class="h2" style="margin: 0 0 6px">Bash</h2>
          <p style="margin: 0; color: var(--muted); font-size: 14px">
            运行命令: <span class="num">npm test</span>
          </p>
        </div>
        <details class="details">
          <summary>查看参数</summary>
          <pre>cwd: /Users/dev/MyApp
command: npm test
timeout: 120s
network: false</pre>
        </details>
        <div class="btn-row">
          <button
            type="button"
            class="btn-primary"
            :disabled="resolved.bash"
            @click="perm('allow', 'bash')"
          >
            允许
          </button>
          <button
            type="button"
            class="btn-secondary"
            :disabled="resolved.bash"
            @click="perm('always', 'bash')"
          >
            总是允许
          </button>
          <button
            type="button"
            class="btn-danger"
            :disabled="resolved.bash"
            @click="perm('deny', 'bash')"
          >
            拒绝
          </button>
        </div>
      </div>

      <div
        class="card soft stack"
        style="gap: 14px"
        :style="resolved.q1 ? { opacity: 0.55 } : undefined"
      >
        <div class="row-between">
          <div class="row" style="gap: 8px">
            <span class="chip tag q">问答</span>
            <span class="meta">步骤 1/2</span>
          </div>
          <span class="meta">Claude Code · MyApp</span>
        </div>
        <div>
          <h2 class="h2" style="margin: 0 0 8px">选择部署环境</h2>
          <p style="margin: 0; color: var(--muted); font-size: 14px">
            你希望部署到哪个环境？
          </p>
        </div>
        <div class="row" style="flex-wrap: wrap; gap: 8px">
          <button
            type="button"
            class="chip choice"
            :class="{ selected: selectedEnv === 'Staging' }"
            :aria-pressed="selectedEnv === 'Staging'"
            @click="selectedEnv = 'Staging'"
          >
            Staging
          </button>
          <button
            type="button"
            class="chip choice"
            :class="{ selected: selectedEnv === 'Production' }"
            :aria-pressed="selectedEnv === 'Production'"
            @click="selectedEnv = 'Production'"
          >
            Production
          </button>
        </div>
        <div class="field">
          <label for="free-answer">或输入回答</label>
          <div class="control">
            <input
              id="free-answer"
              v-model="freeAnswer"
              type="text"
              placeholder="例如：先上 Staging，今晚再 Production"
            />
          </div>
        </div>
        <div class="row" style="gap: 8px">
          <button
            type="button"
            class="btn-primary"
            style="flex: 1"
            :class="{ 'is-loading': answerLoading }"
            :disabled="resolved.q1 || answerLoading"
            @click="answer"
          >
            {{ answerLoading ? "提交中…" : "回答" }}
          </button>
          <button
            type="button"
            class="btn-secondary"
            style="flex: 0 0 96px; width: auto"
            :disabled="resolved.q1"
            @click="
              resolved = { ...resolved, q1: true };
              $emit('toast', '已关闭问题');
            "
          >
            关闭
          </button>
        </div>
      </div>
    </template>

    <template v-else-if="connectionState === 'offline'">
      <div class="empty">
        <div class="icon" aria-hidden="true">
          <svg viewBox="0 0 24 24">
            <path d="M8 6h13" />
            <path d="M8 12h13" />
            <path d="M8 18h13" />
            <path d="M3 6h.01" />
            <path d="M3 12h.01" />
            <path d="M3 18h.01" />
          </svg>
        </div>
        <h2 class="h2">暂无待处理</h2>
        <p>AI 请求权限或提问时会显示在这里。当前离线，连接恢复后自动同步。</p>
      </div>
      <div class="card soft">
        <p class="h3" style="margin: 0 0 6px">会话暂不可用</p>
        <p class="meta" style="margin: 0 0 12px">未连接 Runtime 时无法拉取会话列表。</p>
        <button type="button" class="btn-primary" @click="$emit('goConnect')">去连接</button>
      </div>
    </template>

    <template v-else>
      <div class="empty">
        <div class="icon" aria-hidden="true">
          <svg viewBox="0 0 24 24">
            <path d="M8 6h13" />
            <path d="M8 12h13" />
            <path d="M8 18h13" />
            <path d="M3 6h.01" />
            <path d="M3 12h.01" />
            <path d="M3 18h.01" />
          </svg>
        </div>
        <h2 class="h2">暂无待处理</h2>
        <p>AI 请求权限或提问时会显示在这里。</p>
      </div>
      <button type="button" class="btn-primary" @click="$emit('goConnect')">去连接</button>
    </template>
  </section>
</template>
