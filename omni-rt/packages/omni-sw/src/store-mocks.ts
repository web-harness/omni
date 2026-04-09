import {
  hashAgentConfig as wasmHashAgentConfig,
  mockThreadFiles,
  mockThreadIds,
  mockToolCalls,
  mockToolResults,
  mockWorkspaceFiles,
  scaffoldFiles,
  seedAgentEndpoints as wasmSeedAgentEndpoints,
  seedThreads as wasmSeedThreads,
} from "./deepagents.js";
import type {
  MockFileEntry,
  MockThreadIds,
  MockToolCall,
  MockToolResult,
  ScaffoldFile,
  SeedAgentEndpoint,
  SeedThread,
} from "./deepagents.js";

export type { SeedThread, SeedAgentEndpoint, MockFileEntry, MockToolCall, MockToolResult, ScaffoldFile };

let cachedThreadIds: MockThreadIds | null = null;

export async function hashAgentConfig(url: string, bearerToken: string): Promise<string> {
  return wasmHashAgentConfig(url, bearerToken);
}

export async function seedAgentEndpoints(): Promise<SeedAgentEndpoint[]> {
  return wasmSeedAgentEndpoints();
}

async function ensureThreadIds(): Promise<MockThreadIds> {
  if (!cachedThreadIds) {
    cachedThreadIds = await mockThreadIds();
  }
  return cachedThreadIds;
}

export { ensureThreadIds as _ensureThreadIds };

export const MOCK_THREAD_IDS: MockThreadIds = {
  gtd: "11111111-1111-4111-8111-111111111111",
  auth: "22222222-2222-4222-8222-222222222222",
  db: "33333333-3333-4333-8333-333333333333",
  ci: "44444444-4444-4444-8444-444444444444",
  idea: "55555555-5555-4555-8555-555555555555",
};

export async function scaffoldFilesFromStore(): Promise<ScaffoldFile[]> {
  return scaffoldFiles();
}

export async function getMockWorkspaceFiles(): Promise<Record<string, MockFileEntry[]>> {
  return mockWorkspaceFiles();
}

export async function seedThreads(): Promise<SeedThread[]> {
  return wasmSeedThreads();
}

export async function getMockThreadFiles(threadId: string): Promise<MockFileEntry[]> {
  return mockThreadFiles(threadId);
}

export async function getMockToolCalls(threadId: string): Promise<MockToolCall[]> {
  return mockToolCalls(threadId);
}

export async function getMockToolResults(threadId: string): Promise<MockToolResult[]> {
  return mockToolResults(threadId);
}
