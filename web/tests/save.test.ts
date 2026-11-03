import { expect, test } from "bun:test";
import { saveForm } from "../src/editor/save.ts";

const form = () => {
  const fd = new Map([["body", "x"]]);
  globalThis.FormData = class { constructor() { return fd as never; } } as never;
  return { action: "/admin/edit/en/a" } as HTMLFormElement;
};

test("2xx answer is a success and reports the final url", async () => {
  const r = await saveForm(form(), async () => ({ ok: true, url: "/en/a" }));
  expect(r).toEqual({ ok: true, url: "/en/a" });
});

test("a rejected save is not a success", async () => {
  expect(await saveForm(form(), async () => ({ ok: false, url: "/admin/edit/en/a" }))).toEqual({ ok: false });
});

test("a network failure is not a success", async () => {
  expect(await saveForm(form(), async () => { throw new Error("offline"); })).toEqual({ ok: false });
});
