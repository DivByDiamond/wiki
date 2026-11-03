export type Lang = "en" | "ru" | "zh";

const TEXTS = {
  copy: { en: "Copy", ru: "Копировать", zh: "复制" },
  copied: { en: "Copied", ru: "Скопировано", zh: "已复制" },
  copyFailed: { en: "Copy failed", ru: "Не удалось", zh: "复制失败" },
  copyLabel: { en: "Copy code to clipboard", ru: "Копировать код в буфер обмена", zh: "复制代码到剪贴板" },
  anchor: { en: "Link to this section", ru: "Ссылка на этот раздел", zh: "指向本节的链接" },
  top: { en: "Back to top", ru: "Наверх", zh: "回到顶部" },
  readMin: { en: "{n} min read", ru: "Время чтения: {n} мин", zh: "阅读约 {n} 分钟" },
  print: { en: "Print", ru: "Печать", zh: "打印" },
  paletteTitle: { en: "Quick search", ru: "Быстрый поиск", zh: "快速搜索" },
  palettePlaceholder: {
    en: "Search pages and headings",
    ru: "Поиск по страницам и заголовкам",
    zh: "搜索页面和标题",
  },
  recent: { en: "Recent searches", ru: "Недавние запросы", zh: "最近搜索" },
  headings: { en: "On this page", ru: "На этой странице", zh: "本页标题" },
  pages: { en: "Pages", ru: "Страницы", zh: "页面" },
  results: { en: "Search results", ru: "Результаты поиска", zh: "搜索结果" },
  searchAll: { en: "Search everywhere for “{q}”", ru: "Искать везде: «{q}»", zh: "全站搜索“{q}”" },
  empty: { en: "Nothing found", ru: "Ничего не найдено", zh: "未找到结果" },
  clearRecent: { en: "Clear", ru: "Очистить", zh: "清除" },
  hint: {
    en: "Up/Down to move, Enter to open, Esc to close",
    ru: "Вверх/вниз, Enter — открыть, Esc — закрыть",
    zh: "上下选择，Enter 打开，Esc 关闭",
  },
} as const;

export type TextKey = keyof typeof TEXTS;

export function pageLang(): Lang {
  const l = (typeof document !== "undefined" ? document.documentElement.lang : "") || "en";
  return l === "ru" || l === "zh" ? l : "en";
}

export function t(key: TextKey, vars: Record<string, string | number> = {}, lang: Lang = pageLang()): string {
  let s: string = TEXTS[key][lang];
  for (const [k, v] of Object.entries(vars)) s = s.replace(`{${k}}`, String(v));
  return s;
}
