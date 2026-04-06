import { Serwist } from "serwist";
import { formatError } from "@omni/omni-util";
import { getInferenceEngine } from "./engine.js";
import { handleInferenceRoute } from "./openai-handler.js";
import type {
  InferenceBridgeRequestMessage,
  InferenceBridgeResponseMessage,
  InferenceWorkerMessage,
  InferenceWorkerResponse,
} from "./protocol.js";
import { matchInferenceRoute } from "./runtime.js";

declare const self: ServiceWorkerGlobalScope;

const INFERENCE_BRIDGE_CHANNEL = "omni-inference-bridge";

function respondToClient(port: MessagePort | undefined, payload: InferenceWorkerResponse): void {
  port?.postMessage(payload);
}

function postBridgeMessage(channel: BroadcastChannel, payload: InferenceBridgeResponseMessage): void {
  channel.postMessage(payload);
}

async function forwardBridgeResponse(channel: BroadcastChannel, requestId: string, response: Response): Promise<void> {
  postBridgeMessage(channel, {
    type: "response-start",
    requestId,
    status: response.status,
    statusText: response.statusText,
    headers: [...response.headers.entries()],
  });

  if (!response.body) {
    postBridgeMessage(channel, { type: "response-end", requestId });
    return;
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    if (!value) {
      continue;
    }

    postBridgeMessage(channel, {
      type: "response-chunk",
      requestId,
      chunk: decoder.decode(value, { stream: true }),
    });
  }

  const tail = decoder.decode();
  if (tail) {
    postBridgeMessage(channel, {
      type: "response-chunk",
      requestId,
      chunk: tail,
    });
  }

  postBridgeMessage(channel, { type: "response-end", requestId });
}

function setupBridgeChannel(scope: ServiceWorkerGlobalScope): void {
  if (typeof BroadcastChannel === "undefined") {
    return;
  }

  const channel = new BroadcastChannel(INFERENCE_BRIDGE_CHANNEL);
  channel.addEventListener("message", (event: MessageEvent<InferenceBridgeRequestMessage>) => {
    const message = event.data;
    if (!message || message.type !== "request") {
      return;
    }

    const request = new Request(message.url, {
      method: message.method,
      headers: message.headers,
      body: message.body,
    });

    scope.waitUntil(
      handleInferenceRoute(request, message.route)
        .then((response) => forwardBridgeResponse(channel, message.requestId, response))
        .catch((error) => {
          postBridgeMessage(channel, {
            type: "response-error",
            requestId: message.requestId,
            message: formatError(error),
          });
        }),
    );
  });
}

function setupClientMessages(scope: ServiceWorkerGlobalScope): void {
  scope.addEventListener("message", (event: ExtendableMessageEvent) => {
    const message = event.data as InferenceWorkerMessage | undefined;
    const port = event.ports[0];
    if (!message || !port) {
      return;
    }

    if (message.type === "status") {
      event.waitUntil(
        getInferenceEngine()
          .getStatus()
          .then((status) => respondToClient(port, { ok: true, status }))
          .catch((error) => {
            respondToClient(port, { ok: false, message: formatError(error) });
          }),
      );
      return;
    }

    if (message.type === "download") {
      respondToClient(port, { ok: true, accepted: true });
      event.waitUntil(
        getInferenceEngine()
          .downloadModel(message.modelId)
          .catch(() => {}),
      );
      return;
    }

    if (message.type === "stop-download") {
      event.waitUntil(
        getInferenceEngine()
          .stopDownload(message.modelId)
          .then(() => respondToClient(port, { ok: true, accepted: true }))
          .catch((error) => {
            respondToClient(port, { ok: false, message: formatError(error) });
          }),
      );
      return;
    }

    if (message.type === "delete") {
      event.waitUntil(
        getInferenceEngine()
          .deleteModel(message.modelId)
          .then(() => respondToClient(port, { ok: true, accepted: true }))
          .catch((error) => {
            respondToClient(port, { ok: false, message: formatError(error) });
          }),
      );
    }
  });
}

export function setupInferenceServiceWorker(scope: ServiceWorkerGlobalScope): void {
  const serwist = new Serwist({
    precacheEntries: scope.__SW_MANIFEST ?? [],
    skipWaiting: true,
    clientsClaim: true,
    navigationPreload: false,
  });

  serwist.addEventListeners();
  setupBridgeChannel(scope);
  setupClientMessages(scope);

  scope.addEventListener("fetch", (event: FetchEvent) => {
    const route = matchInferenceRoute(event.request);
    if (!route) {
      return;
    }

    event.respondWith(handleInferenceRoute(event.request, route));
  });
}

function isInferenceServiceWorkerScope(scope: ServiceWorkerGlobalScope): boolean {
  return scope.location.pathname.endsWith("/omni-inference.js");
}

if (typeof self !== "undefined" && "addEventListener" in self && isInferenceServiceWorkerScope(self)) {
  setupInferenceServiceWorker(self);
}
