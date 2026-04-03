import { LitElement, css, html } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { createCachedLoader } from "@omni/omni-util/async-loader";

import type { PptxViewer as PptxViewerInstance } from "@aiden0z/pptx-renderer";

type PptxRendererModule = typeof import("@aiden0z/pptx-renderer");

const loadPptxRendererModule = createCachedLoader<PptxRendererModule>(() => import("@aiden0z/pptx-renderer"));

@customElement("omni-pptx-renderer")
export class OmniPptxRenderer extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      height: 100%;
      min-height: 0;
      background: var(--background, #11111b);
      color: var(--foreground, #cdd6f4);
    }

    .viewer {
      width: 100%;
      height: 100%;
      overflow: auto;
      background: #111827;
    }

    .status {
      padding: 16px;
      font-size: 12px;
      color: var(--muted-foreground, #9399b2);
    }
  `;

  @property({ attribute: "data-source-url" }) dataSourceUrl = "";
  @property({ attribute: "data-filename" }) dataFilename = "presentation.pptx";

  @state() private status = "";

  private viewer: PptxViewerInstance | null = null;
  private loadVersion = 0;
  private loadAbortController: AbortController | null = null;

  render() {
    return html`
      ${this.status ? html`<div class="status">${this.status}</div>` : ""}
      <div id="viewer" class="viewer" aria-label=${this.dataFilename}></div>
    `;
  }

  firstUpdated() {
    void this.loadPresentation();
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has("dataSourceUrl") || changed.has("dataFilename")) {
      void this.loadPresentation();
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.cleanupViewer();
  }

  private getContainer() {
    return this.shadowRoot?.querySelector<HTMLDivElement>("#viewer") ?? null;
  }

  private cleanupViewer() {
    this.loadAbortController?.abort();
    this.loadAbortController = null;
    this.viewer?.destroy();
    this.viewer = null;

    const container = this.getContainer();
    container?.replaceChildren();
  }

  private async loadPresentation() {
    const src = this.dataSourceUrl;
    const container = this.getContainer();

    if (!container) {
      return;
    }

    const loadVersion = ++this.loadVersion;
    this.cleanupViewer();

    if (!src) {
      this.status = "No presentation data available.";
      return;
    }

    const abortController = new AbortController();
    this.loadAbortController = abortController;
    this.status = "Loading…";

    try {
      const response = await fetch(src, { signal: abortController.signal });
      if (!response.ok) {
        throw new Error(`Failed to load presentation: ${response.status} ${response.statusText}`);
      }

      const buffer = await response.arrayBuffer();
      const { PptxViewer } = await loadPptxRendererModule();

      if (loadVersion !== this.loadVersion || abortController.signal.aborted || !this.isConnected) {
        return;
      }

      const viewer = await PptxViewer.open(buffer, container, {
        renderMode: "list",
        fitMode: "contain",
        scrollContainer: container,
        listOptions: {
          windowed: true,
          batchSize: 8,
          initialSlides: 4,
          overscanViewport: 1.5,
        },
        signal: abortController.signal,
      });

      if (loadVersion !== this.loadVersion || abortController.signal.aborted || !this.isConnected) {
        viewer.destroy();
        return;
      }

      this.viewer = viewer;
      this.status = "";
    } catch (error) {
      if (abortController.signal.aborted) {
        return;
      }

      container.replaceChildren();
      this.status = `Failed to load presentation renderer: ${String(error)}`;
    } finally {
      if (this.loadAbortController === abortController) {
        this.loadAbortController = null;
      }
    }
  }
}
