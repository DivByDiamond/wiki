import { expect, test } from "bun:test";
import { filterEntries, score } from "../src/search/match.ts";
import { addRecent, parseRecent } from "../src/search/recent.ts";

test("recent searches: newest first, deduped, capped, trimmed", () => {
  let l: string[] = [];
  for (const q of ["a", "b", "  Lua   api ", "lua API", "c", "d", "e", "f", "g", "h", "i"]) l = addRecent(l, q);
  expect(l[0]).toBe("i");
  expect(l.length).toBe(8);
  expect(l.filter((x) => x.toLowerCase() === "lua api").length).toBeLessThanOrEqual(1);
  expect(addRecent(["x"], "   ")).toEqual(["x"]);
});

test("recent parser ignores bad data", () => {
  expect(parseRecent("{")).toEqual([]);
  expect(parseRecent(null)).toEqual([]);
  expect(parseRecent('["a",1,"",null,"b"]')).toEqual(["a", "b"]);
});

test("scoring prefers prefixes and word starts, requires all terms", () => {
  expect(score("Computer", "comp")).toBe(0);
  expect(score("Flash Memory", "mem")).toBe(1);
  expect(score("Hammer", "mm")).toBe(3);
  expect(score("Computer", "comp disk")).toBe(-1);
  expect(score("x", "")).toBe(-1);
});

test("filterEntries orders by score then position and limits", () => {
  const e = [{ title: "Redstone Interface", href: "/1" }, { title: "Interface", href: "/2" }, { title: "Other", href: "/3" }];
  expect(filterEntries(e, "inter").map((x) => x.href)).toEqual(["/2", "/1"]);
  expect(filterEntries(e, "e", 1).length).toBe(1);
});
