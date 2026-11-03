import { mountMerge } from "./diffview.ts";
import { mountPreview } from "./preview.ts";
import { mountSession } from "./session.ts";
import { mountViewToggle } from "./split.ts";
import { mountToolbar } from "./toolbar.ts";

export function mountEditor(): void {
  const merge = document.getElementById("merge");
  if (merge) mountMerge(merge);
  const area = document.getElementById("editor") as HTMLTextAreaElement | null;
  if (!area) return;
  const bar = document.getElementById("editor-toolbar");
  const out = document.getElementById("editor-preview");
  if (bar) mountToolbar(bar, area);
  if (out) mountPreview(area, out);
  mountSession(area);
  const layout = document.querySelector<HTMLElement>(".admin-editor-layout");
  const pane = document.querySelector<HTMLElement>(".admin-preview-pane");
  if (layout && pane) mountViewToggle(layout, area, pane);
}
