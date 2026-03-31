// URL is injected by Dioxus via <meta name="omni-sw-url"> so the hashed asset path is correct
const SW_URL =
  typeof document !== "undefined"
    ? (document.querySelector<HTMLMetaElement>('meta[name="omni-sw-url"]')?.content ?? "/omni-sw.js")
    : "/omni-sw.js";
const READY_CHANNEL = "omni-sw-ready";

type RegisterEnv = {
  navigator: Navigator;
  channel: BroadcastChannel;
  swUrl: string;
  setReadyFlag: () => void;
};

function markReady(channel: BroadcastChannel, setReadyFlag: () => void): void {
  setReadyFlag();
  channel.postMessage({ type: "ready" });
}

export async function registerServiceWorker(env: RegisterEnv): Promise<void> {
  const { navigator, channel, swUrl, setReadyFlag } = env;
  if (!("serviceWorker" in navigator)) return;

  try {
    const registration = await navigator.serviceWorker.register(swUrl, {
      type: "module",
      scope: "/",
    });

    const sw = registration.installing ?? registration.waiting ?? registration.active;

    if (sw && sw.state === "activated") {
      markReady(channel, setReadyFlag);
      return;
    }

    const target = registration.installing ?? registration.waiting;
    if (target) {
      target.addEventListener("statechange", () => {
        if (target.state === "activated") {
          markReady(channel, setReadyFlag);
        }
      });
    }

    registration.addEventListener("updatefound", () => {
      const newSw = registration.installing;
      if (!newSw) return;
      newSw.addEventListener("statechange", () => {
        if (newSw.state === "activated") {
          setReadyFlag();
          channel.postMessage({ type: "update-available" });
        }
      });
    });
  } catch (err) {
    console.error("[omni-sw] registration failed:", err);
  }
}

if (typeof navigator !== "undefined" && typeof BroadcastChannel !== "undefined") {
  const channel = new BroadcastChannel(READY_CHANNEL);
  registerServiceWorker({
    navigator,
    channel,
    swUrl: SW_URL,
    setReadyFlag: () => {
      (globalThis as Record<string, unknown>).__omni_sw_ready = true;
    },
  });
}
