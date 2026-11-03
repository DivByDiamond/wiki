export type Draft = { text: string; at: number };
export type Store = Pick<Storage, "getItem" | "setItem" | "removeItem">;

export const draftKey = (lang: string, slug: string) => `wiki:draft:${lang}:${slug}`;

export function pageIdFromAction(action: string): { lang: string; slug: string } | null {
  const m = /\/admin\/edit\/([^/]+)\/(.+)$/.exec(action);
  return m ? { lang: m[1]!, slug: decodeURIComponent(m[2]!) } : null;
}

export function saveDraft(s: Store, key: string, text: string, at: number): void {
  try { s.setItem(key, JSON.stringify({ text, at })); } catch { /* storage unavailable or full */ }
}

export function loadDraft(s: Store, key: string): Draft | null {
  try {
    const raw = s.getItem(key);
    if (!raw) return null;
    const v = JSON.parse(raw) as Partial<Draft>;
    return typeof v.text === "string" && typeof v.at === "number" ? { text: v.text, at: v.at } : null;
  } catch { return null; }
}

export function clearDraft(s: Store, key: string): void {
  try { s.removeItem(key); } catch { /* ignore */ }
}

export const shouldOffer = (d: Draft | null, serverText: string): boolean => d !== null && d.text !== serverText;
