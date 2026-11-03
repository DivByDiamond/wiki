export type Action = "save" | "bold" | "italic" | "link";
export type Edit = { value: string; start: number; end: number };
type Keys = { key: string; ctrlKey: boolean; metaKey: boolean; altKey: boolean; shiftKey: boolean };

const MAP: Record<string, Action> = { s: "save", b: "bold", i: "italic", k: "link" };

export function actionFor(e: Keys): Action | null {
  if (!(e.ctrlKey || e.metaKey) || e.altKey || e.shiftKey) return null;
  return MAP[e.key.toLowerCase()] ?? null;
}

/** Wraps the selection in `mark`; unwraps when it is already wrapped. */
export function wrapEdit(value: string, a: number, z: number, mark: string): Edit {
  const n = mark.length;
  if (a >= n && value.slice(a - n, a) === mark && value.slice(z, z + n) === mark) {
    return { value: value.slice(0, a - n) + value.slice(a, z) + value.slice(z + n), start: a - n, end: z - n };
  }
  return { value: value.slice(0, a) + mark + value.slice(a, z) + mark + value.slice(z), start: a + n, end: z + n };
}

export function linkEdit(value: string, a: number, z: number, placeholder = "text"): Edit {
  const sel = value.slice(a, z);
  const text = sel || placeholder;
  const out = value.slice(0, a) + `[${text}](url)` + value.slice(z);
  const start = sel ? a + text.length + 3 : a + 1;
  return { value: out, start, end: start + (sel ? 3 : text.length) };
}
