import cssText from "dockview-core/dist/styles/dockview.css";
import {
  createDockview,
  themeAbyss,
  type DockviewApi,
  type GroupPanelPartInitParameters,
  type IContentRenderer,
  type AddPanelOptions,
} from "dockview-core";

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

class OmniDock extends HTMLElement {
  static observedAttributes = ["data-panels", "data-active-panel", "data-proportions"];

  private api: DockviewApi | null = null;
  private shadowRootRef: ShadowRoot | null = null;
  private observer: MutationObserver | null = null;
  private knownSlots = new Set<string>();
  private permanentPanels = new Set<string>();

  connectedCallback(): void {
    if (this.api) {
      return;
    }

    const shadow = this.attachShadow({ mode: "open" });
    this.shadowRootRef = shadow;

    const style = document.createElement("style");
    style.textContent =
      cssText +
      `
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
        --dv-activegroup-visiblepanel-tab-background-color: var(--sidebar);
        --dv-inactivegroup-visiblepanel-tab-background-color: var(--sidebar);
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
    shadow.appendChild(style);

    const container = document.createElement("div");
    container.className = "dock-root";
    shadow.appendChild(container);

    this.api = createDockview(container, {
      theme: themeAbyss,
      createComponent: (options) => new SlotPanel(options.name),
    });

    this.initializePanels();
    this.observeSlots();
  }

  disconnectedCallback(): void {
    this.observer?.disconnect();
    this.observer = null;
    this.api?.dispose();
    this.api = null;
    this.knownSlots.clear();
  }

  attributeChangedCallback(name: string, oldValue: string | null, newValue: string | null): void {
    if (name === "data-panels" && this.api && newValue !== oldValue) {
      this.resetPanels();
      this.initializePanels();
    } else if (name === "data-active-panel" && this.api) {
      const panelId = this.getAttribute("data-active-panel");
      if (panelId) {
        const panel = this.api.getPanel(panelId);
        panel?.api.setActive();
      }
    } else if (name === "data-proportions" && this.api && newValue !== oldValue) {
      this.applyProportions();
    }
  }

  private resetPanels(): void {
    if (!this.api) {
      return;
    }
    this.api.dispose();
    this.api = null;
    this.knownSlots.clear();
    const container = this.shadowRootRef?.querySelector<HTMLElement>(".dock-root");
    if (container) {
      this.api = createDockview(container, {
        theme: themeAbyss,
        createComponent: (options) => new SlotPanel(options.name),
      });
    }
  }

  private parsePanelSpecs(): PanelSpec[] {
    const raw = this.getAttribute("data-panels");
    if (!raw) {
      return [];
    }

    try {
      const parsed = JSON.parse(raw) as PanelSpec[];
      return Array.isArray(parsed) ? parsed : [];
    } catch {
      return [];
    }
  }

  private addPanel(spec: PanelSpec): void {
    if (!this.api || this.knownSlots.has(spec.slot)) {
      return;
    }

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

    const panel = this.api.addPanel(options);
    this.knownSlots.add(spec.slot);
    this.permanentPanels.add(spec.id);

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
      this.addPanel(spec);
    }
    requestAnimationFrame(() => {
      this.applyProportions();
      this.tagPermanentTabs();
      const initActive = this.getAttribute("data-active-panel");
      if (initActive) {
        this.api?.getPanel(initActive)?.api.setActive();
      }
    });
  }

  private tagPermanentTabs(): void {
    if (!this.shadowRootRef || !this.api) return;
    const permanentTitles = new Set<string>();
    for (const id of this.permanentPanels) {
      const panel = this.api.getPanel(id);
      if (panel?.title) permanentTitles.add(panel.title);
    }
    this.shadowRootRef.querySelectorAll(".dv-tab").forEach((tab) => {
      const title = tab.querySelector(".dv-default-tab-content")?.textContent;
      if (title && permanentTitles.has(title)) {
        tab.setAttribute("data-permanent", "");
      }
    });
  }

  private applyProportions(): void {
    if (!this.api) return;
    const propStr = this.getAttribute("data-proportions");
    if (!propStr) return;

    const proportions = propStr.split(",").map(Number);

    // Ensure dockview knows its container size before reading layout
    const w = this.clientWidth || window.innerWidth;
    const h = this.clientHeight || window.innerHeight;
    this.api.layout(w, h);

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const json = (this.api as any).toJSON();
    const rootChildren = json?.grid?.root?.data;
    if (!rootChildren || proportions.length !== rootChildren.length) return;

    const totalWidth = json.grid.width;
    const totalProp = proportions.reduce((a: number, b: number) => a + b, 0);
    for (let i = 0; i < rootChildren.length; i++) {
      rootChildren[i].size = Math.round((totalWidth * proportions[i]) / totalProp);
    }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (this.api as any).fromJSON(json);
  }

  private observeSlots(): void {
    if (!this.api) {
      return;
    }

    this.observer = new MutationObserver(() => {
      const slotted = Array.from(this.children)
        .map((el) => el.getAttribute("slot"))
        .filter((s): s is string => !!s);

      for (const slot of slotted) {
        if (!this.knownSlots.has(slot)) {
          this.addPanel({
            id: slot,
            slot,
            title: slot,
            position: { referencePanel: "chat", direction: "within" },
          });
        }
      }

      for (const existing of [...this.knownSlots]) {
        if (!slotted.includes(existing)) {
          const panel = this.api?.getPanel(existing);
          panel?.dispose();
          this.knownSlots.delete(existing);
        }
      }
    });

    this.observer.observe(this, {
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
