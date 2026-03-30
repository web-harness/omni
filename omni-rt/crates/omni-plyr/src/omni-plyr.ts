import Plyr from "plyr";
import plyrCss from "plyr/dist/plyr.css";
import plyrSvg from "../node_modules/plyr/dist/plyr.svg";
import { LitElement, html, unsafeCSS } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { unsafeHTML } from "lit/directives/unsafe-html.js";

@customElement("omni-plyr")
export class OmniPlyr extends LitElement {
  static styles = [
    unsafeCSS(plyrCss),
    unsafeCSS(`
      :host {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        height: 100%;
        background: var(--background, #0f0f15);
      }
      .plyr-container {
        width: 100%;
        max-width: 900px;
      }
      .plyr--video {
        --plyr-color-main: #7c6af7;
        --plyr-video-background: #000;
      }
      .plyr--audio {
        --plyr-color-main: #7c6af7;
        --plyr-audio-controls-background: var(--sidebar, #1e1e2e);
        --plyr-audio-control-color: var(--foreground, #cdd6f4);
      }
    `),
  ];

  @property({ attribute: "data-base64" }) dataBase64 = "";
  @property({ attribute: "data-mime" }) dataMime = "";
  @property({ attribute: "data-type" }) dataType: "video" | "audio" = "video";
  @property({ attribute: "data-src" }) dataSrc = "";

  @state() private blobUrl = "";
  private player: Plyr | null = null;
  private pendingBase64 = "";
  private pendingMime = "";

  updated(changed: Map<string, unknown>) {
    if (changed.has("dataBase64") && this.dataBase64) {
      this.pendingBase64 = this.dataBase64;
      this.pendingMime = this.dataMime;
      this.removeAttribute("data-base64");
      return;
    }

    if (changed.has("dataBase64") && !this.dataBase64 && this.pendingBase64 && !this.dataSrc) {
      const oldUrl = this.blobUrl;
      const bytes = Uint8Array.from(atob(this.pendingBase64), (c) => c.charCodeAt(0));
      const blob = new Blob([bytes], { type: this.pendingMime });
      this.blobUrl = URL.createObjectURL(blob);
      this.pendingBase64 = "";
      if (oldUrl) URL.revokeObjectURL(oldUrl);
    }

    if (changed.has("blobUrl") || changed.has("dataSrc")) {
      this.initPlayer();
    }
  }

  private initPlayer() {
    if (this.player) {
      this.player.destroy();
      this.player = null;
    }
    const src = this.dataSrc || this.blobUrl;
    if (!src) return;
    const el = this.shadowRoot?.querySelector<HTMLVideoElement | HTMLAudioElement>(
      this.dataType === "video" ? "video" : "audio",
    );
    if (!el) return;
    el.src = src;
    this.player = new Plyr(el, {
      loadSprite: false,
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.player?.destroy();
    this.player = null;
    if (this.blobUrl) {
      URL.revokeObjectURL(this.blobUrl);
      this.blobUrl = "";
    }
  }

  render() {
    return html`
      <div class="plyr-container" style="display:none">${unsafeHTML(plyrSvg)}</div>
      <div class="plyr-container">
        ${this.dataType === "video" ? html`<video playsinline controls></video>` : html`<audio controls></audio>`}
      </div>
    `;
  }
}
