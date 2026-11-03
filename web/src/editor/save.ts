export type SaveResult = { ok: true; url: string } | { ok: false };

type Fetch = (url: string, init: RequestInit) => Promise<Pick<Response, "ok" | "url">>;

/**
 * Posts the form without leaving the page. Only a 2xx answer after the server's redirect counts as saved;
 * a rejection or a network failure keeps the page (and the draft) intact.
 */
export async function saveForm(form: HTMLFormElement, doFetch: Fetch = fetch): Promise<SaveResult> {
  try {
    const body = new URLSearchParams(new FormData(form) as unknown as Record<string, string>);
    const r = await doFetch(form.action, {
      method: "POST",
      body,
      credentials: "same-origin",
      headers: { "Content-Type": "application/x-www-form-urlencoded" },
    });
    return r.ok ? { ok: true, url: r.url } : { ok: false };
  } catch {
    return { ok: false };
  }
}
