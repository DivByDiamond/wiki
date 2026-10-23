import { debounce } from "../shared/dom.ts";
import { request } from "../shared/http.ts";

export function mountPreview(area: HTMLTextAreaElement, out: HTMLElement): void {
  const lang = typeof document === "undefined" ? "en" : document.documentElement.lang;
  const failed = lang === "ru" ? "Ошибка предпросмотра" : lang === "zh" ? "预览失败" : "Preview failed";
  const offline = lang === "ru" ? "Предпросмотр недоступен (ошибка сети)"
    : lang === "zh" ? "无法预览（网络错误）" : "Preview unavailable (network error)";
  let latest = 0;
  const run = debounce(async (id: number, body: string) => {
    if (id !== latest) return;
    const r = await request("/admin/preview", {
      method: "POST",
      headers: { "content-type": "text/plain" },
      body,
    });
    if (id !== latest) return;
    if (r && r.ok) out.innerHTML = r.text;
    else out.textContent = r ? `${failed} (${r.status})` : offline;
  }, 300);
  const refresh = () => run(++latest, area.value);
  area.addEventListener("input", refresh);
  refresh();
}
