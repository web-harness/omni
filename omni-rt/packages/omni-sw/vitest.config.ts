import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: {
    alias: {
      "./omni-zenfs.js": fileURLToPath(new URL("../../crates/omni-zenfs/src/omni-zenfs.ts", import.meta.url)),
      "./omni-bashkit.js": fileURLToPath(new URL("./src/test/omni-bashkit.mock.ts", import.meta.url)),
      "./omni-deepagents.js": fileURLToPath(new URL("./src/test/omni-deepagents.mock.ts", import.meta.url)),
    },
  },
});
