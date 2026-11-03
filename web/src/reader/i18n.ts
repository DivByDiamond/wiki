export type Lang = "en" | "ru";

const TEXTS = {
  copy: { en: "Copy", ru: "Копировать" },
  copied: { en: "Copied", ru: "Скопировано" },
  copyFailed: { en: "Copy failed", ru: "Не удалось" },
  copyLabel: { en: "Copy code to clipboard", ru: "Копировать код в буфер обмена" },
  anchor: { en: "Link to this section", ru: "Ссылка на этот раздел" },
  top: { en: "Back to top", ru: "Наверх" },
  readMin: { en: "{n} min read", ru: "Время чтения: {n} мин" },
  print: { en: "Print", ru: "Печать" },
  paletteTitle: { en: "Quick search", ru: "Быстрый поиск" },
  palettePlaceholder: {
    en: "Search pages and headings",
    ru: "Поиск по страницам и заголовкам",
  },
  recent: { en: "Recent searches", ru: "Недавние запросы" },
  headings: { en: "On this page", ru: "На этой странице" },
  pages: { en: "Pages", ru: "Страницы" },
  results: { en: "Search results", ru: "Результаты поиска" },
  searchAll: { en: "Search everywhere for “{q}”", ru: "Искать везде: «{q}»" },
  empty: { en: "Nothing found", ru: "Ничего не найдено" },
  clearRecent: { en: "Clear", ru: "Очистить" },
  hint: {
    en: "Up/Down to move, Enter to open, Esc to close",
    ru: "Вверх/вниз, Enter — открыть, Esc — закрыть",
  },
} as const;

export type TextKey = keyof typeof TEXTS;

export function pageLang(): Lang {
  const l = (typeof document !== "undefined" ? document.documentElement.lang : "") || "en";
  return l === "ru" ? l : "en";
}

export function t(key: TextKey, vars: Record<string, string | number> = {}, lang: Lang = pageLang()): string {
  let s: string = TEXTS[key][lang];
  for (const [k, v] of Object.entries(vars)) s = s.replace(`{${k}}`, String(v));
  return s;
}
