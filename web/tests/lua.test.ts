import { expect, test } from "bun:test";
import { looksLikeLua, tokenize } from "../src/reader/lua.ts";

const join = (src: string) => tokenize(src).map((t) => t.text).join("");
const kinds = (src: string) => tokenize(src).filter((t) => t.type !== "plain").map((t) => `${t.type}:${t.text}`);

test("tokens always reassemble to the source", () => {
  const src = `local x = 0x1F + 3.5e-2 -- note\nprint("a\\"b", [[long\nstr]]) --[[ block\n]] return 'x`;
  expect(join(src)).toBe(src);
});

test("classifies keywords, strings, numbers, comments and builtins", () => {
  expect(kinds(`local n = 42 -- hi`)).toEqual(["keyword:local", "number:42", "comment:-- hi"]);
  expect(kinds(`print("x")`)).toEqual(["builtin:print", 'string:"x"']);
  expect(kinds("--[==[ a ]] b ]==] end")).toEqual(["comment:--[==[ a ]] b ]==]", "keyword:end"]);
});

test("field names that match keywords or builtins stay plain", () => {
  expect(kinds("a.end = t.string")).toEqual([]);
});

test("unterminated string and comment do not hang", () => {
  expect(join('"abc')).toBe('"abc');
  expect(join("--[[ open")).toBe("--[[ open");
});

test("detects Lua blocks", () => {
  expect(looksLikeLua("language-lua", "x")).toBe(true);
  expect(looksLikeLua("language-rust", "local x = 1")).toBe(false);
  expect(looksLikeLua("", "local x = component.list()")).toBe(true);
  expect(looksLikeLua("", "just text")).toBe(false);
});
