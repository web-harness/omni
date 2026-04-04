import { createAvatar } from "@dicebear/core";
import { LitElement, css, html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { unsafeHTML } from "lit/directives/unsafe-html.js";

const styleLoaders: Record<string, () => Promise<{ default: unknown } | unknown>> = {
  "bottts-neutral": () => import("@dicebear/bottts-neutral"),
  thumbs: () => import("@dicebear/thumbs"),
};

const DEFAULT_STYLE = "bottts-neutral";

const styleCache = new Map<string, unknown>();

@customElement("omni-dicebear")
export class OmniDicebear extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      height: 100%;
    }

    .avatar {
      width: 100%;
      height: 100%;
    }
  `;

  @property({ type: String }) seed = "";
  @property({ type: String, attribute: "avatar-style" }) avatarStyle = DEFAULT_STYLE;
  @property({ type: Number }) size = 36;

  private svgContent = "";

  connectedCallback(): void {
    super.connectedCallback();
    void this.generateAvatar();
  }

  async updated(changedProps: Map<string, unknown>): Promise<void> {
    if (changedProps.has("seed") || changedProps.has("avatarStyle") || changedProps.has("size")) {
      await this.generateAvatar();
    }
  }

  private async generateAvatar(): Promise<void> {
    const style = this.avatarStyle in styleLoaders ? this.avatarStyle : DEFAULT_STYLE;
    const loader = styleLoaders[style];
    if (!this.seed) {
      this.svgContent = "";
      return;
    }

    let styleDef = styleCache.get(style);
    if (!styleDef) {
      const mod = await loader();
      styleDef = (mod as { default?: unknown }).default ?? mod;
      styleCache.set(style, styleDef);
    }

    const avatar = createAvatar(styleDef as Parameters<typeof createAvatar>[0], {
      seed: this.seed,
      size: this.size,
      randomizeIds: true,
    });
    this.svgContent = avatar.toString();
    this.requestUpdate();
  }

  render() {
    return html`<div class="avatar">${unsafeHTML(this.svgContent)}</div>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "omni-dicebear": OmniDicebear;
  }
}
