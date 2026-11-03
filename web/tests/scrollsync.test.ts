import { expect, test } from "bun:test";
import { mapScroll } from "../src/editor/scrollsync.ts";

test("proportional mapping", () => {
  expect(mapScroll(50, 100, 400)).toBe(200);
  expect(mapScroll(0, 100, 400)).toBe(0);
  expect(mapScroll(100, 100, 400)).toBe(400);
});
test("no scrollable source maps to 0", () => {
  expect(mapScroll(10, 0, 400)).toBe(0);
});
test("clamps", () => {
  expect(mapScroll(150, 100, 400)).toBe(400);
  expect(mapScroll(-5, 100, 400)).toBe(0);
});
