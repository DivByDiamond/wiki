import { expect, test } from "bun:test";
import { editorText } from "../src/editor/i18n.ts";

test("every key is present in all languages", () => {
  const en = Object.keys(editorText("en"));
  for (const l of ["ru"]) expect(Object.keys(editorText(l)).sort()).toEqual(en.sort());
  expect(editorText("zh").bold).toBe(editorText("en").bold);
  expect(editorText("xx").bold).toBe(editorText("en").bold);
});
