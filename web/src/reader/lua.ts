export type TokenType = "comment" | "string" | "number" | "keyword" | "builtin" | "plain";
export interface Token { type: TokenType; text: string }

const KEYWORDS = new Set([
  "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
  "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
]);
const BUILTINS = new Set([
  "print", "pairs", "ipairs", "require", "type", "tostring", "tonumber", "pcall", "error", "assert",
  "select", "setmetatable", "getmetatable", "string", "table", "math", "os", "io", "coroutine",
  "component", "computer", "event", "term", "unpack", "next", "rawget", "rawset",
]);

/** Matches `[[`, `[=[` ... at `i`; returns the closing bracket length info or null. */
function longOpen(src: string, i: number): { level: number; len: number } | null {
  if (src[i] !== "[") return null;
  let j = i + 1;
  while (src[j] === "=") j++;
  return src[j] === "[" ? { level: j - i - 1, len: j - i + 1 } : null;
}

function longEnd(src: string, from: number, level: number): number {
  const close = "]" + "=".repeat(level) + "]";
  const at = src.indexOf(close, from);
  return at < 0 ? src.length : at + close.length;
}

const isIdStart = (c: string) => /[A-Za-z_]/.test(c);
const isIdPart = (c: string) => /[A-Za-z0-9_]/.test(c);
const isDigit = (c: string) => c >= "0" && c <= "9";

/** Splits Lua source into tokens; concatenating all `text` gives the source back. */
export function tokenize(src: string): Token[] {
  const out: Token[] = [];
  const push = (type: TokenType, text: string) => {
    const last = out[out.length - 1];
    if (last && last.type === type && type === "plain") last.text += text;
    else out.push({ type, text });
  };
  let i = 0;
  while (i < src.length) {
    const c = src[i]!;
    if (c === "-" && src[i + 1] === "-") {
      const lo = longOpen(src, i + 2);
      const end = lo ? longEnd(src, i + 2 + lo.len, lo.level) : (src.indexOf("\n", i) < 0 ? src.length : src.indexOf("\n", i));
      push("comment", src.slice(i, end));
      i = end;
    } else if (c === '"' || c === "'") {
      let j = i + 1;
      while (j < src.length && src[j] !== c && src[j] !== "\n") j += src[j] === "\\" ? 2 : 1;
      j = Math.min(src.length, j + (src[j] === c ? 1 : 0));
      push("string", src.slice(i, j));
      i = j;
    } else if (c === "[" && longOpen(src, i)) {
      const lo = longOpen(src, i)!;
      const end = longEnd(src, i + lo.len, lo.level);
      push("string", src.slice(i, end));
      i = end;
    } else if (isDigit(c) || (c === "." && isDigit(src[i + 1] ?? ""))) {
      let j = i + 1;
      if (c === "0" && /[xX]/.test(src[j] ?? "")) {
        j++;
        while (/[0-9a-fA-F.]/.test(src[j] ?? "")) j++;
      } else {
        while (/[0-9.]/.test(src[j] ?? "")) j++;
        if (/[eE]/.test(src[j] ?? "")) {
          j++;
          if (/[+-]/.test(src[j] ?? "")) j++;
          while (isDigit(src[j] ?? "")) j++;
        }
      }
      push("number", src.slice(i, j));
      i = j;
    } else if (isIdStart(c)) {
      let j = i + 1;
      while (j < src.length && isIdPart(src[j]!)) j++;
      const word = src.slice(i, j);
      const afterDot = src[i - 1] === "." || src[i - 1] === ":";
      push(afterDot ? "plain" : KEYWORDS.has(word) ? "keyword" : BUILTINS.has(word) ? "builtin" : "plain", word);
      i = j;
    } else {
      push("plain", c);
      i++;
    }
  }
  return out;
}

/** Fills `code` with highlighted spans; text is only ever set via textContent. */
export function highlightInto(code: HTMLElement, src: string): void {
  const doc = code.ownerDocument;
  const frag = doc.createDocumentFragment();
  for (const tok of tokenize(src)) {
    if (tok.type === "plain") {
      frag.append(doc.createTextNode(tok.text));
    } else {
      const span = doc.createElement("span");
      span.className = `tok-${tok.type}`;
      span.textContent = tok.text;
      frag.append(span);
    }
  }
  code.replaceChildren(frag);
}

/** Decides whether a code block is Lua: explicit class, or untagged and Lua-looking. */
export function looksLikeLua(className: string, src: string): boolean {
  if (/\blanguage-lua\b/.test(className)) return true;
  if (/\blanguage-/.test(className)) return false;
  return /\blocal\s+\w+\s*=|\bfunction\b[^\n]*\)|\bcomponent\.\w+|\bthen\b[\s\S]*\bend\b/.test(src);
}
