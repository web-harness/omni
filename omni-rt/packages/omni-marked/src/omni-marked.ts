import { LitElement, html, css } from "lit";
import { unsafeHTML } from "lit/directives/unsafe-html.js";
import { customElement, property, state } from "lit/decorators.js";
import DOMPurify from "dompurify";
import { marked } from "marked";

@customElement("omni-marked")
export class OmniMarked extends LitElement {
  static styles = css`
    :host {
      display: block;
      width: 100%;
      height: 100%;
      overflow: auto;
      color: var(--foreground, #cdd6f4);
      background: var(--background, #11111b);
    }
    #container {
      width: 100%;
      min-height: 100%;
    }
    .loading,
    .markdown-body {
      box-sizing: border-box;
      width: 100%;
      min-height: 100%;
      padding: 18px 20px 32px;
    }
    .loading {
      color: var(--muted-foreground, #9399b2);
      font-size: 12px;
    }
    .markdown-body {
      line-height: 1.6;
      overflow-wrap: anywhere;
    }
    .markdown-body > :first-child {
      margin-top: 0;
    }
    .markdown-body > :last-child {
      margin-bottom: 0;
    }
    .markdown-body h1,
    .markdown-body h2,
    .markdown-body h3,
    .markdown-body h4,
    .markdown-body h5,
    .markdown-body h6 {
      color: var(--foreground, #f5e0dc);
      line-height: 1.25;
      margin: 1.25em 0 0.5em;
    }
    .markdown-body p,
    .markdown-body ul,
    .markdown-body ol,
    .markdown-body blockquote,
    .markdown-body pre,
    .markdown-body table {
      margin: 0 0 1em;
    }
    .markdown-body ul,
    .markdown-body ol {
      padding-left: 1.5em;
    }
    .markdown-body li + li {
      margin-top: 0.25em;
    }
    .markdown-body blockquote {
      border-left: 3px solid var(--border, #45475a);
      color: var(--muted-foreground, #9399b2);
      margin-left: 0;
      padding-left: 1em;
    }
    .markdown-body code {
      font-family: "Iosevka Term", "SFMono-Regular", Consolas, monospace;
      font-size: 0.92em;
    }
    .markdown-body :not(pre) > code {
      background: var(--background-elevated, #181825);
      border: 1px solid var(--border, #313244);
      border-radius: 6px;
      padding: 0.12em 0.4em;
    }
    .markdown-body pre {
      background: var(--background-elevated, #181825);
      border: 1px solid var(--border, #313244);
      border-radius: 10px;
      overflow: auto;
      padding: 0.9em 1em;
    }
    .markdown-body pre code {
      background: none;
      border: none;
      padding: 0;
    }
    .markdown-body a {
      color: var(--accent, #89b4fa);
      text-decoration: underline;
    }
    .markdown-body hr {
      border: none;
      border-top: 1px solid var(--border, #313244);
      margin: 1.5em 0;
    }
    .markdown-body table {
      border-collapse: collapse;
      width: 100%;
    }
    .markdown-body th,
    .markdown-body td {
      border: 1px solid var(--border, #313244);
      padding: 0.45em 0.6em;
      text-align: left;
      vertical-align: top;
    }
  `;

  @property({ attribute: "data-value" }) dataValue = "";
  @property({ attribute: "data-readonly", type: Boolean }) dataReadonly = true;

  @state() private renderedHtml = "";

  render() {
    return this.renderedHtml
      ? html`<div class="markdown-body">${unsafeHTML(this.renderedHtml)}</div>`
      : html`<div class="loading">Loading markdown...</div>`;
  }

  connectedCallback() {
    super.connectedCallback();
    this.renderMarkdown();
  }

  updated(changed: Map<string, unknown>) {
    if (changed.has("dataValue")) {
      this.renderMarkdown();
    }
  }

  private renderMarkdown(): void {
    const rendered = marked.parse(this.dataValue, { async: false, gfm: true });
    this.renderedHtml = DOMPurify.sanitize(rendered);
  }
}
