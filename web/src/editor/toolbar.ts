import { el } from "../shared/dom.ts";

export const BLOCKS: [label: string, open: string][] = [
  ["Tip", ":::tip"],
  ["Warning", ":::warn"],
  ["Recipe", ":::recipe"],
  ["Version", ":::version 26.x"],
];

export function wrap(open: string, text: string): string {
  return `${open}\n${text}\n:::\n`;
}

export function mountToolbar(bar: HTMLElement, area: HTMLTextAreaElement): void {
  for (const [label, open] of BLOCKS) {
    const b = el("button", { type: "button", class: "btn" }, label);
    b.addEventListener("click", () => {
      const { selectionStart: a, selectionEnd: z, value } = area;
      const block = wrap(open, value.slice(a, z));
      area.setRangeText(block, a, z, "end");
      area.focus();
      area.dispatchEvent(new Event("input", { bubbles: true }));
    });
    bar.append(b);
  }
}
