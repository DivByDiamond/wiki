import { expect, test } from "bun:test";
import { parseOpen } from "../src/shared/sidebar.ts";

test("parseOpen keeps only boolean entries", () => {
  expect(parseOpen('{"a":true,"b":false,"c":1}')).toEqual({ a: true, b: false });
});

test("parseOpen tolerates garbage", () => {
  expect(parseOpen(null)).toEqual({});
  expect(parseOpen("nope")).toEqual({});
  expect(parseOpen("[1]")).toEqual({});
});
