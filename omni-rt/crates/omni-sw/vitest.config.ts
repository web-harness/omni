import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: {
    alias: {
      "./omni-bashkit.js": fileURLToPath(new URL("./src/test/omni-bashkit.mock.ts", import.meta.url)),
      "/omni-bashkit.js": fileURLToPath(new URL("./src/test/omni-bashkit.mock.ts", import.meta.url)),
    },
  },
});
