import * as esbuild from "esbuild";
import { readFileSync, copyFileSync, mkdirSync, writeFileSync } from "fs";
import { createRequire } from "module";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

mkdirSync("dist", { recursive: true });

// Stub for Node built-ins that LangGraph imports but doesn't use in browser
const nodeStubPlugin = {
  name: "node-stub",
  setup(build) {
    const stubs = [
      "node:async_hooks",
      "async_hooks",
      "node:fs",
      "node:path",
      "node:os",
      "node:crypto",
      "node:stream",
      "node:util",
      "node:events",
      "node:buffer",
      "node:process",
      "node:http",
      "node:https",
      "node:net",
      "node:tls",
      "node:zlib",
      "node:url",
      "node:querystring",
      "node:string_decoder",
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
      return { contents: "export default {};", loader: "js" };
    });
  },
};

const sharedOptions = {
  bundle: true,
  platform: "browser",
  target: "es2021",
  define: {
    "process.env.NODE_ENV": '"production"',
    global: "globalThis",
  },
  plugins: [nodeStubPlugin],
};

// Build the main SW bundle
await esbuild.build({
  ...sharedOptions,
  entryPoints: ["src/omni-sw.ts"],
  format: "esm",
  outfile: "dist/omni-sw.js",
});

// Build the registration module
await esbuild.build({
  ...sharedOptions,
  entryPoints: ["src/register.ts"],
  format: "esm",
  outfile: "dist/omni-sw-register.js",
});

// Copy sql-wasm.wasm to dist for serving
const sqlWasmPath = require.resolve("sql.js/dist/sql-wasm.wasm");
copyFileSync(sqlWasmPath, "dist/sql-wasm.wasm");
