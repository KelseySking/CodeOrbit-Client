export interface RuntimeTargetConnection {
  id: string;
  name: string;
  baseUrl: string;
  token: string;
}

export interface ApiHealthDto {
  status: string;
  startedAtUtc: string;
}

export interface ApiVersionDto {
  product: string;
  version: string;
}

export interface ApiCapabilitiesDto {
  hookInjection: boolean;
  approval: boolean;
  question: boolean;
  transcript: boolean;
  realtime: boolean;
  realtimeProtocols: string[];
  securityMode: string;
}

export interface RuntimeAssetsDto {
  runtimeDirectory: string;
  hookScriptPath: string;
  bridgeExePath: string;
  installed: boolean;
}

export interface SourceCapabilitiesDto {
  hookInstall: boolean;
  approval: boolean;
  question: boolean;
  transcript: boolean;
  alwaysAllow: boolean;
}

export interface SourceDto {
  id: string;
  displayName: string;
  iconName: string;
  installed: boolean;
  capabilities: SourceCapabilitiesDto;
  sourceType: string;
}

export interface SourceOperationResultDto {
  source: string;
  success: boolean;
  installed: boolean;
  message: string;
}

export interface ChatMessageDto {
  isUser: boolean;
  text: string;
  timestampUtc: string;
}

export interface ToolHistoryEntryDto {
  toolName: string;
  timestampUtc: string;
  description?: string | null;
  success: boolean;
}

export interface SessionDto {
  sessionId: string;
  source: string;
  sourceDisplayName: string;
  projectName?: string | null;
  workingDirectory?: string | null;
  status: string;
  currentToolName?: string | null;
  currentToolDescription?: string | null;
  createdAtUtc: string;
  lastUpdatedAtUtc: string;
  trackedPid?: number | null;
  trackedProcessStartedAtUtc?: string | null;
  lastUserPrompt?: string | null;
  lastAssistantMessage?: string | null;
  completionText?: string | null;
  transcriptPath?: string | null;
  transcriptPosition: number;
  terminalApp?: string | null;
  terminalSessionId?: string | null;
  recentMessages: ChatMessageDto[];
  toolHistory: ToolHistoryEntryDto[];
}

export interface PermissionRequestDto {
  sessionId: string;
  toolName: string;
  toolUseId?: string | null;
  toolInput?: Record<string, unknown> | null;
  description?: string | null;
  hookEventName: string;
}

export interface QuestionOptionDto {
  label: string;
  description?: string | null;
  value?: string | null;
}

export interface QuestionItemDto {
  id?: string | null;
  question: string;
  header?: string | null;
  options: QuestionOptionDto[];
  multiSelect: boolean;
  allowFreeText: boolean;
}

export interface QuestionDto {
  sessionId: string;
  id?: string | null;
  question: string;
  header?: string | null;
  options: QuestionOptionDto[];
  multiSelect: boolean;
  isMultiQuestion: boolean;
  questions: QuestionItemDto[];
  hookEventName: string;
  isAskUserQuestion: boolean;
  isCodexRequestUserInput: boolean;
  currentQuestionIndex: number;
  currentAnswerKey: string;
}

export interface PendingActionDto {
  actionId: string;
  kind: string;
  sessionId: string;
  source: string;
  sourceDisplayName: string;
  projectName?: string | null;
  workingDirectory?: string | null;
  createdAtUtc: string;
  permission?: PermissionRequestDto | null;
  question?: QuestionDto | null;
}

export interface QuestionCurrentAnswerResultDto {
  success: boolean;
  resolved: boolean;
}

export interface HubEventDto {
  type: string;
  timestampUtc: string;
  data?: unknown;
}

export class RuntimeApiError extends Error {
  constructor(
    message: string,
    readonly status?: number,
  ) {
    super(message);
  }
}

export function createRuntimeClient(target: RuntimeTargetConnection) {
  async function getJson<T>(path: string, auth = true): Promise<T> {
    const response = await fetch(`${target.baseUrl}/api/${path}`, {
      headers: auth ? authHeaders(target.token) : undefined,
    });
    return readResponse<T>(response, path);
  }

  async function postJson<T>(path: string, body?: unknown): Promise<T> {
    const response = await fetch(`${target.baseUrl}/api/${path}`, {
      method: "POST",
      headers: {
        ...authHeaders(target.token),
        ...(body === undefined ? {} : { "content-type": "application/json" }),
      },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    return readResponse<T>(response, path);
  }

  return {
    getHealth: () => getJson<ApiHealthDto>("health", false),
    getVersion: () => getJson<ApiVersionDto>("version"),
    getCapabilities: () => getJson<ApiCapabilitiesDto>("capabilities"),
    getSources: () => getJson<SourceDto[]>("sources"),
    getRuntimeAssets: () => getJson<RuntimeAssetsDto>("runtime-assets"),
    getSessions: () => getJson<SessionDto[]>("sessions"),
    getPending: () => getJson<PendingActionDto[]>("pending"),
    installSource: (source: string) =>
      postJson<SourceOperationResultDto>(`sources/${encodeURIComponent(source)}/install`),
    repairSource: (source: string) =>
      postJson<SourceOperationResultDto>(`sources/${encodeURIComponent(source)}/repair`),
    uninstallSource: (source: string) =>
      postJson<SourceOperationResultDto>(`sources/${encodeURIComponent(source)}/uninstall`),
    repairAllSources: () => postJson<SourceOperationResultDto[]>("sources/repair-all"),
    repairRuntimeAssets: () =>
      postJson<{ success: boolean; assets: RuntimeAssetsDto }>("runtime-assets/repair"),
    allowPermission: (actionId: string, always: boolean) =>
      postJson<{ success: boolean }>(`permissions/${encodeURIComponent(actionId)}/allow`, {
        always,
        actor: "CodeOrbit Client",
      }),
    denyPermission: (actionId: string, reason: string) =>
      postJson<{ success: boolean }>(`permissions/${encodeURIComponent(actionId)}/deny`, {
        reason,
        actor: "CodeOrbit Client",
      }),
    answerQuestion: (actionId: string, answers: string[]) =>
      postJson<QuestionCurrentAnswerResultDto>(
        `questions/${encodeURIComponent(actionId)}/answer-current`,
        { answers, actor: "CodeOrbit Client" },
      ),
    dismissQuestion: (actionId: string) =>
      postJson<{ success: boolean }>(`questions/${encodeURIComponent(actionId)}/dismiss`),
    eventsUrl: () => buildEventsUrl(target),
  };
}

export function summarizeSession(session: SessionDto): string {
  return firstText(
    session.currentToolDescription,
    session.currentToolName,
    session.lastAssistantMessage,
    session.completionText,
    session.lastUserPrompt,
  );
}

export function currentQuestion(action: PendingActionDto): QuestionItemDto | QuestionDto | null {
  const question = action.question;
  if (!question) return null;
  return question.questions[question.currentQuestionIndex] ?? question;
}

export function formatRuntimeError(error: unknown): string {
  if (error instanceof RuntimeApiError) return error.message;
  if (error instanceof Error) return error.message;
  return "Runtime request failed.";
}

export function eventRefreshScope(event: HubEventDto): "sessions" | "pending" | "sources" | "assets" | "all" {
  switch (event.type) {
    case "session.updated":
    case "session.removed":
      return "sessions";
    case "pending.updated":
    case "pending.resolved":
      return "pending";
    case "source.statusChanged":
      return "sources";
    case "runtime.assetsChanged":
      return "assets";
    default:
      return "all";
  }
}

async function readResponse<T>(response: Response, path: string): Promise<T> {
  if (!response.ok) {
    const fallback = `Runtime /api/${path} failed with ${response.status}.`;
    throw new RuntimeApiError(await readErrorMessage(response, fallback), response.status);
  }

  return (await response.json()) as T;
}

async function readErrorMessage(response: Response, fallback: string): Promise<string> {
  try {
    const body = (await response.json()) as { message?: unknown; code?: unknown };
    if (typeof body.message === "string" && body.message.trim()) {
      return body.message;
    }
    if (typeof body.code === "string" && body.code.trim()) {
      return `${fallback} (${body.code})`;
    }
  } catch {
    // Non-JSON error responses are expected from proxies and network tools.
  }

  return fallback;
}

function authHeaders(token: string): HeadersInit {
  return { authorization: `Bearer ${token}` };
}

function buildEventsUrl(target: RuntimeTargetConnection): string {
  const url = new URL(target.baseUrl);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.pathname = "/api/events";
  url.search = "";
  url.searchParams.set("token", target.token);
  return url.toString();
}

function firstText(...values: Array<string | null | undefined>): string {
  return values.find((value) => value?.trim())?.replace(/\s+/g, " ").trim() ?? "";
}
