import editorCss from "monaco-editor/min/vs/editor/editor.main.css";
import { createCachedLoader } from "@omni/omni-util/async-loader";
import { createObjectUrl, revokeObjectUrl } from "@omni/omni-util/object-url";
import { LitElement, html, css, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

type MonacoModule = typeof import("monaco-editor/esm/vs/editor/editor.api");
type MonacoEnvironmentHost = typeof globalThis & {
  MonacoEnvironment?: {
    getWorker: () => Worker;
  };
};

// Tell Monaco not to try to load workers via URL
(globalThis as MonacoEnvironmentHost).MonacoEnvironment = {
  getWorker: () => {
    const workerSrc = `self.onmessage = function() {};`;
    const workerUrl = createObjectUrl([workerSrc], { type: "application/javascript" });
    const worker = new Worker(workerUrl);
    revokeObjectUrl(workerUrl);
    return worker;
  },
};

const loadMonaco = createCachedLoader(() => import("monaco-editor/esm/vs/editor/editor.api"));

@customElement("omni-monaco")
export class OmniMonaco extends LitElement {
  static styles = css`
    ${unsafeCSS(editorCss)}
    :host {
      display: block;
      width: 100%;
      height: 100%;
    }
    #container {
      width: 100%;
      height: 100%;
    }
  `;

  @property({ attribute: "data-value" }) dataValue = "";
  @property({ attribute: "data-language" }) dataLanguage = "plaintext";
  @property({ attribute: "data-readonly", type: Boolean }) dataReadonly = true;
  @property({ attribute: "data-theme" }) dataTheme = "vs-dark";

  private monaco: MonacoModule | null = null;
  private editor: MonacoModule["editor"]["IStandaloneCodeEditor"] | null = null;
  private container: HTMLDivElement | null = null;

  render() {
    return html`<div id="container"></div>`;
  }

  firstUpdated() {
    void this.initEditor();
  }

  private async initEditor() {
    const root = this.shadowRoot;
    if (!root) return;

    const container = root.querySelector<HTMLDivElement>("#container");
    if (!container) return;

    this.container = container;
    const monaco = await loadMonaco();
    if (!this.isConnected || !this.container) return;

    this.monaco = monaco;
    this.editor = monaco.editor.create(this.container, {
      value: this.dataValue,
      language: this.dataLanguage,
      readOnly: this.dataReadonly,
      theme: this.dataTheme,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      fontSize: 12,
      fontFamily: "'JetBrains Mono', monospace",
      automaticLayout: true,
    });

    this.editor.onDidChangeModelContent(() => {
      const editor = this.editor;
      if (!editor) return;

      this.dispatchEvent(new CustomEvent("change", { detail: editor.getValue(), bubbles: true, composed: true }));
    });
  }

  updated(changed: Map<string, unknown>) {
    if (!this.editor || !this.monaco) return;
    if (changed.has("dataValue")) {
      const current = this.editor.getValue();
      if (current !== this.dataValue) {
        this.editor.setValue(this.dataValue);
      }
    }
    if (changed.has("dataLanguage")) {
      const model = this.editor.getModel();
      if (model) this.monaco.editor.setModelLanguage(model, this.dataLanguage);
    }
    if (changed.has("dataReadonly")) {
      this.editor.updateOptions({ readOnly: this.dataReadonly });
    }
    if (changed.has("dataTheme")) {
      this.monaco.editor.setTheme(this.dataTheme);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.editor?.dispose();
    this.editor = null;
    this.monaco = null;
  }
}
