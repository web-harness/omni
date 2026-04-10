import initDeepagentsModule, {
  deepagents_create_thread,
  deepagents_delete_api_key,
  deepagents_delete_default_model,
  deepagents_delete_run,
  deepagents_delete_thread,
  deepagents_delete_thread_messages,
  deepagents_ensure_workspace_scaffold,
  deepagents_get_api_key,
  deepagents_get_default_model,
  deepagents_get_stored_default_model,
  deepagents_get_run,
  deepagents_get_thread,
  deepagents_hash_agent_config,
  deepagents_list_messages,
  deepagents_list_runs,
  deepagents_list_threads,
  deepagents_mock_thread_files,
  deepagents_mock_thread_ids,
  deepagents_mock_tool_calls,
  deepagents_mock_tool_results,
  deepagents_mock_workspace_files,
  deepagents_patch_thread,
  deepagents_save_thread,
  deepagents_save_message,
  deepagents_save_run,
  deepagents_search_runs,
  deepagents_seed_agent_endpoints,
  deepagents_seed_threads,
  deepagents_set_api_key,
  deepagents_set_default_model,
  deepagents_set_thread_status,
  deepagents_workspace_seed_entries,
} from "./omni-deepagents.js";

export type DeepagentsThread = Record<string, unknown>;
export type DeepagentsMessage = Record<string, unknown>;
export type DeepagentsRun = Record<string, unknown>;
export type WorkspaceSeedEntry = { path: string; text?: string | null; fixture?: string | null; size: number };

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

export async function workspaceSeedEntries(): Promise<WorkspaceSeedEntry[]> {
  await ensureReady();
  return (await deepagents_workspace_seed_entries()) as WorkspaceSeedEntry[];
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

export type MockThreadIds = { gtd: string; auth: string; db: string; ci: string; idea: string };
export type SeedThread = {
  id: string;
  title: string;
  status: string;
  updated_at: string;
  workspace?: string;
  messages: Array<{ id: string; role: string; content: string; created_at: string }>;
  todos: Array<{ id: string; content: string; status: string }>;
  subagents: Array<{ id: string; name: string; description: string; status: string }>;
};
export type SeedAgentEndpoint = {
  id: string;
  url: string;
  bearer_token: string;
  name: string;
  removable: boolean;
};
export type MockFileEntry = { path: string; is_dir: boolean; size: number | null };
export type MockToolCall = { id: string; name: string; args: unknown };
export type MockToolResult = { tool_call_id: string; content: string; is_error: boolean };

export async function mockThreadIds(): Promise<MockThreadIds> {
  await ensureReady();
  return deepagents_mock_thread_ids() as MockThreadIds;
}

export async function seedThreads(): Promise<SeedThread[]> {
  await ensureReady();
  return deepagents_seed_threads() as SeedThread[];
}

export async function seedAgentEndpoints(): Promise<SeedAgentEndpoint[]> {
  await ensureReady();
  return deepagents_seed_agent_endpoints() as SeedAgentEndpoint[];
}

export async function hashAgentConfig(url: string, bearerToken: string): Promise<string> {
  await ensureReady();
  return deepagents_hash_agent_config(url, bearerToken);
}

export async function mockThreadFiles(threadId: string): Promise<MockFileEntry[]> {
  await ensureReady();
  return deepagents_mock_thread_files(threadId) as MockFileEntry[];
}

export async function mockToolCalls(threadId: string): Promise<MockToolCall[]> {
  await ensureReady();
  return deepagents_mock_tool_calls(threadId) as MockToolCall[];
}

export async function mockToolResults(threadId: string): Promise<MockToolResult[]> {
  await ensureReady();
  return deepagents_mock_tool_results(threadId) as MockToolResult[];
}

export async function mockWorkspaceFiles(): Promise<Record<string, MockFileEntry[]>> {
  await ensureReady();
  return deepagents_mock_workspace_files() as Record<string, MockFileEntry[]>;
}

export async function ensureWorkspaceScaffold(): Promise<void> {
  await ensureReady();
  await deepagents_ensure_workspace_scaffold();
}
