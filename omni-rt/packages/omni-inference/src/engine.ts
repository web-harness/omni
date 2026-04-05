import {
  LoggerWithoutDebug,
  type ChatCompletionOptions,
  type CompletionChunk,
  type WllamaChatMessage,
  Wllama,
} from "@wllama/wllama";
import {
  deleteModel as deleteCachedModel,
  hasEngaged,
  hasModel,
  initCache,
  listCachedModels,
  markEngaged,
  readModelBlob,
  writeModelData,
} from "./model-cache.js";
import { getModelSpec, modelPartUrls } from "./models.js";

type DownloadPhase = "idle" | "downloading" | "completed" | "error";

type DownloadState = {
  phase: DownloadPhase;
  model_id: string | null;
  loaded_bytes: number | null;
  total_bytes: number | null;
  progress_percent: number | null;
};

export type BrowserInferenceStatus = {
  engaged: boolean;
  loaded_model_id: string | null;
  cached_model_ids: string[];
  download: DownloadState;
  last_error: string | null;
};

const STATUS_CHANNEL = "omni-inference-status";
const WASM_PATHS = {
  "single-thread/wllama.wasm": "/wllama/single-thread/wllama.wasm",
  "multi-thread/wllama.wasm": "/wllama/multi-thread/wllama.wasm",
};

function defaultStatus(): BrowserInferenceStatus {
  return {
    engaged: false,
    loaded_model_id: null,
    cached_model_ids: [],
    download: {
      phase: "idle",
      model_id: null,
      loaded_bytes: null,
      total_bytes: null,
      progress_percent: null,
    },
    last_error: null,
  };
}

function cloneStatus(status: BrowserInferenceStatus): BrowserInferenceStatus {
  return {
    ...status,
    cached_model_ids: [...status.cached_model_ids],
    download: { ...status.download },
  };
}

function joinChunks(chunks: Uint8Array[], totalBytes: number): Uint8Array {
  const output = new Uint8Array(totalBytes);
  let offset = 0;

  for (const chunk of chunks) {
    output.set(chunk, offset);
    offset += chunk.byteLength;
  }

  return output;
}

function hex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

function readUint16LE(bytes: Uint8Array, offset: number): number {
  return bytes[offset] | (bytes[offset + 1] << 8);
}

function readUint32LE(bytes: Uint8Array, offset: number): number {
  return (bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16) | (bytes[offset + 3] << 24)) >>> 0;
}

function extractStoredZipEntry(archive: Uint8Array, expectedFile: string): Uint8Array {
  if (readUint32LE(archive, 0) !== 0x04034b50) {
    throw new Error("Mirror archive is not a ZIP local file record");
  }

  const compressionMethod = readUint16LE(archive, 8);
  if (compressionMethod !== 0) {
    throw new Error(`Unsupported ZIP compression method: ${compressionMethod}`);
  }

  const compressedSize = readUint32LE(archive, 18);
  const fileNameLength = readUint16LE(archive, 26);
  const extraFieldLength = readUint16LE(archive, 28);
  const fileNameOffset = 30;
  const fileName = new TextDecoder().decode(archive.subarray(fileNameOffset, fileNameOffset + fileNameLength));
  if (fileName !== expectedFile) {
    throw new Error(`Mirror archive entry mismatch: expected ${expectedFile}, got ${fileName}`);
  }

  const dataOffset = fileNameOffset + fileNameLength + extraFieldLength;
  return archive.slice(dataOffset, dataOffset + compressedSize);
}

async function sha256(data: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", data);
  return hex(new Uint8Array(digest));
}

async function readResponseBytes(
  response: Response,
  onChunk: (chunk: Uint8Array, expectedBytes: number | null) => void,
): Promise<Uint8Array> {
  if (!response.body) {
    throw new Error("Download response has no body");
  }

  const expectedBytes = Number(response.headers.get("content-length") ?? 0) || null;
  const reader = response.body.getReader();
  const chunks: Uint8Array[] = [];
  let loadedBytes = 0;

  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    if (!value) {
      continue;
    }

    const chunk = new Uint8Array(value.byteLength);
    chunk.set(value);
    chunks.push(chunk);
    loadedBytes += chunk.byteLength;
    onChunk(chunk, expectedBytes);
  }

  return joinChunks(chunks, loadedBytes);
}

export class WllamaEngine {
  private readonly status = defaultStatus();
  private readonly channel = typeof BroadcastChannel === "undefined" ? null : new BroadcastChannel(STATUS_CHANNEL);
  private readonly wllama = new Wllama(WASM_PATHS, {
    logger: LoggerWithoutDebug,
    parallelDownloads: 1,
    allowOffline: false,
  });
  private loadedModelId: string | null = null;
  private downloadPromise: Promise<void> | null = null;
  private downloadAbortController: AbortController | null = null;

  private resetDownloadState(): void {
    this.status.download = {
      phase: "idle",
      model_id: null,
      loaded_bytes: null,
      total_bytes: null,
      progress_percent: null,
    };
  }

  private async refreshStatus(): Promise<void> {
    this.status.engaged = await hasEngaged();
    this.status.cached_model_ids = await listCachedModels();
    this.status.loaded_model_id = this.loadedModelId;
  }

  private broadcast(): void {
    this.channel?.postMessage({ type: "status", payload: cloneStatus(this.status) });
  }

  private async syncAndBroadcast(): Promise<void> {
    await this.refreshStatus();
    this.broadcast();
  }

  async getStatus(): Promise<BrowserInferenceStatus> {
    await initCache();
    await this.refreshStatus();
    return cloneStatus(this.status);
  }

  async loadModel(modelId: string): Promise<void> {
    await initCache();

    if (this.loadedModelId === modelId && this.wllama.isModelLoaded()) {
      return;
    }

    const spec = getModelSpec(modelId);
    if (!spec) {
      throw new Error(`Unknown browser model: ${modelId}`);
    }

    const blob = await readModelBlob(modelId);
    if (!blob) {
      throw new Error(`Model is not cached: ${modelId}`);
    }

    if (this.wllama.isModelLoaded()) {
      await this.wllama.exit();
    }

    await this.wllama.loadModel([blob], {
      n_threads: crossOriginIsolated ? undefined : 1,
    });

    this.loadedModelId = spec.id;
    this.status.last_error = null;
    await this.syncAndBroadcast();
  }

  async unloadModel(): Promise<void> {
    await this.wllama.exit();
    this.loadedModelId = null;
    await this.syncAndBroadcast();
  }

  async deleteModel(modelId: string): Promise<void> {
    await initCache();

    if (this.downloadPromise && this.status.download.model_id === modelId) {
      throw new Error("Cannot delete a model while it is downloading");
    }

    if (this.loadedModelId === modelId && this.wllama.isModelLoaded()) {
      await this.unloadModel();
    }

    await deleteCachedModel(modelId);
    this.status.last_error = null;
    this.resetDownloadState();
    await this.syncAndBroadcast();
  }

  async stopDownload(modelId: string): Promise<void> {
    await initCache();

    if (this.status.download.model_id !== modelId) {
      await deleteCachedModel(modelId).catch(() => {});
      await this.refreshStatus();
      this.resetDownloadState();
      this.status.last_error = null;
      this.broadcast();
      return;
    }

    this.downloadAbortController?.abort();
    await this.downloadPromise.catch(() => {});
    await deleteCachedModel(modelId).catch(() => {});
    await this.refreshStatus();
    this.resetDownloadState();
    this.status.last_error = null;
    this.broadcast();
  }

  async downloadModel(modelId: string, onProgress?: (status: BrowserInferenceStatus) => void): Promise<void> {
    const spec = getModelSpec(modelId);
    if (!spec) {
      throw new Error(`Unknown browser model: ${modelId}`);
    }

    if (this.downloadPromise) {
      throw new Error("A model download is already in progress");
    }

    this.downloadPromise = (async () => {
      const abortController = new AbortController();
      this.downloadAbortController = abortController;
      await initCache();
      await markEngaged();
      await this.refreshStatus();
      this.status.download = {
        phase: "downloading",
        model_id: modelId,
        loaded_bytes: 0,
        total_bytes: spec.size,
        progress_percent: 0,
      };
      this.status.last_error = null;
      this.broadcast();
      onProgress?.(cloneStatus(this.status));

      try {
        const urls = modelPartUrls(modelId);
        const partBuffers = new Array<Uint8Array>(urls.length);
        let loadedBytes = 0;
        let totalBytes = 0;
        const announcedPartSizes = new Set<number>();
        let nextPartIndex = 0;

        const updateProgress = (chunk: Uint8Array, expectedBytes: number | null, partIndex: number): void => {
          loadedBytes += chunk.byteLength;
          if (expectedBytes && !announcedPartSizes.has(partIndex)) {
            announcedPartSizes.add(partIndex);
            totalBytes += expectedBytes;
          }

          this.status.download = {
            phase: "downloading",
            model_id: modelId,
            loaded_bytes: loadedBytes,
            total_bytes: totalBytes > 0 ? totalBytes : null,
            progress_percent: totalBytes > 0 ? Math.min(100, Math.round((loadedBytes / totalBytes) * 100)) : null,
          };
          this.broadcast();
          onProgress?.(cloneStatus(this.status));
        };

        const downloadPart = async (partIndex: number): Promise<void> => {
          const response = await fetch(urls[partIndex], { signal: abortController.signal });
          if (!response.ok) {
            throw new Error(`Download failed with status ${response.status}`);
          }

          partBuffers[partIndex] = await readResponseBytes(response, (chunk, expectedBytes) => {
            updateProgress(chunk, expectedBytes, partIndex);
          });
        };

        const worker = async (): Promise<void> => {
          while (nextPartIndex < urls.length) {
            const partIndex = nextPartIndex;
            nextPartIndex += 1;
            await downloadPart(partIndex);
          }
        };

        await Promise.all(Array.from({ length: Math.min(3, urls.length) }, () => worker()));

        if (partBuffers.some((part) => !part)) {
          throw new Error(`Mirror archive download incomplete for ${spec.file}`);
        }

        const archiveChunks = partBuffers as Uint8Array[];
        const archiveBytes = joinChunks(
          archiveChunks,
          archiveChunks.reduce((sum, chunk) => sum + chunk.byteLength, 0),
        );
        const modelBytes = extractStoredZipEntry(archiveBytes, spec.file);
        const actualSha = await sha256(modelBytes);
        if (actualSha !== spec.sha256) {
          throw new Error(`Mirror checksum mismatch for ${spec.file}`);
        }

        await writeModelData(modelId, modelBytes);
        await this.refreshStatus();
        this.status.download = {
          phase: "completed",
          model_id: modelId,
          loaded_bytes: spec.size,
          total_bytes: spec.size,
          progress_percent: 100,
        };
        this.status.last_error = null;
        this.broadcast();
        onProgress?.(cloneStatus(this.status));
      } catch (error) {
        if (abortController.signal.aborted) {
          await deleteCachedModel(modelId).catch(() => {});
          await this.refreshStatus();
          this.resetDownloadState();
          this.status.last_error = null;
          this.broadcast();
          onProgress?.(cloneStatus(this.status));
          return;
        }

        this.status.download = {
          phase: "error",
          model_id: modelId,
          loaded_bytes: this.status.download.loaded_bytes,
          total_bytes: this.status.download.total_bytes,
          progress_percent: this.status.download.progress_percent,
        };
        this.status.last_error = error instanceof Error ? error.message : String(error);
        this.broadcast();
        onProgress?.(cloneStatus(this.status));
        throw error;
      }
    })();

    try {
      await this.downloadPromise;
    } finally {
      this.downloadAbortController = null;
      this.downloadPromise = null;
    }
  }

  async createChatCompletion(
    modelId: string,
    messages: WllamaChatMessage[],
    options: ChatCompletionOptions & { stream?: false },
  ): Promise<string>;
  async createChatCompletion(
    modelId: string,
    messages: WllamaChatMessage[],
    options: ChatCompletionOptions & { stream: true },
  ): Promise<AsyncIterable<CompletionChunk>>;
  async createChatCompletion(
    modelId: string,
    messages: WllamaChatMessage[],
    options: ChatCompletionOptions,
  ): Promise<string | AsyncIterable<CompletionChunk>> {
    if (!(await hasModel(modelId))) {
      throw new Error(`Model is not cached: ${modelId}`);
    }

    await this.loadModel(modelId);
    return this.wllama.createChatCompletion(messages, {
      useCache: true,
      ...options,
    });
  }
}

let inferenceEngine: WllamaEngine | null = null;

export function getInferenceEngine(): WllamaEngine {
  if (!inferenceEngine) {
    inferenceEngine = new WllamaEngine();
  }
  return inferenceEngine;
}
