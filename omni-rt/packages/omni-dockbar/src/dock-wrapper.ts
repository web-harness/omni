import { LitElement, css, html } from "lit";
import { customElement, property } from "lit/decorators.js";

export type DockPosition = "top" | "right" | "bottom" | "left";
export type DockDirection = "horizontal" | "vertical";

@customElement("dock-wrapper")
export class DockWrapper extends LitElement {
  private _ready = false;
  private _children: Element[] = [];

  @property({ type: Boolean })
  disabled = false;

  @property({ type: Number, attribute: "max-range" })
  maxRange = 200;

  @property({ type: Number, attribute: "max-scale" })
  maxScale = 2;

  @property({ type: String })
  position: DockPosition = "bottom";

  @property({ type: String })
  direction: DockDirection = "horizontal";

  @property({ type: Number })
  size = 40;

  @property({ type: Number })
  padding = 8;

  @property({ type: Number })
  gap = 5;

  @property({ type: String })
  easing = "cubicBezier(0, 0.55, 0.45, 1)";

  private onSlotChange(e: Event) {
    const slot = e.target as HTMLSlotElement;
    const nodes = slot.assignedNodes({ flatten: true });
    this._children = nodes.filter(
      (node): node is Element => node.nodeType === Node.ELEMENT_NODE && node.nodeName.toUpperCase() === "DOCK-ITEM",
    );
    this.observe();
    this.provideSharedProps();
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this._ready = false;
  }

  private observe() {
    if (this._ready) return;
    this._ready = true;
    const host = this.shadowRoot?.host as HTMLElement | undefined;
    if (!host) return;
    host.addEventListener("mousemove", this.handleMousemove);
    host.addEventListener("mouseleave", this.handleMouseleave);
  }

  private resetAll() {
    for (const child of this._children) {
      child.setAttribute("scale", "1");
    }
  }

  private handleMouseleave = () => {
    this.resetAll();
  };

  private handleMousemove = (e: Event) => {
    const { clientX, clientY } = e as MouseEvent;
    if (this.disabled) return;

    for (const child of this._children) {
      const childRect = child.getBoundingClientRect();
      const center =
        this.direction === "horizontal" ? childRect.left + childRect.width / 2 : childRect.top + childRect.height / 2;
      const distance = Math.abs((this.direction === "horizontal" ? clientX : clientY) - center);
      const scale = distance > this.maxRange ? 1 : 1 + (this.maxScale - 1) * (1 - distance / this.maxRange);
      child.setAttribute("scale", `${scale}`);
    }
  };

  private get wrapperStyle() {
    const hOrW = this.direction === "horizontal" ? "height" : "width";
    const styles: Record<string, string> = {
      "--gap": `${this.gap}px`,
      "--size": `${this.size}px`,
      padding: `${this.padding}px`,
      [hOrW]: `${this.padding * 2 + this.size}px`,
    };
    return Object.entries(styles)
      .map(([key, value]) => `${key}: ${value}`)
      .join(";");
  }

  private get className() {
    return ["dock-wrapper", this.position, this.direction].join(" ");
  }

  render() {
    return html`
      <ul class=${this.className} style="${this.wrapperStyle}">
        <slot @slotchange=${this.onSlotChange}></slot>
      </ul>
    `;
  }

  private provideSharedProps() {
    for (const el of this._children) {
      el.setAttribute("size", `${this.size}`);
      el.setAttribute("easing", `${this.easing}`);
      el.setAttribute("gap", `${this.gap}`);
      el.setAttribute("direction", `${this.direction}`);
    }
  }

  updated(changedProperties: Map<string, unknown>) {
    if (["size", "gap", "easing", "direction"].some((key) => changedProperties.has(key))) {
      this.provideSharedProps();
    }
  }

  static styles = css`
    ul.dock-wrapper {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
      display: flex;
      flex-wrap: nowrap;
      align-items: center;
      list-style: none;
      gap: var(--gap, 5px);
      border-radius: inherit;
    }
    ul.dock-wrapper.horizontal.bottom {
      align-items: flex-end;
    }
    ul.dock-wrapper.horizontal.top {
      align-items: flex-start;
    }
    ul.dock-wrapper.vertical.left {
      align-items: flex-start;
    }
    ul.dock-wrapper.vertical.right {
      align-items: flex-end;
    }
    ul.dock-wrapper.left,
    ul.dock-wrapper.right {
      flex-direction: column;
    }
    ul.dock-wrapper.top,
    ul.dock-wrapper.bottom {
      flex-direction: row;
    }
    ul.dock-wrapper.horizontal {
      flex-direction: row;
      max-width: 80vw;
    }
    ul.dock-wrapper.vertical {
      flex-direction: column;
      max-height: 90vh;
    }
    ul.dock-wrapper.horizontal.overflowed {
      overflow-x: auto;
    }
    ul.dock-wrapper.vertical.overflowed {
      overflow-y: auto;
    }
  `;
}

declare global {
  interface HTMLElementTagNameMap {
    "dock-wrapper": DockWrapper;
  }
}
