<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import {
  createRuntimeClient,
  currentQuestion,
  eventRefreshScope,
  formatRuntimeError,
  summarizeSession,
  type ApiCapabilitiesDto,
  type ApiHealthDto,
  type ApiVersionDto,
  type ChatMessageDto,
  type HubEventDto,
  type PendingActionDto,
  type PendingResolutionDto,
  type RuntimeAssetsDto,
  type RuntimeTargetConnection,
  type SessionDto,
  type SourceDto,
} from "./runtimeApi";
import {
  deleteRuntimeTarget,
  getRuntimeTargetToken,
  listRuntimeTargets,
  normalizeBaseUrl,
  saveRuntimeTarget,
  startLocalRuntime,
  stopLocalRuntime,
  type RuntimeTarget,
} from "./targetStore";

type ConnectionState = "idle" | "connecting" | "connected" | "offline" | "unauthorized";
type WsState = "idle" | "connecting" | "connected" | "disconnected" | "disabled";
type Language = "zh" | "en";

const DEFAULT_BASE_URL = "http://127.0.0.1:32145";
const LOCAL_RUNTIME_TARGET_NAME = "CodeOrbit Rust Dev Runtime";
const LOCAL_RUNTIME_TOKEN = "dev-token";

const translations = {
  zh: {
    product: "CodeOrbit Client",
    title: "CodeOrbit 管理台",
    language: "语言",
    zh: "中文",
    en: "English",
    noActiveTarget: "未连接目标",
    websocket: "WebSocket",
    refresh: "刷新",
    startLocalRuntime: "启动本地 Runtime",
    stopLocalRuntime: "关闭本地 Runtime",
    targets: "目标",
    new: "新建",
    noSavedTargets: "暂无已保存目标。",
    name: "名称",
    runtimeUrl: "服务地址",
    token: "Token",
    tokenHelp: "保存后 Token 只存入系统凭据，不会再次显示。",
    save: "保存",
    saving: "保存中",
    connect: "连接",
    delete: "删除",
    health: "健康状态",
    unknown: "未知",
    notConnected: "未连接",
    version: "版本",
    runtime: "CodeOrbit",
    security: "安全模式",
    realtime: "实时",
    enabled: "已启用",
    pending: "待处理",
    sessionsCount: "个会话",
    sources: "来源",
    repairAll: "全部修复",
    installed: "已安装",
    notInstalled: "未安装",
    install: "安装",
    repair: "修复",
    uninstall: "卸载",
    noSourceSnapshot: "暂无来源快照。",
    runtimeAssets: "核心资产",
    missingOrUnknown: "缺失或未知",
    noAssetSnapshot: "暂无核心资产快照。",
    sessions: "会话",
    loadMessages: "消息",
    activateTerminal: "激活终端",
    dismissSession: "移除会话",
    user: "用户",
    assistant: "助手",
    noMessages: "暂无消息。",
    noSummary: "暂无摘要。",
    noSessions: "暂无会话。",
    pendingActions: "待处理动作",
    pendingHistory: "处理历史",
    noPendingHistory: "暂无处理历史。",
    actor: "处理者",
    reason: "原因",
    allow: "允许",
    allowAlways: "总是允许",
    deny: "拒绝",
    question: "问题",
    answerPlaceholder: "回答当前步骤；多个值用英文逗号分隔",
    answer: "回答",
    dismiss: "关闭",
    noPendingActions: "暂无待处理动作。",
    targetSaved: "目标已保存。Token 已存入系统凭据。",
    localRuntimeStarted: "本地 Runtime 已启动，目标已保存。",
    localRuntimeStopped: "本地 Runtime 已关闭。",
    snapshotsRefreshed: "状态快照已刷新。",
    operationCompleted: "操作已完成：",
    deniedReason: "从 CodeOrbit Client 拒绝",
    defaultTargetName: "本地服务",
    connectionStates: {
      idle: "空闲",
      connecting: "连接中",
      connected: "已连接",
      offline: "离线",
      unauthorized: "未授权",
    },
    wsStates: {
      idle: "空闲",
      connecting: "连接中",
      connected: "已连接",
      disconnected: "已断开",
      disabled: "未启用",
    },
  },
  en: {
    product: "CodeOrbit Client",
    title: "CodeOrbit Console",
    language: "Language",
    zh: "中文",
    en: "English",
    noActiveTarget: "No active target",
    websocket: "WebSocket",
    refresh: "Refresh",
    startLocalRuntime: "Start local Runtime",
    stopLocalRuntime: "Stop local Runtime",
    targets: "Targets",
    new: "New",
    noSavedTargets: "No saved targets.",
    name: "Name",
    runtimeUrl: "Service URL",
    token: "Token",
    tokenHelp: "Saved tokens stay in OS credentials and are never shown again.",
    save: "Save",
    saving: "Saving",
    connect: "Connect",
    delete: "Delete",
    health: "Health",
    unknown: "unknown",
    notConnected: "not connected",
    version: "Version",
    runtime: "CodeOrbit",
    security: "Security",
    realtime: "Realtime",
    enabled: "enabled",
    pending: "Pending",
    sessionsCount: "sessions",
    sources: "Sources",
    repairAll: "Repair all",
    installed: "Installed",
    notInstalled: "Not installed",
    install: "Install",
    repair: "Repair",
    uninstall: "Uninstall",
    noSourceSnapshot: "No source snapshot loaded.",
    runtimeAssets: "Core Assets",
    missingOrUnknown: "Missing or unknown",
    noAssetSnapshot: "No runtime asset snapshot loaded.",
    sessions: "Sessions",
    loadMessages: "Messages",
    activateTerminal: "Activate terminal",
    dismissSession: "Dismiss session",
    user: "User",
    assistant: "Assistant",
    noMessages: "No messages.",
    noSummary: "No summary yet.",
    noSessions: "No sessions.",
    pendingActions: "Pending Actions",
    pendingHistory: "History",
    noPendingHistory: "No pending history.",
    actor: "Actor",
    reason: "Reason",
    allow: "Allow",
    allowAlways: "Allow always",
    deny: "Deny",
    question: "Question",
    answerPlaceholder: "Answer current step; comma-separate multiple values",
    answer: "Answer",
    dismiss: "Dismiss",
    noPendingActions: "No pending actions.",
    targetSaved: "Target saved. Token is stored in OS credentials.",
    localRuntimeStarted: "Local Runtime started. Target saved.",
    localRuntimeStopped: "Local Runtime stopped.",
    snapshotsRefreshed: "State snapshots refreshed.",
    operationCompleted: "Operation completed on ",
    deniedReason: "Denied from CodeOrbit Client",
    defaultTargetName: "Local Service",
    connectionStates: {
      idle: "idle",
      connecting: "connecting",
      connected: "connected",
      offline: "offline",
      unauthorized: "unauthorized",
    },
    wsStates: {
      idle: "idle",
      connecting: "connecting",
      connected: "connected",
      disconnected: "disconnected",
      disabled: "disabled",
    },
  },
} as const;

const language = ref<Language>("zh");
const copy = computed(() => translations[language.value]);

const targets = ref<RuntimeTarget[]>([]);
const activeTarget = ref<RuntimeTarget | null>(null);
const activeToken = ref("");
const connectionState = ref<ConnectionState>("idle");
const wsState = ref<WsState>("idle");
const loading = ref(false);
const savingTarget = ref(false);
const startingRuntime = ref(false);
const stoppingRuntime = ref(false);
const operationMessage = ref("");
const errorMessage = ref("");
const activeRequestId = ref(0);
const socket = ref<WebSocket | null>(null);

const form = reactive<{ id: string; name: string; baseUrl: string; token: string }>({
  id: "",
  name: translations.zh.defaultTargetName,
  baseUrl: DEFAULT_BASE_URL,
  token: "",
});

const questionAnswers = reactive<Record<string, string>>({});
const health = ref<ApiHealthDto | null>(null);
const version = ref<ApiVersionDto | null>(null);
const capabilities = ref<ApiCapabilitiesDto | null>(null);
const sources = ref<SourceDto[]>([]);
const runtimeAssets = ref<RuntimeAssetsDto | null>(null);
const sessions = ref<SessionDto[]>([]);
const pending = ref<PendingActionDto[]>([]);
const pendingHistory = ref<PendingResolutionDto[]>([]);
const sessionMessages = reactive<Record<string, ChatMessageDto[]>>({});

const client = computed(() => {
  if (!activeTarget.value || !activeToken.value) return null;
  return createRuntimeClient({
    id: activeTarget.value.id,
    name: activeTarget.value.name,
    baseUrl: activeTarget.value.baseUrl,
    token: activeToken.value,
  });
});

const activeLabel = computed(() =>
  activeTarget.value ? `${activeTarget.value.name} · ${activeTarget.value.baseUrl}` : copy.value.noActiveTarget,
);

const sortedSessions = computed(() =>
  [...sessions.value].sort((a, b) => b.lastUpdatedAtUtc.localeCompare(a.lastUpdatedAtUtc)),
);

const sortedPending = computed(() =>
  [...pending.value].sort((a, b) => a.createdAtUtc.localeCompare(b.createdAtUtc)),
);

const recentPendingHistory = computed(() => pendingHistory.value.slice(0, 20));

onMounted(async () => {
  await loadTargets();
});

onBeforeUnmount(() => {
  closeSocket();
});

async function loadTargets() {
  try {
    targets.value = await listRuntimeTargets();
    if (targets.value[0]) {
      editTarget(targets.value[0]);
    }
  } catch (error) {
    errorMessage.value = String(error);
  }
}

function editTarget(target: RuntimeTarget) {
  form.id = target.id;
  form.name = target.name;
  form.baseUrl = target.baseUrl;
  form.token = "";
}

function newTarget() {
  form.id = "";
  form.name = copy.value.defaultTargetName;
  form.baseUrl = DEFAULT_BASE_URL;
  form.token = "";
}

async function saveTarget() {
  savingTarget.value = true;
  errorMessage.value = "";
  operationMessage.value = "";
  try {
    const target = await saveRuntimeTarget({
      id: form.id || undefined,
      name: form.name,
      baseUrl: form.baseUrl,
      token: form.token,
    });
    form.token = "";
    await loadTargets();
    editTarget(target);
    operationMessage.value = copy.value.targetSaved;
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    savingTarget.value = false;
  }
}

async function removeTarget(target: RuntimeTarget) {
  if (activeTarget.value?.id === target.id) {
    disconnect();
  }

  try {
    await deleteRuntimeTarget(target.id);
    await loadTargets();
    if (form.id === target.id) newTarget();
  } catch (error) {
    errorMessage.value = String(error);
  }
}

async function startLocalRuntimeTarget() {
  startingRuntime.value = true;
  errorMessage.value = "";
  operationMessage.value = "";
  try {
    const result = await startLocalRuntime();
    const existing = targets.value.find(
      (target) =>
        normalizeBaseUrl(target.baseUrl) === DEFAULT_BASE_URL || target.name === LOCAL_RUNTIME_TARGET_NAME,
    );
    const target = await saveRuntimeTarget({
      id: existing?.id,
      name: LOCAL_RUNTIME_TARGET_NAME,
      baseUrl: DEFAULT_BASE_URL,
      token: LOCAL_RUNTIME_TOKEN,
    });
    await loadTargets();
    editTarget(target);
    operationMessage.value = `${copy.value.localRuntimeStarted} ${result.message}`;
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    startingRuntime.value = false;
  }
}

async function stopLocalRuntimeTarget() {
  stoppingRuntime.value = true;
  errorMessage.value = "";
  operationMessage.value = "";
  try {
    const result = await stopLocalRuntime();
    if (activeTarget.value?.baseUrl === DEFAULT_BASE_URL && result.success) {
      disconnect();
    }
    operationMessage.value = result.message || copy.value.localRuntimeStopped;
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    stoppingRuntime.value = false;
  }
}

async function connectSelected() {
  errorMessage.value = "";
  try {
    if (!form.id) {
      const saved = await saveRuntimeTarget({
        name: form.name,
        baseUrl: form.baseUrl,
        token: form.token,
      });
      form.token = "";
      await loadTargets();
      editTarget(saved);
    }

    const target = targets.value.find((candidate) => candidate.id === form.id);
    if (!target) return;
    await connectTarget(target);
  } catch (error) {
    errorMessage.value = String(error);
  }
}

async function connectTarget(target: RuntimeTarget) {
  const requestId = resetTargetState(target);
  connectionState.value = "connecting";
  loading.value = true;

  try {
    const token = await getRuntimeTargetToken(target.id);
    if (!isCurrent(requestId)) return;

    activeToken.value = token;
    await refreshStartup(requestId);
    if (!isCurrent(requestId)) return;

    connectionState.value = "connected";
    openSocket(requestId, {
      id: target.id,
      name: target.name,
      baseUrl: target.baseUrl,
      token,
    });
  } catch (error) {
    if (!isCurrent(requestId)) return;
    const message = formatRuntimeError(error);
    connectionState.value = message.toLowerCase().includes("401") ? "unauthorized" : "offline";
    errorMessage.value = message;
  } finally {
    if (isCurrent(requestId)) loading.value = false;
  }
}

function disconnect() {
  activeRequestId.value += 1;
  activeTarget.value = null;
  activeToken.value = "";
  connectionState.value = "idle";
  resetSnapshots();
  closeSocket();
}

async function refreshStartup(requestId = activeRequestId.value) {
  const api = client.value;
  if (!api) return;

  try {
    health.value = await api.getHealth();
    version.value = await api.getVersion();
    capabilities.value = await api.getCapabilities();
    await refreshAll(requestId);
  } catch (error) {
    if (!isCurrent(requestId)) return;
    const message = formatRuntimeError(error);
    connectionState.value = message.toLowerCase().includes("401") ? "unauthorized" : "offline";
    throw error;
  }
}

async function refreshAll(requestId = activeRequestId.value) {
  await Promise.all([
    refreshSources(requestId),
    refreshAssets(requestId),
    refreshSessions(requestId),
    refreshPendingState(requestId),
  ]);
}

async function refreshSources(requestId = activeRequestId.value) {
  const api = client.value;
  if (!api) return;
  const next = await api.getSources();
  if (isCurrent(requestId)) sources.value = next;
}

async function refreshAssets(requestId = activeRequestId.value) {
  const api = client.value;
  if (!api) return;
  const next = await api.getRuntimeAssets();
  if (isCurrent(requestId)) runtimeAssets.value = next;
}

async function refreshSessions(requestId = activeRequestId.value) {
  const api = client.value;
  if (!api) return;
  const next = await api.getSessions();
  if (isCurrent(requestId)) sessions.value = next;
}

async function refreshPending(requestId = activeRequestId.value) {
  const api = client.value;
  if (!api) return;
  const next = await api.getPending();
  if (isCurrent(requestId)) pending.value = next;
}

async function refreshPendingState(requestId = activeRequestId.value) {
  await refreshPending(requestId);
  await refreshPendingHistory(requestId);
}

async function refreshPendingHistory(requestId = activeRequestId.value) {
  const api = client.value;
  if (!api) return;
  try {
    const next = await api.getPendingHistory(20);
    if (isCurrent(requestId)) pendingHistory.value = next.entries;
  } catch {
    if (isCurrent(requestId)) pendingHistory.value = [];
  }
}

async function manualRefresh() {
  loading.value = true;
  errorMessage.value = "";
  try {
    await refreshStartup();
    connectionState.value = "connected";
    operationMessage.value = copy.value.snapshotsRefreshed;
  } catch (error) {
    errorMessage.value = formatRuntimeError(error);
  } finally {
    loading.value = false;
  }
}

async function runOperation(action: () => Promise<unknown>, refresh: () => Promise<unknown>) {
  const targetName = activeLabel.value;
  loading.value = true;
  errorMessage.value = "";
  operationMessage.value = "";
  try {
    await action();
    await refresh();
    operationMessage.value = `${copy.value.operationCompleted}${targetName}`;
  } catch (error) {
    errorMessage.value = formatRuntimeError(error);
    await refresh().catch(() => undefined);
  } finally {
    loading.value = false;
  }
}

function sourceAction(source: SourceDto, action: "install" | "repair" | "uninstall") {
  const api = client.value;
  if (!api) return;
  const call = {
    install: () => api.installSource(source.id),
    repair: () => api.repairSource(source.id),
    uninstall: () => api.uninstallSource(source.id),
  }[action];
  void runOperation(call, refreshSources);
}

function repairAllSources() {
  const api = client.value;
  if (api) void runOperation(api.repairAllSources, refreshSources);
}

function repairAssets() {
  const api = client.value;
  if (api) void runOperation(api.repairRuntimeAssets, refreshAssets);
}

async function loadSessionMessages(session: SessionDto) {
  const api = client.value;
  if (!api) return;
  const requestId = activeRequestId.value;
  loading.value = true;
  errorMessage.value = "";
  try {
    const next = await api.getSessionMessages(session.sessionId);
    if (isCurrent(requestId)) sessionMessages[session.sessionId] = next;
  } catch (error) {
    if (isCurrent(requestId)) errorMessage.value = formatRuntimeError(error);
  } finally {
    if (isCurrent(requestId)) loading.value = false;
  }
}

function activateSessionTerminal(session: SessionDto) {
  const api = client.value;
  if (api) void runOperation(() => api.activateTerminal(session.sessionId), refreshSessions);
}

function dismissRuntimeSession(session: SessionDto) {
  const api = client.value;
  if (api) void runOperation(() => api.dismissSession(session.sessionId), refreshAll);
}

function allow(action: PendingActionDto, always: boolean) {
  const api = client.value;
  if (api) void runOperation(() => api.allowPermission(action.actionId, always), refreshPendingState);
}

function deny(action: PendingActionDto) {
  const api = client.value;
  if (api) void runOperation(() => api.denyPermission(action.actionId, copy.value.deniedReason), refreshPendingState);
}

function answer(action: PendingActionDto) {
  const api = client.value;
  const answerText = questionAnswers[action.actionId]?.trim();
  if (!api || !answerText) return;

  const answers = answerText
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);

  void runOperation(async () => {
    const result = await api.answerQuestion(action.actionId, answers);
    if (result.resolved) {
      delete questionAnswers[action.actionId];
    }
  }, refreshPendingState);
}

function dismissQuestion(action: PendingActionDto) {
  const api = client.value;
  if (api) void runOperation(() => api.dismissQuestion(action.actionId), refreshPendingState);
}

function resetTargetState(target: RuntimeTarget): number {
  activeRequestId.value += 1;
  activeTarget.value = target;
  activeToken.value = "";
  errorMessage.value = "";
  operationMessage.value = "";
  resetSnapshots();
  closeSocket();
  return activeRequestId.value;
}

function resetSnapshots() {
  health.value = null;
  version.value = null;
  capabilities.value = null;
  sources.value = [];
  runtimeAssets.value = null;
  sessions.value = [];
  pending.value = [];
  pendingHistory.value = [];
  Object.keys(questionAnswers).forEach((key) => delete questionAnswers[key]);
  Object.keys(sessionMessages).forEach((key) => delete sessionMessages[key]);
}

function openSocket(requestId: number, target: RuntimeTargetConnection) {
  if (capabilities.value?.realtime === false) {
    wsState.value = "disabled";
    return;
  }

  const ws = new WebSocket(createRuntimeClient(target).eventsUrl());
  socket.value = ws;
  wsState.value = "connecting";

  ws.onopen = () => {
    if (isCurrent(requestId)) wsState.value = "connected";
  };
  ws.onclose = () => {
    if (isCurrent(requestId)) wsState.value = "disconnected";
  };
  ws.onerror = () => {
    if (isCurrent(requestId)) wsState.value = "disconnected";
  };
  ws.onmessage = (message) => {
    if (!isCurrent(requestId)) return;
    handleEventMessage(String(message.data), requestId);
  };
}

function closeSocket() {
  socket.value?.close();
  socket.value = null;
  wsState.value = "idle";
}

function handleEventMessage(message: string, requestId: number) {
  try {
    const event = JSON.parse(message) as HubEventDto;
    const scope = eventRefreshScope(event);
    const refresh = {
      sessions: refreshSessions,
      pending: refreshPendingState,
      sources: refreshSources,
      assets: refreshAssets,
      all: refreshAll,
    }[scope];
    void refresh(requestId);
  } catch {
    void refreshAll(requestId);
  }
}

function isCurrent(requestId: number): boolean {
  return activeRequestId.value === requestId;
}

function statusClass(value: string | boolean | null | undefined): string {
  if (value === true || value === "connected" || value === "ok" || value === "Installed") return "good";
  if (value === false || value === "offline" || value === "unauthorized") return "bad";
  return "muted";
}

function connectionStateLabel(value: ConnectionState): string {
  return copy.value.connectionStates[value];
}

function wsStateLabel(value: WsState): string {
  return copy.value.wsStates[value];
}

function trimText(value: string | null | undefined, max = 120): string {
  const text = value?.replace(/\s+/g, " ").trim() ?? "";
  return text.length > max ? `${text.slice(0, max - 3)}...` : text;
}
</script>

<template>
  <main class="app-shell">
    <header class="topbar">
      <div>
        <p class="eyebrow">{{ copy.product }}</p>
        <h1>{{ copy.title }}</h1>
      </div>
      <div class="target-badge">
        <span :class="['dot', statusClass(connectionState)]"></span>
        <div>
          <strong>{{ activeLabel }}</strong>
          <small>{{ connectionStateLabel(connectionState) }} · {{ copy.websocket }} {{ wsStateLabel(wsState) }}</small>
        </div>
      </div>
      <div class="toolbar-actions">
        <div class="language-toggle" :aria-label="copy.language">
          <button type="button" :class="{ selected: language === 'zh' }" @click="language = 'zh'">
            {{ copy.zh }}
          </button>
          <button type="button" :class="{ selected: language === 'en' }" @click="language = 'en'">
            {{ copy.en }}
          </button>
        </div>
        <button type="button" :disabled="startingRuntime" @click="startLocalRuntimeTarget">
          {{ copy.startLocalRuntime }}
        </button>
        <button type="button" class="danger" :disabled="stoppingRuntime" @click="stopLocalRuntimeTarget">
          {{ copy.stopLocalRuntime }}
        </button>
        <button type="button" :disabled="!activeTarget || loading" @click="manualRefresh">{{ copy.refresh }}</button>
      </div>
    </header>

    <section v-if="errorMessage || operationMessage" class="message-row">
      <p v-if="errorMessage" class="message error">{{ errorMessage }}</p>
      <p v-if="operationMessage" class="message success">{{ operationMessage }}</p>
    </section>

    <div class="layout">
      <aside class="panel targets-panel">
        <div class="panel-header">
          <h2>{{ copy.targets }}</h2>
          <button type="button" @click="newTarget">{{ copy.new }}</button>
        </div>

        <div class="target-list">
          <button
            v-for="target in targets"
            :key="target.id"
            type="button"
            :class="{ selected: form.id === target.id }"
            @click="editTarget(target)"
          >
            <strong>{{ target.name }}</strong>
            <span>{{ target.baseUrl }}</span>
          </button>
          <p v-if="targets.length === 0" class="empty">{{ copy.noSavedTargets }}</p>
        </div>

        <form class="target-form" @submit.prevent="saveTarget">
          <label>
            {{ copy.name }}
            <input v-model="form.name" autocomplete="off" required />
          </label>
          <label>
            {{ copy.runtimeUrl }}
            <input v-model="form.baseUrl" placeholder="http://127.0.0.1:32145" required />
          </label>
          <label>
            {{ copy.token }}
            <input v-model="form.token" type="password" :required="!form.id" autocomplete="off" />
          </label>
          <small>{{ copy.tokenHelp }}</small>
          <div class="button-row">
            <button type="submit" :disabled="savingTarget">{{ savingTarget ? copy.saving : copy.save }}</button>
            <button type="button" :disabled="loading" @click="connectSelected">{{ copy.connect }}</button>
            <button
              v-if="form.id"
              type="button"
              class="danger"
              :disabled="loading"
              @click="removeTarget({ id: form.id, name: form.name, baseUrl: normalizeBaseUrl(form.baseUrl), createdAtUtc: '', updatedAtUtc: '' })"
            >
              {{ copy.delete }}
            </button>
          </div>
        </form>
      </aside>

      <section class="content">
        <section class="status-grid">
          <article class="stat">
            <span>{{ copy.health }}</span>
            <strong>{{ health?.status ?? copy.unknown }}</strong>
            <small>{{ health?.startedAtUtc ?? copy.notConnected }}</small>
          </article>
          <article class="stat">
            <span>{{ copy.version }}</span>
            <strong>{{ version?.version ?? copy.unknown }}</strong>
            <small>{{ version?.product ?? copy.runtime }}</small>
          </article>
          <article class="stat">
            <span>{{ copy.security }}</span>
            <strong>{{ capabilities?.securityMode ?? copy.unknown }}</strong>
            <small>{{ copy.realtime }} {{ capabilities?.realtime ? copy.enabled : copy.unknown }}</small>
          </article>
          <article class="stat">
            <span>{{ copy.pending }}</span>
            <strong>{{ pending.length }}</strong>
            <small>{{ sessions.length }} {{ copy.sessionsCount }}</small>
          </article>
        </section>

        <section class="panel">
          <div class="panel-header">
            <div>
              <h2>{{ copy.sources }}</h2>
              <small>{{ activeLabel }}</small>
            </div>
            <button type="button" :disabled="!client || loading" @click="repairAllSources">{{ copy.repairAll }}</button>
          </div>
          <div class="table-list">
            <article v-for="source in sources" :key="source.id" class="row-card">
              <div>
                <strong>{{ source.displayName }}</strong>
                <small>{{ source.id }} · {{ source.sourceType }}</small>
              </div>
              <span :class="['pill', source.installed ? 'good' : 'muted']">
                {{ source.installed ? copy.installed : copy.notInstalled }}
              </span>
              <div class="row-actions">
                <button
                  type="button"
                  :disabled="!source.capabilities.hookInstall || loading"
                  @click="sourceAction(source, 'install')"
                >
                  {{ copy.install }}
                </button>
                <button
                  type="button"
                  :disabled="!source.capabilities.hookInstall || loading"
                  @click="sourceAction(source, 'repair')"
                >
                  {{ copy.repair }}
                </button>
                <button
                  type="button"
                  :disabled="!source.installed || loading"
                  @click="sourceAction(source, 'uninstall')"
                >
                  {{ copy.uninstall }}
                </button>
              </div>
            </article>
            <p v-if="sources.length === 0" class="empty">{{ copy.noSourceSnapshot }}</p>
          </div>
        </section>

        <section class="panel">
          <div class="panel-header">
            <div>
              <h2>{{ copy.runtimeAssets }}</h2>
              <small>{{ activeLabel }}</small>
            </div>
            <button type="button" :disabled="!client || loading" @click="repairAssets">{{ copy.repair }}</button>
          </div>
          <div class="assets-grid">
            <span :class="['pill', runtimeAssets?.installed ? 'good' : 'bad']">
              {{ runtimeAssets?.installed ? copy.installed : copy.missingOrUnknown }}
            </span>
            <p>{{ runtimeAssets?.runtimeDirectory ?? copy.noAssetSnapshot }}</p>
            <p>{{ runtimeAssets?.hookScriptPath }}</p>
            <p>{{ runtimeAssets?.bridgeExePath }}</p>
          </div>
        </section>

        <section class="panel">
          <div class="panel-header">
            <div>
              <h2>{{ copy.sessions }}</h2>
              <small>{{ activeLabel }}</small>
            </div>
          </div>
          <div class="table-list">
            <article v-for="session in sortedSessions" :key="session.sessionId" class="session-card">
              <div>
                <strong>{{ session.projectName || session.workingDirectory || session.sessionId }}</strong>
                <small>{{ session.sourceDisplayName }} · {{ session.sessionId }}</small>
              </div>
              <span class="pill">{{ session.status }}</span>
              <p>{{ trimText(summarizeSession(session)) || copy.noSummary }}</p>
              <div class="button-row">
                <button type="button" :disabled="loading" @click="loadSessionMessages(session)">
                  {{ copy.loadMessages }}
                </button>
                <button type="button" :disabled="loading" @click="activateSessionTerminal(session)">
                  {{ copy.activateTerminal }}
                </button>
                <button type="button" class="danger" :disabled="loading" @click="dismissRuntimeSession(session)">
                  {{ copy.dismissSession }}
                </button>
              </div>
              <div v-if="sessionMessages[session.sessionId]" class="chat-list">
                <p v-if="sessionMessages[session.sessionId].length === 0" class="empty">{{ copy.noMessages }}</p>
                <p
                  v-for="(message, index) in sessionMessages[session.sessionId]"
                  :key="`${message.timestampUtc}-${index}`"
                  class="chat-message"
                >
                  <strong>{{ message.isUser ? copy.user : copy.assistant }}</strong>
                  <span>{{ trimText(message.text, 220) }}</span>
                </p>
              </div>
            </article>
            <p v-if="sortedSessions.length === 0" class="empty">{{ copy.noSessions }}</p>
          </div>
        </section>

        <section class="panel">
          <div class="panel-header">
            <div>
              <h2>{{ copy.pendingActions }}</h2>
              <small>{{ activeLabel }}</small>
            </div>
          </div>
          <div class="pending-list">
            <article v-for="action in sortedPending" :key="action.actionId" class="pending-card">
              <div class="pending-title">
                <div>
                  <strong>{{ action.kind }} · {{ action.sourceDisplayName }}</strong>
                  <small>{{ action.projectName || action.workingDirectory || action.sessionId }}</small>
                </div>
                <span class="pill">{{ action.actionId }}</span>
              </div>

              <template v-if="action.permission">
                <p>{{ action.permission.toolName }} · {{ trimText(action.permission.description) }}</p>
                <div class="button-row">
                  <button type="button" :disabled="loading" @click="allow(action, false)">{{ copy.allow }}</button>
                  <button type="button" :disabled="loading" @click="allow(action, true)">{{ copy.allowAlways }}</button>
                  <button type="button" class="danger" :disabled="loading" @click="deny(action)">{{ copy.deny }}</button>
                </div>
              </template>

              <template v-else-if="action.question">
                <p>{{ currentQuestion(action)?.header || action.question.header || copy.question }}</p>
                <p>{{ currentQuestion(action)?.question || action.question.question }}</p>
                <div v-if="currentQuestion(action)?.options?.length" class="options">
                  <button
                    v-for="option in currentQuestion(action)?.options"
                    :key="option.value || option.label"
                    type="button"
                    @click="questionAnswers[action.actionId] = option.value || option.label"
                  >
                    {{ option.label }}
                  </button>
                </div>
                <input
                  v-model="questionAnswers[action.actionId]"
                  :placeholder="copy.answerPlaceholder"
                />
                <div class="button-row">
                  <button type="button" :disabled="loading || !questionAnswers[action.actionId]?.trim()" @click="answer(action)">
                    {{ copy.answer }}
                  </button>
                  <button type="button" class="danger" :disabled="loading" @click="dismissQuestion(action)">
                    {{ copy.dismiss }}
                  </button>
                </div>
              </template>
            </article>
            <p v-if="sortedPending.length === 0" class="empty">{{ copy.noPendingActions }}</p>
          </div>
        </section>

        <section class="panel">
          <div class="panel-header">
            <div>
              <h2>{{ copy.pendingHistory }}</h2>
              <small>{{ activeLabel }}</small>
            </div>
          </div>
          <div class="table-list">
            <article v-for="entry in recentPendingHistory" :key="`${entry.actionId}-${entry.resolvedAtUtc}`" class="history-card">
              <div>
                <strong>{{ entry.kind }} · {{ entry.decision }}</strong>
                <small>{{ entry.source || entry.sessionId || entry.actionId }}</small>
              </div>
              <span class="pill">{{ entry.actor || copy.actor }}</span>
              <p>{{ entry.reason ? `${copy.reason}: ${entry.reason}` : entry.resolvedAtUtc }}</p>
            </article>
            <p v-if="recentPendingHistory.length === 0" class="empty">{{ copy.noPendingHistory }}</p>
          </div>
        </section>
      </section>
    </div>
  </main>
</template>

<style>
:root {
  color: #172026;
  background: #f4f6f4;
  font-family:
    Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 16px;
  font-weight: 400;
  line-height: 1.5;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
}

button,
input {
  font: inherit;
}

button {
  border: 1px solid #bfcbc3;
  border-radius: 6px;
  background: #ffffff;
  color: #172026;
  cursor: pointer;
  padding: 0.45rem 0.7rem;
}

button:hover:not(:disabled) {
  border-color: #51766a;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

input {
  width: 100%;
  border: 1px solid #bfcbc3;
  border-radius: 6px;
  background: #ffffff;
  color: #172026;
  padding: 0.5rem 0.65rem;
}

h1,
h2,
p {
  margin: 0;
}

h1 {
  font-size: 1.4rem;
}

h2 {
  font-size: 1rem;
}

small {
  color: #64736c;
}

.app-shell {
  min-height: 100vh;
  padding: 1rem;
}

.topbar,
.panel,
.stat {
  border: 1px solid #d8e0db;
  border-radius: 8px;
  background: #ffffff;
}

.topbar {
  display: grid;
  grid-template-columns: 1fr minmax(16rem, 1.3fr) auto;
  gap: 1rem;
  align-items: center;
  padding: 0.85rem 1rem;
}

.eyebrow {
  color: #51766a;
  font-size: 0.74rem;
  font-weight: 700;
  letter-spacing: 0;
  text-transform: uppercase;
}

.target-badge {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: 0.65rem;
}

.target-badge strong,
.target-badge small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.toolbar-actions,
.language-toggle {
  display: flex;
  align-items: center;
  gap: 0.45rem;
}

.language-toggle {
  border: 1px solid #bfcbc3;
  border-radius: 6px;
  padding: 0.15rem;
}

.language-toggle button {
  border: 0;
  padding: 0.3rem 0.5rem;
}

.language-toggle button.selected {
  background: #dceee4;
  color: #21533b;
}

.dot {
  width: 0.7rem;
  height: 0.7rem;
  flex: 0 0 auto;
  border-radius: 999px;
  background: #8a9690;
}

.good {
  background: #dceee4;
  color: #21533b;
}

.bad {
  background: #f7dddd;
  color: #7b2929;
}

.muted {
  background: #e8ece9;
  color: #56645e;
}

.message-row {
  display: grid;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

.message {
  border-radius: 6px;
  padding: 0.65rem 0.75rem;
}

.message.error {
  background: #f7dddd;
  color: #7b2929;
}

.message.success {
  background: #dceee4;
  color: #21533b;
}

.layout {
  display: grid;
  grid-template-columns: minmax(16rem, 21rem) minmax(0, 1fr);
  gap: 1rem;
  margin-top: 1rem;
}

.panel {
  padding: 0.9rem;
}

.content {
  display: grid;
  gap: 1rem;
}

.panel-header,
.button-row,
.row-actions,
.pending-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.panel-header,
.pending-title {
  justify-content: space-between;
}

.targets-panel {
  align-self: start;
  display: grid;
  gap: 1rem;
}

.target-list,
.target-form,
.table-list,
.pending-list,
.assets-grid {
  display: grid;
  gap: 0.65rem;
}

.target-list button {
  display: grid;
  gap: 0.15rem;
  text-align: left;
}

.target-list button.selected {
  border-color: #51766a;
  background: #eef6f1;
}

.target-list span,
.row-card small,
.session-card small,
.pending-card small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.target-form label {
  display: grid;
  gap: 0.25rem;
  color: #415049;
  font-size: 0.9rem;
  font-weight: 650;
}

.danger {
  border-color: #d7aaaa;
  color: #8a2d2d;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.75rem;
}

.stat {
  display: grid;
  gap: 0.1rem;
  min-width: 0;
  padding: 0.8rem;
}

.stat span {
  color: #64736c;
  font-size: 0.78rem;
}

.stat strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-card,
.session-card,
.pending-card,
.history-card {
  border: 1px solid #e1e7e3;
  border-radius: 8px;
  background: #fbfcfb;
  padding: 0.75rem;
}

.row-card {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 0.75rem;
  align-items: center;
}

.session-card,
.pending-card,
.history-card {
  display: grid;
  gap: 0.5rem;
}

.pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: #e8ece9;
  color: #56645e;
  font-size: 0.78rem;
  padding: 0.2rem 0.55rem;
  white-space: nowrap;
}

.assets-grid p,
.session-card p,
.pending-card p,
.history-card p,
.chat-message {
  overflow-wrap: anywhere;
}

.chat-list {
  display: grid;
  gap: 0.4rem;
}

.chat-message {
  display: grid;
  gap: 0.15rem;
  border-left: 3px solid #d8e0db;
  padding-left: 0.55rem;
}

.options {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.empty {
  color: #64736c;
  padding: 0.3rem 0;
}

@media (max-width: 900px) {
  .topbar,
  .layout,
  .status-grid,
  .row-card {
    grid-template-columns: 1fr;
  }

  .panel-header,
  .row-actions,
  .button-row {
    flex-wrap: wrap;
  }
}
</style>
