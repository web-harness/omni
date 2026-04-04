import { describe, expect, it, vi } from "vitest";
import { registerServiceWorker } from "./register";

describe("registerServiceWorker", () => {
  it("posts ready and flips flag when already activated", async () => {
    const postMessage = vi.fn();
    const channel = { postMessage } as unknown as BroadcastChannel;
    const register = vi.fn();

    const registration = {
      active: { state: "activated" },
      installing: null,
      waiting: null,
      addEventListener: vi.fn(),
    };
    register.mockResolvedValue(registration);

    const navigator = {
      serviceWorker: {
        register,
      },
    } as unknown as Navigator;

    const setReadyFlag = vi.fn();

    await registerServiceWorker({
      navigator,
      channel,
      swUrl: "/omni-sw.js",
      setReadyFlag,
    });

    expect(register.mock.calls[0][0]).toBe("/omni-sw.js");
    expect(register.mock.calls[0][1]).toEqual({
      type: "module",
      scope: "/",
    });
    expect(setReadyFlag).toHaveBeenCalledTimes(1);
    expect(postMessage).toHaveBeenCalledWith({ type: "ready" });
  });

  it("registers with the service worker directory as scope", async () => {
    const channel = { postMessage: vi.fn() } as unknown as BroadcastChannel;
    const register = vi.fn();

    const registration = {
      active: { state: "activated" },
      installing: null,
      waiting: null,
      addEventListener: vi.fn(),
    };
    register.mockResolvedValue(registration);

    const navigator = {
      serviceWorker: {
        register,
      },
    } as unknown as Navigator;

    await registerServiceWorker({
      navigator,
      channel,
      swUrl: "/omni/app/omni-sw.js",
      setReadyFlag: vi.fn(),
    });

    expect(register.mock.calls[0][1]).toEqual({
      type: "module",
      scope: "/omni/app/",
    });
  });
});
