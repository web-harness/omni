import { describe, expect, it, vi } from "vitest";
import { registerServiceWorker } from "./register";

describe("registerServiceWorker", () => {
  it("posts ready and flips flag when already activated", async () => {
    const postMessage = vi.fn();
    const channel = { postMessage } as unknown as BroadcastChannel;

    const registration = {
      active: { state: "activated" },
      installing: null,
      waiting: null,
      addEventListener: vi.fn(),
    };

    const navigator = {
      serviceWorker: {
        register: vi.fn().mockResolvedValue(registration),
      },
    } as unknown as Navigator;

    const setReadyFlag = vi.fn();

    await registerServiceWorker({
      navigator,
      channel,
      swUrl: "/omni-sw.js",
      setReadyFlag,
    });

    expect((navigator.serviceWorker.register as any).mock.calls[0][0]).toBe("/omni-sw.js");
    expect(setReadyFlag).toHaveBeenCalledTimes(1);
    expect(postMessage).toHaveBeenCalledWith({ type: "ready" });
  });
});
