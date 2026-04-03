import { LitElement, css, html } from "lit";
import { unsafeHTML } from "lit/directives/unsafe-html.js";
import { customElement, property, state } from "lit/decorators.js";
import DOMPurify from "dompurify";
import * as XLSX from "xlsx";

@customElement("omni-sheetjs")
export class OmniSheetjs extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      background: var(--background, #11111b);
      color: var(--foreground, #cdd6f4);
    }
    .toolbar {
      display: flex;
      align-items: center;
      gap: 10px;
      border-bottom: 1px solid var(--border, #313244);
      background: var(--sidebar, #1e1e2e);
      padding: 8px 12px;
      font-size: 11px;
      flex-shrink: 0;
    }
    .filename {
      color: var(--muted-foreground, #9399b2);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .sheet-tabs {
      display: flex;
      gap: 6px;
      flex-wrap: wrap;
      margin-left: auto;
    }
    .sheet-tab {
      appearance: none;
      border: 1px solid var(--border, #45475a);
      border-radius: 999px;
      background: transparent;
      color: var(--muted-foreground, #bac2de);
      cursor: pointer;
      font: inherit;
      padding: 3px 10px;
    }
    .sheet-tab.active {
      background: var(--background-interactive, #313244);
      color: var(--foreground, #f5e0dc);
    }
    .scroll-area {
      flex: 1;
      overflow: auto;
      padding: 16px;
    }
    .status {
      color: var(--muted-foreground, #9399b2);
      font-size: 12px;
      padding: 16px;
    }
    .sheet-html {
      min-width: fit-content;
    }
    .sheet-html table {
      border-collapse: collapse;
      background: white;
      color: #111827;
      box-shadow: 0 12px 40px rgba(0, 0, 0, 0.22);
    }
    .sheet-html td,
    .sheet-html th {
      border: 1px solid #d1d5db;
      min-width: 72px;
      padding: 6px 10px;
      vertical-align: top;
    }
  `;

  @property({ attribute: "data-source-url" }) dataSourceUrl = "";
  @property({ attribute: "data-filename" }) dataFilename = "workbook.xlsx";

  @state() private activeSheet = "";
  @state() private renderedHtml = "";
  @state() private sheetNames: string[] = [];
  @state() private status = "";

  private workbook: XLSX.WorkBook | null = null;

  render() {
    return html`
      <div class="toolbar">
        <div class="filename">${this.dataFilename}</div>
        <div class="sheet-tabs">
          ${this.sheetNames.map(
            (sheetName) => html`
              <button
                class=${sheetName === this.activeSheet ? "sheet-tab active" : "sheet-tab"}
                @click=${() => this.selectSheet(sheetName)}
              >
                ${sheetName}
              </button>
            `,
          )}
        </div>
      </div>
      <div class="scroll-area">
        ${
          this.status
            ? html`<div class="status">${this.status}</div>`
            : html`<div class="sheet-html">${unsafeHTML(this.renderedHtml)}</div>`
        }
      </div>
    `;
  }

  firstUpdated() {
    void this.loadWorkbook();
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has("dataSourceUrl")) {
      void this.loadWorkbook();
    }
  }

  private selectSheet(sheetName: string) {
    this.activeSheet = sheetName;
    this.renderCurrentSheet();
  }

  private async loadWorkbook() {
    const src = this.dataSourceUrl;

    if (!src) {
      this.workbook = null;
      this.sheetNames = [];
      this.renderedHtml = "";
      this.status = "No workbook data available.";
      return;
    }

    try {
      this.status = "Loading workbook...";
      const response = await fetch(src);
      const buffer = await response.arrayBuffer();
      const workbook = XLSX.read(buffer, { type: "array" });
      this.workbook = workbook;
      this.sheetNames = workbook.SheetNames;
      this.activeSheet = workbook.SheetNames[0] ?? "";
      this.renderCurrentSheet();
    } catch (error) {
      this.workbook = null;
      this.sheetNames = [];
      this.renderedHtml = "";
      this.status = `Failed to load workbook: ${String(error)}`;
    }
  }

  private renderCurrentSheet() {
    if (!this.workbook || !this.activeSheet) {
      this.renderedHtml = "";
      this.status = "No worksheets found.";
      return;
    }

    const worksheet = this.workbook.Sheets[this.activeSheet];
    const htmlOutput = XLSX.utils.sheet_to_html(worksheet, { id: "omni-sheet" });
    this.renderedHtml = DOMPurify.sanitize(htmlOutput);
    this.status = "";
  }
}
