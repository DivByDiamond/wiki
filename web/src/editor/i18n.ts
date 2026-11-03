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
const ZH: EditorText = {
  bold: "加粗", italic: "斜体", link: "链接",
  boldTip: "加粗 (Ctrl+B)", italicTip: "斜体 (Ctrl+I)", linkTip: "插入链接 (Ctrl+K)",
  tip: "提示", warn: "警告", recipe: "合成配方", version: "版本",
  blockTip: "插入区块：", save: "保存 (Ctrl+S)",
  restore: "在此浏览器中发现未保存的草稿。", restoreBtn: "恢复草稿", discardBtn: "丢弃",
  unsaved: "有未保存的更改。", saveFailed: "保存失败。草稿已保留，请重试。",
  viewEdit: "编辑", viewPreview: "预览", viewSplit: "分栏",
};

export function editorText(lang: string): EditorText {
  return lang === "ru" ? RU : lang === "zh" ? ZH : EN;
}
