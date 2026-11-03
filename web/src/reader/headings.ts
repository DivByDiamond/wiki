import { el } from "../shared/dom.ts";
import { t } from "./i18n.ts";

/** Adds a permalink after every article heading that has an id. */
export function mountAnchors(root: ParentNode = document): void {
  root.querySelectorAll<HTMLElement>(".prose :is(h2,h3,h4,h5,h6)[id]").forEach((h) => {
    if (h.querySelector(".heading-anchor")) return;
    h.classList.add("has-anchor");
    h.append(el("a", { class: "heading-anchor", href: `#${h.id}`, "aria-label": t("anchor") }, "#"));
  });
}
