<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import TopBar from "./TopBar.vue";
import TabBar, { type TabId } from "./TabBar.vue";
import PendingView from "../views/PendingView.vue";
import SessionsView from "../views/SessionsView.vue";
import ConnectView from "../views/ConnectView.vue";
import SessionDetailView from "../views/SessionDetailView.vue";
import ConfirmDialog from "../components/ConfirmDialog.vue";
import { useRuntimeConnection } from "../composables/useRuntimeConnection";
import { formatRuntimeError } from "../runtimeApi";
import {
  animatePresence,
  cancelMotion,
  MOTION,
} from "../utils/motion";
import {
  keepAliveStart,
  keepAliveStop,
  keepAliveUpdate,
} from "../utils/keepAlive";

const {
  targets,
  activeTarget,
  connectionState,
  health,
  version,
  pending,
  pendingCount,
  sessions,
  sessionsEpoch,
  errorMessage,
  loading,
  wsState,
  isConnected,
  client,
  loadTargets,
  loadPending,
  loadSessions,
  saveAndConnect,
  connectExisting,
  removeTarget,
  disconnect,
  refresh,
} = useRuntimeConnection();

const activeTab = ref<TabId>("connect");
const toastMessage = ref("");
const toastVisible = ref(false);
let toastTimer: ReturnType<typeof setTimeout> | null = null;

const stack = ref<null | {
  type: "session";
  id: string;
  title: string;
  subtitle: string;
}>(null);

/** SPA history: root guard + session detail so Android gesture/back uses WebView.goBack */
let stackHistoryOwned = false;
/** suppress popstate side-effects while we programmatically history.back() */
let suppressPopstate = false;

type ConfirmAction =
  | { kind: "dismissSession"; sessionId: string }
  | { kind: "deleteTarget"; id: string; name: string };

/** root-tab double-back exit window (ms) */
const EXIT_HINT_MS = 2000;
let exitHintUntil = 0;
let exitHintTimer: ReturnType<typeof setTimeout> | null = null;

const confirmOpen = ref(false);
const confirmTitle = ref("");
const confirmMessage = ref("");
const confirmLabel = ref("确认");
const confirmDanger = ref(false);
const confirmAction = ref<ConfirmAction | null>(null);

const titles: Record<TabId, string> = {
  pending: "待处理",
  sessions: "会话",
  connect: "连接",
};

const showPendingBadge = computed(
  () => isConnected.value && pendingCount.value > 0,
);

const chromeTitle = computed(() => {
  if (stack.value?.type === "session") return stack.value.title;
  return titles[activeTab.value];
});

const chromeSubtitle = computed(() => {
  if (stack.value?.type === "session") return stack.value.subtitle;
  if (isConnected.value && wsState.value === "reconnecting") return "实时同步重连中…";
  if (isConnected.value && wsState.value === "closed") return "实时同步暂不可用";
  return undefined;
});

function pushRootGuard() {
  history.pushState({ coRoot: 1 }, "");
}

function onPopState() {
  if (suppressPopstate) {
    suppressPopstate = false;
    return;
  }

  // Confirm has no history entry; re-arm so cancel doesn't drop the page.
  if (confirmOpen.value) {
    closeConfirm();
    if (stack.value) {
      history.pushState({ coStack: "session" }, "");
      stackHistoryOwned = true;
    } else {
      pushRootGuard();
    }
    return;
  }

  if (stack.value) {
    stack.value = null;
    stackHistoryOwned = false;
    // session entry was popped; root guard still under us
    return;
  }

  // Tab root: first back = hint, second within window = exit
  pushRootGuard();
  const now = Date.now();
  if (now < exitHintUntil) {
    exitHintUntil = 0;
    if (exitHintTimer) {
      clearTimeout(exitHintTimer);
      exitHintTimer = null;
    }
    void doExitApp();
    return;
  }
  exitHintUntil = now + EXIT_HINT_MS;
  showToast("再按一次返回退出应用");
  if (exitHintTimer) clearTimeout(exitHintTimer);
  exitHintTimer = setTimeout(() => {
    exitHintUntil = 0;
    exitHintTimer = null;
  }, EXIT_HINT_MS);
}

onMounted(async () => {
  // Keep one history entry under tabs so root back is interceptable.
  pushRootGuard();
  window.addEventListener("popstate", onPopState);
  try {
    await loadTargets();
    if (targets.value.length === 0) activeTab.value = "connect";
  } catch (error) {
    showToast(formatRuntimeError(error));
  }
});

onUnmounted(() => {
  window.removeEventListener("popstate", onPopState);
  if (exitHintTimer) {
    clearTimeout(exitHintTimer);
    exitHintTimer = null;
  }
  void keepAliveStop();
});

// Android FGS + ongoing notification (desktop no-op)
let keepAliveRunning = false;
watch(
  [isConnected, activeTarget, pendingCount],
  ([connected, target, count]) => {
    if (!connected || !target) {
      if (keepAliveRunning) {
        keepAliveRunning = false;
        void keepAliveStop();
      }
      return;
    }
    const payload = { targetName: target.name, pendingCount: count };
    if (!keepAliveRunning) {
      keepAliveRunning = true;
      void keepAliveStart(payload);
    } else {
      void keepAliveUpdate(payload);
    }
  },
  { immediate: true },
);

function showToast(msg: string) {
  toastMessage.value = msg;
  toastVisible.value = true;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toastVisible.value = false;
  }, 1600);
}

function openConfirm(opts: {
  title: string;
  message: string;
  confirmLabel?: string;
  danger?: boolean;
  action: ConfirmAction;
}) {
  confirmTitle.value = opts.title;
  confirmMessage.value = opts.message;
  confirmLabel.value = opts.confirmLabel ?? "确认";
  confirmDanger.value = opts.danger ?? false;
  confirmAction.value = opts.action;
  confirmOpen.value = true;
}

function closeConfirm() {
  confirmOpen.value = false;
  confirmAction.value = null;
}

function onConfirmCancel() {
  closeConfirm();
}

async function onConfirmOk() {
  const action = confirmAction.value;
  closeConfirm();
  if (!action) return;

  if (action.kind === "dismissSession") {
    await doDismissSession(action.sessionId);
    return;
  }
  if (action.kind === "deleteTarget") {
    await doDeleteTarget(action.id);
  }
}

async function doExitApp() {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("exit_app");
  } catch {
    showToast("无法退出，请用系统手势返回桌面");
  }
}

function releaseStackHistory() {
  if (!stackHistoryOwned) {
    return;
  }
  stackHistoryOwned = false;
  suppressPopstate = true;
  history.back();
}

function clearStack() {
  if (!stack.value) return;
  stack.value = null;
  releaseStackHistory();
}

function onTabChange(tab: TabId) {
  clearStack();
  activeTab.value = tab;
}

function goConnect() {
  clearStack();
  activeTab.value = "connect";
}

function goPending() {
  clearStack();
  activeTab.value = "pending";
}

function openSession(session: { id: string; title: string; subtitle: string }) {
  stack.value = { type: "session", ...session };
  if (stackHistoryOwned) {
    history.replaceState({ coStack: "session" }, "");
  } else {
    history.pushState({ coStack: "session" }, "");
    stackHistoryOwned = true;
  }
}

function popStack() {
  clearStack();
}

async function onRefresh() {
  if (!isConnected.value && connectionState.value === "idle") {
    showToast("尚未连接");
    return;
  }
  try {
    await refresh();
    showToast("已刷新");
  } catch {
    showToast(errorMessage.value || "刷新失败");
  }
}

async function onSaveConnect(payload: {
  id?: string;
  name: string;
  baseUrl: string;
  token: string;
}) {
  try {
    const saved = await saveAndConnect(payload);
    showToast(`已连接 · ${saved.name}`);
  } catch {
    showToast(errorMessage.value || "连接失败");
  }
}

async function onConnectExisting(id: string) {
  try {
    await connectExisting(id);
    showToast(`已连接 · ${activeTarget.value?.name ?? ""}`);
  } catch {
    showToast(errorMessage.value || "连接失败");
  }
}

function requestDeleteTarget(payload: { id: string; name: string }) {
  openConfirm({
    title: "删除目标",
    message: `确定删除「${payload.name}」？本地保存的 Token 也会移除。`,
    confirmLabel: "删除",
    danger: true,
    action: { kind: "deleteTarget", id: payload.id, name: payload.name },
  });
}

async function doDeleteTarget(id: string) {
  try {
    await removeTarget(id);
    showToast("已删除目标");
  } catch (error) {
    showToast(formatRuntimeError(error));
  }
}

function onDisconnect() {
  disconnect();
  showToast("已断开连接");
}

async function withPendingAction(
  okMsg: string,
  fn: (api: NonNullable<typeof client.value>) => Promise<void>,
) {
  const api = client.value;
  if (!api) {
    showToast("尚未连接");
    return;
  }
  try {
    await fn(api);
    await loadPending(api);
    showToast(okMsg);
  } catch (error) {
    showToast(formatRuntimeError(error));
    try {
      await loadPending(api);
    } catch {
      // still bump list identity so PendingView can clear busyId
      pending.value = pending.value.slice();
    }
  }
}

function onAllow(actionId: string, always: boolean) {
  void withPendingAction(always ? "已设为总是允许" : "已允许", (api) =>
    api.allowPermission(actionId, always).then(() => undefined),
  );
}

function onDeny(actionId: string, reason: string) {
  void withPendingAction("已拒绝", (api) =>
    api.denyPermission(actionId, reason).then(() => undefined),
  );
}

function onAnswer(actionId: string, answers: string[]) {
  void withPendingAction("已回答", async (api) => {
    await api.answerQuestion(actionId, answers);
  });
}

function onDismiss(actionId: string) {
  void withPendingAction("已关闭问题", (api) =>
    api.dismissQuestion(actionId).then(() => undefined),
  );
}

function requestDismissSession(sessionId: string) {
  openConfirm({
    title: "移除会话",
    message: "从列表移除此会话？",
    confirmLabel: "移除",
    danger: true,
    action: { kind: "dismissSession", sessionId },
  });
}

async function doDismissSession(sessionId: string) {
  const api = client.value;
  if (!api) {
    showToast("尚未连接");
    return;
  }
  try {
    await api.dismissSession(sessionId);
    await loadSessions(api);
    if (stack.value?.type === "session" && stack.value.id === sessionId) {
      clearStack();
    }
    showToast("已移除会话");
  } catch (error) {
    showToast(formatRuntimeError(error));
  }
}

/* —— presence transitions (Anime via motion.ts) —— */

function onTabEnter(el: Element, done: () => void) {
  void animatePresence(el, "fade-up", "in", {
    duration: MOTION.durationFast,
  }).then(done);
}

function onTabLeave(el: Element, done: () => void) {
  void animatePresence(el, "fade", "out", {
    duration: MOTION.durationFast - 30,
  }).then(done);
}

function onStackEnter(el: Element, done: () => void) {
  void animatePresence(el, "fade-right", "in", {
    duration: MOTION.durationSlow,
  }).then(done);
}

function onStackLeave(el: Element, done: () => void) {
  void animatePresence(el, "fade-right", "out", {
    duration: MOTION.duration,
  }).then(done);
}

function onTabBarEnter(el: Element, done: () => void) {
  void animatePresence(el, "tabbar", "in").then(done);
}

function onTabBarLeave(el: Element, done: () => void) {
  void animatePresence(el, "tabbar", "out").then(done);
}

function onPresenceCancelled(el: Element) {
  cancelMotion(el);
}
</script>

<template>
  <div class="app-shell">
    <TopBar
      :title="chromeTitle"
      :subtitle="chromeSubtitle"
      :connection-state="connectionState"
      :show-back="!!stack"
      @back="popStack"
      @refresh="onRefresh"
    />

    <main class="content" :class="{ 'content--detail': !!stack }">
      <Transition
        mode="out-in"
        :css="false"
        @enter="onTabEnter"
        @leave="onTabLeave"
        @enter-cancelled="onPresenceCancelled"
        @leave-cancelled="onPresenceCancelled"
      >
        <div v-if="!stack" :key="activeTab" class="tab-pane">
          <PendingView
            v-if="activeTab === 'pending'"
            :connection-state="connectionState"
            :pending="pending"
            :loading="loading"
            @go-connect="goConnect"
            @toast="showToast"
            @retry="onRefresh"
            @allow="onAllow"
            @deny="onDeny"
            @answer="onAnswer"
            @dismiss="onDismiss"
          />
          <SessionsView
            v-else-if="activeTab === 'sessions'"
            :connected="isConnected"
            :sessions="sessions"
            :loading="loading"
            @go-connect="goConnect"
            @open-session="openSession"
            @dismiss="requestDismissSession"
          />
          <ConnectView
            v-else
            :connection-state="connectionState"
            :pending-count="pendingCount"
            :loading="loading"
            :targets="targets"
            :active-target="activeTarget"
            :health="health"
            :version="version"
            :error-message="errorMessage"
            @save-connect="onSaveConnect"
            @connect-existing="onConnectExisting"
            @delete-target="requestDeleteTarget"
            @disconnect="onDisconnect"
            @go-pending="goPending"
            @toast="showToast"
          />
        </div>
      </Transition>

      <Transition
        :css="false"
        @enter="onStackEnter"
        @leave="onStackLeave"
        @enter-cancelled="onPresenceCancelled"
        @leave-cancelled="onPresenceCancelled"
      >
        <div
          v-if="stack?.type === 'session'"
          :key="stack.id"
          class="stack-pane"
        >
          <SessionDetailView
            :session-id="stack.id"
            :title="stack.title"
            :client="client"
            :reload-token="sessionsEpoch"
          />
        </div>
      </Transition>
    </main>

    <Transition
      :css="false"
      @enter="onTabBarEnter"
      @leave="onTabBarLeave"
      @enter-cancelled="onPresenceCancelled"
      @leave-cancelled="onPresenceCancelled"
    >
      <TabBar
        v-if="!stack"
        :active="activeTab"
        :pending-count="pendingCount"
        :show-badge="showPendingBadge"
        @change="onTabChange"
      />
    </Transition>

    <div class="toast" :class="{ show: toastVisible }" role="status" aria-live="polite">
      {{ toastMessage }}
    </div>

    <ConfirmDialog
      :open="confirmOpen"
      :title="confirmTitle"
      :message="confirmMessage"
      :confirm-label="confirmLabel"
      cancel-label="取消"
      :danger="confirmDanger"
      @confirm="onConfirmOk"
      @cancel="onConfirmCancel"
    />
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 100%;
  max-width: 480px;
  margin: 0 auto;
  background: var(--bg);
  color: var(--fg);
  position: relative;
}

.content {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  -webkit-overflow-scrolling: touch;
  padding: 4px 0 16px;
  position: relative;
}

.content--detail {
  overflow: hidden;
  padding: 0;
  display: flex;
  flex-direction: column;
}

.content::-webkit-scrollbar {
  display: none;
}

.tab-pane {
  min-height: 100%;
  will-change: transform, opacity;
}

.stack-pane {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--bg);
  will-change: transform, opacity;
  z-index: 2;
}
</style>
