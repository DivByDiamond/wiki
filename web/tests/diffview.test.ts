import { expect, test } from "bun:test";
import { lineDiff, threeWay } from "../src/editor/diffview.ts";

test("lineDiff marks inserted and deleted lines", () => {
  expect(lineDiff(["a", "b", "c"], ["a", "x", "c"])).toEqual([
    { kind: "same", line: "a" },
    { kind: "del", line: "b" },
    { kind: "ins", line: "x" },
    { kind: "same", line: "c" },
  ]);
});

test("threeWay diffs both sides against the base", () => {
  const d = threeWay("a\nb", "a\nb\nc", "z\nb");
  expect(d.current.filter((o) => o.kind === "ins").map((o) => o.line)).toEqual(["c"]);
  expect(d.proposed.filter((o) => o.kind === "del").map((o) => o.line)).toEqual(["a"]);
  expect(d.proposed.filter((o) => o.kind === "ins").map((o) => o.line)).toEqual(["z"]);
});
