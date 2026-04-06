import type { StoredSseEvent } from "./run-store.js";

type FrameBuffer = {
  text: string;
};

const textEncoder = new TextEncoder();

export type SseFrame = StoredSseEvent;

export function encodeSseChunk(frame: SseFrame): Uint8Array {
  const idLine = frame.id ? `id: ${frame.id}\n` : "";
  return textEncoder.encode(`${idLine}event: ${frame.event}\ndata: ${JSON.stringify(frame.data)}\n\n`);
}

export async function* readSseEvents(stream: ReadableStream<Uint8Array>): AsyncGenerator<SseFrame, void, void> {
  const reader = stream.getReader();
  const decoder = new TextDecoder();
  const buffer: FrameBuffer = { text: "" };

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        break;
      }

      buffer.text += decoder.decode(value, { stream: true });
      while (true) {
        const frame = takeFrame(buffer);
        if (!frame) {
          break;
        }
        yield frame;
      }
    }

    buffer.text += decoder.decode();
    while (true) {
      const frame = takeFrame(buffer);
      if (!frame) {
        break;
      }
      yield frame;
    }
  } finally {
    reader.releaseLock();
  }
}

export function parseSseFrames(text: string): SseFrame[] {
  const buffer: FrameBuffer = { text };
  const frames: SseFrame[] = [];
  while (true) {
    const frame = takeFrame(buffer);
    if (!frame) {
      break;
    }
    frames.push(frame);
  }
  return frames;
}

function takeFrame(buffer: FrameBuffer): SseFrame | null {
  const separatorIndex = buffer.text.indexOf("\n\n");
  if (separatorIndex < 0) {
    return null;
  }

  const chunk = buffer.text.slice(0, separatorIndex);
  buffer.text = buffer.text.slice(separatorIndex + 2);

  let id: string | undefined;
  let event = "message";
  const dataLines: string[] = [];
  for (const line of chunk.split("\n")) {
    if (line.startsWith("id: ")) {
      id = line.slice(4).trim();
      continue;
    }
    if (line.startsWith("event: ")) {
      event = line.slice(7).trim() || "message";
      continue;
    }
    if (line.startsWith("data: ")) {
      dataLines.push(line.slice(6));
    }
  }

  const rawData = dataLines.join("\n");
  let data: unknown = rawData;
  if (rawData) {
    try {
      data = JSON.parse(rawData);
    } catch {
      data = rawData;
    }
  }

  return { id, event, data };
}
