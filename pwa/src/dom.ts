// Trợ giúp DOM tối giản. Không dùng innerHTML cho dữ liệu (chỉ cho SVG biểu tượng tĩnh).

type Child = Node | string | number | null | undefined | false;
type Attrs = Record<string, string | number | boolean | null | undefined | EventListener>;

export function h<K extends keyof HTMLElementTagNameMap>(tag: K, attrs?: Attrs | null, ...children: (Child | Child[])[]): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  if (attrs) {
    for (const [k, v] of Object.entries(attrs)) {
      if (v === null || v === undefined || v === false) continue;
      if (k.toLowerCase().startsWith("on")) {
        // RS-11: chỉ nhận hàm cho thuộc tính sự kiện; không bao giờ đặt trình xử lý dạng chuỗi.
        if (typeof v !== "function") throw new TypeError(`thuộc tính sự kiện ${k} phải là hàm`);
        el.addEventListener(k.slice(2), v);
      }
      else if (k === "class") el.className = String(v);
      else if (v === true) el.setAttribute(k, "");
      else el.setAttribute(k, String(v));
    }
  }
  append(el, children);
  return el;
}

export function append(el: Element, children: (Child | Child[])[]): void {
  for (const c of children.flat()) {
    if (c === null || c === undefined || c === false) continue;
    el.append(typeof c === "object" ? c : String(c));
  }
}

const parser = typeof DOMParser !== "undefined" ? new DOMParser() : null;

/** Tạo phần tử SVG từ chuỗi tĩnh (chỉ dùng cho biểu tượng viết sẵn trong mã nguồn). */
export function svg(markup: string, cls = "ico"): SVGElement {
  const doc = parser!.parseFromString(markup.replace("<svg", `<svg xmlns="http://www.w3.org/2000/svg" class="${cls}" aria-hidden="true" focusable="false"`), "image/svg+xml");
  return document.importNode(doc.documentElement, true) as unknown as SVGElement;
}

export function clear(el: Element): void {
  el.replaceChildren();
}

export const $ = <T extends Element = HTMLElement>(sel: string, root: ParentNode = document) => root.querySelector(sel) as T;
