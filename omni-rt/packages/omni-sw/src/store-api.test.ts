import { beforeEach, describe, expect, it, vi } from "vitest";

const mockState = vi.hoisted(() => {
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  const files = new Map<string, Uint8Array>();
  const apiKeys = new Map<string, string>();
  let defaultModel = "";
  const checkpoints = new Map<
    string,
    Array<{
      config?: { configurable?: Record<string, unknown> };
      checkpoint: Record<string, unknown>;
      metadata?: Record<string, unknown>;
    }>
  >();

  const normalize = (path: string) => path.replace(/\/+/g, "/").replace(/\/$/, "") || "/";
  const hasDir = (dir: string) => {
    const clean = normalize(dir);
    for (const key of files.keys()) {
      if (key.startsWith(`${clean}/`)) {
        return true;
      }
    }
    return false;
  };

  return {
    encoder,
    decoder,
    files,
    apiKeys,
    checkpoints,
    normalize,
    hasDir,
    get defaultModel() {
      return defaultModel;
    },
    set defaultModel(value: string) {
      defaultModel = value;
    },
    reset() {
      files.clear();
      apiKeys.clear();
      checkpoints.clear();
      defaultModel = "";
    },
  };
});

vi.mock("./store-data.js", () => ({
  buildBootstrap: vi.fn(async () => ({ ok: true })),
  readProvidersWithKeys: vi.fn(async () => []),
  deleteThread: vi.fn(async (threadId: string) => {
    for (const key of [...mockState.files.keys()]) {
      if (key.includes(`/${threadId}`) || key.endsWith(`/${threadId}.json`)) {
        mockState.files.delete(key);
      }
    }
  }),
  getApiKey: vi.fn(async (provider: string) => mockState.apiKeys.get(provider) ?? ""),
  setApiKey: vi.fn(async (provider: string, value: string) => {
    mockState.apiKeys.set(provider, value);
  }),
  deleteApiKey: vi.fn(async (provider: string) => {
    mockState.apiKeys.delete(provider);
  }),
  setDefaultModel: vi.fn(async (modelId: string) => {
    mockState.defaultModel = modelId;
    if (modelId) {
      mockState.files.set("/home/config/default_model", mockState.encoder.encode(modelId));
    }
  }),
  getDefaultModel: vi.fn(async () => mockState.defaultModel || "claude-3-7-sonnet"),
  getStoredDefaultModel: vi.fn(async () => mockState.defaultModel || null),
  deleteDefaultModel: vi.fn(async () => {
    mockState.defaultModel = "";
    mockState.files.delete("/home/config/default_model");
  }),
}));

vi.mock("./zenfs.js", () => ({
  init: vi.fn(async () => {}),
  exists: vi.fn(async (path: string) => {
    const clean = mockState.normalize(path);
    return mockState.files.has(clean) || mockState.hasDir(clean);
  }),
  mkdir: vi.fn(async () => {}),
  readFile: vi.fn(async (path: string) => {
    const clean = mockState.normalize(path);
    const value = mockState.files.get(clean);
    if (!value) {
      throw new Error("ENOENT");
    }
    return value;
  }),
  writeFile: vi.fn(async (path: string, data: Uint8Array) => {
    mockState.files.set(mockState.normalize(path), data);
  }),
  rm: vi.fn(async (path: string, options?: { recursive?: boolean }) => {
    const clean = mockState.normalize(path);
    if (options?.recursive) {
      for (const key of [...mockState.files.keys()]) {
        if (key === clean || key.startsWith(`${clean}/`)) {
          mockState.files.delete(key);
        }
      }
      return;
    }
    mockState.files.delete(clean);
  }),
  readdir: vi.fn(async (path: string) => {
    const clean = mockState.normalize(path);
    const entries = new Map<string, { name: string; is_dir: boolean; is_file: boolean }>();
    for (const key of mockState.files.keys()) {
      if (!key.startsWith(`${clean}/`)) {
        continue;
      }
      const rest = key.slice(clean.length + 1);
      if (!rest) {
        continue;
      }
      const [name, ...tail] = rest.split("/");
      if (!entries.has(name)) {
        entries.set(name, { name, is_dir: tail.length > 0, is_file: tail.length === 0 });
      }
    }
    return [...entries.values()];
  }),
}));

vi.mock("./checkpointer.js", () => ({
  SqlJsSaver: class {
    async getTuple(config: { configurable?: Record<string, unknown> }) {
      const threadId = String(config.configurable?.thread_id ?? "");
      const checkpointId = String(config.configurable?.checkpoint_id ?? "");
      return (mockState.checkpoints.get(threadId) ?? []).find(
        (tuple) => tuple.config?.configurable?.checkpoint_id === checkpointId,
      );
    }

    async *list(config: { configurable?: Record<string, unknown> }) {
      const threadId = String(config.configurable?.thread_id ?? "");
      for (const tuple of mockState.checkpoints.get(threadId) ?? []) {
        yield tuple;
      }
    }
  },
}));

import { handleStoreRoute } from "./store-api.js";

function jsonRequest(url: string, method: string, body?: unknown): Request {
  return new Request(url, {
    method,
    headers: { "Content-Type": "application/json" },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
  });
}

beforeEach(() => {
  mockState.reset();
});

describe("store-api parity", () => {
  it("filters agents by name and returns dotted capability keys", async () => {
    const response = await handleStoreRoute(
      jsonRequest("https://example.test/agents/search", "POST", {
        name: "deep",
        metadata: { provider: "omni" },
        limit: 1,
        offset: 0,
      }),
      "agents-search",
    );

    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body).toHaveLength(1);
    expect(body[0].capabilities["ap.io.messages"]).toBe(true);
    expect(body[0].capabilities["ap.io.streaming"]).toBe(true);

    const nextPage = await handleStoreRoute(
      jsonRequest("https://example.test/agents/search", "POST", {
        name: "deep",
        metadata: { provider: "omni" },
        limit: 1,
        offset: 1,
      }),
      "agents-search",
    );
    expect(await nextPage.json()).toEqual([]);
  });

  it("returns agent payloads and rejects unknown agent ids", async () => {
    const response = await handleStoreRoute(
      new Request("https://example.test/agents/deepagent", { method: "GET" }),
      "agents-get",
    );

    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body.agent_id).toBe("deepagent");
    expect(body.capabilities["ap.io.messages"]).toBe(true);

    const missing = await handleStoreRoute(
      new Request("https://example.test/agents/missing", { method: "GET" }),
      "agents-get",
    );
    expect(missing.status).toBe(404);
  });

  it("returns agent schemas for the default agent", async () => {
    const response = await handleStoreRoute(
      new Request("https://example.test/agents/deepagent/schemas", { method: "GET" }),
      "agents-schemas",
    );

    expect(response.status).toBe(200);
    const body = await response.json();
    expect(body.agent_id).toBe("deepagent");
    expect(body.input_schema.properties.stream_mode).toBeDefined();
  });

  it("enforces thread UUIDs and filters thread search by values", async () => {
    const invalid = await handleStoreRoute(
      jsonRequest("https://example.test/threads", "POST", { thread_id: "not-a-uuid" }),
      "threads-create",
    );
    expect(invalid.status).toBe(422);

    const threadId = "11111111-1111-4111-8111-111111111111";
    const created = await handleStoreRoute(
      jsonRequest("https://example.test/threads", "POST", { thread_id: threadId, metadata: { scope: "tests" } }),
      "threads-create",
    );
    expect(created.status).toBe(200);

    const duplicate = await handleStoreRoute(
      jsonRequest("https://example.test/threads", "POST", { thread_id: threadId }),
      "threads-create",
    );
    expect(duplicate.status).toBe(409);

    const patched = await handleStoreRoute(
      jsonRequest(`https://example.test/threads/${threadId}`, "PATCH", {
        values: { branch: "main", label: "agent" },
        messages: [{ role: "user", content: "hello" }],
      }),
      "threads-patch",
    );
    const patchedBody = await patched.json();
    expect(patchedBody.values).toEqual({ branch: "main", label: "agent" });
    expect(patchedBody.messages).toHaveLength(1);

    const searched = await handleStoreRoute(
      jsonRequest("https://example.test/threads/search", "POST", { values: { branch: "main" } }),
      "threads-search",
    );
    const searchBody = await searched.json();
    expect(searchBody).toHaveLength(1);
    expect(searchBody[0].thread_id).toBe(threadId);

    const checkpointPatch = await handleStoreRoute(
      jsonRequest(`https://example.test/threads/${threadId}`, "PATCH", {
        checkpoint: { checkpoint_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa" },
      }),
      "threads-patch",
    );
    expect(checkpointPatch.status).toBe(422);
  });

  it("supports thread history limit and before parameters", async () => {
    const threadId = "22222222-2222-4222-8222-222222222222";
    mockState.files.set(
      `/home/db/threads/${threadId}.json`,
      mockState.encoder.encode(
        JSON.stringify({
          thread_id: threadId,
          created_at: "2026-01-01T00:00:00.000Z",
          updated_at: "2026-01-01T00:00:00.000Z",
          metadata: {},
          status: "idle",
        }),
      ),
    );
    mockState.checkpoints.set(threadId, [
      {
        config: { configurable: { checkpoint_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa" } },
        checkpoint: { id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", channel_values: { step: 3 }, source: "latest" },
        metadata: { order: 3 },
      },
      {
        config: { configurable: { checkpoint_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb" } },
        checkpoint: { id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", channel_values: { step: 2 } },
        metadata: { order: 2 },
      },
      {
        config: { configurable: { checkpoint_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc" } },
        checkpoint: { id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", channel_values: { step: 1 } },
        metadata: { order: 1 },
      },
    ]);

    const response = await handleStoreRoute(
      jsonRequest(
        `https://example.test/threads/${threadId}/history?limit=1&before=aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa`,
        "GET",
      ),
      "threads-history",
    );
    const body = await response.json();
    expect(body).toHaveLength(1);
    expect(body[0].checkpoint.checkpoint_id).toBe("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb");

    const fullResponse = await handleStoreRoute(
      new Request(`https://example.test/threads/${threadId}/history?limit=3`, { method: "GET" }),
      "threads-history",
    );
    const fullBody = await fullResponse.json();
    expect(fullBody[0].checkpoint.source).toBe("latest");
  });

  it("honors thread if_exists behavior plus metadata, status, limit, and offset filters", async () => {
    const firstThreadId = "55555555-5555-4555-8555-555555555555";
    const secondThreadId = "66666666-6666-4666-8666-666666666666";

    const created = await handleStoreRoute(
      jsonRequest("https://example.test/threads", "POST", {
        thread_id: firstThreadId,
        metadata: { scope: "tests", lane: "a" },
      }),
      "threads-create",
    );
    expect(created.status).toBe(200);

    const duplicate = await handleStoreRoute(
      jsonRequest("https://example.test/threads", "POST", {
        thread_id: firstThreadId,
        if_exists: "do_nothing",
      }),
      "threads-create",
    );
    expect(duplicate.status).toBe(200);

    mockState.files.set(
      `/home/db/threads/${secondThreadId}.json`,
      mockState.encoder.encode(
        JSON.stringify({
          thread_id: secondThreadId,
          created_at: "2026-01-02T00:00:00.000Z",
          updated_at: "2026-01-02T00:00:00.000Z",
          metadata: { scope: "tests", lane: "b" },
          status: "busy",
          values: { branch: "main" },
        }),
      ),
    );

    const searched = await handleStoreRoute(
      jsonRequest("https://example.test/threads/search", "POST", {
        metadata: { scope: "tests" },
        status: "busy",
        offset: 0,
        limit: 1,
      }),
      "threads-search",
    );
    expect(searched.status).toBe(200);
    const searchBody = await searched.json();
    expect(searchBody).toHaveLength(1);
    expect(searchBody[0].thread_id).toBe(secondThreadId);

    const nextPage = await handleStoreRoute(
      jsonRequest("https://example.test/threads/search", "POST", {
        metadata: { scope: "tests" },
        offset: 1,
        limit: 1,
      }),
      "threads-search",
    );
    const nextPageBody = await nextPage.json();
    expect(nextPageBody).toHaveLength(1);

    const invalidGet = await handleStoreRoute(
      new Request("https://example.test/threads/not-a-uuid", { method: "GET" }),
      "threads-get",
    );
    expect(invalidGet.status).toBe(422);
  });

  it("copies thread messages and lists namespaces for config and non-config items", async () => {
    const threadId = "77777777-7777-4777-8777-777777777777";
    mockState.files.set(
      `/home/db/threads/${threadId}.json`,
      mockState.encoder.encode(
        JSON.stringify({
          thread_id: threadId,
          created_at: "2026-01-03T00:00:00.000Z",
          updated_at: "2026-01-03T00:00:00.000Z",
          metadata: { title: "Source" },
          status: "idle",
        }),
      ),
    );
    mockState.files.set(
      `/home/db/messages/${threadId}/m1.json`,
      mockState.encoder.encode(
        JSON.stringify({
          id: "m1",
          thread_id: threadId,
          created_at: "2026-01-03T00:00:00.000Z",
          role: "user",
          content: "copy me",
        }),
      ),
    );

    const copied = await handleStoreRoute(
      new Request(`https://example.test/threads/${threadId}/copy`, { method: "POST" }),
      "threads-copy",
    );
    expect(copied.status).toBe(200);
    const copiedBody = await copied.json();
    const copiedThread = await handleStoreRoute(
      new Request(`https://example.test/threads/${copiedBody.thread_id}`, { method: "GET" }),
      "threads-get",
    );
    const copiedThreadBody = await copiedThread.json();
    expect(copiedThreadBody.messages).toHaveLength(1);
    expect(copiedThreadBody.messages[0].content).toBe("copy me");

    const putCustom = await handleStoreRoute(
      jsonRequest("https://example.test/store/items", "PUT", {
        namespace: ["workspace", "settings"],
        key: "theme",
        value: { mode: "light" },
      }),
      "store-items-put",
    );
    expect(putCustom.status).toBe(204);

    await handleStoreRoute(
      jsonRequest("https://example.test/store/items", "PUT", {
        namespace: ["config"],
        key: "default_model",
        value: { model_id: "gpt-5" },
      }),
      "store-items-put",
    );

    const namespaces = await handleStoreRoute(
      jsonRequest("https://example.test/store/namespaces", "POST", { prefix: [], limit: 10, offset: 0 }),
      "store-namespaces",
    );
    expect(namespaces.status).toBe(200);
    const namespaceBody = await namespaces.json();
    expect(namespaceBody).toContainEqual(["workspace", "settings"]);
    expect(namespaceBody).toContainEqual(["config"]);
  });

  it("stores config-backed items as real items and returns search response envelopes", async () => {
    const emptyConfigSearch = await handleStoreRoute(
      jsonRequest("https://example.test/store/items/search", "POST", {
        namespace_prefix: ["config"],
      }),
      "store-items-search",
    );
    const emptyConfigSearchBody = await emptyConfigSearch.json();
    expect(emptyConfigSearchBody.items).toEqual([]);

    const emptyNamespaces = await handleStoreRoute(
      jsonRequest("https://example.test/store/namespaces", "POST", { prefix: ["config"], limit: 10, offset: 0 }),
      "store-namespaces",
    );
    expect(await emptyNamespaces.json()).toEqual([]);

    const badGet = await handleStoreRoute(
      new Request("https://example.test/store/items?namespace=config", { method: "GET" }),
      "store-items-get",
    );
    expect(badGet.status).toBe(400);

    const put = await handleStoreRoute(
      jsonRequest("https://example.test/store/items", "PUT", {
        namespace: ["config", "api-keys"],
        key: "openai",
        value: { api_key: "secret" },
      }),
      "store-items-put",
    );
    expect(put.status).toBe(204);

    const get = await handleStoreRoute(
      new Request("https://example.test/store/items?namespace=config&namespace=api-keys&key=openai", { method: "GET" }),
      "store-items-get",
    );
    const item = await get.json();
    expect(item.value).toEqual({ api_key: "secret" });

    const search = await handleStoreRoute(
      jsonRequest("https://example.test/store/items/search", "POST", {
        namespace_prefix: ["config"],
        filter: { api_key: "secret" },
      }),
      "store-items-search",
    );
    const searchBody = await search.json();
    expect(searchBody.items).toHaveLength(1);

    const deleted = await handleStoreRoute(
      jsonRequest("https://example.test/store/items", "DELETE", {
        namespace: ["config", "api-keys"],
        key: "openai",
      }),
      "store-items-delete",
    );
    expect(deleted.status).toBe(204);

    const missing = await handleStoreRoute(
      new Request("https://example.test/store/items?namespace=config&namespace=api-keys&key=openai", { method: "GET" }),
      "store-items-get",
    );
    expect(missing.status).toBe(404);
  });
});
