import { computed, ref } from "vue";
import {
  createRuntimeClient,
  formatRuntimeError,
  RuntimeApiError,
  type ApiCapabilitiesDto,
  type ApiHealthDto,
  type ApiVersionDto,
  type PendingActionDto,
  type SessionDto,
} from "../runtimeApi";
import type { ConnectionState } from "../shell/StatusChip.vue";
import {
  deleteRuntimeTarget,
  getRuntimeTargetToken,
  listRuntimeTargets,
  normalizeBaseUrl,
  saveRuntimeTarget,
  type RuntimeTarget,
} from "../targetStore";

export type ConnectFormInput = {
  id?: string;
  name: string;
  baseUrl: string;
  /** empty = keep existing stored token when updating */
  token: string;
};

export function useRuntimeConnection() {
  const targets = ref<RuntimeTarget[]>([]);
  const activeTarget = ref<RuntimeTarget | null>(null);
  const connectionState = ref<ConnectionState>("idle");
  const health = ref<ApiHealthDto | null>(null);
  const version = ref<ApiVersionDto | null>(null);
  const capabilities = ref<ApiCapabilitiesDto | null>(null);
  const pending = ref<PendingActionDto[]>([]);
  const pendingCount = computed(() => pending.value.length);
  const sessions = ref<SessionDto[]>([]);
  const errorMessage = ref("");
  const loading = ref(false);

  let activeToken = "";
  let requestId = 0;

  const isConnected = computed(() => connectionState.value === "connected");

  const client = computed(() => {
    if (!activeTarget.value || !activeToken) return null;
    return createRuntimeClient({
      id: activeTarget.value.id,
      name: activeTarget.value.name,
      baseUrl: activeTarget.value.baseUrl,
      token: activeToken,
    });
  });

  async function loadTargets() {
    targets.value = await listRuntimeTargets();
  }

  function classifyError(error: unknown): ConnectionState {
    if (error instanceof RuntimeApiError && error.status === 401) {
      return "unauthorized";
    }
    const message = formatRuntimeError(error).toLowerCase();
    if (
      message.includes("401") ||
      message.includes("unauthorized") ||
      message.includes("invalid token") ||
      message.includes("forbidden")
    ) {
      return "unauthorized";
    }
    return "offline";
  }

  function clearSnapshots() {
    health.value = null;
    version.value = null;
    capabilities.value = null;
    pending.value = [];
    sessions.value = [];
  }

  function disconnect() {
    requestId += 1;
    activeToken = "";
    activeTarget.value = null;
    connectionState.value = "idle";
    errorMessage.value = "";
    loading.value = false;
    clearSnapshots();
  }

  async function loadPending(api?: ReturnType<typeof createRuntimeClient> | null) {
    const clientApi = api ?? client.value;
    if (!clientApi) {
      pending.value = [];
      return;
    }
    pending.value = await clientApi.getPending();
  }

  async function loadSessions(api?: ReturnType<typeof createRuntimeClient> | null) {
    const clientApi = api ?? client.value;
    if (!clientApi) {
      sessions.value = [];
      return;
    }
    sessions.value = await clientApi.getSessions();
  }

  async function probe(api: ReturnType<typeof createRuntimeClient>) {
    health.value = await api.getHealth();
    version.value = await api.getVersion();
    capabilities.value = await api.getCapabilities();
    try {
      await loadPending(api);
    } catch {
      pending.value = [];
    }
    try {
      await loadSessions(api);
    } catch {
      sessions.value = [];
    }
  }

  async function connectTarget(target: RuntimeTarget, tokenOverride?: string) {
    const rid = ++requestId;
    connectionState.value = "connecting";
    loading.value = true;
    errorMessage.value = "";
    activeTarget.value = target;
    clearSnapshots();

    try {
      const token = (tokenOverride ?? (await getRuntimeTargetToken(target.id))).trim();
      if (!token) {
        throw new RuntimeApiError("Token 未保存，请重新填写 Token。");
      }
      if (rid !== requestId) return;

      activeToken = token;
      const api = createRuntimeClient({
        id: target.id,
        name: target.name,
        baseUrl: target.baseUrl,
        token,
      });

      await probe(api);
      if (rid !== requestId) return;

      connectionState.value = "connected";
    } catch (error) {
      if (rid !== requestId) return;
      activeToken = "";
      connectionState.value = classifyError(error);
      errorMessage.value = formatRuntimeError(error);
      clearSnapshots();
      throw error;
    } finally {
      if (rid === requestId) loading.value = false;
    }
  }

  async function saveAndConnect(input: ConnectFormInput) {
    const name = input.name.trim() || "本地服务";
    const baseUrl = normalizeBaseUrl(input.baseUrl);
    const token = input.token.trim();

    if (!baseUrl) throw new Error("Runtime 地址不能为空");
    if (!input.id && !token) throw new Error("Token 不能为空");

    loading.value = true;
    errorMessage.value = "";
    connectionState.value = "connecting";

    try {
      const saved = await saveRuntimeTarget({
        id: input.id,
        name,
        baseUrl,
        token: token || undefined,
      });
      await loadTargets();
      await connectTarget(saved, token || undefined);
      return saved;
    } catch (error) {
      if (connectionState.value === "connecting") {
        connectionState.value = classifyError(error);
      }
      errorMessage.value = formatRuntimeError(error);
      loading.value = false;
      throw error;
    }
  }

  async function connectExisting(id: string) {
    const target = targets.value.find((item) => item.id === id);
    if (!target) throw new Error("未找到已保存的目标");
    await connectTarget(target);
  }

  async function removeTarget(id: string) {
    if (activeTarget.value?.id === id) {
      disconnect();
    }
    await deleteRuntimeTarget(id);
    await loadTargets();
  }

  async function refresh() {
    const api = client.value;
    if (!api || !activeTarget.value) {
      errorMessage.value = "尚未连接";
      return;
    }

    const rid = requestId;
    loading.value = true;
    errorMessage.value = "";
    try {
      await probe(api);
      if (rid !== requestId) return;
      connectionState.value = "connected";
    } catch (error) {
      if (rid !== requestId) return;
      connectionState.value = classifyError(error);
      errorMessage.value = formatRuntimeError(error);
      throw error;
    } finally {
      if (rid === requestId) loading.value = false;
    }
  }

  return {
    targets,
    activeTarget,
    connectionState,
    health,
    version,
    capabilities,
    pending,
    pendingCount,
    sessions,
    errorMessage,
    loading,
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
  };
}
