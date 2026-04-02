import {
  buildBootstrap,
  createThread,
  deleteApiKey,
  deleteThread,
  getApiKey,
  getDefaultModel,
  listWorkspaceFiles,
  readProvidersWithKeys,
  setApiKey,
  setDefaultModel,
} from "./store-data.js";

export type StoreRoute =
  | "store-bootstrap"
  | "store-create-thread"
  | "store-delete-thread"
  | "store-providers"
  | "store-get-api-key"
  | "store-set-api-key"
  | "store-delete-api-key"
  | "store-get-default-model"
  | "store-set-default-model"
  | "store-files"
  | null;

export function matchStoreRoute(request: Request): StoreRoute {
  const url = new URL(request.url);
  if (url.pathname === "/api/store/bootstrap" && request.method === "GET") return "store-bootstrap";
  if (url.pathname === "/api/store/threads" && request.method === "POST") return "store-create-thread";
  if (url.pathname.startsWith("/api/store/threads/") && request.method === "DELETE") return "store-delete-thread";
  if (url.pathname === "/api/store/providers" && request.method === "GET") return "store-providers";
  if (url.pathname.startsWith("/api/store/config/api-keys/") && request.method === "GET") return "store-get-api-key";
  if (url.pathname.startsWith("/api/store/config/api-keys/") && request.method === "PUT") return "store-set-api-key";
  if (url.pathname.startsWith("/api/store/config/api-keys/") && request.method === "DELETE") {
    return "store-delete-api-key";
  }
  if (url.pathname === "/api/store/config/default-model" && request.method === "GET") return "store-get-default-model";
  if (url.pathname === "/api/store/config/default-model" && request.method === "PUT") return "store-set-default-model";
  if (url.pathname === "/api/store/files" && request.method === "GET") return "store-files";
  return null;
}

function getPathTail(url: URL): string {
  return url.pathname.split("/").pop() ?? "";
}

export async function handleStoreRoute(request: Request, route: Exclude<StoreRoute, null>): Promise<Response> {
  try {
    const url = new URL(request.url);

    if (route === "store-bootstrap") {
      return Response.json(await buildBootstrap());
    }

    if (route === "store-create-thread") {
      return Response.json(await createThread());
    }

    if (route === "store-delete-thread") {
      await deleteThread(getPathTail(url).trim());
      return new Response(null, { status: 204 });
    }

    if (route === "store-providers") {
      return Response.json(await readProvidersWithKeys());
    }

    if (route === "store-get-api-key") {
      return Response.json({ value: await getApiKey(getPathTail(url)) });
    }

    if (route === "store-set-api-key") {
      const body = (await request.json()) as { value?: string };
      await setApiKey(getPathTail(url), body.value ?? "");
      return new Response(null, { status: 204 });
    }

    if (route === "store-delete-api-key") {
      await deleteApiKey(getPathTail(url));
      return new Response(null, { status: 204 });
    }

    if (route === "store-get-default-model") {
      return Response.json({ model_id: await getDefaultModel() });
    }

    if (route === "store-set-default-model") {
      const body = (await request.json()) as { model_id?: string };
      if (!body.model_id) {
        return Response.json({ error: "model_id is required" }, { status: 400 });
      }
      await setDefaultModel(body.model_id);
      return new Response(null, { status: 204 });
    }

    if (route === "store-files") {
      const workspace = url.searchParams.get("workspace") ?? "/home/workspace";
      return Response.json(await listWorkspaceFiles(workspace));
    }

    return Response.json({ error: "Unknown route" }, { status: 404 });
  } catch (err) {
    const message =
      err instanceof Error ? `${err.name}: ${err.message}${err.stack ? `\n${err.stack}` : ""}` : String(err);
    return Response.json({ error: message }, { status: 500 });
  }
}
