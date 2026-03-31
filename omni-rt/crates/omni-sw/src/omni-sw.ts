import { Serwist } from "serwist";

declare const self: ServiceWorkerGlobalScope;

export type RunRoute = "runs-stream" | "runs-wait" | null;

type RuntimeModule = {
  handleRunStream(request: Request): Promise<Response>;
  handleRunWait(request: Request): Promise<Response>;
};

let runtimeModulePromise: Promise<RuntimeModule> | null = null;

function loadRuntimeModule(): Promise<RuntimeModule> {
  if (!runtimeModulePromise) {
    runtimeModulePromise = import("./agent/runtime.js") as Promise<RuntimeModule>;
  }
  return runtimeModulePromise;
}

export function matchRunRoute(request: Request): RunRoute {
  const url = new URL(request.url);
  if (url.pathname === "/api/runs/stream" && request.method === "POST") {
    return "runs-stream";
  }
  if (url.pathname === "/api/runs/wait" && request.method === "POST") {
    return "runs-wait";
  }
  return null;
}

export function setupServiceWorker(scope: ServiceWorkerGlobalScope): void {
  const serwist = new Serwist({
    precacheEntries: scope.__SW_MANIFEST ?? [],
    skipWaiting: true,
    clientsClaim: true,
    navigationPreload: false,
  });

  serwist.addEventListeners();

  scope.addEventListener("fetch", (event: FetchEvent) => {
    const route = matchRunRoute(event.request);

    if (route === "runs-stream") {
      event.respondWith(loadRuntimeModule().then((runtime) => runtime.handleRunStream(event.request)));
      return;
    }

    if (route === "runs-wait") {
      event.respondWith(loadRuntimeModule().then((runtime) => runtime.handleRunWait(event.request)));
      return;
    }
  });
}

if (typeof self !== "undefined" && "addEventListener" in self) {
  setupServiceWorker(self);
}
