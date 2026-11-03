// WCAG contrast check for the theme tokens. Run: bun run dev/contrast.ts (exit 1 on failure).
import { readFileSync } from "node:fs";

const css = readFileSync(new URL("../styles/tokens.css", import.meta.url), "utf8");
const block = (sel: string) => {
  const i = css.indexOf(sel + " {");
  if (i < 0) throw new Error(`missing ${sel}`);
  return css.slice(i, css.indexOf("}", i));
};
const vars = (b: string) => Object.fromEntries([...b.matchAll(/--([\w-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)].map((m) => [m[1], m[2]]));
const lum = (hex: string) => {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((v) => (v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0]! + 0.7152 * c[1]! + 0.0722 * c[2]!;
};
export const ratio = (a: string, b: string) => {
  const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p);
  return (x! + 0.05) / (y! + 0.05);
};

const light = vars(block(":root"));
const dark = { ...light, ...vars(block(':root[data-theme="dark"]')) };
const accents: Record<string, [string, string]> = {
  green: [light.accent!, "#6fd987"], violet: ["#6a3dc2", "#bfa9ff"], orange: ["#9a4400", "#ffa763"], teal: ["#00706a", "#44d6c9"],
};
let failed = 0;
const rows: string[] = [];
function check(theme: string, name: string, fg: string, bg: string, min = 4.5) {
  const r = ratio(fg, bg);
  const ok = r >= min;
  if (!ok) failed++;
  rows.push(`| ${theme} | ${name} | ${fg} on ${bg} | ${r.toFixed(2)} | ${min} | ${ok ? "pass" : "FAIL"} |`);
}
for (const [theme, t] of [["light", light], ["dark", dark]] as const) {
  const i = theme === "light" ? 0 : 1;
  check(theme, "text", t.fg!, t.bg!);
  check(theme, "text on stone panel alt", t.fg!, t["bg-alt"]!);
  check(theme, "text on button slot", t.fg!, t.slot!);
  check(theme, "muted text", t.muted!, t.bg!);
  check(theme, "muted text on alt", t.muted!, t["bg-alt"]!);
  check(theme, "warn on panel", t.warn!, t.bg!);
  check(theme, "note on panel", t.note!, t.bg!);
  check(theme, "primary button", t["on-accent"]!, t.accent!);
  for (const [n, pair] of Object.entries(accents)) {
    check(theme, `link (${n}) on panel`, pair[i]!, t.bg!);
    check(theme, `link (${n}) on alt`, pair[i]!, t["bg-alt"]!);
    check(theme, `button text (${n})`, t["on-accent"]!, pair[i]!);
  }
  check(theme, "focus ring / UI border vs panel", t.accent!, t.bg!, 3);
}
for (const k of ["note", "tip", "warn", "version"]) check("both", `tooltip title ${k}`, light[`tt-${k}`]!, light["tt-bg"]!);
check("both", "tooltip text", light["tt-fg"]!, light["tt-bg"]!);
console.log("| Theme | Pair | Colors | Ratio | Min | Result |\n|---|---|---|---|---|---|\n" + rows.join("\n"));
if (failed) { console.error(`${failed} failing pair(s)`); process.exit(1); }
