import { el } from "../shared/dom.ts";
import { onEscape } from "../shared/keys.ts";
import { ACCENTS, SIZES, THEMES, type Settings } from "./store.ts";

const LABELS = {
  en: { title: "Reading settings", theme: "Theme", accent: "Accent", size: "Font size", pixel: "Pixel borders", light: "Light", dark: "Dark", auto: "System", green: "Green", violet: "Violet", orange: "Orange", teal: "Teal" },
  ru: { title: "Настройки чтения", theme: "Тема", accent: "Акцент", size: "Размер текста", pixel: "Пиксельные рамки", light: "Светлая", dark: "Тёмная", auto: "Системная", green: "Зелёный", violet: "Фиолетовый", orange: "Оранжевый", teal: "Бирюзовый" },
  zh: { title: "阅读设置", theme: "主题", accent: "强调色", size: "字号", pixel: "像素边框", light: "浅色", dark: "深色", auto: "跟随系统", green: "绿色", violet: "紫色", orange: "橙色", teal: "青色" },
} as const;

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
  const lang = document.documentElement.lang;
  const labels = LABELS[lang === "ru" || lang === "zh" ? lang : "en"];
  const panel = el("div", { id: "settings-panel", class: "card", role: "dialog", "aria-label": labels.title, hidden: "" });
  document.body.append(panel);

  const render = () => {
    const s = get();
    const upd = (p: Partial<Settings>) => {
      const controls = Array.from(panel.querySelectorAll<HTMLElement>("button, input"));
      const focused = controls.indexOf(document.activeElement as HTMLElement);
      set({ ...s, ...p }); render();
      if (focused >= 0) panel.querySelectorAll<HTMLElement>("button, input")[focused]?.focus();
    };
    const pixel = el("input", { type: "checkbox" });
    pixel.checked = s.pixel;
    pixel.addEventListener("change", () => upd({ pixel: pixel.checked }));
    panel.replaceChildren(
      group(labels.theme, ...THEMES.map((t) => choice(labels[t], s.theme === t, {}, () => upd({ theme: t })))),
      group(labels.accent, ...ACCENTS.map((a) => choice("", s.accent === a, { "data-swatch": a, "aria-label": labels[a] }, () => upd({ accent: a })))),
      group(labels.size, ...SIZES.map((z) => choice(SIZE_LABEL[z], s.size === z, {}, () => upd({ size: z })))),
      el("label", { class: "settings-check" }, pixel, ` ${labels.pixel}`),
    );
  };

  const setOpen = (open: boolean) => {
    panel.hidden = !open;
    toggle.setAttribute("aria-expanded", String(open));
    if (open) panel.querySelector<HTMLElement>("button")?.focus();
  };
  toggle.setAttribute("aria-expanded", "false");
  toggle.addEventListener("click", () => setOpen(panel.hidden));
  document.addEventListener("click", (event) => {
    const path = event.composedPath();
    if (!panel.hidden && !path.includes(panel) && !path.includes(toggle)) setOpen(false);
  });
  onEscape(() => { if (!panel.hidden) { setOpen(false); toggle.focus(); } });
  render();
}
