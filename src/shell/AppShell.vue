<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import TopBar from "./TopBar.vue";
import TabBar, { type TabId } from "./TabBar.vue";
import PendingView from "../views/PendingView.vue";
import SessionsView from "../views/SessionsView.vue";
import ConnectView from "../views/ConnectView.vue";
import SessionDetailView from "../views/SessionDetailView.vue";
import { useRuntimeConnection } from "../composables/useRuntimeConnection";
import { formatRuntimeError } from "../runtimeApi";

const {
  targets,
  activeTarget,
  connectionState,
  health,
  version,
  pendingCount,
  errorMessage,
  loading,
  isConnected,
  loadTargets,
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
  return undefined;
});

onMounted(async () => {
  try {
    await loadTargets();
    if (targets.value.length === 0) activeTab.value = "connect";
  } catch (error) {
    showToast(formatRuntimeError(error));
  }
});

function showToast(msg: string) {
  toastMessage.value = msg;
  toastVisible.value = true;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toastVisible.value = false;
  }, 1600);
}

function onTabChange(tab: TabId) {
  stack.value = null;
  activeTab.value = tab;
}

function goConnect() {
  stack.value = null;
  activeTab.value = "connect";
}

function goPending() {
  stack.value = null;
  activeTab.value = "pending";
}

function openSession(session: { id: string; title: string; subtitle: string }) {
  stack.value = { type: "session", ...session };
}

function popStack() {
  stack.value = null;
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

async function onDeleteTarget(id: string) {
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

    <main class="content">
      <SessionDetailView
        v-if="stack?.type === 'session'"
        :session-id="stack.id"
        :title="stack.title"
      />
      <template v-else>
        <PendingView
          v-if="activeTab === 'pending'"
          :connection-state="connectionState"
          @go-connect="goConnect"
          @toast="showToast"
          @retry="onRefresh"
        />
        <SessionsView
          v-else-if="activeTab === 'sessions'"
          :connected="isConnected"
          @go-connect="goConnect"
          @open-session="openSession"
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
          @delete-target="onDeleteTarget"
          @disconnect="onDisconnect"
          @go-pending="goPending"
          @toast="showToast"
        />
      </template>
    </main>

    <TabBar
      v-if="!stack"
      :active="activeTab"
      :pending-count="pendingCount"
      :show-badge="showPendingBadge"
      @change="onTabChange"
    />

    <div class="toast" :class="{ show: toastVisible }" role="status" aria-live="polite">
      {{ toastMessage }}
    </div>
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
}

.content::-webkit-scrollbar {
  display: none;
}
</style>
