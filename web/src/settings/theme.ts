import type { Settings } from "./store.ts";

export function apply(root: HTMLElement, s: Settings, prefersDark: boolean): void {
  root.dataset.theme = s.theme === "auto" ? (prefersDark ? "dark" : "light") : s.theme;
  root.dataset.accent = s.accent;
  root.dataset.size = s.size;
  root.dataset.pixel = s.pixel ? "1" : "0";
}
