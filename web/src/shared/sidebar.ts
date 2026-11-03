/** Sidebar behaviour: remembered section state and the mobile off-canvas drawer. */

const KEY = "oc2r-wiki-nav";
const MOBILE = "(max-width: 719px)";

export function parseOpen(raw: string | null): Record<string, boolean> {
  try {
    const v: unknown = JSON.parse(raw ?? "{}");
    if (!v || typeof v !== "object" || Array.isArray(v)) return {};
    return Object.fromEntries(Object.entries(v).filter(([, o]) => typeof o === "boolean")) as Record<string, boolean>;
  } catch {
    return {};
  }
}

function store(): Storage | null {
  try { return window.localStorage; } catch { return null; }
}

export function mountSidebar(root: Document = document): void {
  const sidebar = root.getElementById("sidebar");
  if (!sidebar) return;
  // On a phone the outline starts collapsed so the article is the first thing on screen.
  if (window.matchMedia(MOBILE).matches) {
    const toc = root.querySelector<HTMLDetailsElement>(".toc details");
    if (toc) toc.open = false;
  }
  const html = root.documentElement;
  const toggle = root.querySelector<HTMLButtonElement>("[data-sidebar-toggle]");
  const backdrop = root.querySelector<HTMLElement>("[data-sidebar-backdrop]");
  const closeBtn = sidebar.querySelector<HTMLButtonElement>("[data-sidebar-close]");
  const mobile = window.matchMedia(MOBILE);
  const ls = store();

  const saved = parseOpen(ls?.getItem(KEY) ?? null);
  const sections = Array.from(sidebar.querySelectorAll<HTMLDetailsElement>("details[data-nav-section]"));
  for (const d of sections) {
    const name = d.dataset.navSection ?? "";
    const hasCurrent = d.querySelector('[aria-current="page"]') !== null;
    if (!hasCurrent && name in saved) d.open = saved[name] as boolean;
    if (hasCurrent) d.open = true;
    d.addEventListener("toggle", () => {
      const next = parseOpen(ls?.getItem(KEY) ?? null);
      next[name] = d.open;
      try { ls?.setItem(KEY, JSON.stringify(next)); } catch { /* storage unavailable */ }
    });
  }
  sidebar.querySelector('[aria-current="page"]')?.scrollIntoView({ block: "nearest" });

  const isOpen = () => html.classList.contains("sidebar-open");
  const setOpen = (open: boolean, restoreFocus = true) => {
    html.classList.toggle("sidebar-open", open);
    toggle?.setAttribute("aria-expanded", String(open));
    if (open) sidebar.removeAttribute("inert");
    if (open) (closeBtn ?? sidebar.querySelector<HTMLElement>("a, summary"))?.focus();
    else if (restoreFocus) toggle?.focus();
  };
  // Off-screen drawer content must not take focus or be read out.
  const sync = () => {
    if (mobile.matches && !isOpen()) sidebar.setAttribute("inert", "");
    else sidebar.removeAttribute("inert");
    if (!mobile.matches && isOpen()) setOpen(false, false);
  };

  toggle?.addEventListener("click", () => setOpen(!isOpen()));
  closeBtn?.addEventListener("click", () => setOpen(false));
  backdrop?.addEventListener("click", () => setOpen(false));
  sidebar.addEventListener("click", (e) => {
    if (mobile.matches && (e.target as Element).closest("a")) setOpen(false, false);
  });
  root.addEventListener("keydown", (e) => {
    if (!isOpen() || !mobile.matches) return;
    if (e.key === "Escape") { e.preventDefault(); setOpen(false); return; }
    if (e.key !== "Tab") return;
    // Keep Tab inside the open drawer.
    const items = Array.from(sidebar.querySelectorAll<HTMLElement>("a[href], summary, button")).filter((n) => n.offsetParent !== null);
    const first = items[0], last = items[items.length - 1];
    if (!first || !last) return;
    const active = root.activeElement;
    if (e.shiftKey && active === first) { e.preventDefault(); last.focus(); }
    else if (!e.shiftKey && active === last) { e.preventDefault(); first.focus(); }
  });
  mobile.addEventListener("change", sync);
  const observer = new MutationObserver(sync);
  observer.observe(html, { attributes: true, attributeFilter: ["class"] });
  sync();
}
