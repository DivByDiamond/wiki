import { expect, test } from "bun:test";
import { DEFAULTS, load, save } from "../src/settings/store.ts";

const fake = (v: string | null) => ({ getItem: () => v });

test("returns defaults when storage is empty or broken", () => {
  expect(load(fake(null))).toEqual(DEFAULTS);
  expect(load(fake("{not json"))).toEqual(DEFAULTS);
  expect(load({ getItem: () => { throw new Error("blocked"); } })).toEqual(DEFAULTS);
});

test("ignores unknown values and keeps valid ones", () => {
  const s = load(fake(JSON.stringify({ theme: "dark", accent: "pink", size: "l", pixel: false })));
  expect(s.theme).toBe("dark");
  expect(s.accent).toBe(DEFAULTS.accent);
  expect(s.size).toBe("l");
  expect(s.pixel).toBe(false);
});

test("save swallows storage errors", () => {
  expect(() => save({ setItem: () => { throw new Error("full"); } }, DEFAULTS)).not.toThrow();
});
