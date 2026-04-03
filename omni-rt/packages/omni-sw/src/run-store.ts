import { deleteRun, getRun, listRuns, saveRun, searchRuns } from "./deepagents.js";

export type ProtocolMessage = {
  role: string;
  content: unknown;
  id?: string;
  metadata?: unknown;
  [key: string]: unknown;
};

export type StoredSseEvent = {
  id?: string;
  event: string;
  data: unknown;
};

export type PersistedRunStatus = "pending" | "error" | "success" | "timeout" | "interrupted";

export type PersistedStreamMode = "values" | "messages" | "updates" | "custom";

export type PersistedRun = {
  run: {
    run_id: string;
    created_at: string;
    updated_at: string;
    status: PersistedRunStatus;
    metadata: Record<string, unknown>;
    thread_id?: string;
    agent_id?: string;
    input?: unknown;
    messages?: ProtocolMessage[];
    config?: Record<string, unknown>;
    webhook?: string;
    on_completion?: "delete" | "keep";
    on_disconnect?: "cancel" | "continue";
    if_not_exists?: "create" | "reject";
    stream_mode?: PersistedStreamMode | PersistedStreamMode[];
  };
  values?: unknown;
  messages?: ProtocolMessage[];
  error?: string;
  events?: StoredSseEvent[];
};

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function matchesMetadata(actual: Record<string, unknown>, expected: Record<string, unknown> | undefined): boolean {
  if (!expected) {
    return true;
  }

  for (const [key, value] of Object.entries(expected)) {
    if (JSON.stringify(actual[key]) !== JSON.stringify(value)) {
      return false;
    }
  }

  return true;
}

export function createRunRecord(params: {
  threadId?: string;
  agentId?: string;
  input?: unknown;
  inputMessages?: ProtocolMessage[];
  metadata?: Record<string, unknown>;
  config?: Record<string, unknown>;
  webhook?: string;
  onCompletion?: "delete" | "keep";
  onDisconnect?: "cancel" | "continue";
  ifNotExists?: "create" | "reject";
  streamMode?: PersistedStreamMode | PersistedStreamMode[];
}): PersistedRun {
  const now = new Date().toISOString();
  return {
    run: {
      run_id: crypto.randomUUID(),
      created_at: now,
      updated_at: now,
      status: "pending",
      metadata: params.metadata ?? {},
      ...(params.threadId ? { thread_id: params.threadId } : {}),
      ...(params.agentId ? { agent_id: params.agentId } : {}),
      ...(params.input === undefined ? {} : { input: params.input }),
      ...(params.inputMessages?.length ? { messages: params.inputMessages } : {}),
      ...(params.config ? { config: params.config } : {}),
      ...(params.webhook ? { webhook: params.webhook } : {}),
      ...(params.onCompletion ? { on_completion: params.onCompletion } : {}),
      ...(params.onDisconnect ? { on_disconnect: params.onDisconnect } : {}),
      ...(params.ifNotExists ? { if_not_exists: params.ifNotExists } : {}),
      ...(params.streamMode ? { stream_mode: params.streamMode } : {}),
    },
    events: [],
  };
}

export async function saveRunRecord(record: PersistedRun): Promise<void> {
  await saveRun(record as Record<string, unknown>);
}

export async function getRunRecord(runId: string): Promise<PersistedRun | null> {
  const record = await getRun(runId);
  if (!isObject(record) || !isObject(record.run)) {
    return null;
  }

  return record as PersistedRun;
}

export async function listRunRecords(): Promise<PersistedRun[]> {
  return (await listRuns()).filter(
    (record): record is PersistedRun => isObject(record) && isObject(record.run),
  ) as PersistedRun[];
}

export async function searchRunRecords(body: {
  metadata?: Record<string, unknown>;
  status?: string;
  thread_id?: string;
  agent_id?: string;
  limit?: number;
  offset?: number;
}): Promise<PersistedRun[]> {
  const runs = await searchRuns(body);
  return runs.filter((record): record is PersistedRun => {
    if (!isObject(record) || !isObject(record.run)) {
      return false;
    }
    return matchesMetadata(record.run.metadata as Record<string, unknown>, body.metadata);
  }) as PersistedRun[];
}

export async function deleteRunRecord(runId: string): Promise<void> {
  await deleteRun(runId);
}
