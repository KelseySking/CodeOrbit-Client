export type KeepAlivePayload = {
  targetName: string;
  pendingCount: number;
};

export type PendingAlertPayload = {
  title: string;
  body: string;
  count: number;
};

async function invokeCmd(cmd: string, args?: Record<string, unknown>): Promise<void> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    if (args) await invoke(cmd, args);
    else await invoke(cmd);
  } catch {
    // desktop / missing plugin — no-op
  }
}

// plugin command names: plugin:keepalive|<snake_case>
export function keepAliveStart(payload: KeepAlivePayload): Promise<void> {
  return invokeCmd("plugin:keepalive|keep_alive_start", { payload });
}

export function keepAliveUpdate(payload: KeepAlivePayload): Promise<void> {
  return invokeCmd("plugin:keepalive|keep_alive_update", { payload });
}

export function keepAliveStop(): Promise<void> {
  return invokeCmd("plugin:keepalive|keep_alive_stop");
}

export function pendingAlertNotify(payload: PendingAlertPayload): Promise<void> {
  return invokeCmd("plugin:keepalive|pending_alert_notify", { payload });
}

export function pendingAlertClear(): Promise<void> {
  return invokeCmd("plugin:keepalive|pending_alert_clear");
}
