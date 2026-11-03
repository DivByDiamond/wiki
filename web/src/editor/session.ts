import { clearDraft, draftKey, loadDraft, pageIdFromAction, saveDraft, shouldOffer } from "./draft.ts";
import { actionFor } from "./hotkeys.ts";
import { editorText } from "./i18n.ts";
import { saveForm } from "./save.ts";
import { bold, italic, link } from "./toolbar.ts";
import { debounce, el } from "../shared/dom.ts";

function store(): Storage | null {
  try { return window.localStorage; } catch { return null; }
}

/** Draft autosave, restore offer, unload warning and hotkeys for one editor. */
export function mountSession(area: HTMLTextAreaElement): void {
  const form = area.form;
  if (!form) return;
  const t = editorText(document.documentElement.lang);
  const id = pageIdFromAction(form.getAttribute("action") ?? "");
  const s = store();
  const key = id ? draftKey(id.lang, id.slug) : null;
  const original = area.value;
  let submitting = false;
  const dirty = () => area.value !== original;

  if (s && key) {
    const d = loadDraft(s, key);
    if (shouldOffer(d, original) && d) {
      const note = el("div", { class: "banner editor-restore", role: "status" }, t.restore + " ");
      const yes = el("button", { type: "button", class: "btn" }, t.restoreBtn);
      const no = el("button", { type: "button", class: "btn" }, t.discardBtn);
      yes.addEventListener("click", () => {
        area.value = d.text;
        area.dispatchEvent(new Event("input", { bubbles: true }));
        note.remove();
      });
      no.addEventListener("click", () => { clearDraft(s, key); note.remove(); });
      note.append(yes, " ", no);
      form.before(note);
    } else if (d) clearDraft(s, key);
    const persist = debounce(() => {
      if (submitting) return;
      if (dirty()) saveDraft(s, key, area.value, Date.now());
      else clearDraft(s, key);
    }, 500);
    area.addEventListener("input", persist);
  }

  // The draft is cleared only after the server accepted the save (2xx after its redirect).
  form.addEventListener("submit", (e) => {
    e.preventDefault();
    if (submitting) return;
    submitting = true;
    form.querySelector(".editor-error")?.remove();
    void saveForm(form).then((r) => {
      if (r.ok) {
        if (s && key) clearDraft(s, key);
        window.location.assign(r.url);
        return;
      }
      submitting = false;
      if (s && key && dirty()) saveDraft(s, key, area.value, Date.now());
      form.before(el("div", { class: "banner editor-error", role: "alert" }, t.saveFailed));
    });
  });

  window.addEventListener("beforeunload", (e) => {
    if (submitting || !dirty()) return;
    e.preventDefault();
    e.returnValue = t.unsaved;
  });

  area.addEventListener("keydown", (e) => {
    const action = actionFor(e);
    if (!action) return;
    e.preventDefault();
    if (action === "save") form.requestSubmit();
    else if (action === "bold") bold(area);
    else if (action === "italic") italic(area);
    else link(area);
  });
}
