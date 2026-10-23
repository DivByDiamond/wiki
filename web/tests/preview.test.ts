import { expect, test } from "bun:test";
import { mountPreview } from "../src/editor/preview.ts";

test("an edit invalidates the previous preview before the debounce finishes", async () => {
  const original = globalThis.fetch;
  const pending: Array<(response: Response) => void> = [];
  globalThis.fetch = Object.assign(
    () => new Promise<Response>((resolve) => pending.push(resolve)),
    { preconnect: original.preconnect },
  );
  const area = Object.assign(new EventTarget(), { value: "old draft" });
  const out = { innerHTML: "", textContent: "" };
  try {
    mountPreview(area as HTMLTextAreaElement, out as HTMLElement);
    await Bun.sleep(340);
    expect(pending).toHaveLength(1);
    area.value = "new draft";
    area.dispatchEvent(new Event("input"));
    pending[0]!(new Response("<p>old draft</p>"));
    await Bun.sleep(10);
    expect(out.innerHTML).toBe("");
    await Bun.sleep(340);
    expect(pending).toHaveLength(2);
    pending[1]!(new Response("<p>new draft</p>"));
    await Bun.sleep(10);
    expect(out.innerHTML).toBe("<p>new draft</p>");
  } finally {
    globalThis.fetch = original;
  }
});
