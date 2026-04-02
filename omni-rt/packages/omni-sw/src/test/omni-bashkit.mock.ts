export default async function initBashkit(): Promise<void> {}

export async function execute(
  command: string,
  cwd?: string,
): Promise<{ output: string; exitCode: number; truncated: boolean }> {
  return {
    output: `[mock bashkit] ${cwd ?? "/home/workspace"}: ${command}`,
    exitCode: 0,
    truncated: false,
  };
}
