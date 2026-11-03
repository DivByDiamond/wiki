import { el } from "../shared/dom.ts";
import { t } from "./i18n.ts";
import { highlightInto, looksLikeLua } from "./lua.ts";

export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    const area = el("textarea", { "aria-hidden": "true", tabindex: "-1" });
    area.value = text;
    area.style.cssText = "position:fixed;opacity:0;top:0;left:0";
    document.body.append(area);
    area.select();
    let ok = false;
    try { ok = document.execCommand("copy"); } catch { ok = false; }
    area.remove();
    return ok;
  }
}

function enhance(pre: HTMLPreElement): void {
  const code = pre.querySelector("code");
  if (!code || pre.parentElement?.classList.contains("code-block")) return;
  const source = code.textContent ?? "";
  if (looksLikeLua(code.className, source)) {
    code.classList.add("lua");
    highlightInto(code, source);
  }
  const wrap = el("div", { class: "code-block" });
  pre.replaceWith(wrap);
  const button = el("button", { type: "button", class: "code-copy", "aria-label": t("copyLabel") }, t("copy"));
  const live = el("span", { class: "visually-hidden", role: "status" });
  let timer: ReturnType<typeof setTimeout> | undefined;
  button.addEventListener("click", async () => {
    const ok = await copyText(source.replace(/\n$/, ""));
    button.textContent = ok ? t("copied") : t("copyFailed");
    live.textContent = button.textContent;
    clearTimeout(timer);
    timer = setTimeout(() => { button.textContent = t("copy"); live.textContent = ""; }, 1800);
  });
  wrap.append(pre, button, live);
}

export function mountCode(root: ParentNode = document): void {
  root.querySelectorAll<HTMLPreElement>(".prose pre").forEach(enhance);
}
