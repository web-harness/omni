import { getScopedRequestPathParts } from "@omni/omni-util/service-worker";
import { handleInferenceRoute as handleRuntimeInferenceRoute } from "./openai-handler.js";
import type { InferenceRoute } from "./protocol.js";

export type MatchedInferenceRoute = InferenceRoute | null;

const ROUTE_ROOTS = new Set(["v1", "inference"]);

export function matchInferenceRoute(request: Request): MatchedInferenceRoute {
  const parts = getScopedRequestPathParts(request, ROUTE_ROOTS);
  if (parts[0] === "v1" && parts[1] === "chat" && parts[2] === "completions" && request.method === "POST") {
    return "chat-completions";
  }
  if (parts[0] === "v1" && parts[1] === "completions" && request.method === "POST") {
    return "completions";
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

export async function handleInferenceRoute(request: Request, route: InferenceRoute): Promise<Response> {
  return handleRuntimeInferenceRoute(request, route);
}
