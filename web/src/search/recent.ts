export const RECENT_KEY = "oc2r-wiki-recent-searches";
export const RECENT_MAX = 8;

export function parseRecent(raw: string | null): string[] {
  try {
    const v: unknown = JSON.parse(raw ?? "[]");
    return Array.isArray(v) ? v.filter((x): x is string => typeof x === "string" && x.trim() !== "").slice(0, RECENT_MAX) : [];
  } catch {
    return [];
  }
}

/** Newest first, case-insensitive duplicates collapsed, capped. */
export function addRecent(list: string[], query: string): string[] {
  const q = query.trim().replace(/\s+/g, " ");
  if (!q) return list;
  const rest = list.filter((x) => x.toLowerCase() !== q.toLowerCase());
  return [q, ...rest].slice(0, RECENT_MAX);
}

function storage(): Storage | null {
  try { return window.localStorage; } catch { return null; }
}

export function loadRecent(): string[] {
  return parseRecent(storage()?.getItem(RECENT_KEY) ?? null);
}

export function saveRecent(list: string[]): void {
  try { storage()?.setItem(RECENT_KEY, JSON.stringify(list)); } catch { /* blocked */ }
}
