//! Local diff: current buffer vs. last saved snapshot.
//!
//! A bounded Myers O(ND) diff over lines. Large inputs fall back to a fast
//! prefix/suffix trim so the UI never hangs on huge files.

/// One row of a two-sided diff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffRow {
    pub tag: DiffTag,
    /// 0-based line number in the old (saved) text.
    pub old_no: Option<usize>,
    /// 0-based line number in the new (current) text.
    pub new_no: Option<usize>,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffTag {
    Equal,
    Delete,
    Insert,
}

const MYERS_LIMIT: usize = 4_000_000; // cells of the edit graph

/// Diff `old` against `new`, returning rows in display order.
pub fn diff_lines(old: &[String], new: &[String]) -> Vec<DiffRow> {
    // Cheap fast path.
    if old == new {
        return old
            .iter()
            .enumerate()
            .map(|(i, l)| DiffRow {
                tag: DiffTag::Equal,
                old_no: Some(i),
                new_no: Some(i),
                text: l.clone(),
            })
            .collect();
    }
    // Trim common prefix/suffix so Myers only sees the changed middle.
    let mut pre = 0;
    while pre < old.len() && pre < new.len() && old[pre] == new[pre] {
        pre += 1;
    }
    let mut suf = 0;
    while suf < old.len() - pre
        && suf < new.len() - pre
        && old[old.len() - 1 - suf] == new[new.len() - 1 - suf]
    {
        suf += 1;
    }
    let mid_old = &old[pre..old.len() - suf];
    let mid_new = &new[pre..new.len() - suf];

    let mid = if mid_old.len() * mid_new.len() > MYERS_LIMIT {
        naive_rows(mid_old, mid_new)
    } else {
        myers_rows(mid_old, mid_new)
    };

    let mut rows = Vec::with_capacity(old.len().max(new.len()));
    for (i, l) in old[..pre].iter().enumerate() {
        rows.push(DiffRow {
            tag: DiffTag::Equal,
            old_no: Some(i),
            new_no: Some(i),
            text: l.clone(),
        });
    }
    let mut o = pre;
    let mut n = pre;
    for m in mid {
        match m.tag {
            DiffTag::Equal => {
                rows.push(DiffRow {
                    tag: DiffTag::Equal,
                    old_no: Some(o),
                    new_no: Some(n),
                    text: m.text,
                });
                o += 1;
                n += 1;
            }
            DiffTag::Delete => {
                rows.push(DiffRow {
                    tag: DiffTag::Delete,
                    old_no: Some(o),
                    new_no: None,
                    text: m.text,
                });
                o += 1;
            }
            DiffTag::Insert => {
                rows.push(DiffRow {
                    tag: DiffTag::Insert,
                    old_no: None,
                    new_no: Some(n),
                    text: m.text,
                });
                n += 1;
            }
        }
    }
    for k in 0..suf {
        let i = old.len() - suf + k;
        let j = new.len() - suf + k;
        rows.push(DiffRow {
            tag: DiffTag::Equal,
            old_no: Some(i),
            new_no: Some(j),
            text: old[i].clone(),
        });
    }
    rows
}

/// Fallback for giant inputs: all old lines deleted, all new lines inserted.
fn naive_rows(old: &[String], new: &[String]) -> Vec<DiffRow> {
    let mut rows = Vec::with_capacity(old.len() + new.len());
    for l in old {
        rows.push(DiffRow {
            tag: DiffTag::Delete,
            old_no: None,
            new_no: None,
            text: l.clone(),
        });
    }
    for l in new {
        rows.push(DiffRow {
            tag: DiffTag::Insert,
            old_no: None,
            new_no: None,
            text: l.clone(),
        });
    }
    rows
}

/// Myers greedy diff on the trimmed middle. Returns rows with local numbering.
fn myers_rows(a: &[String], b: &[String]) -> Vec<DiffRow> {
    let n = a.len();
    let m = b.len();
    if n == 0 {
        return b
            .iter()
            .map(|l| DiffRow {
                tag: DiffTag::Insert,
                old_no: None,
                new_no: None,
                text: l.clone(),
            })
            .collect();
    }
    if m == 0 {
        return a
            .iter()
            .map(|l| DiffRow {
                tag: DiffTag::Delete,
                old_no: None,
                new_no: None,
                text: l.clone(),
            })
            .collect();
    }

    let max = n + m;
    let size = 2 * max + 1;
    let idx = |k: isize| (k + max as isize) as usize;
    let mut v = vec![0isize; size];
    let mut trace: Vec<Vec<isize>> = Vec::new();

    let mut found = None;
    'outer: for d in 0..=max as isize {
        let mut cur = v.clone();
        for k in (-d..=d).step_by(2) {
            let mut x = if k == -d || (k != d && v[idx(k - 1)] < v[idx(k + 1)]) {
                v[idx(k + 1)]
            } else {
                v[idx(k - 1)] + 1
            };
            let mut y = x - k;
            while x < n as isize && y < m as isize && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }
            cur[idx(k)] = x;
            if x >= n as isize && y >= m as isize {
                trace.push(cur);
                found = Some(d);
                break 'outer;
            }
        }
        trace.push(cur.clone());
        v = cur;
    }
    let d = found.unwrap_or(max as isize);

    // Backtrack.
    let mut x = n as isize;
    let mut y = m as isize;
    let mut rev: Vec<DiffTag> = Vec::new();
    for dd in (1..=d).rev() {
        let prev = &trace[(dd - 1) as usize];
        let k = x - y;
        let prev_k = if k == -dd || (k != dd && prev[idx(k - 1)] < prev[idx(k + 1)]) {
            k + 1
        } else {
            k - 1
        };
        let prev_x = prev[idx(prev_k)];
        let prev_y = prev_x - prev_k;
        while x > prev_x && y > prev_y {
            rev.push(DiffTag::Equal);
            x -= 1;
            y -= 1;
        }
        if x == prev_x {
            rev.push(DiffTag::Insert);
        } else {
            rev.push(DiffTag::Delete);
        }
        x = prev_x;
        y = prev_y;
    }
    while x > 0 && y > 0 {
        rev.push(DiffTag::Equal);
        x -= 1;
        y -= 1;
    }
    rev.reverse();

    // Emit rows.
    let mut rows = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    for tag in rev {
        match tag {
            DiffTag::Equal => {
                rows.push(DiffRow {
                    tag,
                    old_no: Some(i),
                    new_no: Some(j),
                    text: a[i].clone(),
                });
                i += 1;
                j += 1;
            }
            DiffTag::Delete => {
                rows.push(DiffRow {
                    tag,
                    old_no: Some(i),
                    new_no: None,
                    text: a[i].clone(),
                });
                i += 1;
            }
            DiffTag::Insert => {
                rows.push(DiffRow {
                    tag,
                    old_no: None,
                    new_no: Some(j),
                    text: b[j].clone(),
                });
                j += 1;
            }
        }
    }
    rows
}

/// (added_lines, removed_lines) for the status display.
pub fn summarize(rows: &[DiffRow]) -> (usize, usize) {
    let mut a = 0;
    let mut r = 0;
    for row in rows {
        match row.tag {
            DiffTag::Insert => a += 1,
            DiffTag::Delete => r += 1,
            DiffTag::Equal => {}
        }
    }
    (a, r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn identical_text_is_all_equal() {
        let rows = diff_lines(&s(&["a", "b"]), &s(&["a", "b"]));
        assert!(rows.iter().all(|r| r.tag == DiffTag::Equal));
    }

    #[test]
    fn detects_insert_and_delete() {
        let rows = diff_lines(&s(&["a", "b", "c"]), &s(&["a", "x", "c", "d"]));
        let (added, removed) = summarize(&rows);
        assert_eq!((added, removed), (2, 1));
        // Reconstruct new text from Equal+Insert rows.
        let rebuilt: Vec<&str> = rows
            .iter()
            .filter(|r| r.tag != DiffTag::Delete)
            .map(|r| r.text.as_str())
            .collect();
        assert_eq!(rebuilt, vec!["a", "x", "c", "d"]);
    }

    #[test]
    fn empty_old_is_all_insert() {
        let rows = diff_lines(&[], &s(&["a"]));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].tag, DiffTag::Insert);
        assert_eq!(rows[0].new_no, Some(0));
    }

    #[test]
    fn line_numbers_track_positions() {
        let rows = diff_lines(&s(&["a", "b"]), &s(&["a", "c"]));
        let del = rows.iter().find(|r| r.tag == DiffTag::Delete).unwrap();
        let ins = rows.iter().find(|r| r.tag == DiffTag::Insert).unwrap();
        assert_eq!(del.old_no, Some(1));
        assert_eq!(ins.new_no, Some(1));
    }
}
