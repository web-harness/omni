import { configure, fs } from "@zenfs/core";
import { IndexedDB } from "@zenfs/dom";

export { fs };

interface StatInfo {
  is_file: boolean;
  is_dir: boolean;
  is_symlink: boolean;
  size: number;
  mode: number;
  mtime_ms: number;
  atime_ms: number;
  ctime_ms: number;
}

interface DirEntryInfo {
  name: string;
  is_file: boolean;
  is_dir: boolean;
  is_symlink: boolean;
}

function splitPath(value: string): string[] {
  return String(value).split("/").filter(Boolean);
}

function normalizePath(value: string): string {
  const input = String(value || ".");
  const absolute = input.startsWith("/");
  const segments: string[] = [];

  for (const segment of splitPath(input)) {
    if (segment === ".") {
      continue;
    }

    if (segment === "..") {
      if (segments.length > 0 && segments[segments.length - 1] !== "..") {
        segments.pop();
      } else if (!absolute) {
        segments.push("..");
      }
      continue;
    }

    segments.push(segment);
  }

  if (absolute) {
    return segments.length > 0 ? `/${segments.join("/")}` : "/";
  }

  return segments.join("/") || ".";
}

function isAbsolutePath(value: string): boolean {
  return String(value).startsWith("/");
}

function joinPath(...parts: string[]): string {
  return normalizePath(parts.filter(Boolean).join("/"));
}

function dirnamePath(value: string): string {
  const normalized = normalizePath(value);
  if (normalized === "/") {
    return "/";
  }
  if (normalized === ".") {
    return ".";
  }

  const segments = splitPath(normalized);
  segments.pop();

  if (isAbsolutePath(normalized)) {
    return segments.length > 0 ? `/${segments.join("/")}` : "/";
  }

  return segments.join("/") || ".";
}

function basenamePath(value: string, suffix = ""): string {
  const normalized = normalizePath(value);
  const base = splitPath(normalized).at(-1) || (normalized === "/" ? "/" : "");
  if (suffix && base.endsWith(suffix)) {
    return base.slice(0, -suffix.length);
  }
  return base;
}

function extnamePath(value: string): string {
  const base = basenamePath(value);
  const index = base.lastIndexOf(".");
  if (index <= 0) {
    return "";
  }
  return base.slice(index);
}

function resolvePath(...parts: string[]): string {
  let current = "/";
  for (const part of parts.filter(Boolean)) {
    current = isAbsolutePath(part) ? part : `${current}/${part}`;
  }
  return normalizePath(current);
}

function relativePath(from: string, to: string): string {
  const normalizedFrom = normalizePath(from);
  const normalizedTo = normalizePath(to);
  if (isAbsolutePath(normalizedFrom) !== isAbsolutePath(normalizedTo)) {
    return normalizedTo;
  }

  const fromSegments = splitPath(normalizedFrom);
  const toSegments = splitPath(normalizedTo);

  while (fromSegments.length > 0 && toSegments.length > 0 && fromSegments[0] === toSegments[0]) {
    fromSegments.shift();
    toSegments.shift();
  }

  return [...Array(fromSegments.length).fill(".."), ...toSegments].join("/") || ".";
}

export const path = {
  sep: "/",
  delimiter: ":",
  basename: basenamePath,
  dirname: dirnamePath,
  extname: extnamePath,
  isAbsolute: isAbsolutePath,
  join: joinPath,
  normalize: normalizePath,
  relative: relativePath,
  resolve: resolvePath,
};

export async function init(): Promise<void> {
  await configure({
    mounts: { "/": IndexedDB },
    defaultDirectories: true,
  });
  for (const dir of [
    "/tmp",
    "/home",
    "/home/user",
    "/home/db",
    "/home/db/threads",
    "/home/db/messages",
    "/home/db/todos",
    "/home/db/subagents",
    "/home/config",
    "/dev",
  ]) {
    await fs.promises.mkdir(dir, { recursive: true }).catch(() => {});
  }
}

export async function readFile(path: string): Promise<Uint8Array> {
  return fs.promises.readFile(path);
}

export async function writeFile(path: string, data: Uint8Array): Promise<void> {
  await fs.promises.writeFile(path, data);
}

export async function appendFile(path: string, data: Uint8Array): Promise<void> {
  await fs.promises.appendFile(path, data);
}

export async function mkdir(path: string, opts?: { recursive?: boolean }): Promise<void> {
  await fs.promises.mkdir(path, opts);
}

export async function rm(path: string, opts?: { recursive?: boolean }): Promise<void> {
  await fs.promises.rm(path, opts);
}

async function statPath(path: string, doLstat: boolean): Promise<StatInfo> {
  const s = await (doLstat ? fs.promises.lstat(path) : fs.promises.stat(path));
  return {
    is_file: s.isFile(),
    is_dir: s.isDirectory(),
    is_symlink: s.isSymbolicLink(),
    size: Number(s.size),
    mode: s.mode,
    mtime_ms: s.mtimeMs,
    atime_ms: s.atimeMs,
    ctime_ms: s.ctimeMs,
  };
}

export async function stat(path: string): Promise<StatInfo> {
  return statPath(path, false);
}

export async function lstat(path: string): Promise<StatInfo> {
  return statPath(path, true);
}

export async function readdir(path: string): Promise<DirEntryInfo[]> {
  const entries = await fs.promises.readdir(path, { withFileTypes: true });
  return entries.map((e) => ({
    name: e.name,
    is_file: e.isFile(),
    is_dir: e.isDirectory(),
    is_symlink: e.isSymbolicLink(),
  }));
}

export async function exists(path: string): Promise<boolean> {
  return fs.promises.exists(path);
}

export async function rename(from: string, to: string): Promise<void> {
  await fs.promises.rename(from, to);
}

export async function copyFile(from: string, to: string): Promise<void> {
  await fs.promises.copyFile(from, to);
}

export async function symlink(target: string, path: string): Promise<void> {
  await fs.promises.symlink(target, path);
}

export async function readlink(path: string): Promise<string> {
  return fs.promises.readlink(path, { encoding: "utf8" }) as Promise<string>;
}

export async function chmod(path: string, mode: number): Promise<void> {
  await fs.promises.chmod(path, mode);
}
