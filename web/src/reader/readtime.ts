import { el } from "../shared/dom.ts";
import { t } from "./i18n.ts";

/** Minutes to read: 200 words/min for alphabetic text, 400 characters/min for CJK. */
export function readingMinutes(text: string): number {
  const cjk = (text.match(/[぀-ヿ㐀-鿿豈-﫿]/g) ?? []).length;
  const words = (text.replace(/[぀-ヿ㐀-鿿豈-﫿]/g, " ").match(/[\p{L}\p{N}]+/gu) ?? []).length;
  return Math.max(1, Math.round(words / 200 + cjk / 400));
}

export function mountReadingTime(): void {
  const prose = document.querySelector<HTMLElement>(".prose");
  if (!prose || document.querySelector(".reading-time")) return;
  const note = el("p", { class: "reading-time" }, t("readMin", { n: readingMinutes(prose.textContent ?? "") }));
  const h1 = prose.querySelector("h1");
  if (h1) h1.after(note);
  else prose.prepend(note);
}
