import process from "./process.ts";
import { Buffer } from "./buffer.ts";

const globalScope = globalThis as typeof globalThis & {
  process?: typeof process;
  Buffer?: typeof Buffer;
};

globalScope.process ??= process;
globalScope.Buffer ??= Buffer;

export { process, Buffer };
