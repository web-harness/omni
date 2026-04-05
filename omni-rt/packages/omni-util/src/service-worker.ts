export function resolveServiceWorkerScope(swUrl: string, scopeSuffix = ""): string {
  const baseUrl = globalThis.location?.href ?? "https://example.test/";
  const resolvedUrl = new URL(swUrl, baseUrl);
  const lastSlash = resolvedUrl.pathname.lastIndexOf("/");
  const basePath = lastSlash < 0 ? "/" : resolvedUrl.pathname.slice(0, lastSlash + 1) || "/";

  if (!scopeSuffix) {
    return basePath;
  }

  return `${basePath}${scopeSuffix}`.replace(/\/+/g, "/");
}

export function getScopedRequestPathParts(request: Request, routeRoots?: Iterable<string>): string[] {
  const parts = new URL(request.url).pathname.split("/").filter(Boolean);
  if (!routeRoots) {
    return parts;
  }

  const roots = routeRoots instanceof Set ? routeRoots : new Set(routeRoots);
  const rootIndex = parts.findIndex((part) => roots.has(part));
  return rootIndex >= 0 ? parts.slice(rootIndex) : [];
}
