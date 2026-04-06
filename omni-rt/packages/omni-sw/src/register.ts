import { resolveServiceWorkerScope } from "@omni/omni-util/service-worker";

import { buildBootstrap } from "./store-data.js";

// URL is injected by Dioxus via <meta name="omni-sw-url"> so the hashed asset path is correct
const SW_URL =
  typeof document !== "undefined"
    ? (document.querySelector<HTMLMetaElement>('meta[name="omni-sw-url"]')?.content ?? "/omni-sw.js")
    : "/omni-sw.js";
const READY_CHANNEL = "omni-sw-ready";
const FIRST_ACTIVATION_RELOAD_KEY = "__omni_sw_first_activation_reload";
let ready = false;
let readyWaiters: Array<() => void> = [];

type RegisterEnv = {
  navigator: Navigator;
  channel: BroadcastChannel;
  swUrl: string;
  setReadyFlag: () => void;
  reloadPage: () => void;
};

function markReady(channel: BroadcastChannel, setReadyFlag: () => void): void {
  ready = true;
  for (const resolve of readyWaiters) {
    resolve();
  }
  readyWaiters = [];
  setReadyFlag();
  channel.postMessage({ type: "ready" });
}

export function waitForServiceWorkerReady(): Promise<void> {
  if (ready || (globalThis as Record<string, unknown>).__omni_sw_ready === true) {
    return Promise.resolve();
  }

  return new Promise((resolve) => {
    readyWaiters.push(resolve);
  });
}

export async function loadBootstrapPayloadJson(): Promise<string> {
  return JSON.stringify(await buildBootstrap());
}

function shouldReloadOnFirstActivation(hadController: boolean): boolean {
  if (hadController || typeof sessionStorage === "undefined") {
    return false;
  }

  if (sessionStorage.getItem(FIRST_ACTIVATION_RELOAD_KEY) === "done") {
    return false;
  }

  sessionStorage.setItem(FIRST_ACTIVATION_RELOAD_KEY, "done");
  return true;
}

function clearReloadFlag(): void {
  if (typeof sessionStorage === "undefined") {
    return;
  }
  sessionStorage.removeItem(FIRST_ACTIVATION_RELOAD_KEY);
}

export async function registerServiceWorker(env: RegisterEnv): Promise<void> {
  const { navigator, channel, swUrl, setReadyFlag, reloadPage } = env;
  if (!("serviceWorker" in navigator)) {
    return;
  }

  const hadController = Boolean(navigator.serviceWorker.controller);
  if (hadController) {
    clearReloadFlag();
  }

  try {
    const registration = await navigator.serviceWorker.register(swUrl, {
      type: "module",
      scope: resolveServiceWorkerScope(swUrl),
    });

    const sw = registration.installing ?? registration.waiting ?? registration.active;

    if (sw && sw.state === "activated") {
      if (shouldReloadOnFirstActivation(hadController)) {
        reloadPage();
        return;
      }
      markReady(channel, setReadyFlag);
      return;
    }

    const target = registration.installing ?? registration.waiting;
    if (target) {
      target.addEventListener("statechange", () => {
        if (target.state === "activated") {
          if (shouldReloadOnFirstActivation(hadController)) {
            reloadPage();
            return;
          }
          markReady(channel, setReadyFlag);
        }
      });
    }

    registration.addEventListener("updatefound", () => {
      const newSw = registration.installing;
      if (!newSw) return;
      newSw.addEventListener("statechange", () => {
        if (newSw.state === "activated") {
          if (shouldReloadOnFirstActivation(hadController)) {
            reloadPage();
            return;
          }
          setReadyFlag();
          channel.postMessage({ type: "update-available" });
        }
      });
    });
  } catch (err) {
    const details =
      err instanceof Error ? `${err.name}: ${err.message}${err.stack ? `\n${err.stack}` : ""}` : String(err);
    console.error(`[omni-sw] registration failed: ${details}`);
  }
}

if (typeof navigator !== "undefined" && typeof BroadcastChannel !== "undefined") {
  const channel = new BroadcastChannel(READY_CHANNEL);
  registerServiceWorker({
    navigator,
    channel,
    swUrl: SW_URL,
    reloadPage: () => location.reload(),
    setReadyFlag: () => {
      (globalThis as Record<string, unknown>).__omni_sw_ready = true;
    },
  });
}
