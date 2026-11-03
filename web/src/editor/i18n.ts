const EN = {
  bold: "Bold", italic: "Italic", link: "Link",
  boldTip: "Bold (Ctrl+B)", italicTip: "Italic (Ctrl+I)", linkTip: "Insert link (Ctrl+K)",
  tip: "Tip", warn: "Warning", recipe: "Recipe", version: "Version",
  blockTip: "Insert block: ", save: "Save (Ctrl+S)",
  restore: "An unsaved draft from this browser was found.", restoreBtn: "Restore draft", discardBtn: "Discard",
  unsaved: "You have unsaved changes.", saveFailed: "Save failed. Your draft is kept, try again.",
  viewEdit: "Editor", viewPreview: "Preview", viewSplit: "Split view",
};
export type EditorText = typeof EN;

const RU: EditorText = {
  bold: "Жирный", italic: "Курсив", link: "Ссылка",
  boldTip: "Жирный (Ctrl+B)", italicTip: "Курсив (Ctrl+I)", linkTip: "Вставить ссылку (Ctrl+K)",
  tip: "Подсказка", warn: "Предупреждение", recipe: "Рецепт", version: "Версия",
  blockTip: "Вставить блок: ", save: "Сохранить (Ctrl+S)",
  restore: "Найден несохранённый черновик в этом браузере.", restoreBtn: "Восстановить черновик", discardBtn: "Отбросить",
  unsaved: "Есть несохранённые изменения.", saveFailed: "Не удалось сохранить. Черновик сохранён, попробуйте ещё раз.",
  viewEdit: "Редактор", viewPreview: "Предпросмотр", viewSplit: "Рядом",
};

export function editorText(lang: string): EditorText {
  return lang === "ru" ? RU : EN;
}
