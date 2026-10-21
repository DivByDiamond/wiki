import { debounce } from "../shared/dom.ts";
import { onSlash } from "../shared/keys.ts";
import { fetchFragment } from "./index.ts";

export function mountSearch(): void {
  const input = document.getElementById("search-input") as HTMLInputElement | null;
  const box = document.getElementById("search-results");
  if (!input) return;
  onSlash(() => input.focus());
  if (!box) return;

  let latest = 0;
  const run = debounce(async () => {
    const id = ++latest;
    const html = await fetchFragment(document.documentElement.lang || "en", input.value);
    if (id === latest) box.innerHTML = html;
  }, 200);
  input.addEventListener("input", run);
}
