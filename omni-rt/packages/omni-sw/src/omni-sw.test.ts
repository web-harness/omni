import { describe, expect, it } from "vitest";
import { matchRunRoute } from "./omni-sw";
import { matchStoreRoute } from "./store-api";

describe("matchRunRoute", () => {
  it("matches create endpoint", () => {
    const req = new Request("https://example.test/runs", { method: "POST" });
    expect(matchRunRoute(req)).toBe("runs-create");
  });

  it("matches search endpoint", () => {
    const req = new Request("https://example.test/runs/search", { method: "POST" });
    expect(matchRunRoute(req)).toBe("runs-search");
  });

  it("matches stream endpoint", () => {
    const req = new Request("https://example.test/runs/stream", { method: "POST" });
    expect(matchRunRoute(req)).toBe("runs-stream");
  });

  it("matches GitHub Pages-prefixed run endpoints", () => {
    const req = new Request("https://example.test/omni/app/runs/stream", { method: "POST" });
    expect(matchRunRoute(req)).toBe("runs-stream");
  });

  it("matches wait endpoint", () => {
    const req = new Request("https://example.test/runs/wait", { method: "POST" });
    expect(matchRunRoute(req)).toBe("runs-wait");
  });

  it("matches run id endpoints", () => {
    expect(matchRunRoute(new Request("https://example.test/runs/123", { method: "GET" }))).toBe("run-get");
    expect(matchRunRoute(new Request("https://example.test/runs/123", { method: "DELETE" }))).toBe("run-delete");
    expect(matchRunRoute(new Request("https://example.test/runs/123/wait", { method: "GET" }))).toBe("run-wait");
    expect(matchRunRoute(new Request("https://example.test/runs/123/stream", { method: "GET" }))).toBe("run-stream");
    expect(matchRunRoute(new Request("https://example.test/runs/123/cancel", { method: "POST" }))).toBe("run-cancel");
  });

  it("ignores non-POST requests", () => {
    const req = new Request("https://example.test/runs/stream", { method: "PUT" });
    expect(matchRunRoute(req)).toBeNull();
  });

  it("passes through unrelated paths", () => {
    const req = new Request("https://example.test/threads", { method: "POST" });
    expect(matchRunRoute(req)).toBeNull();
  });

  it("matches GitHub Pages-prefixed store endpoints", () => {
    expect(matchStoreRoute(new Request("https://example.test/omni/app/x/bootstrap", { method: "GET" }))).toBe(
      "x-bootstrap",
    );
    expect(
      matchStoreRoute(
        new Request("https://example.test/omni/app/x/files?workspace=%2Fhome%2Fworkspace&", { method: "GET" }),
      ),
    ).toBe("x-files");
  });
});
