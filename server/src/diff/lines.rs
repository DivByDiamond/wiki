#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Line<'a> {
    Same(&'a str),
    Added(&'a str),
    Removed(&'a str),
}

/// Largest LCS table (in cells) the reader is willing to build.
pub const MAX_DIFF_CELLS: usize = 4_000_000;

/// Table size for `n` and `m` lines; saturates instead of overflowing.
fn cells(n: usize, m: usize) -> usize {
    n.saturating_add(1).saturating_mul(m.saturating_add(1))
}

/// Line diff; `None` when the differing middle needs more than `max_cells`.
pub fn diff_lines_bounded<'a>(
    old: &'a str,
    new: &'a str,
    max_cells: usize,
) -> Option<Vec<Line<'a>>> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let mid_a = &a[prefix..a.len() - suffix];
    let mid_b = &b[prefix..b.len() - suffix];
    if cells(mid_a.len(), mid_b.len()) > max_cells {
        return None;
    }
    let mut out: Vec<Line<'a>> = a[..prefix].iter().map(|l| Line::Same(l)).collect();
    out.extend(lcs_diff(mid_a, mid_b));
    out.extend(a[a.len() - suffix..].iter().map(|l| Line::Same(l)));
    Some(out)
}

fn lcs_diff<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<Line<'a>> {
    let (n, m) = (a.len(), b.len());

    // t[i][j] — длина наибольшей общей подпоследовательности a[i..] и b[j..]
    let mut t = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            t[i][j] = if a[i] == b[j] {
                t[i + 1][j + 1] + 1
            } else {
                t[i + 1][j].max(t[i][j + 1])
            };
        }
    }

    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < n && j < m {
        if a[i] == b[j] {
            out.push(Line::Same(a[i]));
            i += 1;
            j += 1;
        } else if t[i + 1][j] >= t[i][j + 1] {
            out.push(Line::Removed(a[i]));
            i += 1;
        } else {
            out.push(Line::Added(b[j]));
            j += 1;
        }
    }
    out.extend(a[i..].iter().map(|l| Line::Removed(l)));
    out.extend(b[j..].iter().map(|l| Line::Added(l)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full<'a>(old: &'a str, new: &'a str) -> Vec<Line<'a>> {
        diff_lines_bounded(old, new, MAX_DIFF_CELLS).unwrap()
    }

    #[test]
    fn identical_is_all_same() {
        assert_eq!(full("a\nb", "a\nb"), vec![Line::Same("a"), Line::Same("b")]);
    }

    #[test]
    fn changed_line_is_removed_then_added() {
        assert_eq!(
            full("a\nb\nc", "a\nX\nc"),
            vec![
                Line::Same("a"),
                Line::Removed("b"),
                Line::Added("X"),
                Line::Same("c")
            ]
        );
    }

    #[test]
    fn appended_and_deleted_lines() {
        assert_eq!(full("a", "a\nb"), vec![Line::Same("a"), Line::Added("b")]);
        assert_eq!(full("a\nb", "a"), vec![Line::Same("a"), Line::Removed("b")]);
    }

    #[test]
    fn empty_old_is_all_added() {
        assert_eq!(full("", "a"), vec![Line::Added("a")]);
    }

    #[test]
    fn identical_huge_inputs_are_trimmed() {
        let text = "x\n".repeat(200_000);
        let out = full(&text, &text);
        assert_eq!(out.len(), 200_000);
        assert!(out.iter().all(|l| *l == Line::Same("x")));
    }

    #[test]
    fn completely_different_large_inputs_exceed_cap() {
        let a: String = (0..5_000).map(|i| format!("a{i}\n")).collect();
        let b: String = (0..5_000).map(|i| format!("b{i}\n")).collect();
        assert!(diff_lines_bounded(&a, &b, MAX_DIFF_CELLS).is_none());
    }

    #[test]
    fn small_change_in_huge_file_keeps_prefix_and_suffix() {
        let lines: Vec<String> = (0..100_000).map(|i| format!("l{i}")).collect();
        let old = lines.join("\n");
        let mut changed = lines.clone();
        changed[50_000] = "CHANGED".into();
        let new = changed.join("\n");
        let out = full(&old, &new);
        assert_eq!(out.len(), 100_001);
        assert_eq!(out[49_999], Line::Same("l49999"));
        assert_eq!(out[50_000], Line::Removed("l50000"));
        assert_eq!(out[50_001], Line::Added("CHANGED"));
        assert_eq!(out[50_002], Line::Same("l50001"));
        assert_eq!(out[100_000], Line::Same("l99999"));
    }

    #[test]
    fn cell_count_does_not_overflow() {
        let half = usize::MAX / 2;
        assert_eq!(cells(half, half), usize::MAX);
        assert_eq!(cells(usize::MAX, 1), usize::MAX);
        assert_eq!(cells(2, 3), 12);
    }
}
