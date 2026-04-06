import { describe, expect, it, vi } from "vitest";

const DEFAULT_MODEL_ITEM_PATH = "/home/store/config/default_model.json";

vi.mock("./zenfs.js", () => ({
  exists: vi.fn(),
  init: vi.fn(),
  mkdir: vi.fn(),
  readFile: vi.fn(),
  readdir: vi.fn(),
  rm: vi.fn(),
  writeFile: vi.fn(),
}));

vi.mock("./deepagents.js", () => ({
  createThread: vi.fn(),
  deleteApiKey: vi.fn(),
  deleteDefaultModel: vi.fn(),
  deleteThread: vi.fn(),
  deleteThreadMessages: vi.fn(),
  getApiKey: vi.fn(),
  getDefaultModel: vi.fn(),
  getStoredDefaultModel: vi.fn(),
  listMessages: vi.fn(),
  listThreads: vi.fn(),
  saveMessage: vi.fn(),
  setApiKey: vi.fn(),
  setDefaultModel: vi.fn(),
  setThreadStatus: vi.fn(),
}));

vi.mock("./store-mocks.js", () => ({
  DEFAULT_WORKSPACE_ORDER: [],
  MOCK_THREAD_IDS: {},
  SCAFFOLD_FILES: [],
  getMockThreadFiles: vi.fn(() => []),
  getMockToolCalls: vi.fn(() => []),
  getMockToolResults: vi.fn(() => []),
  getMockWorkspaceFiles: vi.fn(() => ({})),
  seedThreads: vi.fn(() => []),
}));

describe("store-data thread titles", () => {
  it("derives a readable title from the first user message when the current title is a placeholder", async () => {
    const { deriveThreadTitle } = await import("./store-data.js");

    const title = deriveThreadTitle(
      [
        { role: "assistant", content: "ignored" },
        { role: "user", content: "Investigate why Storybook assets 404 on pages deploy" },
      ],
      "New Thread",
    );

    expect(title).toBe("Investigate why Storybook assets 404 on pages...");
  });

  it("keeps an existing non-placeholder title", async () => {
    const { deriveThreadTitle } = await import("./store-data.js");

    const title = deriveThreadTitle([{ role: "user", content: "Rename me" }], "Existing Title");

    expect(title).toBe("Existing Title");
  });

  it("reads the current stored default model without falling back to legacy storage", async () => {
    const zenfs = await import("./zenfs.js");
    const deepagents = await import("./deepagents.js");
    vi.mocked(zenfs.readFile).mockImplementation(async (path: string) => {
      if (path === DEFAULT_MODEL_ITEM_PATH) {
        return new TextEncoder().encode(JSON.stringify({ value: { model_id: "lfm2-1.2b" } }));
      }
      throw new Error("missing");
    });
    vi.mocked(deepagents.getStoredDefaultModel).mockResolvedValue("deepseek-r1");

    const { getStoredDefaultModel } = await import("./store-data.js");

    await expect(getStoredDefaultModel()).resolves.toBe("lfm2-1.2b");
    expect(deepagents.getStoredDefaultModel).not.toHaveBeenCalled();
  });

  it("keeps the legacy fallback only for migration reads", async () => {
    const zenfs = await import("./zenfs.js");
    const deepagents = await import("./deepagents.js");
    vi.mocked(zenfs.readFile).mockRejectedValue(new Error("missing"));
    vi.mocked(deepagents.getStoredDefaultModel).mockResolvedValue("deepseek-r1");

    const { getMigratableStoredDefaultModel, getStoredDefaultModel } = await import("./store-data.js");

    await expect(getStoredDefaultModel()).resolves.toBeNull();
    await expect(getMigratableStoredDefaultModel()).resolves.toBe("deepseek-r1");
  });
});
