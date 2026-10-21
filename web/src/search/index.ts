import { request } from "../shared/http.ts";

/** Returns the server-rendered result fragment, or "" on any failure. */
export async function fetchFragment(lang: string, q: string): Promise<string> {
  const query = q.trim();
  if (!query) return "";
  const r = await request(`/${lang}/search?q=${encodeURIComponent(query)}&fragment=1`);
  return r && r.ok ? r.text : "";
}
