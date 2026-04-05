import { resolveServiceWorkerScope } from "@omni/omni-util/service-worker";
import { parseInferenceWorkerResponse, type InferenceWorkerMessage, type InferenceWorkerResponse } from "./protocol.js";

const WORKER_RESPONSE_TIMEOUT_MS = 10_000;

function inferenceWorkerUrl(): string {
  return document.querySelector<HTMLMetaElement>('meta[name="omni-inference-url"]')?.content ?? "/omni-inference.js";
}

function inferenceScope(): string {
  return resolveServiceWorkerScope(inferenceWorkerUrl(), "inference/");
}

async function getInferenceWorker(): Promise<ServiceWorker> {
  if (!("serviceWorker" in navigator)) {
    throw new Error("Service worker API is unavailable");
  }

  const registration = await navigator.serviceWorker.getRegistration(inferenceScope());
  const worker = registration?.active ?? registration?.waiting ?? registration?.installing;
  if (!worker) {
    throw new Error("Inference service worker is not active");
  }

  return worker;
}

function requestInferenceWorker(message: InferenceWorkerMessage): Promise<InferenceWorkerResponse> {
  return getInferenceWorker().then(
    (worker) =>
      new Promise((resolve, reject) => {
        const channel = new MessageChannel();
        const timeoutId = globalThis.setTimeout(() => {
          channel.port1.close();
          channel.port2.close();
          reject(new Error("Inference worker timed out"));
        }, WORKER_RESPONSE_TIMEOUT_MS);

        const finish = (callback: () => void): void => {
          globalThis.clearTimeout(timeoutId);
          channel.port1.close();
          channel.port2.close();
          callback();
        };

        channel.port1.onmessage = (event) => {
          finish(() => {
            const response = parseInferenceWorkerResponse(event.data);
            if (!response) {
              reject(new Error("Inference worker returned an invalid response payload"));
              return;
            }
            resolve(response);
          });
        };

        channel.port1.onmessageerror = () => {
          finish(() => {
            reject(new Error("Inference worker returned an unreadable response"));
          });
        };

        try {
          worker.postMessage(message, [channel.port2]);
        } catch (error) {
          finish(() => {
            reject(error instanceof Error ? error : new Error(String(error)));
          });
        }
      }),
  );
}

export async function getBrowserInferenceStatus(): Promise<unknown> {
  const response = await requestInferenceWorker({ type: "status" });
  if (!response.ok) {
    throw new Error(response.message);
  }
  if (!("status" in response)) {
    throw new Error("Inference worker returned an invalid status payload");
  }
  return response.status;
}

export async function startBrowserModelDownload(modelId: string): Promise<void> {
  const response = await requestInferenceWorker({ type: "download", modelId });
  if (!response.ok) {
    throw new Error(response.message);
  }
  if (!("accepted" in response)) {
    throw new Error("Inference worker returned an invalid download response payload");
  }
}

export async function stopBrowserModelDownload(modelId: string): Promise<void> {
  const response = await requestInferenceWorker({ type: "stop-download", modelId });
  if (!response.ok) {
    throw new Error(response.message);
  }
  if (!("accepted" in response)) {
    throw new Error("Inference worker returned an invalid stop-download response payload");
  }
}

export async function deleteBrowserModel(modelId: string): Promise<void> {
  const response = await requestInferenceWorker({ type: "delete", modelId });
  if (!response.ok) {
    throw new Error(response.message);
  }
  if (!("accepted" in response)) {
    throw new Error("Inference worker returned an invalid delete response payload");
  }
}
