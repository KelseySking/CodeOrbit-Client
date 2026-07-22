<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import type { ConnectionState } from "../shell/StatusChip.vue";
import {
  currentQuestion,
  type PendingActionDto,
  type QuestionItemDto,
  type QuestionOptionDto,
} from "../runtimeApi";

const props = defineProps<{
  connectionState: ConnectionState;
  pending: PendingActionDto[];
  loading: boolean;
}>();

const emit = defineEmits<{
  goConnect: [];
  toast: [message: string];
  retry: [];
  allow: [actionId: string, always: boolean];
  deny: [actionId: string, reason: string];
  answer: [actionId: string, answers: string[]];
  dismiss: [actionId: string];
}>();

const busyId = ref("");
const freeAnswers = reactive<Record<string, string>>({});
const selectedOptions = reactive<Record<string, string[]>>({});

const isConnected = computed(() => props.connectionState === "connected");

// loadPending always replaces the array; reset draft answers + busy on each refresh
// (multi-step keeps the same actionId, so id-only watches would stick busy / old choices)
watch(
  () => props.pending,
  (list) => {
    busyId.value = "";
    for (const action of list) {
      freeAnswers[action.actionId] = "";
      selectedOptions[action.actionId] = [];
    }
  },
);

watch(
  () => props.loading,
  () => {
    busyId.value = "";
  },
);

function isPermission(action: PendingActionDto): boolean {
  return !!action.permission || action.kind.toLowerCase().includes("permission");
}

function isQuestion(action: PendingActionDto): boolean {
  return !!action.question || action.kind.toLowerCase().includes("question");
}

function sourceLine(action: PendingActionDto): string {
  return [action.sourceDisplayName || action.source, action.projectName].filter(Boolean).join(" · ");
}

function relativeTime(iso: string): string {
  const t = Date.parse(iso);
  if (Number.isNaN(t)) return "";
  const sec = Math.round((Date.now() - t) / 1000);
  if (sec < 60) return "刚刚";
  if (sec < 3600) return `${Math.floor(sec / 60)} 分钟前`;
  if (sec < 86400) return `${Math.floor(sec / 3600)} 小时前`;
  return new Date(t).toLocaleString();
}

function toolInputText(action: PendingActionDto): string {
  const input = action.permission?.toolInput;
  if (!input) return "";
  try {
    return JSON.stringify(input, null, 2);
  } catch {
    return String(input);
  }
}

function questionStep(action: PendingActionDto): string {
  const q = action.question;
  if (!q?.isMultiQuestion || !q.questions?.length) return "";
  return `步骤 ${q.currentQuestionIndex + 1}/${q.questions.length}`;
}

function currentQ(action: PendingActionDto): QuestionItemDto | null {
  const q = currentQuestion(action);
  if (!q) return null;
  return {
    id: "id" in q ? ((q as QuestionItemDto).id ?? null) : null,
    question: q.question,
    header: q.header,
    options: q.options ?? [],
    multiSelect: q.multiSelect,
    allowFreeText: "allowFreeText" in q ? !!(q as QuestionItemDto).allowFreeText : true,
  };
}

function optionValue(opt: QuestionOptionDto): string {
  return (opt.value || opt.label || "").trim();
}

function isSelected(actionId: string, value: string, multi: boolean): boolean {
  const sel = selectedOptions[actionId] ?? [];
  return multi ? sel.includes(value) : sel[0] === value;
}

function toggleOption(actionId: string, value: string, multi: boolean) {
  if (!value) return;
  if (multi) {
    const cur = selectedOptions[actionId] ?? [];
    selectedOptions[actionId] = cur.includes(value)
      ? cur.filter((v) => v !== value)
      : [...cur, value];
  } else {
    selectedOptions[actionId] = [value];
  }
}

function emitAllow(actionId: string, always: boolean) {
  if (busyId.value) return;
  busyId.value = actionId;
  emit("allow", actionId, always);
}

function emitDeny(actionId: string) {
  if (busyId.value) return;
  busyId.value = actionId;
  emit("deny", actionId, "用户拒绝");
}

function emitAnswer(action: PendingActionDto) {
  if (busyId.value) return;
  const q = currentQ(action);
  const free = (freeAnswers[action.actionId] ?? "").trim();
  const sel = selectedOptions[action.actionId] ?? [];
  const multi = !!q?.multiSelect;
  let answers: string[] = [];
  if (free) answers = [free];
  else if (sel.length) answers = multi ? sel : [sel[0]!];
  if (!answers.length) {
    emit("toast", "请选择或输入回答");
    return;
  }
  busyId.value = action.actionId;
  emit("answer", action.actionId, answers);
}

function emitDismiss(actionId: string) {
  if (busyId.value) return;
  busyId.value = actionId;
  emit("dismiss", actionId);
}
</script>

<template>
  <section class="pad stack" style="gap: 12px">
    <div v-if="connectionState === 'offline'" class="banner">
      <span>无法访问 CodeOrbit</span>
      <button type="button" @click="$emit('retry')">重试</button>
    </div>
    <div
      v-else-if="connectionState === 'unauthorized'"
      class="banner"
      style="color: var(--danger)"
    >
      <span>Token 无效，请重新连接</span>
      <button type="button" @click="$emit('goConnect')">去连接</button>
    </div>

    <template v-if="isConnected">
      <div v-if="loading && !pending.length" class="meta" style="padding: 12px 4px">加载中…</div>

      <template v-for="action in pending" :key="action.actionId">
        <div v-if="isPermission(action)" class="card soft stack" style="gap: 12px">
          <div class="row-between">
            <span class="chip tag">权限</span>
            <span class="meta">{{ relativeTime(action.createdAtUtc) }}</span>
          </div>
          <div>
            <p class="meta" style="margin: 0 0 4px">{{ sourceLine(action) }}</p>
            <h2 class="h2" style="margin: 0 0 6px">
              {{ action.permission?.toolName || "工具权限" }}
            </h2>
            <p
              v-if="action.permission?.description"
              style="margin: 0; color: var(--muted); font-size: 14px"
            >
              {{ action.permission.description }}
            </p>
          </div>
          <details v-if="toolInputText(action)" class="details">
            <summary>查看参数</summary>
            <pre>{{ toolInputText(action) }}</pre>
          </details>
          <div class="btn-row">
            <button
              type="button"
              class="btn-primary"
              :disabled="!!busyId"
              @click="emitAllow(action.actionId, false)"
            >
              允许
            </button>
            <button
              type="button"
              class="btn-secondary"
              :disabled="!!busyId"
              @click="emitAllow(action.actionId, true)"
            >
              总是允许
            </button>
            <button
              type="button"
              class="btn-danger"
              :disabled="!!busyId"
              @click="emitDeny(action.actionId)"
            >
              拒绝
            </button>
          </div>
        </div>

        <div v-else-if="isQuestion(action)" class="card soft stack" style="gap: 14px">
          <div class="row-between">
            <div class="row" style="gap: 8px">
              <span class="chip tag q">问答</span>
              <span v-if="questionStep(action)" class="meta">{{ questionStep(action) }}</span>
            </div>
            <span class="meta">{{ sourceLine(action) }}</span>
          </div>
          <div>
            <h2 class="h2" style="margin: 0 0 8px">
              {{ currentQ(action)?.header || action.question?.header || "需要你的回答" }}
            </h2>
            <p style="margin: 0; color: var(--muted); font-size: 14px">
              {{ currentQ(action)?.question || action.question?.question }}
            </p>
          </div>
          <div
            v-if="(currentQ(action)?.options?.length ?? 0) > 0"
            class="row"
            style="flex-wrap: wrap; gap: 8px"
          >
            <button
              v-for="(opt, idx) in currentQ(action)?.options ?? []"
              :key="idx"
              type="button"
              class="chip choice"
              :class="{
                selected: isSelected(
                  action.actionId,
                  optionValue(opt),
                  !!currentQ(action)?.multiSelect,
                ),
              }"
              :aria-pressed="
                isSelected(action.actionId, optionValue(opt), !!currentQ(action)?.multiSelect)
              "
              @click="
                toggleOption(action.actionId, optionValue(opt), !!currentQ(action)?.multiSelect)
              "
            >
              {{ opt.label || opt.value }}
            </button>
          </div>
          <div
            v-if="currentQ(action)?.allowFreeText !== false"
            class="field"
          >
            <label :for="`free-${action.actionId}`">或输入回答</label>
            <div class="control">
              <input
                :id="`free-${action.actionId}`"
                v-model="freeAnswers[action.actionId]"
                type="text"
                placeholder="自由输入"
              />
            </div>
          </div>
          <div class="row" style="gap: 8px">
            <button
              type="button"
              class="btn-primary"
              style="flex: 1"
              :class="{ 'is-loading': busyId === action.actionId }"
              :disabled="!!busyId"
              @click="emitAnswer(action)"
            >
              {{ busyId === action.actionId ? "提交中…" : "回答" }}
            </button>
            <button
              type="button"
              class="btn-secondary"
              style="flex: 0 0 96px; width: auto"
              :disabled="!!busyId"
              @click="emitDismiss(action.actionId)"
            >
              关闭
            </button>
          </div>
        </div>

        <div v-else class="card soft stack" style="gap: 8px">
          <span class="chip tag">{{ action.kind }}</span>
          <p class="meta" style="margin: 0">{{ sourceLine(action) }} · {{ action.actionId }}</p>
        </div>
      </template>

      <div v-if="!loading && !pending.length" class="empty">
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
        <p>当前离线，连接恢复后自动同步。</p>
      </div>
      <div class="card soft">
        <p class="h3" style="margin: 0 0 6px">暂不可用</p>
        <p class="meta" style="margin: 0 0 12px">未连接 CodeOrbit 时无法拉取待处理。</p>
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
