import * as esbuild from "esbuild";
import { copyFileSync, mkdirSync } from "fs";
import { fileURLToPath } from "url";

mkdirSync("dist", { recursive: true });

// Stub for Node built-ins that LangGraph imports but doesn't use in browser
const nodeStubPlugin: esbuild.Plugin = {
  name: "node-stub",
  setup(build) {
    const stubs = [
      "node:async_hooks",
      "async_hooks",
      "path",
      "node:fs",
      "node:fs/promises",
      "fs",
      "node:path",
      "node:os",
      "os",
      "node:crypto",
      "crypto",
      "node:stream",
      "stream",
      "node:util",
      "util",
      "node:events",
      "events",
      "node:buffer",
      "buffer",
      "node:process",
      "process",
      "child_process",
      "node:child_process",
      "node:http",
      "http",
      "node:https",
      "https",
      "node:net",
      "net",
      "node:tls",
      "tls",
      "node:zlib",
      "zlib",
      "node:url",
      "url",
      "node:querystring",
      "querystring",
      "node:string_decoder",
      "string_decoder",
    ];
    for (const mod of stubs) {
      build.onResolve({ filter: new RegExp(`^${mod.replace(/\//g, "\\/")}$`) }, () => ({
        path: mod,
        namespace: "node-stub",
      }));
    }
    build.onLoad({ filter: /.*/, namespace: "node-stub" }, (args) => {
      if (args.path === "node:async_hooks" || args.path === "async_hooks") {
        return {
          contents:
            "export class AsyncLocalStorage { getStore() { return undefined; } run(s, fn, ...a) { return fn(...a); } } export class AsyncResource { static bind(fn) { return fn; } }",
          loader: "js",
        };
      }
      if (args.path === "path" || args.path === "node:path") {
        return {
          contents:
            "export const basename = (p='') => (String(p).split('/').filter(Boolean).pop() || ''); export const dirname = (p='') => { const s=String(p).split('/'); s.pop(); return s.join('/') || '/'; }; export const join = (...p) => p.filter(Boolean).join('/'); export default { basename, dirname, join };",
          loader: "js",
        };
      }
      if (args.path === "child_process" || args.path === "node:child_process") {
        return {
          contents:
            "export const spawn = () => { throw new Error('child_process is unavailable in service worker'); }; const d = { spawn }; export default d;",
          loader: "js",
        };
      }
      if (args.path === "buffer" || args.path === "node:buffer") {
        return {
          contents:
            "export class Buffer extends Uint8Array { static from(v) { if (typeof v === 'string') return new TextEncoder().encode(v); return new Uint8Array(v); } static alloc(n) { return new Uint8Array(n); } static concat(chunks) { const size = chunks.reduce((n, c) => n + c.length, 0); const out = new Uint8Array(size); let o = 0; for (const c of chunks) { out.set(c, o); o += c.length; } return out; } } export default { Buffer };",
          loader: "js",
        };
      }
      if (args.path === "process" || args.path === "node:process") {
        return {
          contents: "export const env = {}; const p = { env }; export default p;",
          loader: "js",
        };
      }
      if (args.path === "node:fs/promises" || args.path === "node:fs" || args.path === "fs") {
        return {
          contents:
            "const e = async () => { throw new Error('fs is unavailable in service worker'); }; export const readFile=e, writeFile=e, readdir=e, mkdir=e, stat=e, rm=e, unlink=e, access=e; export const existsSync=() => false; export default { readFile, writeFile, readdir, mkdir, stat, rm, unlink, access, existsSync };",
          loader: "js",
        };
      }
      return { contents: "export default {};", loader: "js" };
    });
  },
};

const sharedOptions: esbuild.BuildOptions = {
  bundle: true,
  platform: "browser",
  target: "es2021",
  external: ["./omni-bashkit.js", "/omni-bashkit.js", "./omni-zenfs.js", "/omni-zenfs.js"],
  define: {
    process: "globalThis.__omni_process_polyfill",
    "process.env.NODE_ENV": '"production"',
    global: "globalThis",
  },
  banner: {
    js: "globalThis.__omni_process_polyfill ??= { env: {}, platform: 'browser', version: 'v0.0.0', cwd: () => '/', nextTick: (fn, ...args) => queueMicrotask(() => fn(...args)) };",
  },
  plugins: [nodeStubPlugin],
};

await esbuild.build({
  ...sharedOptions,
  entryPoints: ["src/omni-sw.ts"],
  format: "esm",
  outfile: "dist/omni-sw.js",
});

await esbuild.build({
  ...sharedOptions,
  entryPoints: ["src/register.ts"],
  format: "esm",
  outfile: "dist/omni-sw-register.js",
});

const sqlWasmPath = fileURLToPath(import.meta.resolve("sql.js/dist/sql-wasm.wasm"));
copyFileSync(sqlWasmPath, "dist/sql-wasm.wasm");
