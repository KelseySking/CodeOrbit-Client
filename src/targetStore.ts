import { invoke } from "@tauri-apps/api/core";

export interface RuntimeTarget {
  id: string;
  name: string;
  baseUrl: string;
  createdAtUtc: string;
  updatedAtUtc: string;
}

export interface SaveRuntimeTargetInput {
  id?: string;
  name: string;
  baseUrl: string;
  token?: string;
}

export interface LocalRuntimeStartResult {
  success: boolean;
  message: string;
}

export function listRuntimeTargets(): Promise<RuntimeTarget[]> {
  return invoke("list_runtime_targets");
}

export function saveRuntimeTarget(input: SaveRuntimeTargetInput): Promise<RuntimeTarget> {
  return invoke("save_runtime_target", {
    request: {
      ...input,
      baseUrl: normalizeBaseUrl(input.baseUrl),
      token: input.token?.trim() || undefined,
    },
  });
}

export function deleteRuntimeTarget(id: string): Promise<void> {
  return invoke("delete_runtime_target", { id });
}

export function getRuntimeTargetToken(id: string): Promise<string> {
  return invoke("get_runtime_target_token", { id });
}

export function startLocalRuntime(): Promise<LocalRuntimeStartResult> {
  return invoke("start_local_runtime");
}

export function stopLocalRuntime(): Promise<LocalRuntimeStartResult> {
  return invoke("stop_local_runtime");
}

export function normalizeBaseUrl(value: string): string {
  return value.trim().replace(/\/+$/, "");
}
