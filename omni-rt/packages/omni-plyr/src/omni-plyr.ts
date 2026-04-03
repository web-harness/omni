import plyrCss from "plyr/dist/plyr.css";
import plyrSvg from "plyr/dist/plyr.svg";
import { LitElement, html, unsafeCSS } from "lit";
import { createCachedLoader } from "@omni/omni-util/async-loader";
import { customElement, property } from "lit/decorators.js";
import { unsafeHTML } from "lit/directives/unsafe-html.js";

import type Plyr from "plyr";

type PlyrModule = typeof import("plyr");

const loadPlyrModule = createCachedLoader(() => import("plyr"));

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

  @property({ attribute: "data-source-url" }) dataSourceUrl = "";
  @property({ attribute: "data-mime" }) dataMime = "";
  @property({ attribute: "data-type" }) dataType: "video" | "audio" = "video";
  private player: Plyr | null = null;
  private initVersion = 0;

  updated(changed: Map<string, unknown>) {
    if (changed.has("dataSourceUrl") || changed.has("dataType")) {
      void this.initPlayer();
    }
  }

  private async initPlayer() {
    if (this.player) {
      this.player.destroy();
      this.player = null;
    }
    const src = this.dataSourceUrl;
    if (!src) return;
    const initVersion = ++this.initVersion;
    const plyr = await loadPlyrModule();
    if (initVersion !== this.initVersion || !this.isConnected) return;
    const el = this.shadowRoot?.querySelector<HTMLVideoElement | HTMLAudioElement>(
      this.dataType === "video" ? "video" : "audio",
    );
    if (!el) return;
    el.src = src;
    this.player = new plyr.default(el, {
      loadSprite: false,
    });
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.player?.destroy();
    this.player = null;
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
