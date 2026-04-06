export function formatError(error: unknown): string {
  return error instanceof Error ? `${error.name}: ${error.message}` : String(error);
}
