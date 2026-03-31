import { prepare, layout } from "@chenglou/pretext";

function buildFont(style: string, weight: string, size: number, family: string): string {
  const s = style !== "normal" ? style + " " : "";
  return `${s}${weight} ${size}px ${family}`;
}

function lineHeightPx(cs: CSSStyleDeclaration): number {
  const lh = cs.lineHeight;
  if (lh === "normal") return parseFloat(cs.fontSize) * 1.2;
  return parseFloat(lh);
}

function doTruncate(text: string, font: string, width: number, maxLines: number, lh: number): string {
  if (layout(prepare(text, font), width, lh).lineCount <= maxLines) return text;
  let lo = 0;
  let hi = text.length;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    const candidate = text.slice(0, mid) + "\u2026";
    if (layout(prepare(candidate, font), width, lh).lineCount <= maxLines) lo = mid;
    else hi = mid - 1;
  }
  return text.slice(0, lo) + "\u2026";
}

function findFitSize(
  text: string,
  style: string,
  weight: string,
  family: string,
  startSize: number,
  minSize: number,
  width: number,
  maxLines: number,
  baseLH: number,
): number {
  for (let size = startSize; size > minSize; size -= 0.5) {
    const font = buildFont(style, weight, size, family);
    const lh = baseLH * (size / startSize);
    if (layout(prepare(text, font), width, lh).lineCount <= maxLines) return size;
  }
  return minSize;
}

class OmniText extends HTMLElement {
  static observedAttributes = ["data-text", "data-strategy", "data-max-lines", "data-min-size"];

  private _observer: ResizeObserver | null = null;
  private _rafId: number | null = null;

  connectedCallback() {
    this._observer = new ResizeObserver(() => this._schedule());
    this._observer.observe(this);
    this._schedule();
    document.fonts.ready.then(() => this._schedule());
  }

  disconnectedCallback() {
    this._observer?.disconnect();
    this._observer = null;
    if (this._rafId !== null) {
      cancelAnimationFrame(this._rafId);
      this._rafId = null;
    }
  }

  attributeChangedCallback() {
    this._schedule();
  }

  private _schedule() {
    if (this._rafId !== null) return;
    this._rafId = requestAnimationFrame(() => {
      this._rafId = null;
      this._render();
    });
  }

  private _render() {
    const text = this.getAttribute("data-text") ?? "";
    const strategy = this.getAttribute("data-strategy") ?? "truncate";
    const maxLines = parseInt(this.getAttribute("data-max-lines") ?? "1", 10);
    const minSize = parseFloat(this.getAttribute("data-min-size") ?? "9");
    const ownWidth = this.getBoundingClientRect().width;
    const parentWidth = this.parentElement?.getBoundingClientRect().width ?? 0;
    const width = ownWidth > 0 ? ownWidth : parentWidth;

    if (width <= 0) {
      if (this.textContent !== text) this.textContent = text;
      // Retry next frame until we have a layout
      this._rafId = requestAnimationFrame(() => {
        this._rafId = null;
        this._render();
      });
      return;
    }

    const cs = getComputedStyle(this);
    const baseFontSize = parseFloat(cs.fontSize) || 12;
    const baseLH = lineHeightPx(cs);
    const weight = cs.fontWeight;
    const style = cs.fontStyle;
    const family = cs.fontFamily;
    const font = buildFont(style, weight, baseFontSize, family);

    if (strategy === "truncate") {
      const result = doTruncate(text, font, width, maxLines, baseLH);
      if (this.textContent !== result) this.textContent = result;
      if (result !== text) this.setAttribute("title", text);
      else this.removeAttribute("title");
    } else if (strategy === "shrink") {
      const size = findFitSize(text, style, weight, family, baseFontSize, minSize, width, maxLines, baseLH);
      if (Math.abs(parseFloat(this.style.fontSize) - size) > 0.1) this.style.fontSize = size + "px";
      if (this.textContent !== text) this.textContent = text;
      this.removeAttribute("title");
    } else if (strategy === "shrink-truncate") {
      const size = findFitSize(text, style, weight, family, baseFontSize, minSize, width, maxLines, baseLH);
      if (Math.abs(parseFloat(this.style.fontSize) - size) > 0.1) this.style.fontSize = size + "px";
      const shrunkLH = baseLH * (size / baseFontSize);
      const shrunkFont = buildFont(style, weight, size, family);
      const result = doTruncate(text, shrunkFont, width, maxLines, shrunkLH);
      if (this.textContent !== result) this.textContent = result;
      if (result !== text) this.setAttribute("title", text);
      else this.removeAttribute("title");
    } else {
      if (this.textContent !== text) this.textContent = text;
      this.removeAttribute("title");
    }
  }
}

customElements.define("omni-text", OmniText);
