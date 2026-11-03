import { expect, test } from "bun:test";
import { readingMinutes } from "../src/reader/readtime.ts";
import { parseMap, remember } from "../src/reader/scroll.ts";
import { currentHeading } from "../src/reader/toc.ts";

test("reading time uses words for text and characters for CJK, minimum 1", () => {
  expect(readingMinutes("")).toBe(1);
  expect(readingMinutes("слово ".repeat(600))).toBe(3);
  expect(readingMinutes("汉".repeat(800))).toBe(2);
});

test("scroll memory stores, clears near the top, and caps entries", () => {
  let m = remember({}, "/ru/a", 500, 1);
  expect(m["/ru/a"]).toEqual({ y: 500, at: 1 });
  m = remember(m, "/ru/a", 10, 2);
  expect(m["/ru/a"]).toBeUndefined();
  for (let i = 0; i < 100; i++) m = remember(m, `/p${i}`, 300, 10 + i);
  expect(Object.keys(m).length).toBe(60);
  expect(m["/p99"]).toBeDefined();
  expect(m["/p0"]).toBeUndefined();
});

test("scroll map parser survives garbage", () => {
  expect(parseMap("nope")).toEqual({});
  expect(parseMap("[1]")).toEqual({});
  expect(parseMap(JSON.stringify({ a: { y: 1, at: 2 }, b: { y: "x" } }))).toEqual({ a: { y: 1, at: 2 } });
});

test("current heading is first visible, else previous", () => {
  const ids = ["a", "b", "c"];
  expect(currentHeading(ids, new Set(["b", "c"]), "a")).toBe("b");
  expect(currentHeading(ids, new Set(), "b")).toBe("b");
  expect(currentHeading(ids, new Set(), null)).toBeNull();
  expect(currentHeading(ids, new Set(), "zzz")).toBeNull();
});
