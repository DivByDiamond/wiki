export type Theme = "light" | "dark" | "auto";
export type Accent = "green" | "violet" | "orange" | "teal";
export type Size = "s" | "m" | "l";
export interface Settings { theme: Theme; accent: Accent; size: Size; pixel: boolean }

export const DEFAULTS: Settings = { theme: "auto", accent: "green", size: "m", pixel: true };
const KEY = "oc2r-wiki-settings";

export const THEMES: Theme[] = ["light", "dark", "auto"];
export const ACCENTS: Accent[] = ["green", "violet", "orange", "teal"];
export const SIZES: Size[] = ["s", "m", "l"];

function pick<T>(allowed: T[], value: unknown, fallback: T): T {
  return allowed.includes(value as T) ? (value as T) : fallback;
}

export function load(storage: Pick<Storage, "getItem">): Settings {
  try {
    const raw = storage.getItem(KEY);
    if (!raw) return { ...DEFAULTS };
    const v = JSON.parse(raw) as Record<string, unknown>;
    return {
      theme: pick(THEMES, v.theme, DEFAULTS.theme),
      accent: pick(ACCENTS, v.accent, DEFAULTS.accent),
      size: pick(SIZES, v.size, DEFAULTS.size),
      pixel: typeof v.pixel === "boolean" ? v.pixel : DEFAULTS.pixel,
    };
  } catch {
    return { ...DEFAULTS };
  }
}

export function save(storage: Pick<Storage, "setItem">, s: Settings): void {
  try {
    storage.setItem(KEY, JSON.stringify(s));
  } catch {
    /* storage unavailable: the site keeps working with defaults */
  }
}
