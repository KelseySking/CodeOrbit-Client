<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { ConnectionState } from "../shell/StatusChip.vue";
import type { ApiHealthDto, ApiVersionDto } from "../runtimeApi";
import type { RuntimeTarget } from "../targetStore";
import { normalizeBaseUrl } from "../targetStore";

const props = defineProps<{
  connectionState: ConnectionState;
  pendingCount: number;
  loading: boolean;
  targets: RuntimeTarget[];
  activeTarget: RuntimeTarget | null;
  health: ApiHealthDto | null;
  version: ApiVersionDto | null;
  errorMessage: string;
}>();

const emit = defineEmits<{
  saveConnect: [payload: { id?: string; name: string; baseUrl: string; token: string }];
  connectExisting: [id: string];
  deleteTarget: [id: string];
  disconnect: [];
  goPending: [];
  toast: [message: string];
}>();

const formId = ref("");
const name = ref("家里电脑");
const url = ref("http://127.0.0.1:32145");
const token = ref("");
const showToken = ref(false);
const errUrl = ref("");
const errToken = ref("");

function applyTarget(target: RuntimeTarget | null) {
  if (!target) {
    formId.value = "";
    name.value = "家里电脑";
    url.value = "http://127.0.0.1:32145";
    token.value = "";
    return;
  }
  formId.value = target.id;
  name.value = target.name;
  url.value = target.baseUrl;
  token.value = "";
}

onMounted(() => {
  if (props.activeTarget) applyTarget(props.activeTarget);
  else if (props.targets[0]) applyTarget(props.targets[0]);
});

watch(
  () => props.activeTarget?.id,
  () => {
    if (props.activeTarget) applyTarget(props.activeTarget);
  },
);

function validate(requireToken: boolean): boolean {
  errUrl.value = "";
  errToken.value = "";
  let ok = true;
  const u = url.value.trim();
  const t = token.value.trim();
  if (!u) {
    ok = false;
    errUrl.value = "请填写 Runtime 地址";
  } else if (!/^https?:\/\//i.test(u)) {
    ok = false;
    errUrl.value = "地址需以 http:// 或 https:// 开头";
  }
  if (requireToken && !t) {
    ok = false;
    errToken.value = formId.value ? "更新 Token 或使用「连接」读取已存 Token" : "请填写 Token";
  }
  return ok;
}

function onSaveConnect() {
  // new target needs token; existing may omit token (keep stored)
  if (!validate(!formId.value)) return;
  emit("saveConnect", {
    id: formId.value || undefined,
    name: name.value,
    baseUrl: url.value,
    token: token.value,
  });
}

function selectTarget(target: RuntimeTarget) {
  applyTarget(target);
}

function newTarget() {
  applyTarget(null);
}

function removeSelected() {
  if (!formId.value) return;
  emit("deleteTarget", formId.value);
  newTarget();
}
</script>

<template>
  <section class="pad stack" style="gap: 14px">
    <div
      v-if="errorMessage && connectionState !== 'connected'"
      class="banner"
      :style="
        connectionState === 'unauthorized'
          ? {
              background: 'var(--danger-soft)',
              borderColor: 'color-mix(in srgb, var(--danger) 28%, var(--border))',
              color: 'var(--danger)',
            }
          : undefined
      "
    >
      <span>{{ errorMessage }}</span>
    </div>

    <template v-if="connectionState === 'connected' && activeTarget">
      <div class="card soft">
        <div class="row-between" style="align-items: flex-start">
          <div>
            <div class="row" style="gap: 8px; margin-bottom: 6px">
              <span
                style="
                  width: 8px;
                  height: 8px;
                  border-radius: 50%;
                  background: var(--accent);
                  display: inline-block;
                "
              />
              <span class="h3">已连接 · {{ activeTarget.name }}</span>
            </div>
            <p class="meta num" style="margin: 0">{{ activeTarget.baseUrl }}</p>
            <p v-if="version" class="meta" style="margin: 6px 0 0">
              {{ version.product }} {{ version.version }}
              <template v-if="health"> · {{ health.status }}</template>
            </p>
          </div>
          <button type="button" class="btn-text" @click="$emit('disconnect')">断开</button>
        </div>
      </div>

      <div v-if="pendingCount > 0" class="card soft row-between">
        <div>
          <p class="h3" style="margin: 0 0 4px">有 {{ pendingCount }} 条待处理</p>
          <p class="meta" style="margin: 0">权限审批与 AI 提问等待你确认</p>
        </div>
        <button
          type="button"
          class="btn-primary"
          style="width: auto; min-width: 96px; padding: 10px 14px; min-height: 44px"
          @click="$emit('goPending')"
        >
          去处理
        </button>
      </div>

      <p class="hint">连接保持在局域网内。切换网络后如掉线，点顶栏刷新重试。</p>
    </template>

    <template v-else>
      <div class="card soft">
        <p class="h3" style="margin: 0 0 6px">连接到电脑上的 Runtime</p>
        <p class="meta" style="margin: 0">
          在手机上查看 AI 编程会话、处理权限审批与提问。Token 仅保存在本机。
        </p>
      </div>

      <div v-if="targets.length" class="card soft stack" style="gap: 8px">
        <div class="row-between">
          <span class="h3">已保存目标</span>
          <button type="button" class="btn-text" @click="newTarget">新建</button>
        </div>
        <button
          v-for="t in targets"
          :key="t.id"
          type="button"
          class="session-row"
          style="min-height: 56px"
          :style="
            formId === t.id
              ? { borderColor: 'color-mix(in srgb, var(--accent) 40%, var(--border))' }
              : undefined
          "
          @click="selectTarget(t)"
        >
          <span class="dot" aria-hidden="true" />
          <div>
            <div class="title">{{ t.name }}</div>
            <div class="sub">{{ t.baseUrl }}</div>
          </div>
          <div class="right">
            <button
              type="button"
              class="btn-text"
              style="min-height: 32px"
              @click.stop="emit('connectExisting', t.id)"
            >
              连接
            </button>
          </div>
        </button>
      </div>

      <form class="card soft stack" style="gap: 14px" @submit.prevent="onSaveConnect">
        <div class="field">
          <label for="runtime-name">名称（可选）</label>
          <div class="control">
            <input
              id="runtime-name"
              v-model="name"
              type="text"
              placeholder="家里电脑"
              autocomplete="off"
            />
          </div>
        </div>
        <div class="field">
          <label for="runtime-url">Runtime 地址</label>
          <div class="control">
            <input
              id="runtime-url"
              v-model="url"
              type="url"
              inputmode="url"
              placeholder="http://192.168.1.8:32145"
              :class="{ invalid: !!errUrl }"
            />
          </div>
          <div class="field-error" role="alert">{{ errUrl }}</div>
        </div>
        <div class="field">
          <label for="runtime-token">Token{{ formId ? "（留空则用已存）" : "" }}</label>
          <div class="control">
            <input
              id="runtime-token"
              v-model="token"
              :type="showToken ? 'text' : 'password'"
              placeholder="粘贴访问 Token"
              style="padding-right: 48px"
              :class="{ invalid: !!errToken }"
              autocomplete="off"
            />
            <button
              type="button"
              class="eye"
              :aria-label="showToken ? '隐藏 Token' : '显示 Token'"
              @click="showToken = !showToken"
            >
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
            </button>
          </div>
          <div class="field-error" role="alert">{{ errToken }}</div>
        </div>
        <button
          type="submit"
          class="btn-primary"
          :class="{ 'is-loading': loading || connectionState === 'connecting' }"
          :disabled="loading || connectionState === 'connecting'"
        >
          {{
            loading || connectionState === "connecting" ? "连接中…" : "保存并连接"
          }}
        </button>
        <button
          v-if="formId"
          type="button"
          class="btn-danger"
          :disabled="loading"
          @click="removeSelected"
        >
          删除此目标
        </button>
      </form>

      <p class="hint">
        Token 仅保存在本机凭据库，请勿把 Runtime 直接暴露到公网。当前地址将规范为
        {{ url.trim() ? normalizeBaseUrl(url) : "—" }}。
      </p>
    </template>
  </section>
</template>
