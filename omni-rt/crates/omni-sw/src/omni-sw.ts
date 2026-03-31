import { Serwist } from "serwist";
import { handleRunStream, handleRunWait } from "./agent/runtime.js";

declare const self: ServiceWorkerGlobalScope;

const serwist = new Serwist({
  precacheEntries: self.__SW_MANIFEST ?? [],
  skipWaiting: true,
  clientsClaim: true,
  navigationPreload: false,
});

serwist.addEventListeners();

self.addEventListener("fetch", (event: FetchEvent) => {
  const url = new URL(event.request.url);

  if (url.pathname === "/api/runs/stream" && event.request.method === "POST") {
    event.respondWith(handleRunStream(event.request));
    return;
  }

  if (url.pathname === "/api/runs/wait" && event.request.method === "POST") {
    event.respondWith(handleRunWait(event.request));
    return;
  }
});
