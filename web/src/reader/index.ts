import { el } from "../shared/dom.ts";
import { mountCode } from "./code.ts";
import { mountAnchors } from "./headings.ts";
import { t } from "./i18n.ts";
import { mountReadingTime } from "./readtime.ts";
import { mountScrollMemory } from "./scroll.ts";
import { mountToc } from "./toc.ts";
import { mountBackToTop } from "./totop.ts";

/** Article-only enhancements; the page stays fully readable without them. */
export function mountReader(): void {
  if (!document.querySelector(".prose")) return;
  mountCode();
  mountAnchors();
  mountReadingTime();
  mountToc();
  mountBackToTop();
  mountScrollMemory();
  const actions = document.querySelector(".page-actions");
  if (actions) {
    const print = el("button", { type: "button", class: "linklike" }, t("print"));
    print.addEventListener("click", () => window.print());
    actions.append(print);
  }
}
