#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Line<'a> {
    Same(&'a str),
    Added(&'a str),
    Removed(&'a str),
}

pub fn diff_lines<'a>(old: &'a str, new: &'a str) -> Vec<Line<'a>> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
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

    #[test]
    fn identical_is_all_same() {
        assert_eq!(
            diff_lines("a\nb", "a\nb"),
            vec![Line::Same("a"), Line::Same("b")]
        );
    }

    #[test]
    fn changed_line_is_removed_then_added() {
        assert_eq!(
            diff_lines("a\nb\nc", "a\nX\nc"),
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
        assert_eq!(
            diff_lines("a", "a\nb"),
            vec![Line::Same("a"), Line::Added("b")]
        );
        assert_eq!(
            diff_lines("a\nb", "a"),
            vec![Line::Same("a"), Line::Removed("b")]
        );
    }

    #[test]
    fn empty_old_is_all_added() {
        assert_eq!(diff_lines("", "a"), vec![Line::Added("a")]);
    }
}
