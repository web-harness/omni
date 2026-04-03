import { beforeEach, describe, expect, it, vi } from "vitest";

const mockState = vi.hoisted(() => {
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  const files = new Map<string, string>();
  const normalize = (path: string) => path.replace(/\/+/g, "/").replace(/\/$/, "") || "/";
  return {
    encoder,
    decoder,
    files,
    normalize,
    reset() {
      files.clear();
    },
  };
});

vi.mock("deepagents", () => ({
  createDeepAgent: vi.fn(() => ({
    stream: async function* () {
      yield [{ content: "Hello" }];
      yield [{ content: " world" }];
    },
  })),
}));

vi.mock("../store-data.js", () => ({
  getApiKey: vi.fn(async () => "test-key"),
  getDefaultModel: vi.fn(async () => "gpt-5"),
}));

vi.mock("../zenfs.js", () => ({
  init: vi.fn(async () => {}),
  exists: vi.fn(async (path: string) => {
    const clean = mockState.normalize(path);
    return [...mockState.files.keys()].some((key) => key === clean || key.startsWith(`${clean}/`));
  }),
  mkdir: vi.fn(async () => {}),
  readFile: vi.fn(async (path: string) => {
    const clean = mockState.normalize(path);
    const value = mockState.files.get(clean);
    if (value === undefined) {
      throw new Error("ENOENT");
    }
    return mockState.encoder.encode(value);
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
  writeFile: vi.fn(async (path: string, data: Uint8Array) => {
    mockState.files.set(mockState.normalize(path), mockState.decoder.decode(data));
  }),
  rm: vi.fn(async (path: string) => {
    const clean = mockState.normalize(path);
    for (const key of [...mockState.files.keys()]) {
      if (key === clean || key.startsWith(`${clean}/`)) {
        mockState.files.delete(key);
      }
    }
  }),
  fs: {
    promises: {
      readFile: vi.fn(async (path: string) => {
        const clean = mockState.normalize(path);
        const value = mockState.files.get(clean);
        if (value === undefined) {
          throw new Error("ENOENT");
        }
        return value;
      }),
      writeFile: vi.fn(async (path: string, data: string | Uint8Array) => {
        const clean = mockState.normalize(path);
        if (typeof data === "string") {
          mockState.files.set(clean, data);
          return;
        }
        mockState.files.set(clean, mockState.decoder.decode(data));
      }),
      mkdir: vi.fn(async () => {}),
      rm: vi.fn(async (path: string) => {
        const clean = mockState.normalize(path);
        for (const key of [...mockState.files.keys()]) {
          if (key === clean || key.startsWith(`${clean}/`)) {
            mockState.files.delete(key);
          }
        }
      }),
    },
  },
}));

vi.mock("../checkpointer.js", () => ({
  SqlJsSaver: class {
    async getTuple() {
      return {
        checkpoint: {
          id: "99999999-9999-4999-8999-999999999999",
          channel_values: { session: "ok" },
        },
      };
    }
  },
}));

vi.mock("./omni-bashkit.js", () => ({
  default: vi.fn(async () => {}),
  execute: vi.fn(async () => ({ output: "", exitCode: 0, truncated: false })),
}));

vi.mock("./sandbox.js", () => ({
  BashkitSandboxBackend: class {
    constructor(readonly cwd: string) {}
  },
}));

vi.mock("./system-prompt.js", () => ({
  buildSystemPrompt: vi.fn(() => "system"),
}));

async function readResponseStream(response: Response): Promise<string> {
  const reader = response.body?.getReader();
  if (!reader) {
    return "";
  }
  let output = "";
  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      return output;
    }
    output += new TextDecoder().decode(value);
  }
}

function jsonRequest(url: string, method: string, body?: unknown): Request {
  return new Request(url, {
    method,
    headers: { "Content-Type": "application/json" },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
  });
}

function parseSseFrames(text: string): Array<{ id?: string; event?: string; data?: unknown }> {
  return text
    .trim()
    .split("\n\n")
    .filter(Boolean)
    .map((frame) => {
      const entry: { id?: string; event?: string; data?: unknown } = {};
      for (const line of frame.split("\n")) {
        if (line.startsWith("id: ")) {
          entry.id = line.slice(4);
        } else if (line.startsWith("event: ")) {
          entry.event = line.slice(7);
        } else if (line.startsWith("data: ")) {
          entry.data = JSON.parse(line.slice(6));
        }
      }
      return entry;
    });
}

beforeEach(() => {
  mockState.reset();
  vi.resetModules();
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response(null, { status: 200 })),
  );
});

describe("runtime parity", () => {
  it("rejects unsupported agent ids", async () => {
    const { handleRunRoute } = await import("./runtime.js");
    const response = await handleRunRoute(
      jsonRequest("https://example.test/runs", "POST", { input: "hello", agent_id: "other" }),
      "runs-create",
    );

    expect(response.status).toBe(404);
  });

  it("preserves full run request fields and returns spec-shaped runs", async () => {
    const { handleRunRoute } = await import("./runtime.js");
    const threadId = "33333333-3333-4333-8333-333333333333";
    const createResponse = await handleRunRoute(
      jsonRequest("https://example.test/runs", "POST", {
        thread_id: threadId,
        input: { task: "summarize" },
        agent_id: "deepagent",
        metadata: { trace: "abc" },
        config: { configurable: { model_id: "gpt-5" }, tags: ["tests"] },
        webhook: "https://example.test/webhook",
        on_completion: "keep",
        on_disconnect: "continue",
        if_not_exists: "create",
        stream_mode: ["messages", "values"],
      }),
      "runs-create",
    );

    expect(createResponse.status).toBe(200);
    const createdRun = await createResponse.json();
    expect(createdRun.thread_id).toBe(threadId);
    expect(createdRun.agent_id).toBe("deepagent");
    expect(createdRun.input).toEqual({ task: "summarize" });
    expect(createdRun.metadata).toEqual({ trace: "abc" });
    expect(createdRun.config).toEqual({ configurable: { model_id: "gpt-5" }, tags: ["tests"] });
    expect(createdRun.webhook).toBe("https://example.test/webhook");
    expect(createdRun.on_completion).toBe("keep");
    expect(createdRun.on_disconnect).toBe("continue");
    expect(createdRun.if_not_exists).toBe("create");
    expect(createdRun.stream_mode).toEqual(["messages", "values"]);

    const waitResponse = await handleRunRoute(
      new Request(`https://example.test/runs/${createdRun.run_id}/wait`, { method: "GET" }),
      "run-wait",
    );

    expect(waitResponse.status).toBe(200);
    const body = await waitResponse.json();
    expect(body.run.thread_id).toBe(threadId);
    expect(body.run.agent_id).toBe("deepagent");
    expect(body.run.input).toEqual({ task: "summarize" });
    expect(body.run.metadata).toEqual({ trace: "abc" });
    expect(body.run.config).toEqual({ configurable: { model_id: "gpt-5" }, tags: ["tests"] });
    expect(body.run.webhook).toBe("https://example.test/webhook");
    expect(body.run.on_completion).toBe("keep");
    expect(body.run.on_disconnect).toBe("continue");
    expect(body.run.if_not_exists).toBe("create");
    expect(body.run.status).toBe("success");
    expect(body.values).toEqual({ session: "ok" });
    expect(body.messages[0].role).toBe("assistant");

    const fetched = await handleRunRoute(
      new Request(`https://example.test/runs/${body.run.run_id}`, { method: "GET" }),
      "run-get",
    );
    const fetchedBody = await fetched.json();
    expect(fetchedBody.run_id).toBe(body.run.run_id);
    expect(fetchedBody.input).toEqual({ task: "summarize" });
  });

  it("streams spec-style message and values events", async () => {
    const { handleRunRoute } = await import("./runtime.js");
    const response = await handleRunRoute(
      jsonRequest("https://example.test/runs/stream", "POST", {
        input: "hello",
        stream_mode: ["messages", "values"],
      }),
      "runs-stream",
    );

    expect(response.status).toBe(200);
    const text = await readResponseStream(response);
    const frames = parseSseFrames(text);
    expect(frames[0]).toMatchObject({ id: "1", event: "message", data: { role: "assistant", content: "Hello" } });
    expect(frames[1]).toMatchObject({ id: "2", event: "message", data: { role: "assistant", content: " world" } });
    expect(frames[2]).toMatchObject({ id: "3", event: "values", data: { output: "Hello world" } });
    expect(frames.at(-1)).toMatchObject({ event: "end", data: null });
  });

  it("replays persisted stream events from GET /runs/{run_id}/stream", async () => {
    const { handleRunRoute } = await import("./runtime.js");
    const created = await handleRunRoute(
      jsonRequest("https://example.test/runs", "POST", {
        input: "hello",
        stream_mode: ["messages", "values"],
      }),
      "runs-create",
    );

    expect(created.status).toBe(200);
    const run = await created.json();

    const waited = await handleRunRoute(
      new Request(`https://example.test/runs/${run.run_id}/wait`, { method: "GET" }),
      "run-wait",
    );
    expect(waited.status).toBe(200);

    const replay = await handleRunRoute(
      new Request(`https://example.test/runs/${run.run_id}/stream`, { method: "GET" }),
      "run-stream",
    );
    const frames = parseSseFrames(await readResponseStream(replay));
    expect(frames.map((frame) => frame.event)).toEqual(["message", "message", "values", "messages/complete", "end"]);
    expect(frames[0]).toMatchObject({ id: "1", data: { role: "assistant", content: "Hello" } });
    expect(frames[2]).toMatchObject({ id: "3", data: { output: "Hello world" } });
  });

  it("returns spec errors for missing threads, invalid ids, and invalid cancel actions", async () => {
    const { handleRunRoute } = await import("./runtime.js");
    const missingThread = await handleRunRoute(
      jsonRequest("https://example.test/runs/wait", "POST", {
        thread_id: "88888888-8888-4888-8888-888888888888",
        input: "hello",
        if_not_exists: "reject",
      }),
      "runs-wait",
    );
    expect(missingThread.status).toBe(404);

    const invalidRunId = await handleRunRoute(
      new Request("https://example.test/runs/not-a-uuid", { method: "GET" }),
      "run-get",
    );
    expect(invalidRunId.status).toBe(422);

    const created = await handleRunRoute(
      jsonRequest("https://example.test/runs", "POST", { input: "hello" }),
      "runs-create",
    );
    const run = await created.json();
    const cancelled = await handleRunRoute(
      new Request(`https://example.test/runs/${run.run_id}/cancel?action=stop`, { method: "POST" }),
      "run-cancel",
    );
    expect(cancelled.status).toBe(422);
  });

  it("supports runs search and delete routes", async () => {
    const { handleRunRoute } = await import("./runtime.js");
    const threadId = "99999999-9999-4999-8999-999999999999";
    const created = await handleRunRoute(
      jsonRequest("https://example.test/runs", "POST", {
        thread_id: threadId,
        input: "search me",
        if_not_exists: "create",
        metadata: { trace: "searchable" },
      }),
      "runs-create",
    );
    const run = await created.json();

    const waited = await handleRunRoute(
      new Request(`https://example.test/runs/${run.run_id}/wait`, { method: "GET" }),
      "run-wait",
    );
    expect(waited.status).toBe(200);

    const search = await handleRunRoute(
      jsonRequest("https://example.test/runs/search", "POST", {
        thread_id: threadId,
        metadata: { trace: "searchable" },
        status: "success",
        limit: 10,
        offset: 0,
      }),
      "runs-search",
    );
    expect(search.status).toBe(200);
    const searchBody = await search.json();
    expect(searchBody).toHaveLength(1);
    expect(searchBody[0].run_id).toBe(run.run_id);

    const deleted = await handleRunRoute(
      new Request(`https://example.test/runs/${run.run_id}`, { method: "DELETE" }),
      "run-delete",
    );
    expect(deleted.status).toBe(204);

    const missing = await handleRunRoute(
      new Request(`https://example.test/runs/${run.run_id}`, { method: "GET" }),
      "run-get",
    );
    expect(missing.status).toBe(404);
  });

  it("restores existing thread state on rollback cancellation", async () => {
    const { handleRunRoute } = await import("./runtime.js");
    const threadId = "44444444-4444-4444-8444-444444444444";
    mockState.files.set(
      `/home/db/threads/${threadId}.json`,
      JSON.stringify({
        thread_id: threadId,
        created_at: "2026-01-01T00:00:00.000Z",
        updated_at: "2026-01-01T00:00:00.000Z",
        metadata: { title: "Original" },
        status: "idle",
        messages: [{ id: "m0", role: "user", content: "before" }],
      }),
    );
    mockState.files.set(
      `/home/db/messages/${threadId}/m0.json`,
      JSON.stringify({ id: "m0", thread_id: threadId, role: "user", content: "before" }),
    );
    mockState.files.set(`/home/checkpoints/${threadId}.sqlite`, "checkpoint-bytes");

    const waitResponse = await handleRunRoute(
      jsonRequest("https://example.test/runs/wait", "POST", {
        thread_id: threadId,
        input: "mutate thread",
        on_completion: "keep",
        if_not_exists: "reject",
      }),
      "runs-wait",
    );
    expect(waitResponse.status).toBe(200);
    const waitBody = await waitResponse.json();

    const rollback = await handleRunRoute(
      new Request(`https://example.test/runs/${waitBody.run.run_id}/cancel?action=rollback`, { method: "POST" }),
      "run-cancel",
    );
    expect(rollback.status).toBe(204);

    const restoredThread = JSON.parse(mockState.files.get(`/home/db/threads/${threadId}.json`) ?? "{}");
    expect(restoredThread.metadata.title).toBe("Original");
    expect(restoredThread.messages).toHaveLength(1);
    expect(mockState.files.get(`/home/db/messages/${threadId}/m0.json`)).toContain("before");
    expect(mockState.files.get(`/home/checkpoints/${threadId}.sqlite`)).toBe("checkpoint-bytes");
  });
});
