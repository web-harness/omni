import { Serwist } from "serwist";
import { formatError } from "@omni/omni-util";
import {
  matchInferenceRoute,
  type InferenceBridgeMessage,
  type InferenceBridgeRequestMessage,
  type InferenceRoute,
} from "@omni/omni-inference/runtime";
import { getScopedRequestPathParts } from "@omni/omni-util/service-worker";
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

const ROUTE_ROOTS = new Set(["agents", "threads", "store", "x", "runs"]);
const INFERENCE_BRIDGE_CHANNEL = "omni-inference-bridge";
const BRIDGE_IDLE_TIMEOUT_MS = 30_000;
const STREAM_CONTENT_TYPE = "text/event-stream";
const NAVIGATION_ERROR_HTML =
  '<!doctype html><html><head><meta charset="utf-8"><title>Application Unavailable</title></head><body><h1>Application Unavailable</h1><p>The application shell could not be loaded.</p></body></html>';
const textEncoder = new TextEncoder();

let bridgeCounter = 0;

function loadRuntimeModule(): Promise<RuntimeModule> {
  if (!runtimeModulePromise) {
    runtimeModulePromise = import("./agent/runtime.js") as Promise<RuntimeModule>;
  }
  return runtimeModulePromise;
}

function inferenceErrorResponse(error: unknown): Response {
  const message = formatError(error);
  return Response.json({ error: { message } }, { status: 500 });
}

function nextBridgeRequestId(): string {
  bridgeCounter += 1;
  return `inference-${Date.now()}-${bridgeCounter}`;
}

async function handleInferenceViaBridge(request: Request, route: InferenceRoute): Promise<Response> {
  if (typeof BroadcastChannel === "undefined") {
    throw new Error("BroadcastChannel is unavailable in this service worker");
  }

  const requestId = nextBridgeRequestId();
  const channel = new BroadcastChannel(INFERENCE_BRIDGE_CHANNEL);
  const body = request.method === "GET" || request.method === "HEAD" ? null : await request.clone().text();

  return new Promise<Response>((resolve, reject) => {
    let started = false;
    let resolved = false;
    let responseInit: ResponseInit | null = null;
    let buffered = "";
    let streamController: ReadableStreamDefaultController<Uint8Array> | null = null;
    let timeoutSignal: AbortSignal | null = null;

    const cleanup = () => {
      timeoutSignal?.removeEventListener("abort", onTimeout);
      channel.close();
    };

    const fail = (error: unknown) => {
      const normalized = error instanceof Error ? error : new Error(String(error));
      cleanup();
      if (streamController) {
        streamController.error(normalized);
        return;
      }
      if (!resolved) {
        reject(normalized);
      }
    };

    const onTimeout = () => {
      fail(new Error("Inference bridge timed out"));
    };

    const armTimeout = () => {
      timeoutSignal?.removeEventListener("abort", onTimeout);
      timeoutSignal = AbortSignal.timeout(BRIDGE_IDLE_TIMEOUT_MS);
      timeoutSignal.addEventListener("abort", onTimeout, { once: true });
    };

    armTimeout();

    channel.addEventListener("message", (event: MessageEvent<InferenceBridgeMessage>) => {
      const message = event.data;
      if (!message || message.requestId !== requestId || message.type === "request") {
        return;
      }

      armTimeout();

      if (message.type === "response-error") {
        fail(new Error(message.message));
        return;
      }

      if (message.type === "response-start") {
        started = true;
        responseInit = {
          status: message.status,
          statusText: message.statusText,
          headers: message.headers,
        };

        const contentType = new Headers(message.headers).get("content-type") ?? "";
        if (contentType.includes(STREAM_CONTENT_TYPE)) {
          const stream = new ReadableStream<Uint8Array>({
            start(controller) {
              streamController = controller;
            },
          });
          resolved = true;
          resolve(new Response(stream, responseInit));
        }
        return;
      }

      if (message.type === "response-chunk") {
        if (streamController) {
          streamController.enqueue(textEncoder.encode(message.chunk));
        } else {
          buffered += message.chunk;
        }
        return;
      }

      if (message.type === "response-end") {
        cleanup();
        if (streamController && responseInit) {
          streamController.close();
          return;
        }
        if (started && responseInit) {
          resolved = true;
          resolve(new Response(buffered, responseInit));
          return;
        }
        fail(new Error("Inference bridge ended before sending a response"));
      }
    });

    channel.start?.();
    channel.postMessage({
      type: "request",
      requestId,
      route,
      url: request.url,
      method: request.method,
      headers: [...request.headers.entries()],
      body,
    } satisfies InferenceBridgeRequestMessage);
  });
}

export function matchRunRoute(request: Request): RunRoute {
  const parts = getScopedRequestPathParts(request, ROUTE_ROOTS);
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

  async function handleNavigationRequest(request: Request): Promise<Response> {
    try {
      const response = await fetch(request);
      const headers = new Headers(response.headers);
      headers.set("Cross-Origin-Embedder-Policy", "require-corp");
      headers.set("Cross-Origin-Opener-Policy", "same-origin");

      return new Response(response.body, {
        status: response.status,
        statusText: response.statusText,
        headers,
      });
    } catch {
      return new Response(NAVIGATION_ERROR_HTML, {
        status: 503,
        statusText: "Service Unavailable",
        headers: {
          "content-type": "text/html; charset=utf-8",
          "Cross-Origin-Embedder-Policy": "require-corp",
          "Cross-Origin-Opener-Policy": "same-origin",
        },
      });
    }
  }

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

    const inferenceRoute = matchInferenceRoute(event.request);
    if (inferenceRoute) {
      event.respondWith(
        Promise.resolve()
          .then(() => handleInferenceViaBridge(event.request, inferenceRoute))
          .catch((error) => inferenceErrorResponse(error)),
      );
      return;
    }

    if (event.request.mode === "navigate") {
      event.respondWith(handleNavigationRequest(event.request));
      return;
    }
  });
}

if (typeof self !== "undefined" && "addEventListener" in self) {
  setupServiceWorker(self);
}
