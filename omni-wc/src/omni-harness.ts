import { LitElement, html, css } from "lit";
import { customElement, property } from "lit/decorators.js";

export type AgentConfig = {
  url: string;
  apiKey: string;
};

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

  private get iframeSrc(): string {
    const url = new URL(this.src, window.location.href);
    url.searchParams.set("theme", this.theme);
    return url.toString();
  }

  private onLoad(e: Event) {
    const iframe = e.target as HTMLIFrameElement;
    iframe.contentWindow?.postMessage({ type: "omni-config", agents: this.agents, theme: this.theme }, "*");
  }

  updated(changedProps: Map<string, unknown>) {
    if (changedProps.has("theme") || changedProps.has("agents")) {
      const iframe = this.shadowRoot?.querySelector("iframe");
      iframe?.contentWindow?.postMessage({ type: "omni-config", agents: this.agents, theme: this.theme }, "*");
    }
  }

  render() {
    return html`<iframe src=${this.iframeSrc} @load=${this.onLoad}></iframe>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "omni-harness": OmniHarness;
  }
}
