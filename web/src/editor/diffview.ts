export type Op = { kind: "same" | "ins" | "del"; line: string };

/** Line diff via LCS: lines only in `to` are "ins", lines only in `from` are "del". */
export function lineDiff(from: string[], to: string[]): Op[] {
  const n = from.length, m = to.length;
  const lcs: number[][] = Array.from({ length: n + 1 }, () => new Array<number>(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      lcs[i]![j] = from[i] === to[j] ? lcs[i + 1]![j + 1]! + 1 : Math.max(lcs[i + 1]![j]!, lcs[i]![j + 1]!);
    }
  }
  const ops: Op[] = [];
  let i = 0, j = 0;
  while (i < n && j < m) {
    if (from[i] === to[j]) { ops.push({ kind: "same", line: from[i]! }); i++; j++; }
    else if (lcs[i + 1]![j]! >= lcs[i]![j + 1]!) ops.push({ kind: "del", line: from[i++]! });
    else ops.push({ kind: "ins", line: to[j++]! });
  }
  while (i < n) ops.push({ kind: "del", line: from[i++]! });
  while (j < m) ops.push({ kind: "ins", line: to[j++]! });
  return ops;
}

const lines = (s: string) => s.split("\n");

/** Both sides of a merge are shown as changes against the common base. */
export function threeWay(base: string, current: string, proposed: string): { current: Op[]; proposed: Op[] } {
  return {
    current: lineDiff(lines(base), lines(current)),
    proposed: lineDiff(lines(base), lines(proposed)),
  };
}

function fill(pre: HTMLElement, ops: Op[]): void {
  pre.classList.add("diff");
  pre.replaceChildren(...ops.map((o) => {
    const node = document.createElement(o.kind === "ins" ? "ins" : o.kind === "del" ? "del" : "span");
    node.textContent = o.line || " ";
    return node;
  }));
}

export function mountMerge(root: HTMLElement): void {
  const get = (role: string) => root.querySelector<HTMLElement>(`pre[data-role=${role}]`);
  const base = get("base"), cur = get("current"), prop = get("proposed");
  if (!base || !cur || !prop) return;
  const d = threeWay(base.textContent ?? "", cur.textContent ?? "", prop.textContent ?? "");
  fill(cur, d.current);
  fill(prop, d.proposed);
}
