import { Serwist } from "serwist";
import { handleRunStream, handleRunWait } from "./agent/runtime.js";

declare const self: ServiceWorkerGlobalScope;

export type RunRoute = "runs-stream" | "runs-wait" | null;

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
      event.respondWith(handleRunStream(event.request));
      return;
    }

    if (route === "runs-wait") {
      event.respondWith(handleRunWait(event.request));
      return;
    }
  });
}

if (typeof self !== "undefined" && "addEventListener" in self) {
  setupServiceWorker(self);
}
