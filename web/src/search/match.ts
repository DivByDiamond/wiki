export interface Entry { title: string; href: string; hint?: string }

/** Lower score is better; -1 means no match. Word starts beat inner matches. */
export function score(title: string, query: string): number {
  const t = title.toLowerCase();
  const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (terms.length === 0) return -1;
  let total = 0;
  for (const term of terms) {
    const at = t.indexOf(term);
    if (at < 0) return -1;
    total += at === 0 ? 0 : /[\s\-_./(]/.test(t[at - 1]!) ? 1 : 3;
  }
  return total;
}

export function filterEntries(entries: Entry[], query: string, limit = 6): Entry[] {
  return entries
    .map((e, i) => ({ e, s: score(e.title, query), i }))
    .filter((x) => x.s >= 0)
    .sort((a, b) => a.s - b.s || a.i - b.i)
    .slice(0, limit)
    .map((x) => x.e);
}

/** Reads `<li><a>` rows of the server's search fragment. */
export function parseHits(html: string): Entry[] {
  const doc = new DOMParser().parseFromString(html, "text/html");
  const out: Entry[] = [];
  doc.querySelectorAll("li").forEach((li) => {
    const a = li.querySelector("a");
    const href = a?.getAttribute("href") ?? "";
    if (!a || !href.startsWith("/") || href.startsWith("//")) return;
    out.push({ title: a.textContent ?? href, href, hint: li.querySelector("small")?.textContent ?? "" });
  });
  return out;
}
