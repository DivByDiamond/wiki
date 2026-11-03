/** Picks the section to highlight: the first visible heading, else the previous one. */
export function currentHeading(ids: string[], visible: Set<string>, previous: string | null): string | null {
  for (const id of ids) if (visible.has(id)) return id;
  return previous && ids.includes(previous) ? previous : null;
}

export function mountToc(): void {
  const links = new Map<string, HTMLAnchorElement>();
  document.querySelectorAll<HTMLAnchorElement>(".toc a[href^='#']").forEach((a) => {
    links.set(decodeURIComponent(a.getAttribute("href")!.slice(1)), a);
  });
  const headings = [...links.keys()]
    .map((id) => document.getElementById(id))
    .filter((h): h is HTMLElement => h !== null);
  if (headings.length === 0 || typeof IntersectionObserver === "undefined") return;
  const ids = headings.map((h) => h.id);
  const visible = new Set<string>();
  let current: string | null = null;
  const show = () => {
    const next = currentHeading(ids, visible, current);
    if (next === current) return;
    if (current) {
      links.get(current)?.classList.remove("is-current");
      links.get(current)?.removeAttribute("aria-current");
    }
    current = next;
    if (current) {
      const a = links.get(current)!;
      a.classList.add("is-current");
      a.setAttribute("aria-current", "location");
    }
  };
  const observer = new IntersectionObserver((entries) => {
    for (const e of entries) {
      if (e.isIntersecting) visible.add(e.target.id);
      else visible.delete(e.target.id);
    }
    show();
  }, { rootMargin: "-64px 0px -65% 0px" });
  headings.forEach((h) => observer.observe(h));
}
