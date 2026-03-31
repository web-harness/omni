// URL is injected by Dioxus via <meta name="omni-sw-url"> so the hashed asset path is correct
const SW_URL = document.querySelector<HTMLMetaElement>('meta[name="omni-sw-url"]')?.content ?? "/omni-sw.js";
const READY_CHANNEL = "omni-sw-ready";

const channel = new BroadcastChannel(READY_CHANNEL);

async function register(): Promise<void> {
  if (!("serviceWorker" in navigator)) return;

  try {
    const registration = await navigator.serviceWorker.register(SW_URL, {
      type: "module",
      scope: "/",
    });

    const sw = registration.installing ?? registration.waiting ?? registration.active;

    if (sw && sw.state === "activated") {
      channel.postMessage({ type: "ready" });
      return;
    }

    const target = registration.installing ?? registration.waiting;
    if (target) {
      target.addEventListener("statechange", () => {
        if (target.state === "activated") {
          channel.postMessage({ type: "ready" });
        }
      });
    }

    registration.addEventListener("updatefound", () => {
      const newSw = registration.installing;
      if (!newSw) return;
      newSw.addEventListener("statechange", () => {
        if (newSw.state === "activated") {
          channel.postMessage({ type: "update-available" });
        }
      });
    });
  } catch (err) {
    console.error("[omni-sw] registration failed:", err);
  }
}

register();
