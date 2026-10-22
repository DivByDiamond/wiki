import { mountSearch } from "./search/ui.ts";
import { mountPanel } from "./settings/panel.ts";
import { apply } from "./settings/theme.ts";
import { load, save, type Settings } from "./settings/store.ts";

function storage(): Storage | null {
  try { return window.localStorage; } catch { return null; }
}

const store = storage();
let current: Settings = load(store ?? { getItem: () => null });
const dark = window.matchMedia("(prefers-color-scheme: dark)");
const refresh = () => apply(document.documentElement, current, dark.matches);

refresh();
dark.addEventListener("change", refresh);

function start(): void {
  mountPanel(() => current, (s) => {
    current = s;
    if (store) save(store, s);
    refresh();
  });
  mountSearch();
  if (document.getElementById("editor") || document.getElementById("merge")) {
    void import("./editor/editor.ts").then((m) => m.mountEditor());
  }
}

if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start);
else start();
