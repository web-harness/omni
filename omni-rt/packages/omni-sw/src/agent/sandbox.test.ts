import { describe, expect, it } from "vitest";
import { BashkitSandboxBackend } from "./sandbox";

describe("BashkitSandboxBackend", () => {
  it("returns explicit error when executor is missing", async () => {
    const sandbox = new BashkitSandboxBackend("/home/workspace");
    const result = await sandbox.execute("echo hello");
    expect(result.exitCode).toBe(1);
    expect(result.output).toContain("executor");
  });

  it("uses injected SW-local executor", async () => {
    const sandbox = new BashkitSandboxBackend("/home/workspace", async (command, cwd) => ({
      output: `${cwd}:${command}`,
      exitCode: 0,
      truncated: false,
    }));

    const result = await sandbox.execute("echo hello");
    expect(result.exitCode).toBe(0);
    expect(result.output).toContain("/home/workspace:echo hello");
  });
});
