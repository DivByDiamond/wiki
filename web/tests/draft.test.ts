import { expect, test } from "bun:test";
import { clearDraft, draftKey, loadDraft, pageIdFromAction, saveDraft, shouldOffer } from "../src/editor/draft.ts";

const mem = () => {
  const m = new Map<string, string>();
  return { getItem: (k: string) => m.get(k) ?? null, setItem: (k: string, v: string) => void m.set(k, v), removeItem: (k: string) => void m.delete(k) };
};

test("key combines locale and slug", () => {
  expect(draftKey("ru", "a/b")).toBe("wiki:draft:ru:a/b");
});
test("page id parsed from form action", () => {
  expect(pageIdFromAction("/admin/edit/ru/robots/cpu")).toEqual({ lang: "ru", slug: "robots/cpu" });
  expect(pageIdFromAction("/other")).toBeNull();
});
test("save/load/clear roundtrip", () => {
  const s = mem();
  saveDraft(s, "k", "hello", 5);
  expect(loadDraft(s, "k")).toEqual({ text: "hello", at: 5 });
  clearDraft(s, "k");
  expect(loadDraft(s, "k")).toBeNull();
});
test("corrupt data is ignored", () => {
  const s = mem();
  s.setItem("k", "{bad");
  expect(loadDraft(s, "k")).toBeNull();
  s.setItem("k", JSON.stringify({ text: 1 }));
  expect(loadDraft(s, "k")).toBeNull();
});
test("throwing storage does not throw", () => {
  const s = { getItem: () => { throw new Error("x"); }, setItem: () => { throw new Error("x"); }, removeItem: () => { throw new Error("x"); } };
  expect(() => saveDraft(s, "k", "t", 1)).not.toThrow();
  expect(loadDraft(s, "k")).toBeNull();
  expect(() => clearDraft(s, "k")).not.toThrow();
});
test("offer only when draft differs from server text", () => {
  expect(shouldOffer({ text: "a", at: 1 }, "a")).toBe(false);
  expect(shouldOffer({ text: "b", at: 1 }, "a")).toBe(true);
  expect(shouldOffer(null, "a")).toBe(false);
});
