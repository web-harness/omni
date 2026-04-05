import {
  BROWSER_MODELS,
  getBrowserModelDownloadUrl,
  getBrowserModelPartUrls,
  getBrowserModelSourceLabel,
  getBrowserModelSpec,
  type BrowserModelId,
  type BrowserModelSpec,
} from "@omni/omni-util/browser-models";

export { BROWSER_MODELS };
export type { BrowserModelId, BrowserModelSpec };

export function getModelSpec(modelId: string): BrowserModelSpec | null {
  return getBrowserModelSpec(modelId);
}

export function getModelPath(modelId: string): string {
  const spec = getModelSpec(modelId);
  if (!spec) {
    throw new Error(`Unknown browser model: ${modelId}`);
  }
  return `/models/${spec.file}`;
}

export function modelDownloadUrl(modelId: string): string {
  return getBrowserModelDownloadUrl(modelId);
}

export function modelSourceLabel(modelId: string): string {
  return getBrowserModelSourceLabel(modelId);
}

export function modelPartUrls(modelId: string): string[] {
  return getBrowserModelPartUrls(modelId);
}
