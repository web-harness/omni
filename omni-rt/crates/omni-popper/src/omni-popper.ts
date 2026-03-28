import { createPopper, Instance, Placement } from "@popperjs/core";
import { LitElement, html, css, type PropertyValues } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("omni-popper")
class OmniPopper extends LitElement {
  static styles = css`
    :host { display: inline-flex; position: relative; }
    .popper-content { display: none; z-index: 110; }
    .popper-content[data-show] { display: block; }
  `;

  @property({ type: Boolean, reflect: true }) open = false;
  @property({ type: String }) placement: Placement = "bottom-start";
  @property({ type: String }) offset = "0,8";
  @property({ type: String }) strategy: "fixed" | "absolute" = "fixed";

  private popperInstance: Instance | null = null;
  private clickAway: ((e: MouseEvent) => void) | null = null;
  private keyUp: ((e: KeyboardEvent) => void) | null = null;

  render() {
    return html`
      <slot name="trigger"></slot>
      <div class="popper-content">
        <slot name="content"></slot>
      </div>
    `;
  }

  updated(changedProps: PropertyValues) {
    if (changedProps.has("open")) {
      if (this.open) {
        this.show();
      } else {
        this.hide();
      }
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.cleanup();
  }

  private get triggerEl(): Element | null {
    const slot = this.shadowRoot?.querySelector<HTMLSlotElement>('slot[name="trigger"]');
    return slot?.assignedElements()[0] ?? null;
  }

  private get contentEl(): HTMLElement | null {
    return this.shadowRoot?.querySelector(".popper-content") ?? null;
  }

  private show() {
    if (!this.contentEl || !this.triggerEl) return;
    this.contentEl.setAttribute("data-show", "");

    const [ox, oy] = this.offset.split(",").map(Number);

    if (this.popperInstance) {
      this.popperInstance.destroy();
    }
    this.popperInstance = createPopper(this.triggerEl as HTMLElement, this.contentEl, {
      placement: this.placement,
      strategy: this.strategy,
      modifiers: [
        { name: "offset", options: { offset: [ox, oy] } },
        { name: "flip", enabled: true },
        { name: "preventOverflow", options: { padding: 8 } },
      ],
    });

    this.clickAway = (e: MouseEvent) => {
      if (!this.contains(e.target as Node)) {
        this.dispatchEvent(new CustomEvent("popper-close", { bubbles: true }));
      }
    };
    this.keyUp = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        this.dispatchEvent(new CustomEvent("popper-close", { bubbles: true }));
      }
    };
    setTimeout(() => {
      document.addEventListener("click", this.clickAway!);
      document.addEventListener("keyup", this.keyUp!);
    }, 0);
  }

  private hide() {
    this.contentEl?.removeAttribute("data-show");
    this.cleanup();
  }

  private cleanup() {
    this.popperInstance?.destroy();
    this.popperInstance = null;
    if (this.clickAway) document.removeEventListener("click", this.clickAway);
    if (this.keyUp) document.removeEventListener("keyup", this.keyUp);
    this.clickAway = null;
    this.keyUp = null;
  }
}

if (!customElements.get("omni-popper")) {
  customElements.define("omni-popper", OmniPopper);
}
