import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import editorCss from "monaco-editor/min/vs/editor/editor.main.css";
import { LitElement, html, css, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

// Tell Monaco not to try to load workers via URL
(self as any).MonacoEnvironment = {
  getWorker: () => {
    const workerSrc = `self.onmessage = function() {};`;
    const blob = new Blob([workerSrc], { type: "application/javascript" });
    return new Worker(URL.createObjectURL(blob));
  },
};

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

  private editor: monaco.editor.IStandaloneCodeEditor | null = null;
  private container: HTMLDivElement | null = null;

  render() {
    return html`<div id="container"></div>`;
  }

  firstUpdated() {
    this.container = this.shadowRoot!.querySelector<HTMLDivElement>("#container")!;
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
      this.dispatchEvent(new CustomEvent("change", { detail: this.editor!.getValue(), bubbles: true, composed: true }));
    });
  }

  updated(changed: Map<string, unknown>) {
    if (!this.editor) return;
    if (changed.has("dataValue")) {
      const current = this.editor.getValue();
      if (current !== this.dataValue) {
        this.editor.setValue(this.dataValue);
      }
    }
    if (changed.has("dataLanguage")) {
      const model = this.editor.getModel();
      if (model) monaco.editor.setModelLanguage(model, this.dataLanguage);
    }
    if (changed.has("dataReadonly")) {
      this.editor.updateOptions({ readOnly: this.dataReadonly });
    }
    if (changed.has("dataTheme")) {
      monaco.editor.setTheme(this.dataTheme);
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.editor?.dispose();
    this.editor = null;
  }
}
