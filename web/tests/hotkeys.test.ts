import { expect, test } from "bun:test";
import { actionFor, linkEdit, wrapEdit } from "../src/editor/hotkeys.ts";

const ev = (key: string, o: object = {}) => ({ key, ctrlKey: true, metaKey: false, altKey: false, shiftKey: false, ...o });

test("maps ctrl combos", () => {
  expect(actionFor(ev("s"))).toBe("save");
  expect(actionFor(ev("B"))).toBe("bold");
  expect(actionFor(ev("i"))).toBe("italic");
  expect(actionFor(ev("k"))).toBe("link");
  expect(actionFor(ev("s", { ctrlKey: false, metaKey: true }))).toBe("save");
});
test("ignores plain and alt/shift combos", () => {
  expect(actionFor(ev("s", { ctrlKey: false }))).toBeNull();
  expect(actionFor(ev("s", { shiftKey: true }))).toBeNull();
  expect(actionFor(ev("x"))).toBeNull();
});
test("wrap selection", () => {
  expect(wrapEdit("a word b", 2, 6, "**")).toEqual({ value: "a **word** b", start: 4, end: 8 });
});
test("wrap empty selection places caret inside", () => {
  expect(wrapEdit("ab", 1, 1, "*")).toEqual({ value: "a**b", start: 2, end: 2 });
});
test("wrap toggles off when already wrapped", () => {
  expect(wrapEdit("a **word** b", 4, 8, "**")).toEqual({ value: "a word b", start: 2, end: 6 });
});
test("link with selection selects url placeholder", () => {
  const r = linkEdit("see docs", 4, 8);
  expect(r.value).toBe("see [docs](url)");
  expect(r.value.slice(r.start, r.end)).toBe("url");
});
test("link without selection selects text placeholder", () => {
  const r = linkEdit("", 0, 0, "text");
  expect(r.value).toBe("[text](url)");
  expect(r.value.slice(r.start, r.end)).toBe("text");
});
