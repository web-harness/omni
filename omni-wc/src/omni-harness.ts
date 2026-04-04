import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";

export type AgentConfig = {
  url: string;
  apiKey: string;
};

const ALLOWED_DICEBEAR_STYLES = new Set(["bottts-neutral", "thumbs"]);
const IFRAME_BOOTSTRAP_EVENT = "omni-iframe-config";
const IFRAME_READY_EVENT = "omni-iframe-ready";

@customElement("omni-harness")
export class OmniHarness extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      height: 100%;
    }
    iframe {
      border: none;
      width: 100%;
      height: 100%;
      display: block;
    }
  `;

  @property({ type: Array }) agents: AgentConfig[] = [];
  @property({ type: String }) src = "http://127.0.0.1:8080";
  @property({ type: String }) theme: "light" | "dark" = "dark";
  @property({ type: String }) dicebearStyle = "bottts-neutral";

  private sendBootstrap(): void {
    const iframe = this.renderRoot.querySelector("iframe");
    iframe?.contentWindow?.postMessage(this.bootstrapPayload, this.iframeOrigin);
  }

  private readonly handleFrameLoad = () => {
    this.sendBootstrap();
  };

  private readonly handleWindowMessage = (event: MessageEvent) => {
    if (event.origin !== this.iframeOrigin || typeof event.data !== "string") {
      return;
    }

    let envelope: { type?: string } | null = null;
    try {
      envelope = JSON.parse(event.data) as { type?: string };
    } catch {
      return;
    }

    if (envelope?.type !== IFRAME_READY_EVENT) {
      return;
    }

    this.sendBootstrap();
  };

  private get iframeSrc(): string {
    return new URL(this.src, window.location.href).toString();
  }

  private get bootstrapPayload(): string {
    return JSON.stringify({
      type: IFRAME_BOOTSTRAP_EVENT,
      payload: {
        theme: this.theme,
        dicebearStyle: ALLOWED_DICEBEAR_STYLES.has(this.dicebearStyle) ? this.dicebearStyle : "bottts-neutral",
        agents: this.agents,
      },
    });
  }

  private get iframeOrigin(): string {
    return new URL(this.src, window.location.href).origin;
  }

  connectedCallback(): void {
    super.connectedCallback();
    window.addEventListener("message", this.handleWindowMessage);
  }

  disconnectedCallback(): void {
    window.removeEventListener("message", this.handleWindowMessage);
    super.disconnectedCallback();
  }

  protected updated(changedProperties: Map<PropertyKey, unknown>): void {
    if (
      !changedProperties.has("src") &&
      (changedProperties.has("theme") || changedProperties.has("dicebearStyle") || changedProperties.has("agents"))
    ) {
      this.sendBootstrap();
    }
  }

  render() {
    return html`<iframe src=${this.iframeSrc} @load=${this.handleFrameLoad}></iframe>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "omni-harness": OmniHarness;
  }
}
