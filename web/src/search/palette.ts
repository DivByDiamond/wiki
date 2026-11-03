import { debounce, el } from "../shared/dom.ts";
import { isTextTarget } from "../shared/keys.ts";
import { pageLang, t } from "../reader/i18n.ts";
import { fetchFragment } from "./index.ts";
import { type Entry, filterEntries, parseHits } from "./match.ts";
import { addRecent, loadRecent, saveRecent } from "./recent.ts";

interface Row extends Entry { kind: "recent" | "heading" | "page" | "hit" | "all"; query?: string }

function localEntries(): { headings: Entry[]; pages: Entry[] } {
  const seen = new Set<string>();
  const headings: Entry[] = [];
  document.querySelectorAll<HTMLAnchorElement>(".toc a[href^='#']").forEach((a) => {
    const href = a.getAttribute("href")!;
    if (!seen.has(href)) { seen.add(href); headings.push({ title: a.textContent?.trim() ?? href, href }); }
  });
  const pages: Entry[] = [];
  document.querySelectorAll<HTMLAnchorElement>(".sidebar .nav-section a").forEach((a) => {
    pages.push({ title: a.textContent?.trim() ?? "", href: a.getAttribute("href") ?? "" });
  });
  return { headings, pages };
}

export function mountPalette(): void {
  if (document.getElementById("editor") || document.getElementById("merge")) return;
  const lang = pageLang();
  const input = el("input", {
    type: "search", class: "palette-input", autocomplete: "off", spellcheck: "false",
    role: "combobox", "aria-expanded": "true", "aria-controls": "palette-list", "aria-label": t("paletteTitle"),
    placeholder: t("palettePlaceholder"),
  });
  const list = el("div", { id: "palette-list", class: "palette-list", role: "listbox", "aria-label": t("results") });
  const dialog = el("div", { class: "palette", role: "dialog", "aria-modal": "true", "aria-label": t("paletteTitle") },
    input, list, el("p", { class: "palette-hint" }, t("hint")));
  const overlay = el("div", { class: "palette-overlay", hidden: "" }, dialog);
  document.body.append(overlay);

  let rows: Row[] = [];
  let active = 0;
  let serverHits: Entry[] = [];
  let latest = 0;
  let opener: Element | null = null;

  const go = (row: Row) => {
    if (row.kind === "recent") {
      input.value = row.query ?? row.title;
      refresh();
      return;
    }
    const q = input.value.trim();
    if (q && (row.kind === "all" || row.kind === "hit")) saveRecent(addRecent(loadRecent(), q));
    close();
    if (row.href.startsWith("#")) location.hash = row.href;
    else location.assign(row.href);
  };

  function paint(): void {
    list.replaceChildren();
    let group = "";
    rows.forEach((row, i) => {
      const label = row.kind === "recent" ? t("recent") : row.kind === "heading" ? t("headings")
        : row.kind === "page" ? t("pages") : row.kind === "hit" ? t("results") : "";
      if (label && label !== group) { group = label; list.append(el("div", { class: "palette-group", role: "presentation" }, label)); }
      const item = el("a", {
        id: `palette-opt-${i}`, class: "palette-item", role: "option", href: row.href || "#",
        "aria-selected": String(i === active),
      }, el("span", { class: "palette-title" }, row.title));
      if (row.hint) item.append(el("small", {}, row.hint));
      item.addEventListener("click", (e) => { e.preventDefault(); go(row); });
      item.addEventListener("mousemove", () => { if (active !== i) { active = i; mark(); } });
      list.append(item);
    });
    if (rows.length === 0) list.append(el("p", { class: "palette-empty" }, t("empty")));
    mark();
  }

  function mark(): void {
    list.querySelectorAll<HTMLElement>(".palette-item").forEach((n, i) => {
      n.setAttribute("aria-selected", String(i === active));
      if (i === active) { input.setAttribute("aria-activedescendant", n.id); n.scrollIntoView({ block: "nearest" }); }
    });
  }

  function refresh(): void {
    const q = input.value.trim();
    const local = localEntries();
    if (!q) {
      rows = [
        ...loadRecent().map((r): Row => ({ kind: "recent", title: r, href: "", query: r })),
        ...local.headings.slice(0, 8).map((e): Row => ({ ...e, kind: "heading" })),
      ];
    } else {
      const pages = filterEntries(local.pages, q);
      const taken = new Set(pages.map((p) => p.href));
      rows = [
        ...filterEntries(local.headings, q, 5).map((e): Row => ({ ...e, kind: "heading" })),
        ...pages.map((e): Row => ({ ...e, kind: "page" })),
        ...serverHits.filter((h) => !taken.has(h.href)).slice(0, 8).map((e): Row => ({ ...e, kind: "hit" })),
        { kind: "all", title: t("searchAll", { q }), href: `/${lang}/search?q=${encodeURIComponent(q)}` },
      ];
    }
    active = 0;
    paint();
  }

  const fetchHits = debounce(async (id: number, q: string) => {
    if (id !== latest) return;
    const html = await fetchFragment(lang, q);
    if (id !== latest) return;
    serverHits = parseHits(html);
    refresh();
  }, 150);

  input.addEventListener("input", () => {
    const q = input.value.trim();
    latest++;
    serverHits = [];
    refresh();
    if (q) fetchHits(latest, q);
  });

  function open(): void {
    if (!overlay.hidden) return;
    opener = document.activeElement;
    overlay.hidden = false;
    document.documentElement.classList.add("palette-open");
    input.value = "";
    serverHits = [];
    refresh();
    input.focus();
  }

  function close(): void {
    if (overlay.hidden) return;
    latest++;
    overlay.hidden = true;
    document.documentElement.classList.remove("palette-open");
    if (opener instanceof HTMLElement) opener.focus();
  }

  overlay.addEventListener("mousedown", (e) => { if (e.target === overlay) close(); });
  document.addEventListener("keydown", (e) => {
    const isOpen = !overlay.hidden;
    if ((e.key === "k" || e.key === "K") && (e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey) {
      e.preventDefault();
      if (isOpen) close(); else open();
      return;
    }
    if (!isOpen) {
      if (e.key === "/" && !e.ctrlKey && !e.metaKey && !e.altKey && !isTextTarget(e.target)) { e.preventDefault(); open(); }
      return;
    }
    if (e.key === "Escape") { e.preventDefault(); close(); }
    else if (e.key === "ArrowDown") { e.preventDefault(); active = rows.length ? (active + 1) % rows.length : 0; mark(); }
    else if (e.key === "ArrowUp") { e.preventDefault(); active = rows.length ? (active - 1 + rows.length) % rows.length : 0; mark(); }
    else if (e.key === "Enter") { e.preventDefault(); const r = rows[active]; if (r) go(r); }
    else if (e.key === "Tab") { e.preventDefault(); input.focus(); }
  });
}
