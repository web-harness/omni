import type { ReactiveController, ReactiveControllerHost } from "lit";

export function createObjectUrl(parts: BlobPart[] | Blob, options?: BlobPropertyBag): string {
  return URL.createObjectURL(parts instanceof Blob ? parts : new Blob(parts, options));
}

export function revokeObjectUrl(url: string) {
  if (url) {
    URL.revokeObjectURL(url);
  }
}

export class ObjectUrlController implements ReactiveController {
  private url = "";

  constructor(private readonly host: ReactiveControllerHost) {
    host.addController(this);
  }

  get value() {
    return this.url;
  }

  setBlob(blob: Blob) {
    this.setObjectUrl(createObjectUrl(blob));
  }

  setParts(parts: BlobPart[], options?: BlobPropertyBag) {
    this.setObjectUrl(createObjectUrl(parts, options));
  }

  clear() {
    revokeObjectUrl(this.url);
    this.url = "";
    this.host.requestUpdate();
  }

  hostDisconnected() {
    this.clear();
  }

  private setObjectUrl(nextUrl: string) {
    revokeObjectUrl(this.url);
    this.url = nextUrl;
    this.host.requestUpdate();
  }
}
