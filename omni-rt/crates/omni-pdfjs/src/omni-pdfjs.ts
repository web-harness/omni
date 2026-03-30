import * as pdfjsLib from "pdfjs-dist";
import { LitElement, html, css } from "lit";
import { customElement, property, state } from "lit/decorators.js";

const _workerMeta = document.querySelector<HTMLMetaElement>('meta[name="omni-pdfjs-worker"]');
pdfjsLib.GlobalWorkerOptions.workerSrc =
  _workerMeta?.content ?? new URL("./omni-pdfjs.worker.js", import.meta.url).href;

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

  private pdfDoc: pdfjsLib.PDFDocumentProxy | null = null;

  render() {
    return html`
      <div class="toolbar">
        <span>${this.dataFilename}</span>
        ${this.pageCount > 0 ? html`<span>${this.pageCount}p</span>` : ""}
        <span class="spacer"></span>
        <button @click=${() => this.setScale(this.scale - 0.25)}>−</button>
        <span>${Math.round(this.scale * 100)}%</span>
        <button @click=${() => this.setScale(this.scale + 0.25)}>+</button>
        <button @click=${() => this.setScale(1.5)}>Reset</button>
      </div>
      <div class="scroll-area" id="scroll">
        ${this.status ? html`<div class="status">${this.status}</div>` : ""}
      </div>
    `;
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has("dataSrc") && this.dataSrc) {
      this.loadPdf();
    }
    if (changed.has("scale") && this.pdfDoc) {
      this.renderAllPages();
    }
  }

  private setScale(s: number) {
    this.scale = Math.max(0.5, Math.min(4.0, s));
  }

  private async loadPdf() {
    this.status = "Loading…";
    this.pdfDoc?.destroy();
    this.pdfDoc = null;
    this.pageCount = 0;

    try {
      this.pdfDoc = await pdfjsLib.getDocument({ url: this.dataSrc }).promise;
    } catch {
      console.warn("[omni-pdfjs] Worker unavailable, retrying without worker (sync fallback)");
      pdfjsLib.GlobalWorkerOptions.workerSrc = "";
      try {
        this.pdfDoc = await pdfjsLib.getDocument({ url: this.dataSrc }).promise;
      } catch (e) {
        this.status = `Failed to load PDF: ${e}`;
        return;
      }
    }

    this.pageCount = this.pdfDoc.numPages;
    this.status = "";
    await this.renderAllPages();
  }

  private async renderAllPages() {
    if (!this.pdfDoc) return;
    const scrollArea = this.shadowRoot!.querySelector<HTMLDivElement>("#scroll")!;
    scrollArea.querySelectorAll("canvas").forEach((c) => c.remove());

    for (let i = 1; i <= this.pdfDoc.numPages; i++) {
      const page = await this.pdfDoc.getPage(i);
      const viewport = page.getViewport({ scale: this.scale });
      const canvas = document.createElement("canvas");
      canvas.width = viewport.width;
      canvas.height = viewport.height;
      await page.render({ canvasContext: canvas.getContext("2d")!, viewport }).promise;
      scrollArea.appendChild(canvas);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.pdfDoc?.destroy();
    this.pdfDoc = null;
  }
}
