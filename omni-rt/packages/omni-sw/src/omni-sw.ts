import { Serwist } from "serwist";
import { handleStoreRoute, matchStoreRoute } from "./store-api.js";

declare const self: ServiceWorkerGlobalScope;

export type RunRoute =
  | "runs-create"
  | "runs-search"
  | "runs-stream"
  | "runs-wait"
  | "run-get"
  | "run-delete"
  | "run-wait"
  | "run-stream"
  | "run-cancel"
  | null;

type RuntimeModule = {
  handleRunRoute(request: Request, route: Exclude<RunRoute, null>): Promise<Response>;
};

let runtimeModulePromise: Promise<RuntimeModule> | null = null;

function loadRuntimeModule(): Promise<RuntimeModule> {
  if (!runtimeModulePromise) {
    runtimeModulePromise = import("./agent/runtime.js") as Promise<RuntimeModule>;
  }
  return runtimeModulePromise;
}

export function matchRunRoute(request: Request): RunRoute {
  const parts = new URL(request.url).pathname.split("/").filter(Boolean);
  if (parts[0] !== "runs") {
    return null;
  }

  if (parts.length === 1 && request.method === "POST") {
    return "runs-create";
  }
  if (parts[1] === "search" && request.method === "POST") {
    return "runs-search";
  }
  if (parts[1] === "stream" && request.method === "POST") {
    return "runs-stream";
  }
  if (parts[1] === "wait" && request.method === "POST") {
    return "runs-wait";
  }
  if (parts.length === 2 && request.method === "GET") {
    return "run-get";
  }
  if (parts.length === 2 && request.method === "DELETE") {
    return "run-delete";
  }
  if (parts[2] === "wait" && request.method === "GET") {
    return "run-wait";
  }
  if (parts[2] === "stream" && request.method === "GET") {
    return "run-stream";
  }
  if (parts[2] === "cancel" && request.method === "POST") {
    return "run-cancel";
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
    const storeRoute = matchStoreRoute(event.request);
    if (storeRoute) {
      event.respondWith(handleStoreRoute(event.request, storeRoute));
      return;
    }

    const route = matchRunRoute(event.request);

    if (route) {
      event.respondWith(loadRuntimeModule().then((runtime) => runtime.handleRunRoute(event.request, route)));
      return;
    }
  });
}

if (typeof self !== "undefined" && "addEventListener" in self) {
  setupServiceWorker(self);
}
