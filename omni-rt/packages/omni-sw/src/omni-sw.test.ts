import { describe, expect, it } from "vitest";
import { matchRunRoute } from "./omni-sw";

describe("matchRunRoute", () => {
  it("matches stream endpoint", () => {
    const req = new Request("https://example.test/api/runs/stream", { method: "POST" });
    expect(matchRunRoute(req)).toBe("runs-stream");
  });

  it("matches wait endpoint", () => {
    const req = new Request("https://example.test/api/runs/wait", { method: "POST" });
    expect(matchRunRoute(req)).toBe("runs-wait");
  });

  it("ignores non-POST requests", () => {
    const req = new Request("https://example.test/api/runs/stream", { method: "GET" });
    expect(matchRunRoute(req)).toBeNull();
  });

  it("passes through unrelated paths", () => {
    const req = new Request("https://example.test/api/threads", { method: "POST" });
    expect(matchRunRoute(req)).toBeNull();
  });
});
