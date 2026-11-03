export const SCROLL_KEY = "oc2r-wiki-scroll";
const CAP = 60;

export interface ScrollEntry { y: number; at: number }
export type ScrollMap = Record<string, ScrollEntry>;

export function parseMap(raw: string | null): ScrollMap {
  try {
    const v: unknown = JSON.parse(raw ?? "{}");
    if (!v || typeof v !== "object" || Array.isArray(v)) return {};
    const out: ScrollMap = {};
    for (const [k, e] of Object.entries(v as Record<string, unknown>)) {
      const entry = e as Partial<ScrollEntry> | null;
      if (entry && Number.isFinite(entry.y) && Number.isFinite(entry.at)) out[k] = { y: entry.y!, at: entry.at! };
    }
    return out;
  } catch {
    return {};
  }
}

/** Records a position (near the top clears it) and keeps only the newest entries. */
export function remember(map: ScrollMap, path: string, y: number, now: number): ScrollMap {
  const next: ScrollMap = { ...map };
  if (y < 80) delete next[path];
  else next[path] = { y: Math.round(y), at: now };
  const keys = Object.keys(next).sort((a, b) => next[b]!.at - next[a]!.at);
  for (const k of keys.slice(CAP)) delete next[k];
  return next;
}

export function mountScrollMemory(): void {
  let store: Storage | null = null;
  try { store = window.localStorage; } catch { return; }
  const path = location.pathname + location.search;
  const read = () => parseMap(store!.getItem(SCROLL_KEY));
  const saved = read()[path];
  if (saved && !location.hash) {
    requestAnimationFrame(() => window.scrollTo(0, saved.y));
  }
  let timer: ReturnType<typeof setTimeout> | undefined;
  const write = () => {
    try { store!.setItem(SCROLL_KEY, JSON.stringify(remember(read(), path, window.scrollY, Date.now()))); } catch { /* quota or blocked */ }
  };
  window.addEventListener("scroll", () => { clearTimeout(timer); timer = setTimeout(write, 250); }, { passive: true });
  window.addEventListener("pagehide", write);
}
