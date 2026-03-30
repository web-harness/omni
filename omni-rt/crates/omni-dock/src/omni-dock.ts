import cssText from "dockview-core/dist/styles/dockview.css";
import {
  createDockview,
  themeAbyss,
  type DockviewApi,
  type GroupPanelPartInitParameters,
  type IContentRenderer,
  type AddPanelOptions,
} from "dockview-core";
import { LitElement, html, css, unsafeCSS, type PropertyValues } from "lit";
import { customElement, property } from "lit/decorators.js";

type PanelSpec = {
  id: string;
  slot: string;
  title?: string;
  hideHeader?: boolean;
  position?: {
    referencePanel?: string;
    direction?: "left" | "right" | "above" | "below" | "within";
  };
};

class SlotPanel implements IContentRenderer {
  private readonly container: HTMLDivElement;

  constructor(slotName: string) {
    this.container = document.createElement("div");
    this.container.style.cssText = "position:relative;width:100%;height:100%;overflow:hidden;";
    const slot = document.createElement("slot");
    slot.name = slotName;
    this.container.appendChild(slot);
  }

  get element(): HTMLElement {
    return this.container;
  }

  init(_parameters: GroupPanelPartInitParameters): void {}

  dispose(): void {}
}

@customElement("omni-dock")
class OmniDock extends LitElement {
  static styles = css`
    ${unsafeCSS(cssText)}
    :host {
      display: block;
      width: 100%;
      height: 100%;
    }
    .dock-root {
      width: 100%;
      height: 100%;
    }
    .dockview-theme-abyss {
      --dv-background-color: var(--background);
      --dv-group-view-background-color: var(--background-elevated);
      --dv-tabs-and-actions-container-background-color: var(--sidebar);
      --dv-activegroup-visiblepanel-tab-background-color: var(--background-elevated);
      --dv-inactivegroup-visiblepanel-tab-background-color: var(--background-elevated);
      --dv-activegroup-hiddenpanel-tab-background-color: var(--sidebar);
      --dv-inactivegroup-hiddenpanel-tab-background-color: var(--sidebar);
      --dv-activegroup-visiblepanel-tab-border-color: var(--border);
      --dv-separator-border: var(--border);
      --dv-tab-divider-color: var(--border);
      --dv-sash-hover-background-color: var(--border);
    }
    .dv-sash {
      border-color: var(--border) !important;
    }
    .dv-default-tab-content {
      color: var(--foreground);
    }
    .dv-tab[data-permanent] .dv-default-tab-action {
      display: none !important;
    }
    .dv-sash-container {
      pointer-events: none;
    }
    .dv-sash-container > .dv-sash {
      pointer-events: auto;
    }
    ::slotted(*) {
      position: absolute;
      inset: 0;
      overflow: auto;
    }
  `;

  @property({ attribute: "data-panels" }) dataPanels = "";
  @property({ attribute: "data-active-panel" }) dataActivePanel = "";
  @property({ attribute: "data-proportions" }) dataProportions = "";

  private api: DockviewApi | null = null;
  private slotObserver: MutationObserver | null = null;
  private tabObserver: MutationObserver | null = null;
  private knownSlots: Set<string> = new Set();
  private permanentPanels: Set<string> = new Set();
  private closedByUser: Set<string> = new Set();

  render() {
    return html`<div class="dock-root"></div>`;
  }

  firstUpdated() {
    const container = this.shadowRoot!.querySelector<HTMLElement>(".dock-root")!;
    this.api = createDockview(container, {
      theme: themeAbyss,
      createComponent: (options) => new SlotPanel(options.name),
    });
    this.tabObserver = new MutationObserver(() => this.tagPermanentTabs());
    this.tabObserver.observe(container, { childList: true, subtree: true });
    this.initializePanels();
    this.observeSlots();
    this.api.onDidRemovePanel((panel) => {
      if (!this.permanentPanels.has(panel.id)) {
        this.knownSlots.delete(panel.id);
        this.closedByUser.add(panel.id);
      }
    });
  }

  updated(changedProps: PropertyValues) {
    if (changedProps.has("dataPanels") && this.api) {
      this.resetPanels();
      this.initializePanels();
    }
    if (changedProps.has("dataActivePanel") && this.api && this.dataActivePanel) {
      this.api.getPanel(this.dataActivePanel)?.api.setActive();
    }
    if (changedProps.has("dataProportions") && this.api) {
      this.applyProportions();
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.slotObserver?.disconnect();
    this.slotObserver = null;
    this.tabObserver?.disconnect();
    this.tabObserver = null;
    this.api?.dispose();
    this.api = null;
    this.knownSlots.clear();
  }

  private resetPanels(): void {
    if (!this.api) return;
    this.api.dispose();
    this.api = null;
    this.knownSlots.clear();
    const container = this.shadowRoot?.querySelector<HTMLElement>(".dock-root");
    if (container) {
      this.api = createDockview(container, {
        theme: themeAbyss,
        createComponent: (options) => new SlotPanel(options.name),
      });
    }
  }

  private parsePanelSpecs(): PanelSpec[] {
    if (!this.dataPanels) return [];
    try {
      const parsed = JSON.parse(this.dataPanels) as PanelSpec[];
      return Array.isArray(parsed) ? parsed : [];
    } catch {
      return [];
    }
  }

  private addPanel(spec: PanelSpec, permanent = false): void {
    if (!this.api || this.knownSlots.has(spec.slot)) return;

    const options: AddPanelOptions = {
      id: spec.id,
      component: spec.slot,
      title: spec.title ?? spec.id,
    };

    if (spec.position?.direction && spec.position.referencePanel) {
      options.position = {
        direction: spec.position.direction,
        referencePanel: spec.position.referencePanel,
      } as AddPanelOptions["position"];
    } else if (spec.position?.direction) {
      options.position = {
        direction: spec.position.direction,
      } as AddPanelOptions["position"];
    }

    this.api.addPanel(options);
    this.knownSlots.add(spec.slot);
    if (permanent) this.permanentPanels.add(spec.id);

    if (spec.hideHeader) {
      const group = this.api.groups.find((g) => g.panels.some((p) => p.id === spec.id));
      if (group) {
        (group.model as any).header.hidden = true;
        group.locked = true;
      }
    }
  }

  private initializePanels(): void {
    for (const spec of this.parsePanelSpecs()) {
      this.addPanel(spec, true);
    }
    requestAnimationFrame(() => {
      this.applyProportions();
      this.tagPermanentTabs();
      if (this.dataActivePanel) {
        this.api?.getPanel(this.dataActivePanel)?.api.setActive();
      }
    });
  }

  private tagPermanentTabs(): void {
    if (!this.shadowRoot || !this.api) return;
    const permanentTitles = new Set<string>();
    for (const id of this.permanentPanels) {
      const panel = this.api.getPanel(id);
      if (panel?.title) permanentTitles.add(panel.title);
    }
    this.shadowRoot.querySelectorAll(".dv-tab:not([data-permanent])").forEach((tab) => {
      const title = tab.querySelector(".dv-default-tab-content")?.textContent?.trim();
      if (title && permanentTitles.has(title)) {
        tab.setAttribute("data-permanent", "");
      }
    });
  }

  private applyProportions(): void {
    if (!this.api) return;
    const propStr = this.dataProportions;
    if (!propStr) return;

    const proportions = propStr.split(",").map(Number);
    const w = this.clientWidth || window.innerWidth;
    const h = this.clientHeight || window.innerHeight;
    this.api.layout(w, h);

    const json = (this.api as any).toJSON();
    const rootChildren = json?.grid?.root?.data;
    if (!rootChildren || proportions.length !== rootChildren.length) return;

    const totalWidth = json.grid.width;
    const totalProp = proportions.reduce((a: number, b: number) => a + b, 0);
    for (let i = 0; i < rootChildren.length; i++) {
      rootChildren[i].size = Math.round((totalWidth * proportions[i]) / totalProp);
    }
    (this.api as any).fromJSON(json);
  }

  private observeSlots(): void {
    if (!this.api) return;

    this.slotObserver = new MutationObserver(() => {
      const slotted = Array.from(this.children)
        .map((el) => el.getAttribute("slot"))
        .filter((s): s is string => !!s);

      for (const slot of slotted) {
        if (!this.knownSlots.has(slot) && !this.closedByUser.has(slot)) {
          this.addPanel({
            id: slot,
            slot,
            title: slot,
            position: { referencePanel: "chat", direction: "within" },
          });
        } else if (!this.permanentPanels.has(slot) && !this.api?.getPanel(slot) && !this.closedByUser.has(slot)) {
          // Panel was removed from dockview (user closed it) but Dioxus slot still exists.
          // Mark as closed so future mutations don't re-add it.
          this.knownSlots.delete(slot);
          this.closedByUser.add(slot);
        }
      }

      for (const existing of [...this.knownSlots]) {
        if (!slotted.includes(existing)) {
          this.api?.getPanel(existing)?.dispose();
          this.knownSlots.delete(existing);
        }
      }
      for (const closed of [...this.closedByUser]) {
        if (!slotted.includes(closed)) {
          this.closedByUser.delete(closed);
        }
      }
    });

    this.slotObserver.observe(this, {
      childList: true,
      subtree: false,
      attributes: true,
      attributeFilter: ["slot"],
    });
  }
}

if (!customElements.get("omni-dock")) {
  customElements.define("omni-dock", OmniDock);
}
