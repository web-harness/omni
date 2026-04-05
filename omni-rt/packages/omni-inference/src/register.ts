import { resolveServiceWorkerScope } from "@omni/omni-util/service-worker";

const SW_URL =
  typeof document !== "undefined"
    ? (document.querySelector<HTMLMetaElement>('meta[name="omni-inference-url"]')?.content ?? "/omni-inference.js")
    : "/omni-inference.js";
const READY_CHANNEL = "omni-inference-ready";

type RegisterEnv = {
  navigator: Navigator;
  channel: BroadcastChannel;
  swUrl: string;
};

function markReady(channel: BroadcastChannel): void {
  channel.postMessage({ type: "ready" });
}

export async function registerServiceWorker(env: RegisterEnv): Promise<void> {
  const { navigator, channel, swUrl } = env;
  if (!("serviceWorker" in navigator)) {
    return;
  }

  try {
    const registration = await navigator.serviceWorker.register(swUrl, {
      type: "module",
      scope: resolveServiceWorkerScope(swUrl, "inference/"),
    });

    void navigator.storage?.persist?.();

    const sw = registration.installing ?? registration.waiting ?? registration.active;
    if (sw?.state === "activated") {
      markReady(channel);
      return;
    }

    const target = registration.installing ?? registration.waiting;
    if (target) {
      target.addEventListener("statechange", () => {
        if (target.state === "activated") {
          markReady(channel);
        }
      });
    }
  } catch (error) {
    const details = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
    console.error(`[omni-inference] registration failed: ${details}`);
  }
}

if (typeof navigator !== "undefined" && typeof BroadcastChannel !== "undefined") {
  void registerServiceWorker({
    navigator,
    channel: new BroadcastChannel(READY_CHANNEL),
    swUrl: SW_URL,
  });
}
