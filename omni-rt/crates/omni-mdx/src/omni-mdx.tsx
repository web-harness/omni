import React from "react";
import * as ReactDOM from "react-dom/client";
import {
  MDXEditor,
  headingsPlugin,
  listsPlugin,
  quotePlugin,
  thematicBreakPlugin,
  markdownShortcutPlugin,
  linkPlugin,
  tablePlugin,
  codeBlockPlugin,
} from "@mdxeditor/editor";
import editorCss from "@mdxeditor/editor/style.css";
import { LitElement, html, css, unsafeCSS } from "lit";
import { customElement, property } from "lit/decorators.js";

@customElement("omni-mdx")
export class OmniMdx extends LitElement {
  static styles = css`
    ${unsafeCSS(editorCss)}
    :host {
      display: block;
      width: 100%;
      height: 100%;
      overflow: auto;
    }
    #container {
      width: 100%;
      height: 100%;
    }
  `;

  @property({ attribute: "data-value" }) dataValue = "";
  @property({ attribute: "data-readonly", type: Boolean }) dataReadonly = true;

  private root: ReactDOM.Root | null = null;

  render() {
    return html`<div id="container"></div>`;
  }

  private renderEditor() {
    const container = this.shadowRoot!.querySelector<HTMLDivElement>("#container");
    if (!container) return;
    if (!this.root) {
      this.root = ReactDOM.createRoot(container);
    }
    this.root.render(
      React.createElement(MDXEditor, {
        markdown: this.dataValue,
        readOnly: this.dataReadonly,
        plugins: [
          headingsPlugin(),
          listsPlugin(),
          quotePlugin(),
          thematicBreakPlugin(),
          markdownShortcutPlugin(),
          linkPlugin(),
          tablePlugin(),
          codeBlockPlugin({ defaultCodeBlockLanguage: "text" }),
        ],
        onChange: (value: string) => {
          this.dispatchEvent(new CustomEvent("change", { detail: value, bubbles: true, composed: true }));
        },
      }),
    );
  }

  firstUpdated() {
    this.renderEditor();
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has("dataValue") || changed.has("dataReadonly")) {
      this.renderEditor();
    }
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.root?.unmount();
    this.root = null;
  }
}
