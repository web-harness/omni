import { LitElement, html, css } from "lit";
import { customElement, property, state } from "lit/decorators.js";

import type { PDFDocumentProxy } from "pdfjs-dist/types/src/display/api";

type PdfJsModule = typeof import("pdfjs-dist");

let pdfjsPromise: Promise<PdfJsModule> | null = null;

function loadPdfJsModule(): Promise<PdfJsModule> {
  pdfjsPromise ??= import("pdfjs-dist");
  return pdfjsPromise;
}

@customElement("omni-pdfjs")
export class OmniPdfjs extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      background: #525659;
    }
    .toolbar {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 6px 12px;
      background: var(--sidebar, #1e1e2e);
      border-bottom: 1px solid var(--border, #333);
      flex-shrink: 0;
    }
    .toolbar span {
      font-size: 11px;
      color: var(--muted-foreground, #888);
    }
    .toolbar button {
      font-size: 11px;
      padding: 2px 8px;
      border-radius: 3px;
      border: none;
      background: transparent;
      color: var(--muted-foreground, #888);
      cursor: pointer;
    }
    .toolbar button:hover {
      background: var(--background-interactive, #2a2a3e);
    }
    .spacer { flex: 1; }
    .scroll-area {
      flex: 1;
      overflow-y: auto;
      display: flex;
      flex-direction: column;
      align-items: center;
      padding: 16px;
      gap: 12px;
    }
    canvas {
      box-shadow: 0 2px 8px rgba(0,0,0,0.5);
      background: white;
      display: block;
    }
    .status {
      font-size: 12px;
      color: var(--muted-foreground, #888);
      padding: 32px;
    }
  `;

  @property({ attribute: "data-src" }) dataSrc = "";
  @property({ attribute: "data-filename" }) dataFilename = "document.pdf";

  @state() private pageCount = 0;
  @state() private scale = 1.5;
  @state() private status = "";
  @state() private fitToWidth = true;

  private pdfDoc: PDFDocumentProxy | null = null;
  private loadVersion = 0;
  private resizeObserver: ResizeObserver | null = null;

  render() {
    return html`
      <div class="toolbar">
        <span>${this.dataFilename}</span>
        ${this.pageCount > 0 ? html`<span>${this.pageCount}p</span>` : ""}
        <span class="spacer"></span>
        <button @click=${() => this.setScale(this.scale - 0.25)}>−</button>
        <span>${Math.round(this.scale * 100)}%</span>
        <button @click=${() => this.setScale(this.scale + 0.25)}>+</button>
        <button @click=${() => void this.resetScale()}>Fit</button>
      </div>
      <div class="scroll-area" id="scroll">
        ${this.status ? html`<div class="status">${this.status}</div>` : ""}
      </div>
    `;
  }

  firstUpdated() {
    const scrollArea = this.getScrollArea();
    if (!scrollArea) return;

    this.resizeObserver = new ResizeObserver(() => {
      if (!this.fitToWidth || !this.pdfDoc) return;
      void this.syncFitScale();
    });
    this.resizeObserver.observe(scrollArea);
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has("dataSrc") && this.dataSrc) {
      void this.loadPdf();
    }
    if (changed.has("scale") && this.pdfDoc) {
      void this.renderAllPages();
    }
  }

  private setScale(s: number) {
    this.fitToWidth = false;
    this.scale = Math.max(0.5, Math.min(4.0, s));
  }

  private async resetScale() {
    this.fitToWidth = true;
    await this.syncFitScale();
  }

  private async getPdfJs(): Promise<PdfJsModule> {
    const pdfjs = await loadPdfJsModule();
    const workerMeta = document.querySelector<HTMLMetaElement>('meta[name="omni-pdfjs-worker"]');
    pdfjs.GlobalWorkerOptions.workerSrc =
      workerMeta?.content ?? new URL("./omni-pdfjs.worker.js", import.meta.url).href;
    return pdfjs;
  }

  private async loadPdf() {
    const loadVersion = ++this.loadVersion;
    this.status = "Loading…";
    this.pdfDoc?.destroy();
    this.pdfDoc = null;
    this.pageCount = 0;
    const pdfjs = await this.getPdfJs();
    let nextPdfDoc: PDFDocumentProxy | null = null;

    try {
      nextPdfDoc = await pdfjs.getDocument({ url: this.dataSrc }).promise;
    } catch {
      console.warn("[omni-pdfjs] Worker unavailable, retrying without worker (sync fallback)");
      pdfjs.GlobalWorkerOptions.workerSrc = "";
      try {
        nextPdfDoc = await pdfjs.getDocument({ url: this.dataSrc }).promise;
      } catch (e) {
        this.status = `Failed to load PDF: ${e}`;
        return;
      }
    }

    if (!nextPdfDoc) return;
    if (loadVersion !== this.loadVersion || !this.isConnected) {
      nextPdfDoc.destroy();
      return;
    }

    this.pdfDoc = nextPdfDoc;
    this.pageCount = this.pdfDoc.numPages;
    this.status = "";
    await this.syncFitScale();
    await this.renderAllPages();
  }

  private getScrollArea(): HTMLDivElement | null {
    return this.shadowRoot?.querySelector<HTMLDivElement>("#scroll") ?? null;
  }

  private async syncFitScale() {
    if (!this.pdfDoc) return;
    const fitScale = await this.getFitScale();
    if (fitScale === null) return;
    this.scale = fitScale;
  }

  private async getFitScale(): Promise<number | null> {
    if (!this.pdfDoc) return null;
    const scrollArea = this.getScrollArea();
    if (!scrollArea) return null;

    const styles = getComputedStyle(scrollArea);
    const horizontalPadding = parseFloat(styles.paddingLeft || "0") + parseFloat(styles.paddingRight || "0");
    const availableWidth = scrollArea.clientWidth - horizontalPadding;
    if (availableWidth <= 0) return null;

    const firstPage = await this.pdfDoc.getPage(1);
    const baseViewport = firstPage.getViewport({ scale: 1 });
    const fitScale = availableWidth / baseViewport.width;
    return Math.max(0.5, Math.min(1.5, fitScale));
  }

  private async renderAllPages() {
    if (!this.pdfDoc) return;
    const scrollArea = this.getScrollArea();
    if (!scrollArea) return;
    scrollArea.querySelectorAll("canvas").forEach((canvas) => {
      canvas.remove();
    });

    for (let i = 1; i <= this.pdfDoc.numPages; i++) {
      const page = await this.pdfDoc.getPage(i);
      const viewport = page.getViewport({ scale: this.scale });
      const canvas = document.createElement("canvas");
      canvas.width = viewport.width;
      canvas.height = viewport.height;
      const canvasContext = canvas.getContext("2d");
      if (!canvasContext) continue;
      await page.render({ canvasContext, viewport }).promise;
      scrollArea.appendChild(canvas);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.resizeObserver?.disconnect();
    this.resizeObserver = null;
    this.pdfDoc?.destroy();
    this.pdfDoc = null;
  }
}
