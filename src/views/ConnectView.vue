<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { ConnectionState } from "../shell/StatusChip.vue";
import type { ApiHealthDto, ApiVersionDto } from "../runtimeApi";
import {
  getRuntimeTargetToken,
  normalizeBaseUrl,
  type RuntimeTarget,
} from "../targetStore";

const props = defineProps<{
  connectionState: ConnectionState;
  pendingCount: number;
  loading: boolean;
  targets: RuntimeTarget[];
  activeTarget: RuntimeTarget | null;
  health: ApiHealthDto | null;
  version: ApiVersionDto | null;
}>();

const emit = defineEmits<{
  saveConnect: [payload: { id?: string; name: string; baseUrl: string; token: string }];
  connectExisting: [id: string];
  deleteTarget: [payload: { id: string; name: string }];
  disconnect: [];
  goPending: [];
  toast: [message: string];
}>();

const formId = ref("");
const name = ref("家里电脑");
const url = ref("http://127.0.0.1:32145");
const token = ref("");
const showToken = ref(false);
const detailsOpen = ref(false);
const errUrl = ref("");
const errToken = ref("");
/** load stored token into the form when selecting a saved target */
const formTokenLoading = ref(false);
const formTokenHint = ref("");

// reconnect / disconnect → collapse details (default closed)
watch(
  () => props.connectionState,
  (state) => {
    if (state !== "connected") detailsOpen.value = false;
  },
);

async function loadFormToken(id: string) {
  formTokenLoading.value = true;
  formTokenHint.value = "";
  try {
    const t = (await getRuntimeTargetToken(id)).trim();
    token.value = t;
    formTokenHint.value = t ? "" : "未保存 Token，请填写";
  } catch {
    token.value = "";
    formTokenHint.value = "无法读取已存 Token";
  } finally {
    formTokenLoading.value = false;
  }
}

function applyTarget(target: RuntimeTarget | null) {
  if (!target) {
    formId.value = "";
    name.value = "家里电脑";
    url.value = "http://127.0.0.1:32145";
    token.value = "";
    formTokenHint.value = "";
    formTokenLoading.value = false;
    return;
  }
  formId.value = target.id;
  name.value = target.name;
  url.value = target.baseUrl;
  token.value = "";
  void loadFormToken(target.id);
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

// 删除目标成功后列表会刷新；若当前表单仍指向已删 id，清空为新建态
watch(
  () => props.targets,
  (list) => {
    if (formId.value && !list.some((t) => t.id === formId.value)) {
      applyTarget(null);
    }
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
    errUrl.value = "请填写 CodeOrbit 地址";
  } else if (!/^https?:\/\//i.test(u)) {
    ok = false;
    errUrl.value = "地址需以 http:// 或 https:// 开头";
  }
  if (requireToken && !t) {
    ok = false;
    errToken.value = formId.value ? "请填写或更新 Token" : "请填写 Token";
  }
  return ok;
}

function onSaveConnect() {
  // new target needs token; existing may omit if we still have value loaded
  if (!validate(!formId.value && !token.value.trim())) return;
  // editing existing: empty token after load means user cleared — still send what they typed
  if (!validate(false)) return;
  if (!formId.value && !token.value.trim()) {
    errToken.value = "请填写 Token";
    return;
  }
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
  const targetName =
    name.value.trim() ||
    props.targets.find((t) => t.id === formId.value)?.name ||
    "此目标";
  emit("deleteTarget", { id: formId.value, name: targetName });
}
</script>

<template>
  <section class="pad stack" style="gap: 14px">
    <template v-if="connectionState === 'connected' && activeTarget">
      <div class="card soft conn-card">
        <div class="row-between conn-head">
          <button
            type="button"
            class="conn-title"
            :aria-expanded="detailsOpen"
            aria-controls="conn-details"
            @click="detailsOpen = !detailsOpen"
          >
            <span class="row" style="gap: 8px">
              <span
                style="
                  width: 8px;
                  height: 8px;
                  border-radius: 50%;
                  background: var(--accent);
                  display: inline-block;
                  flex-shrink: 0;
                "
                aria-hidden="true"
              />
              <span class="h3">已连接 · {{ activeTarget.name }}</span>
              <svg
                class="conn-chev"
                :class="{ open: detailsOpen }"
                viewBox="0 0 24 24"
                aria-hidden="true"
              >
                <path d="M6 9l6 6 6-6" />
              </svg>
            </span>
          </button>
          <button type="button" class="btn-text" @click="$emit('disconnect')">断开</button>
        </div>
        <div
          id="conn-details"
          class="conn-collapse"
          :class="{ open: detailsOpen }"
        >
          <div class="conn-collapse-inner">
            <div class="conn-details">
              <p class="meta num" style="margin: 0">{{ activeTarget.baseUrl }}</p>
              <p v-if="version" class="meta" style="margin: 6px 0 0">
                {{ version.product }} {{ version.version }}
                <template v-if="health"> · {{ health.status }}</template>
              </p>
            </div>
          </div>
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
        <p class="h3" style="margin: 0 0 6px">连接到电脑上的 CodeOrbit</p>
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
          <label for="runtime-url">CodeOrbit 地址</label>
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
          <label for="runtime-token">
            Token
            <template v-if="formId && formTokenLoading">（读取中…）</template>
            <template v-else-if="formId && token">（已存，可改）</template>
            <template v-else-if="formId">（需填写）</template>
          </label>
          <div class="control">
            <input
              id="runtime-token"
              v-model="token"
              :type="showToken ? 'text' : 'password'"
              :placeholder="formTokenLoading ? '读取已存 Token…' : '粘贴访问 Token'"
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
              <!-- 显示=睁眼；隐藏=闭眼（斜线） -->
              <svg v-if="showToken" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
              <svg v-else viewBox="0 0 24 24" aria-hidden="true">
                <path d="M3 3l18 18" />
                <path d="M10.6 10.6a2 2 0 0 0 2.8 2.8" />
                <path
                  d="M9.9 5.1A10.4 10.4 0 0 1 12 5c6.5 0 10 7 10 7a17.3 17.3 0 0 1-3.2 4.1"
                />
                <path
                  d="M6.1 6.1C3.7 7.8 2 12 2 12s3.5 7 10 7a10.4 10.4 0 0 0 4.1-.8"
                />
              </svg>
            </button>
          </div>
          <div v-if="formTokenHint && !errToken" class="meta" style="margin-top: 4px">
            {{ formTokenHint }}
          </div>
          <div class="field-error" role="alert">{{ errToken }}</div>
        </div>
        <div class="form-actions">
          <button
            type="submit"
            class="btn-primary"
            :class="{ 'is-loading': loading || connectionState === 'connecting' }"
            :disabled="loading || connectionState === 'connecting' || formTokenLoading"
          >
            {{
              loading || connectionState === "connecting" ? "连接中…" : "保存并连接"
            }}
          </button>
          <button
            v-if="formId"
            type="button"
            class="icon-btn danger"
            aria-label="删除目标"
            :disabled="loading"
            @click="removeSelected"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M4 7h16" />
              <path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
              <path d="M8 7l1 12a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1l1-12" />
              <path d="M10 11v6M14 11v6" />
            </svg>
          </button>
        </div>
      </form>

      <p class="hint">
        Token 仅保存在本机（系统凭据库，不可用时回退应用私有目录），请勿把 CodeOrbit
        直接暴露到公网。当前地址将规范为
        {{ url.trim() ? normalizeBaseUrl(url) : "—" }}。
      </p>
    </template>
  </section>
</template>

<style scoped>
.form-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}
.form-actions .btn-primary {
  flex: 1 1 auto;
  width: auto;
}
.form-actions .icon-btn {
  flex: 0 0 40px;
}
.form-actions .icon-btn:disabled {
  opacity: 0.45;
  pointer-events: none;
}
.conn-head {
  align-items: center;
}
.conn-title {
  border: 0;
  background: transparent;
  padding: 0;
  margin: 0;
  text-align: left;
  cursor: pointer;
  color: inherit;
  min-width: 0;
  flex: 1 1 auto;
}
.conn-chev {
  width: 16px;
  height: 16px;
  stroke: var(--muted);
  fill: none;
  stroke-width: 1.8;
  flex-shrink: 0;
  transition: transform var(--motion-duration-fast, 180ms) var(--motion-ease, ease);
}
.conn-chev.open {
  transform: rotate(180deg);
}
.conn-collapse {
  display: grid;
  grid-template-rows: 0fr;
  opacity: 0;
  pointer-events: none;
  transition:
    grid-template-rows var(--motion-duration-fast, 180ms) var(--motion-ease, ease),
    opacity var(--motion-duration-fast, 180ms) var(--motion-ease, ease);
}
.conn-collapse.open {
  grid-template-rows: 1fr;
  opacity: 1;
  pointer-events: auto;
}
.conn-collapse-inner {
  overflow: hidden;
  min-height: 0;
}
.conn-details {
  margin-top: 10px;
  padding-top: 2px;
}
@media (prefers-reduced-motion: reduce) {
  .conn-collapse {
    transition: none;
  }
  .conn-chev {
    transition: none;
  }
}
</style>
