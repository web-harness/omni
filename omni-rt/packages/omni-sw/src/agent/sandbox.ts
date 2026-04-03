import { BaseSandbox, type ExecuteResponse, type FileDownloadResponse, type FileUploadResponse } from "deepagents";
import { fs, init as initZenfs } from "../zenfs.js";

type ExecuteFn = (command: string, cwd: string) => Promise<ExecuteResponse>;

export class BashkitSandboxBackend extends BaseSandbox {
  readonly id: string;

  constructor(
    private readonly cwd: string,
    private readonly executeFn?: ExecuteFn,
  ) {
    super();
    this.id = `bashkit-${Math.random().toString(16).slice(2)}`;
  }

  async execute(command: string): Promise<ExecuteResponse> {
    if (!this.executeFn) {
      return {
        output: "Error: SW bashkit executor is not configured for this sandbox instance.",
        exitCode: 1,
        truncated: false,
      };
    }
    return this.executeFn(command, this.cwd);
  }

  async uploadFiles(files: Array<[string, Uint8Array]>): Promise<FileUploadResponse[]> {
    await this.ensureFs();
    const responses: FileUploadResponse[] = [];
    for (const [path, content] of files) {
      try {
        const parent = path.split("/").slice(0, -1).join("/") || "/";
        await fs.promises.mkdir(parent, { recursive: true });
        await fs.promises.writeFile(path, content);
        responses.push({ path, success: true });
      } catch (error) {
        responses.push({
          path,
          success: false,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }
    return responses;
  }

  async downloadFiles(paths: string[]): Promise<FileDownloadResponse[]> {
    await this.ensureFs();
    const responses: FileDownloadResponse[] = [];
    for (const path of paths) {
      try {
        const content = (await fs.promises.readFile(path)) as Uint8Array;
        responses.push({ path, success: true, content });
      } catch (error) {
        responses.push({
          path,
          success: false,
          error: error instanceof Error ? error.message : String(error),
        });
      }
    }
    return responses;
  }

  private async ensureFs(): Promise<void> {
    await initZenfs();
  }
}
