import { LitElement, css, html } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { renderAsync } from "docx-preview";

@customElement("omni-docxjs")
export class OmniDocxjs extends LitElement {
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
      border-bottom: 1px solid var(--border, #313244);
      background: var(--sidebar, #1e1e2e);
      color: var(--muted-foreground, #9399b2);
      font-size: 11px;
      padding: 8px 12px;
      flex-shrink: 0;
    }
    .viewer {
      flex: 1;
      min-height: 0;
      overflow: auto;
      padding: 16px;
    }
    .status {
      font-size: 12px;
      color: var(--muted-foreground, #9399b2);
      padding: 12px 0;
    }
    #docx-body {
      min-height: 100%;
    }
  `;

  @property({ attribute: "data-source-url" }) dataSourceUrl = "";
  @property({ attribute: "data-filename" }) dataFilename = "document.docx";

  @state() private status = "";

  private renderVersion = 0;
  private resizeObserver: ResizeObserver | null = null;
  private ignoreDocumentWidth = false;

  render() {
    return html`
      <div class="toolbar">${this.dataFilename}</div>
      <div class="viewer">
        ${this.status ? html`<div class="status">${this.status}</div>` : null}
        <div id="docx-body"></div>
      </div>
    `;
  }

  firstUpdated() {
    const viewer = this.getViewer();
    if (viewer) {
      this.resizeObserver = new ResizeObserver(() => {
        const nextIgnoreWidth = this.shouldIgnoreDocumentWidth();
        if (nextIgnoreWidth !== this.ignoreDocumentWidth && this.dataSourceUrl) {
          void this.renderDocument();
        }
      });
      this.resizeObserver.observe(viewer);
    }

    void this.renderDocument();
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has("dataSourceUrl")) {
      void this.renderDocument();
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.resizeObserver?.disconnect();
    this.resizeObserver = null;
  }

  private getViewer() {
    return this.shadowRoot?.querySelector<HTMLElement>(".viewer") ?? null;
  }

  private shouldIgnoreDocumentWidth() {
    const viewer = this.getViewer();
    if (!viewer) {
      return false;
    }

    const styles = getComputedStyle(viewer);
    const horizontalPadding = parseFloat(styles.paddingLeft || "0") + parseFloat(styles.paddingRight || "0");
    const availableWidth = viewer.clientWidth - horizontalPadding;

    return availableWidth > 0 && availableWidth < 860;
  }

  private async renderDocument() {
    const version = ++this.renderVersion;
    const bodyContainer = this.shadowRoot?.querySelector<HTMLElement>("#docx-body");
    if (!bodyContainer) return;
    const src = this.dataSourceUrl;
    const ignoreWidth = this.shouldIgnoreDocumentWidth();

    this.ignoreDocumentWidth = ignoreWidth;

    bodyContainer.innerHTML = "";

    if (!src) {
      this.status = "No document data available.";
      return;
    }

    try {
      this.status = "Loading document...";
      const response = await fetch(src);
      if (!response.ok) {
        throw new Error(`Failed to load document: ${response.status} ${response.statusText}`);
      }
      const blob = await response.blob();
      await renderAsync(blob, bodyContainer, bodyContainer, {
        breakPages: true,
        inWrapper: true,
        ignoreWidth,
      });
      if (version !== this.renderVersion) {
        return;
      }
      this.status = "";
    } catch (error) {
      if (version !== this.renderVersion) {
        return;
      }
      this.status = `Failed to load document: ${String(error)}`;
    }
  }
}
