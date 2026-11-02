//! Global search and quick-open support. Shared compiled `Matcher` used
//! by the sidebar search and editor find; walking honours gitignore via
//! the `ignore` crate, never follows symlinks, always skips `.git`.

use std::ops::Range;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSetBuilder};
use regex::RegexBuilder;

pub const MAX_MATCHES: usize = 2000;
pub const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchOptions {
    pub query: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
    /// Comma-separated root-relative glob patterns; empty = all files.
    pub include: String,
    pub exclude: String,
    pub use_ignore_files: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            case_sensitive: false,
            whole_word: false,
            regex: false,
            include: String::new(),
            exclude: String::new(),
            use_ignore_files: true,
        }
    }
}

/// One compiled matcher shared by workspace search and editor find.
pub struct Matcher {
    re: regex::Regex,
    pub whole_word: bool,
}

impl Matcher {
    /// Returns Err for invalid regex/globs — callers must not mutate files.
    pub fn new(opts: &SearchOptions) -> Result<Matcher, String> {
        let pat = if opts.regex {
            opts.query.clone()
        } else {
            regex::escape(&opts.query)
        };
        // Whole word is checked via neighbours, not \b (punctuation-safe).
        let re = RegexBuilder::new(&pat)
            .case_insensitive(!opts.case_sensitive)
            .unicode(true)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Matcher {
            re,
            whole_word: opts.whole_word,
        })
    }

    /// Whole-word: neighbours must not be alphanumeric or '_'.
    fn word_ok(&self, text: &str, m: &regex::Match) -> bool {
        if !self.whole_word {
            return true;
        }
        let before = text[..m.start()].chars().next_back();
        let after = text[m.end()..].chars().next();
        let ok = |c: Option<char>| !c.is_some_and(|c| c.is_alphanumeric() || c == '_');
        ok(before) && ok(after)
    }

    /// All match byte ranges in `text`.
    pub fn find_iter<'a>(&'a self, text: &'a str) -> impl Iterator<Item = Range<usize>> + 'a {
        self.re
            .find_iter(text)
            .filter(move |m| self.word_ok(text, m))
            .map(|m| m.start()..m.end())
    }

    /// Expand `replacement` for match at `range` — literal `$` stays
    /// literal unless the matcher is regex (then captures expand).
    /// Replacement text for the match at `range` in the FROZEN original
    /// `text`. Regex captures come from the original (anchors/boundaries
    /// preserved); literal mode inserts verbatim (including '$').
    pub fn replacement_text(
        &self,
        text: &str,
        range: &Range<usize>,
        replacement: &str,
        regex_mode: bool,
    ) -> Result<String, String> {
        if range.start > range.end || range.end > text.len() {
            return Err("match range out of bounds".into());
        }
        if !text.is_char_boundary(range.start) || !text.is_char_boundary(range.end) {
            return Err("match range is not a char boundary".into());
        }
        if !regex_mode {
            return Ok(replacement.to_string());
        }
        let caps = self
            .re
            .captures_at(text, range.start)
            .ok_or_else(|| "stale match range".to_string())?;
        let m0 = caps.get(0).ok_or_else(|| "stale match range".to_string())?;
        if m0.start() != range.start || m0.end() != range.end {
            return Err("match range no longer matches".into());
        }
        if !self.word_ok(text, &m0) {
            return Err("match no longer whole-word".into());
        }
        let mut out = String::new();
        caps.expand(replacement, &mut out);
        Ok(out)
    }
}

#[derive(Clone, Debug)]
pub struct SearchMatch {
    /// Byte range in the file's frozen `text`.
    pub range: Range<usize>,
    pub line: usize,
    pub column: usize,
    pub preview: String,
}

#[derive(Clone, Debug)]
pub struct SearchFile {
    pub path: PathBuf,
    /// Frozen text the matches were computed against (for prevalidation).
    pub text: String,
    pub matches: Vec<SearchMatch>,
}

#[derive(Clone, Debug, Default)]
pub struct SearchResults {
    /// Frozen options that produced these results (replace must use them).
    pub options: SearchOptions,
    pub files: Vec<SearchFile>,
    pub truncated: bool,
    pub skipped: usize,
}

impl SearchResults {
    pub fn total_matches(&self) -> usize {
        self.files.iter().map(|f| f.matches.len()).sum()
    }
}

fn globset(patterns: &str) -> Result<globset::GlobSet, String> {
    let mut b = GlobSetBuilder::new();
    for p in patterns
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
    {
        let g = Glob::new(p).map_err(|e| e.to_string())?;
        b.add(g);
    }
    b.build().map_err(|e| e.to_string())
}

/// Byte offset -> (line, column) both 1-based, columns in chars.
pub fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for c in text[..offset.min(text.len())].chars() {
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Search one text (already loaded, e.g. an open buffer).
pub fn search_text(path: &Path, text: &str, m: &Matcher, budget: &mut usize) -> Option<SearchFile> {
    let mut matches = Vec::new();
    for r in m.find_iter(text) {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        let (line, column) = line_col(text, r.start);
        let line_start = text[..r.start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_end = text[r.end..]
            .find('\n')
            .map(|i| r.end + i)
            .unwrap_or(text.len());
        let preview = text[line_start..line_end].trim().to_string();
        matches.push(SearchMatch {
            range: r,
            line,
            column,
            preview,
        });
    }
    (!matches.is_empty()).then(|| SearchFile {
        path: path.to_path_buf(),
        text: text.to_string(),
        matches,
    })
}

/// Walk `root` (no symlinks, gitignore honored when use_ignore_files,
/// `.git` always skipped) and search every UTF-8/no-NUL text file plus
/// the supplied in-memory buffers (unsaved edits count).
#[cfg(test)]
pub fn search(
    root: &Path,
    opts: &SearchOptions,
    buffers: &[(PathBuf, String)],
) -> Result<SearchResults, String> {
    let cancelled = std::sync::atomic::AtomicBool::new(false);
    search_cancellable(root, opts, buffers, &cancelled)
}

/// Cancellable variant: `cancelled` is polled once per filesystem entry,
/// per supplied buffer, and per match iteration.
pub fn search_cancellable(
    root: &Path,
    opts: &SearchOptions,
    buffers: &[(PathBuf, String)],
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<SearchResults, String> {
    let cancelled = || {
        (!cancelled.load(std::sync::atomic::Ordering::Relaxed))
            .then_some(())
            .ok_or_else(|| "Search cancelled".to_string())
    };
    if opts.query.is_empty() {
        return Ok(SearchResults::default());
    }
    let m = Matcher::new(opts)?;
    let inc = globset(&opts.include)?;
    let exc = globset(&opts.exclude)?;
    let mut out = SearchResults {
        options: opts.clone(),
        ..Default::default()
    };
    let mut budget = MAX_MATCHES;

    // Buffers overlay walked paths (absolute → text), so every ignore /
    // glob rule applies identically to live edits. shortcut: buffers whose
    // file no longer exists on disk are omitted entirely.
    let mut overlay: std::collections::HashMap<PathBuf, &str> = buffers
        .iter()
        .map(|(p, t)| (p.clone(), t.as_str()))
        .collect();
    // Enforce the same text constraints on buffers up front.
    for (p, t) in buffers {
        cancelled()?;
        let abs = if p.is_absolute() {
            p.clone()
        } else {
            root.join(p)
        };
        if abs.exists() && (t.len() as u64 > MAX_FILE_BYTES || t.contains('\0')) {
            overlay.remove(&abs);
            overlay.remove(p);
            out.skipped += 1;
        }
    }

    let mut walker = ignore::WalkBuilder::new(root);
    walker
        .follow_links(false)
        .hidden(false)
        .git_ignore(opts.use_ignore_files)
        .git_exclude(opts.use_ignore_files)
        .git_global(opts.use_ignore_files)
        .ignore(opts.use_ignore_files)
        .filter_entry(|e| e.file_name() != ".git");
    for entry in walker.build() {
        cancelled()?;
        if out.truncated {
            break;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => {
                out.skipped += 1;
                continue;
            }
        };
        // file_type() never follows the link — symlinks are skipped
        // rather than read through.
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let p = entry.path();
        let rel = p.strip_prefix(root).unwrap_or(p).to_path_buf();
        if (!inc.is_empty() && !inc.is_match(&rel)) || exc.is_match(&rel) {
            continue;
        }
        let text: String = if let Some(t) = overlay.remove(p) {
            t.to_string()
        } else {
            let md = match std::fs::metadata(p) {
                Ok(m) => m,
                Err(_) => {
                    out.skipped += 1;
                    continue;
                }
            };
            if md.len() > MAX_FILE_BYTES {
                out.skipped += 1;
                continue;
            }
            let bytes = match std::fs::read(p) {
                Ok(b) => b,
                Err(_) => {
                    out.skipped += 1;
                    continue;
                }
            };
            if bytes.contains(&0) {
                out.skipped += 1;
                continue; // binary
            }
            match String::from_utf8(bytes) {
                Ok(t) => t,
                Err(_) => {
                    out.skipped += 1;
                    continue;
                }
            }
        };
        if budget == 0 {
            out.truncated = true;
            break;
        }
        if let Some(f) = search_text(&rel, &text, &m, &mut budget) {
            out.files.push(f);
        }
    }
    if budget == 0 {
        out.truncated = true;
    }
    out.files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Quick-open query parsing: `>` commands, `:` line[:col], `@` symbols,
/// `path:line[:col]`, else fuzzy files.
#[derive(Clone, Debug, PartialEq)]
pub enum QuickMode {
    Files(String),
    Commands(String),
    Goto {
        line: usize,
        col: Option<usize>,
    },
    Symbols(String),
    FileAt {
        path: String,
        line: Option<usize>,
        col: Option<usize>,
    },
}

pub fn parse_query(q: &str) -> QuickMode {
    if let Some(rest) = q.strip_prefix('>') {
        return QuickMode::Commands(rest.to_string());
    }
    if let Some(rest) = q.strip_prefix(':') {
        let mut it = rest.split(':');
        let line = it.next().and_then(|s| s.parse().ok()).unwrap_or(1).max(1);
        let col = it.next().and_then(|s| s.parse().ok());
        return QuickMode::Goto { line, col };
    }
    if let Some(rest) = q.strip_prefix('@') {
        return QuickMode::Symbols(rest.to_string());
    }
    // path:line[:col] — strip trailing :digits up to twice; a lone drive
    // letter ("C:") is not a line suffix.
    let mut rest = q;
    let mut nums: Vec<usize> = Vec::new();
    while let Some(idx) = rest.rfind(':') {
        let suf = &rest[idx + 1..];
        if suf.is_empty() || !suf.chars().all(|c| c.is_ascii_digit()) {
            break;
        }
        nums.insert(0, suf.parse().unwrap_or(usize::MAX));
        rest = &rest[..idx];
    }
    let drive_letter = rest.len() == 1 && rest.as_bytes()[0].is_ascii_alphabetic();
    if !nums.is_empty() && !rest.is_empty() && !drive_letter {
        return QuickMode::FileAt {
            path: rest.to_string(),
            line: Some(nums[0]),
            col: nums.get(1).copied(),
        };
    }
    QuickMode::Files(q.to_string())
}

/// Fuzzy subsequence match; higher is better. Contiguous runs and
/// basename hits rank higher. None if `q` is not a subsequence.
pub fn fuzzy_score(q: &str, candidate: &str) -> Option<i64> {
    if q.is_empty() {
        return Some(0);
    }
    let qc: Vec<char> = q.chars().collect();
    let cc: Vec<char> = candidate.chars().collect();
    let mut qi = 0usize;
    let mut score = 0i64;
    let mut streak = 0i64;
    let base = candidate.rsplit(['/', '\\']).next().unwrap_or(candidate);
    for (i, c) in cc.iter().enumerate() {
        if qi < qc.len() && c.eq_ignore_ascii_case(&qc[qi]) {
            streak += 1;
            score += 10 + streak * 5;
            // basename start measured in CHARS, not bytes
            if i >= cc.len() - base.chars().count() {
                score += 30; // basename hit
            }
            if i == 0 || cc[i - 1] == '/' || cc[i - 1] == '\\' || cc[i - 1] == '_' {
                score += 10; // word-start hit
            }
            qi += 1;
        } else {
            streak = 0;
        }
    }
    (qi == qc.len()).then_some(score - candidate.len() as i64 / 10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn opts(q: &str) -> SearchOptions {
        SearchOptions {
            query: q.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn literal_case_and_word() {
        let m = Matcher::new(&opts("foo")).unwrap();
        let hits: Vec<_> = m.find_iter("foo FOO food foo").collect();
        assert_eq!(hits.len(), 4); // case-insensitive, 'food' counts
        let m = Matcher::new(&SearchOptions {
            case_sensitive: true,
            ..opts("foo")
        })
        .unwrap();
        assert_eq!(m.find_iter("foo FOO food foo").count(), 3); // "food" still matches
        let m = Matcher::new(&SearchOptions {
            whole_word: true,
            ..opts("foo")
        })
        .unwrap();
        assert_eq!(m.find_iter("foo FOO food foo_bar foo").count(), 3);
        // punctuation-ending query still works as whole word
        let m = Matcher::new(&SearchOptions {
            whole_word: true,
            ..opts("foo.")
        })
        .unwrap();
        assert_eq!(m.find_iter("foo. foo.x").count(), 1);
    }

    #[test]
    fn regex_and_captures_and_dollar() {
        let m = Matcher::new(&SearchOptions {
            regex: true,
            ..opts(r"fn (\w+)")
        })
        .unwrap();
        assert_eq!(m.find_iter("fn main fn x").count(), 2);
        // literal mode: '$' stays literal
        let m = Matcher::new(&opts("$x")).unwrap();
        let out = m
            .replacement_text("let $x = 1;", &(4..6), "y$0z", false)
            .unwrap();
        assert_eq!(out, "y$0z");
        // regex mode: captures expand
        let m = Matcher::new(&SearchOptions {
            regex: true,
            ..opts(r"(a)(b)")
        })
        .unwrap();
        let out = m.replacement_text("ab", &(0..2), "$2$1", true).unwrap();
        assert_eq!(out, "ba");
        assert!(Matcher::new(&SearchOptions {
            regex: true,
            ..opts("(")
        })
        .is_err());
    }

    struct TmpDir(PathBuf);
    impl TmpDir {
        fn new() -> Self {
            static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let d = std::env::temp_dir().join(format!(
                "oxide-search-test-{}-{}",
                std::process::id(),
                N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            fs::create_dir_all(&d).unwrap();
            TmpDir(d)
        }
        fn w(&self, rel: &str, text: &str) {
            let p = self.0.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
    }
    impl Drop for TmpDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn include_exclude_and_binary_skip() {
        let t = TmpDir::new();
        t.w("src/a.rs", "needle\n");
        t.w("target/b.rs", "needle\n");
        t.w("bin.o", "n\x00eedle");
        t.w("note.txt", "needle\n");
        // include limits
        let r = search(
            &t.0,
            &SearchOptions {
                include: "src/**".into(),
                ..opts("needle")
            },
            &[],
        )
        .unwrap();
        assert_eq!(r.files.len(), 1);
        assert_eq!(r.files[0].path, PathBuf::from("src/a.rs"));
        // exclude drops it
        let r = search(
            &t.0,
            &SearchOptions {
                exclude: "src/**".into(),
                ..opts("needle")
            },
            &[],
        )
        .unwrap();
        assert!(!r.files.iter().any(|f| f.path == Path::new("src/a.rs")));
        // binary skipped
        let r = search(&t.0, &opts("needle"), &[]).unwrap();
        assert!(r.skipped >= 1);
        // unsaved buffer contents are searched
        let r = search(&t.0, &opts("zz"), &[(t.0.join("src/a.rs"), "zz\n".into())]).unwrap();
        assert_eq!(r.files.len(), 1);
    }

    #[test]
    fn quickmode_parse() {
        assert_eq!(parse_query("foo.rs"), QuickMode::Files("foo.rs".into()));
        assert_eq!(parse_query(">save"), QuickMode::Commands("save".into()));
        assert_eq!(
            parse_query(":12"),
            QuickMode::Goto {
                line: 12,
                col: None
            }
        );
        assert_eq!(
            parse_query(":12:4"),
            QuickMode::Goto {
                line: 12,
                col: Some(4)
            }
        );
        assert_eq!(parse_query("@main"), QuickMode::Symbols("main".into()));
        assert_eq!(
            parse_query("src/a.rs:7"),
            QuickMode::FileAt {
                path: "src/a.rs".into(),
                line: Some(7),
                col: None
            }
        );
        assert_eq!(
            parse_query("a.rs:7:2"),
            QuickMode::FileAt {
                path: "a.rs".into(),
                line: Some(7),
                col: Some(2)
            }
        );
    }

    #[test]
    fn fuzzy_scoring() {
        assert!(fuzzy_score("main", "src/main.rs").is_some());
        assert!(fuzzy_score("zz", "src/main.rs").is_none());
        // basename/contiguous ranks above scattered
        let a = fuzzy_score("main", "src/main.rs").unwrap();
        let b = fuzzy_score("main", "src/mxaxixn.rs").unwrap();
        assert!(a > b);
    }

    #[test]
    fn replace_prevalidation_aborts_on_stale() {
        let t = TmpDir::new();
        t.w("a.rs", "foo\n");
        let res = search(&t.0, &opts("foo"), &[]).unwrap();
        // File changed on disk after search — must reject everything.
        t.w("a.rs", "foo changed\n");
        let mut ws = crate::project::Workspace::open_folder(&t.0).unwrap();
        let m = Matcher::new(&opts("foo")).unwrap();
        assert!(ws.apply_replacements(&res, &m, "bar", false).is_err());
        assert_eq!(
            fs::read_to_string(t.0.join("a.rs")).unwrap(),
            "foo changed\n"
        );
    }
}
