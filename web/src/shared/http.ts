export interface Reply { ok: boolean; status: number; text: string }

/** Never throws: a network failure yields null. */
export async function request(url: string, init?: RequestInit): Promise<Reply | null> {
  try {
    const r = await fetch(url, init);
    return { ok: r.ok, status: r.status, text: await r.text() };
  } catch {
    return null;
  }
}
