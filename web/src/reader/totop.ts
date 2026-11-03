import { el } from "../shared/dom.ts";
import { t } from "./i18n.ts";

export function mountBackToTop(): void {
  const button = el("button", { type: "button", class: "to-top", "aria-label": t("top"), title: t("top"), hidden: "" }, "↑");
  document.body.append(button);
  const update = () => { button.hidden = window.scrollY < 600; };
  window.addEventListener("scroll", update, { passive: true });
  button.addEventListener("click", () => {
    const calm = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    window.scrollTo({ top: 0, behavior: calm ? "auto" : "smooth" });
  });
  update();
}
