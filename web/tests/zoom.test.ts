import { expect, test } from "bun:test";
import { limitPan, MAX_SCALE, RESET, zoomAt } from "../src/lightbox/zoom.ts";

test("zoom in keeps the pointer point fixed", () => {
  const v = zoomAt(RESET, 2, 100, 50);
  expect(v.scale).toBe(2);
  expect(v.x).toBe(-100);
  expect(v.y).toBe(-50);
});

test("scale is clamped and returns to reset at minimum", () => {
  expect(zoomAt({ scale: 7, x: 0, y: 0 }, 10, 0, 0).scale).toBe(MAX_SCALE);
  expect(zoomAt({ scale: 2, x: 30, y: 30 }, 0.1, 5, 5)).toEqual(RESET);
});

test("pan is limited so the image cannot leave the stage", () => {
  const v = limitPan({ scale: 2, x: 9999, y: -9999 }, 800, 600, 400, 300);
  expect(v.x).toBeLessThan(9999);
  expect(v.y).toBeGreaterThan(-9999);
  expect(limitPan({ scale: 1, x: 500, y: 500 }, 800, 600, 400, 300).x).toBe(0);
});
