import { resolveServiceWorkerScope } from "@omni/omni-util/service-worker";
import { formatError } from "@omni/omni-util";

const DEFAULT_SW_URL = new URL("./omni-inference.js", import.meta.url).href;

const SW_URL =
  typeof document !== "undefined"
    ? (document.querySelector<HTMLMetaElement>('meta[name="omni-inference-url"]')?.content ?? DEFAULT_SW_URL)
    : DEFAULT_SW_URL;
const READY_CHANNEL = "omni-inference-ready";

type RegisterEnv = {
  navigator: Navigator;
  channel: BroadcastChannel;
  swUrl: string;
};

function absoluteUrl(path: string): string {
  return new URL(path, globalThis.location.href).href;
}

function registrationScriptUrl(registration: ServiceWorkerRegistration): string | null {
  return (
    registration.installing?.scriptURL ?? registration.waiting?.scriptURL ?? registration.active?.scriptURL ?? null
  );
}

function samePathDifferentUrl(left: string, right: string): boolean {
  const leftUrl = new URL(left);
  const rightUrl = new URL(right);
  return leftUrl.pathname === rightUrl.pathname && leftUrl.href !== rightUrl.href;
}

function markReady(channel: BroadcastChannel): void {
  channel.postMessage({ type: "ready" });
}

export async function registerServiceWorker(env: RegisterEnv): Promise<void> {
  const { navigator, channel, swUrl } = env;
  if (!("serviceWorker" in navigator)) {
    return;
  }

  const scope = resolveServiceWorkerScope(swUrl, "inference/");
  const scopeUrl = absoluteUrl(scope);
  const scriptUrl = absoluteUrl(swUrl);

  const registrations = await navigator.serviceWorker.getRegistrations();
  await Promise.all(
    registrations
      .filter((registration) => {
        const registeredScriptUrl = registrationScriptUrl(registration);
        if (!registeredScriptUrl) {
          return registration.scope === scopeUrl;
        }

        return registration.scope === scopeUrl || samePathDifferentUrl(registeredScriptUrl, scriptUrl);
      })
      .filter((registration) => registrationScriptUrl(registration) !== scriptUrl)
      .map((registration) => registration.unregister()),
  );

  const registration = await navigator.serviceWorker.register(swUrl, {
    type: "module",
    scope,
    updateViaCache: "none",
  });

  await registration.update();

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
}

if (typeof navigator !== "undefined" && typeof BroadcastChannel !== "undefined") {
  void registerServiceWorker({
    navigator,
    channel: new BroadcastChannel(READY_CHANNEL),
    swUrl: SW_URL,
  }).catch((error) => {
    console.error(`[omni-inference] registration failed: ${formatError(error)}`);
  });
}
