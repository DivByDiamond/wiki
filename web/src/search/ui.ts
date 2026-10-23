import { debounce } from "../shared/dom.ts";
import { onEscape, onSlash } from "../shared/keys.ts";
import { fetchFragment } from "./index.ts";

export function mountSearch(): void {
  const input = document.getElementById("search-input") as HTMLInputElement | null;
  const box = document.getElementById("search-results");
  if (!input) return;
  onSlash(() => input.focus());
  if (!box) return;

  let latest = 0;
  const close = () => { latest++; box.replaceChildren(); };
  onEscape(close);
  document.addEventListener("click", (event) => {
    if (event.target instanceof Node && !box.contains(event.target) && event.target !== input) close();
  });
  const run = debounce(async (id: number, query: string) => {
    if (id !== latest) return;
    const html = await fetchFragment(document.documentElement.lang || "en", query);
    if (id === latest) box.innerHTML = html;
  }, 200);
  input.addEventListener("input", () => run(++latest, input.value));
}
