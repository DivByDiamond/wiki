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
  const lang = typeof document === "undefined" ? "en" : document.documentElement.lang;
  const labels = lang === "ru" ? ["Подсказка", "Предупреждение", "Рецепт", "Версия"]
    : lang === "zh" ? ["提示", "警告", "合成配方", "版本"] : BLOCKS.map(([label]) => label);
  for (const [index, [, open]] of BLOCKS.entries()) {
    const label = labels[index]!;
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
