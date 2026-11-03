import { el } from "../shared/dom.ts";
import { editorText } from "./i18n.ts";
import { mapScroll } from "./scrollsync.ts";

const SPLIT_QUERY = "(min-width: 1100px)";

/** Proportional two-way scroll sync; `lock` swallows the echo event of the other side. */
export function syncScroll(a: HTMLElement, b: HTMLElement, active: () => boolean): void {
  let lock: HTMLElement | null = null;
  const link = (src: HTMLElement, dst: HTMLElement) => {
    src.addEventListener("scroll", () => {
      if (!active()) return;
      if (lock === src) { lock = null; return; }
      const target = mapScroll(src.scrollTop, src.scrollHeight - src.clientHeight, dst.scrollHeight - dst.clientHeight);
      if (Math.abs(dst.scrollTop - target) < 1) return;
      lock = dst;
      dst.scrollTop = target;
    }, { passive: true });
  };
  link(a, b);
  link(b, a);
}

/** Single-pane toggle for narrow screens; the split layout takes over on wide ones. */
export function mountViewToggle(layout: HTMLElement, area: HTMLTextAreaElement, pane: HTMLElement): void {
  const t = editorText(document.documentElement.lang);
  const views: [string, string][] = [["edit", t.viewEdit], ["preview", t.viewPreview]];
  const group = el("div", { class: "editor-views", role: "group", "aria-label": `${t.viewEdit} / ${t.viewPreview}` });
  const buttons = views.map(([v, label]) => {
    const b = el("button", { type: "button", class: "btn", "data-view": v, "aria-pressed": "false" }, label);
    b.addEventListener("click", () => set(v));
    group.append(b);
    return b;
  });
  function set(v: string): void {
    layout.dataset.view = v;
    for (const b of buttons) b.setAttribute("aria-pressed", String(b.dataset.view === v));
  }
  set("edit");
  layout.before(group);
  syncScroll(area, pane, () => window.matchMedia(SPLIT_QUERY).matches);
}
