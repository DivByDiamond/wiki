import { expect, test } from "bun:test";
import { apply } from "../src/settings/theme.ts";
import { DEFAULTS } from "../src/settings/store.ts";

const root = () => ({ dataset: {} as Record<string, string> });

test("auto follows the system preference", () => {
  const a = root();
  apply(a as unknown as HTMLElement, DEFAULTS, true);
  expect(a.dataset.theme).toBe("dark");
  const b = root();
  apply(b as unknown as HTMLElement, DEFAULTS, false);
  expect(b.dataset.theme).toBe("light");
});

test("explicit settings are written as data attributes", () => {
  const r = root();
  apply(r as unknown as HTMLElement, { theme: "light", accent: "teal", size: "l", pixel: false }, true);
  expect(r.dataset).toEqual({ theme: "light", accent: "teal", size: "l", pixel: "0" });
});
