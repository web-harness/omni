export function createCachedLoader<T>(load: () => Promise<T>): () => Promise<T> {
  let promise: Promise<T> | null = null;

  return () => {
    promise ??= load();
    return promise;
  };
}
