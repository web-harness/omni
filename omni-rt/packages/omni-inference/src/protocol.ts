import type { BrowserInferenceStatus } from "./engine.js";

export type InferenceRoute = "chat-completions" | "completions" | "models-list" | "download-model" | "inference-status";

export type InferenceBridgeRequestMessage = {
  type: "request";
  requestId: string;
  route: InferenceRoute;
  url: string;
  method: string;
  headers: [string, string][];
  body: string | null;
};

export type InferenceBridgeResponseStartMessage = {
  type: "response-start";
  requestId: string;
  status: number;
  statusText: string;
  headers: [string, string][];
};

export type InferenceBridgeResponseChunkMessage = {
  type: "response-chunk";
  requestId: string;
  chunk: string;
};

export type InferenceBridgeResponseEndMessage = {
  type: "response-end";
  requestId: string;
};

export type InferenceBridgeResponseErrorMessage = {
  type: "response-error";
  requestId: string;
  message: string;
};

export type InferenceBridgeResponseMessage =
  | InferenceBridgeResponseStartMessage
  | InferenceBridgeResponseChunkMessage
  | InferenceBridgeResponseEndMessage
  | InferenceBridgeResponseErrorMessage;

export type InferenceBridgeMessage = InferenceBridgeRequestMessage | InferenceBridgeResponseMessage;

export type InferenceWorkerMessage =
  | { type: "status" }
  | { type: "download"; modelId: string }
  | { type: "stop-download"; modelId: string }
  | { type: "delete"; modelId: string };

export type InferenceWorkerResponse =
  | { ok: true; status: BrowserInferenceStatus }
  | { ok: true; accepted: true }
  | { ok: false; message: string };

function isObjectRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object";
}

export function parseInferenceWorkerResponse(value: unknown): InferenceWorkerResponse | null {
  if (!isObjectRecord(value) || typeof value.ok !== "boolean") {
    return null;
  }

  if (!value.ok) {
    return typeof value.message === "string" ? { ok: false, message: value.message } : null;
  }

  if ("status" in value) {
    return { ok: true, status: value.status as BrowserInferenceStatus };
  }

  if (value.accepted === true) {
    return { ok: true, accepted: true };
  }

  return null;
}
