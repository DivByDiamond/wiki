import { el } from "../shared/dom.ts";
import { onEscape } from "../shared/keys.ts";
import { ACCENTS, SIZES, THEMES, type Settings } from "./store.ts";

const SIZE_LABEL = { s: "A-", m: "A", l: "A+" } as const;

function group(title: string, ...items: Node[]): HTMLElement {
  return el("fieldset", { class: "settings-group" }, el("legend", {}, title), ...items);
}

function choice(label: string, active: boolean, extra: Record<string, string>, on: () => void): HTMLButtonElement {
  const b = el("button", { type: "button", class: "choice", "aria-pressed": String(active), ...extra }, label);
  b.addEventListener("click", on);
  return b;
}

/** Builds #settings-panel and wires it to the #settings-toggle button. */
export function mountPanel(get: () => Settings, set: (s: Settings) => void): void {
  const toggle = document.getElementById("settings-toggle");
  if (!toggle) return;
  const panel = el("div", { id: "settings-panel", class: "card", role: "dialog", "aria-label": "Settings", hidden: "" });
  document.body.append(panel);

  const render = () => {
    const s = get();
    const upd = (p: Partial<Settings>) => { set({ ...s, ...p }); render(); };
    const pixel = el("input", { type: "checkbox" });
    pixel.checked = s.pixel;
    pixel.addEventListener("change", () => upd({ pixel: pixel.checked }));
    panel.replaceChildren(
      group("Theme", ...THEMES.map((t) => choice(t, s.theme === t, {}, () => upd({ theme: t })))),
      group("Accent", ...ACCENTS.map((a) => choice("", s.accent === a, { "data-swatch": a, "aria-label": a }, () => upd({ accent: a })))),
      group("Font size", ...SIZES.map((z) => choice(SIZE_LABEL[z], s.size === z, {}, () => upd({ size: z })))),
      el("label", { class: "settings-check" }, pixel, " Pixel borders"),
    );
  };

  const setOpen = (open: boolean) => {
    panel.hidden = !open;
    toggle.setAttribute("aria-expanded", String(open));
  };
  toggle.setAttribute("aria-expanded", "false");
  toggle.addEventListener("click", () => setOpen(panel.hidden));
  onEscape(() => { if (!panel.hidden) { setOpen(false); toggle.focus(); } });
  render();
}
