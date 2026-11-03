import { el } from "../shared/dom.ts";
import { type Edit, linkEdit, wrapEdit } from "./hotkeys.ts";
import { editorText } from "./i18n.ts";

export const BLOCKS: [label: string, open: string][] = [
  ["Tip", ":::tip"],
  ["Warning", ":::warn"],
  ["Recipe", ":::recipe"],
  ["Version", ":::version 26.x"],
];

export function wrap(open: string, text: string): string {
  return `${open}\n${text}\n:::\n`;
}

function touch(area: HTMLTextAreaElement): void {
  area.focus();
  area.dispatchEvent(new Event("input", { bubbles: true }));
}

/** Applies a pure edit result, keeping the browser undo stack where possible. */
export function applyEdit(area: HTMLTextAreaElement, make: (v: string, a: number, z: number) => Edit): void {
  const r = make(area.value, area.selectionStart, area.selectionEnd);
  area.value = r.value;
  area.setSelectionRange(r.start, r.end);
  touch(area);
}

export const bold = (area: HTMLTextAreaElement) => applyEdit(area, (v, a, z) => wrapEdit(v, a, z, "**"));
export const italic = (area: HTMLTextAreaElement) => applyEdit(area, (v, a, z) => wrapEdit(v, a, z, "*"));
export const link = (area: HTMLTextAreaElement) => applyEdit(area, (v, a, z) => linkEdit(v, a, z));

function button(label: string, tip: string, onClick: () => void, text = label): HTMLButtonElement {
  const b = el("button", { type: "button", class: "btn", title: tip, "aria-label": tip }, text);
  b.addEventListener("click", onClick);
  return b;
}

export function mountToolbar(bar: HTMLElement, area: HTMLTextAreaElement): void {
  const lang = typeof document === "undefined" ? "en" : document.documentElement.lang;
  const t = editorText(lang);
  bar.setAttribute("role", "toolbar");
  bar.append(
    button(t.bold, t.boldTip, () => bold(area), "B"),
    button(t.italic, t.italicTip, () => italic(area), "I"),
    button(t.link, t.linkTip, () => link(area), t.link),
  );
  bar.querySelectorAll("button")[0]?.classList.add("editor-btn-bold");
  bar.querySelectorAll("button")[1]?.classList.add("editor-btn-italic");
  const labels = [t.tip, t.warn, t.recipe, t.version];
  for (const [index, [, open]] of BLOCKS.entries()) {
    const label = labels[index]!;
    bar.append(button(label, t.blockTip + label, () => {
      const { selectionStart: a, selectionEnd: z, value } = area;
      area.setRangeText(wrap(open, value.slice(a, z)), a, z, "end");
      touch(area);
    }));
  }
}
