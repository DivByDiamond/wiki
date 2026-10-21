import { debounce } from "../shared/dom.ts";
import { request } from "../shared/http.ts";

export function mountPreview(area: HTMLTextAreaElement, out: HTMLElement): void {
  let latest = 0;
  const run = debounce(async () => {
    const id = ++latest;
    const r = await request("/admin/preview", {
      method: "POST",
      headers: { "content-type": "text/plain" },
      body: area.value,
    });
    if (id !== latest) return;
    if (r && r.ok) out.innerHTML = r.text;
    else out.textContent = r ? `Preview failed (${r.status})` : "Preview unavailable (network error)";
  }, 300);
  area.addEventListener("input", run);
  run();
}
