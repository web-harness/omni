import { configure, fs } from "@zenfs/core";
import { IndexedDB } from "@zenfs/dom";
import { BROWSER_MODELS, getModelPath } from "./models.js";

type EngagedState = {
  engaged: boolean;
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();
const ENGAGED_PATH = "/config/engaged.json";

let cacheReady: Promise<void> | null = null;

async function ensureDirs(): Promise<void> {
  for (const dir of ["/models", "/config"]) {
    await fs.promises.mkdir(dir, { recursive: true }).catch(() => {});
  }
}

async function readJson<T>(path: string): Promise<T | null> {
  try {
    const data = await fs.promises.readFile(path);
    return JSON.parse(decoder.decode(data)) as T;
  } catch {
    return null;
  }
}

function cloneBytes(data: Uint8Array): Uint8Array {
  const bytes = new Uint8Array(data.byteLength);
  bytes.set(data);
  return bytes;
}

export async function initCache(): Promise<void> {
  if (!cacheReady) {
    cacheReady = configure({
      mounts: {
        "/": {
          backend: IndexedDB,
          storeName: "omni-inference",
        },
      },
      defaultDirectories: true,
    }).then(ensureDirs);
  }

  await cacheReady;
}

export async function markEngaged(): Promise<void> {
  await initCache();
  await fs.promises.writeFile(ENGAGED_PATH, encoder.encode(JSON.stringify({ engaged: true } satisfies EngagedState)));
}

export async function hasEngaged(): Promise<boolean> {
  await initCache();
  const state = await readJson<EngagedState>(ENGAGED_PATH);
  return Boolean(state?.engaged);
}

export async function hasModel(modelId: string): Promise<boolean> {
  await initCache();
  return fs.promises.exists(getModelPath(modelId));
}

export async function readModelBlob(modelId: string): Promise<Blob | null> {
  await initCache();
  if (!(await hasModel(modelId))) {
    return null;
  }

  const data = await fs.promises.readFile(getModelPath(modelId));
  return new Blob([cloneBytes(data)], { type: "application/octet-stream" });
}

export async function writeModelData(modelId: string, data: Uint8Array): Promise<void> {
  await initCache();
  await fs.promises.writeFile(getModelPath(modelId), cloneBytes(data));
}

export async function deleteModel(modelId: string): Promise<void> {
  await initCache();
  if (await hasModel(modelId)) {
    await fs.promises.unlink(getModelPath(modelId));
  }

  if (await hasModel(modelId)) {
    throw new Error(`Failed to delete cached model: ${modelId}`);
  }
}

export async function listCachedModels(): Promise<string[]> {
  await initCache();

  const cached: string[] = [];
  for (const model of BROWSER_MODELS) {
    if (await hasModel(model.id)) {
      cached.push(model.id);
    }
  }

  return cached;
}
