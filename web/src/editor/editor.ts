import { mountMerge } from "./diffview.ts";
import { mountPreview } from "./preview.ts";
import { mountToolbar } from "./toolbar.ts";

export function mountEditor(): void {
  const area = document.getElementById("editor") as HTMLTextAreaElement | null;
  if (!area) return;
  const bar = document.getElementById("editor-toolbar");
  const out = document.getElementById("editor-preview");
  const merge = document.getElementById("merge");
  if (bar) mountToolbar(bar, area);
  if (out) mountPreview(area, out);
  if (merge) mountMerge(merge);
}
