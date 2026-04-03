import process from "../../../../node_modules/process/browser.js";

const processShim = process as typeof process & {
  getBuiltinModule?: (name: string) => unknown;
};

processShim.env ??= {};
processShim.cwd ??= () => "/";
processShim.getBuiltinModule ??= () => undefined;
processShim.nextTick ??= ((fn: (...args: unknown[]) => void, ...args: unknown[]) => {
  queueMicrotask(() => fn(...args));
}) as typeof processShim.nextTick;
processShim.platform ??= "browser";
processShim.version ??= "v0.0.0";

export const env = processShim.env;
export default processShim;
