export type BrowserModelId = "lfm2-1.2b" | "deepseek-r1-1.5b";

export type BrowserModelSpec = {
  id: BrowserModelId;
  name: string;
  file: string;
  size: number;
  sha256: string;
  mirrorParts: number;
};

const MIRROR_BASE_URL = "https://raw.githubusercontent.com/web-harness/models/main/models";

const MODEL_CATALOG: Record<BrowserModelId, BrowserModelSpec> = {
  "lfm2-1.2b": {
    id: "lfm2-1.2b",
    name: "LFM2 1.2B",
    file: "LFM2-1.2B-Q4_K_M.gguf",
    size: 730_910_720,
    sha256: "55175400e3f509a9616227afeffd58d87e80b9f628a5d3d54ada884d85221fed",
    mirrorParts: 8,
  },
  "deepseek-r1-1.5b": {
    id: "deepseek-r1-1.5b",
    name: "DeepSeek R1 1.5B",
    file: "DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf",
    size: 924_844_032,
    sha256: "b1094879ee7e222cba5be013e6de2d0658c64f91aef5d3c9bff0db5cefbfa020",
    mirrorParts: 10,
  },
};

export const BROWSER_MODELS = Object.values(MODEL_CATALOG);

export function getBrowserModelSpec(modelId: string): BrowserModelSpec | null {
  return MODEL_CATALOG[modelId as BrowserModelId] ?? null;
}

export function getBrowserModelDownloadUrl(modelId: string): string {
  const spec = getBrowserModelSpec(modelId);
  if (!spec) {
    throw new Error(`Unknown browser model: ${modelId}`);
  }
  return `${MIRROR_BASE_URL}/${spec.file}.zip.part-000`;
}

export function getBrowserModelSourceLabel(modelId: string): string {
  const spec = getBrowserModelSpec(modelId);
  return spec ? `web-harness/models/${spec.file}.zip.part-*` : modelId;
}

export function getBrowserModelPartUrls(modelId: string): string[] {
  const spec = getBrowserModelSpec(modelId);
  if (!spec) {
    throw new Error(`Unknown browser model: ${modelId}`);
  }

  return Array.from({ length: spec.mirrorParts }, (_, index) => {
    const part = String(index).padStart(3, "0");
    return `${MIRROR_BASE_URL}/${spec.file}.zip.part-${part}`;
  });
}
