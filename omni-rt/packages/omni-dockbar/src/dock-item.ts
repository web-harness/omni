import { LitElement, css, html } from "lit";
import { customElement, property } from "lit/decorators.js";
import anime from "animejs/lib/anime.es.js";

@customElement("dock-item")
export class DockItem extends LitElement {
  @property({ type: Number })
  size = 40;

  @property({ type: String })
  easing = "linear";

  @property({ type: Number })
  scale = 1;

  @property({ type: Number })
  gap = 8;

  @property({ type: String })
  direction = "horizontal";

  private get sizeStyle() {
    return `width: ${this.size}px; height: ${this.size}px;`;
  }

  private get gapStyle() {
    return `--gap: ${this.gap}px`;
  }

  private get liStyle() {
    return `${this.sizeStyle};${this.gapStyle}`;
  }

  private get className() {
    return `${this.direction} dock-item`;
  }

  render() {
    return html`
      <li class=${this.className} style=${this.liStyle}>
        <div class="dock-item__pos">
          <div class="dock-item__scale" style=${this.sizeStyle}>
            <slot></slot>
          </div>
        </div>
      </li>
    `;
  }

  updated(changedProperties: Map<string, unknown>) {
    if (changedProperties.has("scale")) {
      this.onScaleChanged(this.scale);
    }
  }

  private onScaleChanged(scale: number) {
    const sizeEl = this.shadowRoot?.querySelector(".dock-item");
    const scaleEl = this.shadowRoot?.querySelector(".dock-item__scale");
    const isVertical = this.direction === "vertical";
    anime({
      targets: sizeEl,
      width: isVertical ? `${this.size}px` : `${this.size * scale}px`,
      height: isVertical ? `${this.size * scale}px` : `${this.size}px`,
      duration: 100,
      easing: this.easing,
    });
    anime({
      targets: scaleEl,
      scale,
      duration: 100,
      easing: this.easing,
    });
  }

  static styles = css`
    li.dock-item {
      position: relative;
    }
    li.dock-item .dock-item__pos {
      position: absolute;
      left: 50%;
      top: 50%;
      transform: translateX(-50%) translateY(-50%) scale(var(--scale, 1));
    }
    li::before,
    li::after {
      content: "";
      position: absolute;
    }
    li.horizontal::before,
    li.horizontal::after {
      width: var(--gap, 0px);
      height: 100%;
      top: 0;
    }
    li.horizontal::before {
      right: 100%;
    }
    li.horizontal::after {
      left: 100%;
    }
    li.vertical::before,
    li.vertical::after {
      width: 100%;
      height: var(--gap, 0px);
      left: 0;
    }
    li.vertical::before {
      bottom: 100%;
    }
    li.vertical::after {
      top: 100%;
    }
    .dock-item__scale {
      display: flex;
      justify-content: center;
      align-items: center;
    }
    .dock-item__scale > ::slotted(*) {
      width: 100%;
      height: 100%;
    }
  `;
}

declare global {
  interface HTMLElementTagNameMap {
    "dock-item": DockItem;
  }
}
