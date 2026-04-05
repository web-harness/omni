import { Serwist } from "serwist";
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

type InferenceRoute = "chat-completions" | "models-list" | "download-model" | "inference-status";

type BridgeRequestMessage = {
  type: "request";
  requestId: string;
  route: InferenceRoute;
  url: string;
  method: string;
  headers: [string, string][];
  body: string | null;
};

type BridgeResponseStartMessage = {
  type: "response-start";
  requestId: string;
  status: number;
  statusText: string;
  headers: [string, string][];
};

type BridgeResponseChunkMessage = {
  type: "response-chunk";
  requestId: string;
  chunk: string;
};

type BridgeResponseEndMessage = {
  type: "response-end";
  requestId: string;
};

type BridgeResponseErrorMessage = {
  type: "response-error";
  requestId: string;
  message: string;
};

type BridgeMessage =
  | BridgeRequestMessage
  | BridgeResponseStartMessage
  | BridgeResponseChunkMessage
  | BridgeResponseEndMessage
  | BridgeResponseErrorMessage;

let runtimeModulePromise: Promise<RuntimeModule> | null = null;

const ROUTE_ROOTS = new Set(["agents", "threads", "store", "x", "runs"]);
const INFERENCE_ROUTE_ROOTS = new Set(["v1", "inference"]);
const INFERENCE_BRIDGE_CHANNEL = "omni-inference-bridge";

let bridgeCounter = 0;

function loadRuntimeModule(): Promise<RuntimeModule> {
  if (!runtimeModulePromise) {
    runtimeModulePromise = import("./agent/runtime.js") as Promise<RuntimeModule>;
  }
  return runtimeModulePromise;
}

function inferenceErrorResponse(error: unknown): Response {
  const message = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
  return Response.json({ error: { message } }, { status: 500 });
}

function nextBridgeRequestId(): string {
  bridgeCounter += 1;
  return `inference-${Date.now()}-${bridgeCounter}`;
}

function matchInferenceRoute(request: Request): InferenceRoute | null {
  const parts = getScopedRequestPathParts(request, INFERENCE_ROUTE_ROOTS);
  if (parts[0] === "v1" && parts[1] === "chat" && parts[2] === "completions" && request.method === "POST") {
    return "chat-completions";
  }
  if (parts[0] === "v1" && parts[1] === "models" && request.method === "GET") {
    return "models-list";
  }
  if (parts[0] === "inference" && parts[1] === "download" && request.method === "POST") {
    return "download-model";
  }
  if (parts[0] === "inference" && parts[1] === "status" && request.method === "GET") {
    return "inference-status";
  }
  return null;
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
    let responseInit: ResponseInit | null = null;
    let buffered = "";
    let streamController: ReadableStreamDefaultController<Uint8Array> | null = null;

    const cleanup = () => {
      channel.close();
    };

    const fail = (error: unknown) => {
      cleanup();
      reject(error);
    };

    channel.addEventListener("message", (event: MessageEvent<BridgeMessage>) => {
      const message = event.data;
      if (!message || message.requestId !== requestId || message.type === "request") {
        return;
      }

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
        if (contentType.includes("text/event-stream")) {
          const stream = new ReadableStream<Uint8Array>({
            start(controller) {
              streamController = controller;
            },
          });
          resolve(new Response(stream, responseInit));
        }
        return;
      }

      if (message.type === "response-chunk") {
        if (streamController) {
          streamController.enqueue(new TextEncoder().encode(message.chunk));
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
    } satisfies BridgeRequestMessage);
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
    const response = await fetch(request);
    const headers = new Headers(response.headers);
    headers.set("Cross-Origin-Embedder-Policy", "require-corp");
    headers.set("Cross-Origin-Opener-Policy", "same-origin");

    return new Response(response.body, {
      status: response.status,
      statusText: response.statusText,
      headers,
    });
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

    const inferenceParts = getScopedRequestPathParts(event.request, INFERENCE_ROUTE_ROOTS);
    if (inferenceParts.length > 0) {
      event.respondWith(
        Promise.resolve()
          .then(async () => {
            const inferenceRoute = matchInferenceRoute(event.request);
            if (inferenceRoute) {
              return handleInferenceViaBridge(event.request, inferenceRoute);
            }

            return fetch(event.request);
          })
          .catch((error) => inferenceErrorResponse(error)),
      );
      return;
    }

    if (event.request.mode === "navigate") {
      event.respondWith(handleNavigationRequest(event.request));
      return;
    }

    event.respondWith(fetch(event.request));
  });
}

if (typeof self !== "undefined" && "addEventListener" in self) {
  setupServiceWorker(self);
}
