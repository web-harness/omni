import { createPopper, Instance, Placement } from "@popperjs/core";

class OmniPopper extends HTMLElement {
  private popperInstance: Instance | null = null;
  private contentEl: HTMLElement | null = null;
  private clickAway: ((e: MouseEvent) => void) | null = null;
  private keyUp: ((e: KeyboardEvent) => void) | null = null;

  static get observedAttributes() {
    return ["open", "placement", "offset", "strategy"];
  }

  connectedCallback() {
    this.attachShadow({ mode: "open" });
    this.shadowRoot!.innerHTML = `
      <style>
        :host { display: inline-flex; position: relative; }
        .popper-content { display: none; z-index: 110; }
        .popper-content[data-show] { display: block; }
      </style>
      <slot name="trigger"></slot>
      <div class="popper-content">
        <slot name="content"></slot>
      </div>
    `;
    this.contentEl = this.shadowRoot!.querySelector(".popper-content");
    this.syncOpen();
  }

  disconnectedCallback() {
    this.cleanup();
  }

  attributeChangedCallback() {
    if (this.shadowRoot) this.syncOpen();
  }

  private get triggerEl(): Element | null {
    const slot = this.shadowRoot?.querySelector<HTMLSlotElement>('slot[name="trigger"]');
    return slot?.assignedElements()[0] ?? null;
  }

  private syncOpen() {
    const isOpen = this.hasAttribute("open") && this.getAttribute("open") !== "";
    if (isOpen) {
      this.show();
    } else {
      this.hide();
    }
  }

  private show() {
    if (!this.contentEl || !this.triggerEl) return;
    this.contentEl.setAttribute("data-show", "");

    const placement = (this.getAttribute("placement") ?? "bottom-start") as Placement;
    const strategy = (this.getAttribute("strategy") ?? "fixed") as "fixed" | "absolute";
    const [ox, oy] = (this.getAttribute("offset") ?? "0,8").split(",").map(Number);

    if (this.popperInstance) {
      this.popperInstance.destroy();
    }
    this.popperInstance = createPopper(this.triggerEl as HTMLElement, this.contentEl, {
      placement,
      strategy,
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

customElements.define("omni-popper", OmniPopper);
