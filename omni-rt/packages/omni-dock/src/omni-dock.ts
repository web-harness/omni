import cssText from "dockview-core/dist/styles/dockview.css";
import xSvg from "./icons/x.svg";
import { createCachedLoader } from "@omni/omni-util/async-loader";

import { LitElement, html, css, unsafeCSS, type PropertyValues } from "lit";
import { customElement, property } from "lit/decorators.js";
import type {
  AddGroupOptions,
  AddPanelOptions,
  CreateComponentOptions,
  DockviewApi,
  DockviewGroupPanel,
  GroupPanelPartInitParameters,
  IContentRenderer,
  ITabRenderer,
  TabPartInitParameters,
} from "dockview-core";

const loadDockviewModule = createCachedLoader(() => import("dockview-core"));

const PERMANENT_PANELS = new Set(["sidebar", "chat", "tasks", "files", "bg-tasks"]);

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

type DockGroupOptions = AddGroupOptions & {
  direction?: NonNullable<PanelSpec["position"]>["direction"];
  referencePanel?: string;
};

class SlotPanel implements IContentRenderer {
  private readonly container: HTMLDivElement;

  constructor(slotName: string) {
    this.container = document.createElement("div");
    this.container.classList.add("slot-panel");
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

class OmniTab implements ITabRenderer {
  private readonly el: HTMLElement;

  constructor() {
    this.el = document.createElement("div");
    this.el.classList.add("tab-row");
  }

  get element(): HTMLElement {
    return this.el;
  }

  init(params: TabPartInitParameters): void {
    const title = document.createElement("span");
    title.textContent = params.title ?? "";
    this.el.appendChild(title);

    if (!PERMANENT_PANELS.has(params.api.id)) {
      const btn = document.createElement("button");
      btn.classList.add("tab-close");
      btn.innerHTML = xSvg;
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        params.api.close();
      });
      this.el.appendChild(btn);
    }
  }
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
    .dv-sash-container {
      pointer-events: none;
    }
    .dv-sash-container > .dv-sash {
      pointer-events: auto;
    }
    .slot-panel {
      position: relative;
      width: 100%;
      height: 100%;
      overflow: hidden;
    }
    .tab-row {
      display: flex;
      align-items: center;
      padding: 0 8px;
      height: 100%;
      color: var(--foreground);
      gap: 6px;
    }
    .tab-close {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 14px;
      height: 14px;
      border: none;
      background: none;
      cursor: pointer;
      color: inherit;
      padding: 0;
      opacity: 0.6;
    }
    .tab-close:hover {
      opacity: 1;
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
  private _programmaticClose = false;
  value = "";

  render() {
    return html`<div class="dock-root"></div>`;
  }

  private getDockRoot(): HTMLElement | null {
    return this.shadowRoot?.querySelector<HTMLElement>(".dock-root") ?? null;
  }

  firstUpdated() {
    void this.initializeDockview();
  }

  private async initializeDockview(): Promise<void> {
    const container = this.getDockRoot();
    if (!container) return;
    const dockview = await loadDockviewModule();
    if (!this.isConnected) return;
    this.api = dockview.createDockview(container, {
      theme: dockview.themeAbyss,
      createComponent: (options: CreateComponentOptions) => new SlotPanel(options.name),
      createTabComponent: (_options: CreateComponentOptions) => new OmniTab(),
    });
    this.api.onDidRemovePanel((panel) => {
      if (!this._programmaticClose) {
        const relay = this.querySelector<HTMLInputElement>("[data-dock-relay]");
        if (relay) {
          relay.value = panel.id;
          relay.dispatchEvent(new Event("input", { bubbles: true }));
        }
      }
    });
    this.initializePanels();
  }

  updated(changedProps: PropertyValues) {
    if (!this.api) return;
    if (changedProps.has("dataPanels")) {
      this.diffPanels();
    }
    if (changedProps.has("dataActivePanel") && this.dataActivePanel) {
      this.api.getPanel(this.dataActivePanel)?.api.setActive();
    }
    if (changedProps.has("dataProportions")) {
      this.applyProportions();
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.api?.dispose();
    this.api = null;
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

  private addPanelFromSpec(spec: PanelSpec): void {
    if (!this.api) return;

    if (spec.hideHeader) {
      const groupOpts: DockGroupOptions = { hideHeader: true };
      if (spec.position?.direction && spec.position.referencePanel) {
        groupOpts.referencePanel = spec.position.referencePanel;
        groupOpts.direction = spec.position.direction;
      } else if (spec.position?.direction) {
        groupOpts.direction = spec.position.direction;
      }
      const group: DockviewGroupPanel = this.api.addGroup(groupOpts);
      this.api.addPanel({
        id: spec.id,
        component: spec.slot,
        tabComponent: "omni-tab",
        title: spec.title ?? spec.id,
        position: { referenceGroup: group.id },
      });
      group.locked = true;
    } else {
      const options: AddPanelOptions = {
        id: spec.id,
        component: spec.slot,
        tabComponent: "omni-tab",
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
    }
  }

  private initializePanels(): void {
    for (const spec of this.parsePanelSpecs()) {
      this.addPanelFromSpec(spec);
    }
    setTimeout(() => {
      const container = this.getDockRoot();
      if (!container) return;
      this.api?.layout(container.offsetWidth, container.offsetHeight);
      this.applyProportions();
      if (this.dataActivePanel) {
        this.api?.getPanel(this.dataActivePanel)?.api.setActive();
      }
    }, 0);
  }

  private diffPanels(): void {
    if (!this.api) return;
    const specs = this.parsePanelSpecs();
    const specIds = new Set(specs.map((s) => s.id));
    const existingIds = new Set(this.api.panels.map((p) => p.id));

    for (const panel of [...this.api.panels]) {
      if (!specIds.has(panel.id)) {
        this._programmaticClose = true;
        panel.api.close();
        this._programmaticClose = false;
      }
    }

    for (const spec of specs) {
      if (!existingIds.has(spec.id)) {
        this.addPanelFromSpec(spec);
      }
    }
  }

  private applyProportions(): void {
    if (!this.api) return;
    const propStr = this.dataProportions;
    if (!propStr) return;
    const proportions = propStr
      .split(",")
      .map(Number)
      .filter((n) => !Number.isNaN(n) && n > 0);
    if (proportions.length === 0) return;
    const total = proportions.reduce((a, b) => a + b, 0);
    const container = this.getDockRoot();
    if (!container) return;
    const w = container.offsetWidth || this.clientWidth || window.innerWidth;
    const h = container.offsetHeight || this.clientHeight || window.innerHeight;
    this.api.layout(w, h);

    const specs = this.parsePanelSpecs();
    const anchors = specs.slice(0, proportions.length);
    const seen = new Set<string>();
    for (let i = 0; i < anchors.length; i++) {
      const panel = this.api.getPanel(anchors[i].id);
      if (!panel) continue;
      const groupId = panel.group.id;
      if (seen.has(groupId)) continue;
      seen.add(groupId);
      const width = Math.round((w * proportions[i]) / total);
      panel.group.api.setSize({ width });
    }
  }
}

if (!customElements.get("omni-dock")) {
  customElements.define("omni-dock", OmniDock);
}
