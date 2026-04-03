import initDeepagentsModule, {
  deepagents_create_thread,
  deepagents_delete_api_key,
  deepagents_delete_default_model,
  deepagents_delete_run,
  deepagents_delete_thread,
  deepagents_delete_thread_messages,
  deepagents_get_api_key,
  deepagents_get_default_model,
  deepagents_get_stored_default_model,
  deepagents_get_run,
  deepagents_get_thread,
  deepagents_list_messages,
  deepagents_list_runs,
  deepagents_list_threads,
  deepagents_patch_thread,
  deepagents_save_thread,
  deepagents_save_message,
  deepagents_save_run,
  deepagents_search_runs,
  deepagents_set_api_key,
  deepagents_set_default_model,
  deepagents_set_thread_status,
} from "./omni-deepagents.js";

export type DeepagentsThread = Record<string, unknown>;
export type DeepagentsMessage = Record<string, unknown>;
export type DeepagentsRun = Record<string, unknown>;

let readyPromise: Promise<void> | null = null;

async function ensureReady(): Promise<void> {
  if (!readyPromise) {
    readyPromise = initDeepagentsModule().catch((error) => {
      readyPromise = null;
      throw error;
    });
  }
  await readyPromise;
}

export async function listThreads(): Promise<DeepagentsThread[]> {
  await ensureReady();
  return (await deepagents_list_threads()) as DeepagentsThread[];
}

export async function getThread(threadId: string): Promise<DeepagentsThread | null> {
  await ensureReady();
  return ((await deepagents_get_thread(threadId)) as DeepagentsThread | null) ?? null;
}

export async function createThread(payload: Record<string, unknown>): Promise<DeepagentsThread> {
  await ensureReady();
  return (await deepagents_create_thread(payload)) as DeepagentsThread;
}

export async function patchThread(
  threadId: string,
  payload: Record<string, unknown>,
): Promise<DeepagentsThread | null> {
  await ensureReady();
  return ((await deepagents_patch_thread(threadId, payload)) as DeepagentsThread | null) ?? null;
}

export async function saveThread(payload: Record<string, unknown>): Promise<void> {
  await ensureReady();
  await deepagents_save_thread(payload);
}

export async function setThreadStatus(threadId: string, status: string): Promise<DeepagentsThread | null> {
  await ensureReady();
  return ((await deepagents_set_thread_status(threadId, status)) as DeepagentsThread | null) ?? null;
}

export async function deleteThread(threadId: string): Promise<void> {
  await ensureReady();
  await deepagents_delete_thread(threadId);
}

export async function listMessages(threadId: string): Promise<DeepagentsMessage[]> {
  await ensureReady();
  return (await deepagents_list_messages(threadId)) as DeepagentsMessage[];
}

export async function saveMessage(
  threadId: string,
  createdAt: string,
  payload: Record<string, unknown>,
): Promise<void> {
  await ensureReady();
  await deepagents_save_message(threadId, createdAt, payload);
}

export async function deleteThreadMessages(threadId: string): Promise<void> {
  await ensureReady();
  await deepagents_delete_thread_messages(threadId);
}

export async function getDefaultModel(): Promise<string> {
  await ensureReady();
  return String(await deepagents_get_default_model());
}

export async function getStoredDefaultModel(): Promise<string | null> {
  await ensureReady();
  return ((await deepagents_get_stored_default_model()) as string | null) ?? null;
}

export async function setDefaultModel(modelId: string): Promise<void> {
  await ensureReady();
  await deepagents_set_default_model(modelId);
}

export async function deleteDefaultModel(): Promise<void> {
  await ensureReady();
  await deepagents_delete_default_model();
}

export async function getApiKey(provider: string): Promise<string | null> {
  await ensureReady();
  return ((await deepagents_get_api_key(provider)) as string | null) ?? null;
}

export async function setApiKey(provider: string, value: string): Promise<void> {
  await ensureReady();
  await deepagents_set_api_key(provider, value);
}

export async function deleteApiKey(provider: string): Promise<void> {
  await ensureReady();
  await deepagents_delete_api_key(provider);
}

export async function listRuns(): Promise<DeepagentsRun[]> {
  await ensureReady();
  return (await deepagents_list_runs()) as DeepagentsRun[];
}

export async function getRun(runId: string): Promise<DeepagentsRun | null> {
  await ensureReady();
  return ((await deepagents_get_run(runId)) as DeepagentsRun | null) ?? null;
}

export async function searchRuns(payload: Record<string, unknown>): Promise<DeepagentsRun[]> {
  await ensureReady();
  return (await deepagents_search_runs(payload)) as DeepagentsRun[];
}

export async function saveRun(payload: Record<string, unknown>): Promise<void> {
  await ensureReady();
  await deepagents_save_run(payload);
}

export async function deleteRun(runId: string): Promise<void> {
  await ensureReady();
  await deepagents_delete_run(runId);
}
