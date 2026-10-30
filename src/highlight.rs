//! Syntax highlighting: a small hand-written tokenizer plus an egui
//! [`LayoutJob`] builder. No tree-sitter, no WASM, no JavaScript — one
//! scanner per line family that is fast enough for interactive typing.

use std::ops::Range;

use eframe::egui;
use eframe::egui::text::{LayoutJob, TextFormat};

use crate::languages::Language;
use crate::theme::Theme;

/// Token classes produced by the scanner.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tok {
    Keyword,
    Type,
    Str,
    Comment,
    Number,
    Macro,
    Annotation,
    Punct,
    Ident,
    Whitespace,
}

/// Scan one line, returning (token, byte-range) pairs that tile the line.
/// `in_block` carries multi-line block-comment state across lines.
pub fn tokenize_line(
    line: &str,
    lang: Language,
    in_block: &mut Option<String>,
) -> Vec<(Tok, Range<usize>)> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0usize;

    // Continuation of a block comment started on an earlier line.
    if let Some(close) = in_block.clone() {
        if let Some(end) = line.find(close.as_str()) {
            let e = end + close.len();
            out.push((Tok::Comment, 0..e));
            i = e;
            *in_block = None;
        } else {
            out.push((Tok::Comment, 0..line.len()));
            return out;
        }
    }

    // Line comment starter at this position?
    let is_line_comment = |s: &str| lang.line_comments().iter().any(|p| s.starts_with(p));

    while i < bytes.len() {
        let rest = &line[i..];

        if is_line_comment(rest) {
            out.push((Tok::Comment, i..line.len()));
            break;
        }
        if let Some((open, close)) = lang.block_comment() {
            if rest.starts_with(open) {
                let after = i + open.len();
                if let Some(end) = line[after..].find(close) {
                    let e = after + end + close.len();
                    out.push((Tok::Comment, i..e));
                    i = e;
                    continue;
                }
                out.push((Tok::Comment, i..line.len()));
                *in_block = Some(close.to_string());
                break;
            }
        }

        let c = rest.chars().next().unwrap();
        let clen = c.len_utf8();

        // Strings.
        if matches!(c, '"' | '\'' | '`') && lang != Language::Plain {
            let mut j = i + clen;
            let mut closed = false;
            let chars: Vec<(usize, char)> = line[i..].char_indices().skip(1).collect();
            let mut k = 0;
            while k < chars.len() {
                let (off, ch) = chars[k];
                if ch == '\\' {
                    k += 2; // skip escaped char
                    continue;
                }
                if ch == c {
                    j = i + off + ch.len_utf8();
                    closed = true;
                    break;
                }
                k += 1;
            }
            if !closed {
                j = line.len();
            }
            out.push((Tok::Str, i..j));
            i = j;
            continue;
        }

        // Numbers.
        if c.is_ascii_digit() {
            let mut j = i;
            while j < bytes.len() {
                let ch = bytes[j] as char;
                if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                    j += 1;
                } else {
                    break;
                }
            }
            out.push((Tok::Number, i..j));
            i = j;
            continue;
        }

        // Identifiers / keywords / types / macros / annotations.
        if c.is_alphabetic() || c == '_' || c == '#' || c == '@' || c == '$' {
            let mut j = i;
            while j < bytes.len() {
                let ch2 = line[j..].chars().next().unwrap();
                if ch2.is_alphanumeric() || matches!(ch2, '_' | '#' | '@' | '$') {
                    j += ch2.len_utf8();
                } else {
                    break;
                }
            }
            let word = &line[i..j];
            let kind = classify_word(word, lang);
            // Rust macro call: `name!`
            if kind == Tok::Ident && lang == Language::Rust && line[j..].starts_with('!') {
                out.push((Tok::Macro, i..j + 1));
                i = j + 1;
                continue;
            }
            out.push((kind, i..j));
            i = j;
            continue;
        }

        if c.is_whitespace() {
            let mut j = i + clen;
            while j < bytes.len() {
                let ch2 = line[j..].chars().next().unwrap();
                if ch2.is_whitespace() {
                    j += ch2.len_utf8();
                } else {
                    break;
                }
            }
            out.push((Tok::Whitespace, i..j));
            i = j;
            continue;
        }

        out.push((Tok::Punct, i..i + clen));
        i += clen;
    }
    out
}

fn classify_word(word: &str, lang: Language) -> Tok {
    if word.starts_with('#') || word.starts_with('@') {
        return Tok::Annotation;
    }
    let hit = if lang.keywords_case_insensitive() {
        let lower = word.to_ascii_lowercase();
        lang.keywords().iter().any(|k| *k == lower)
    } else {
        lang.keywords().contains(&word)
    };
    if hit {
        return Tok::Keyword;
    }
    // Capitalized identifiers read as types; SCREAMING as constants.
    if word.chars().next().is_some_and(|c| c.is_uppercase()) {
        return Tok::Type;
    }
    Tok::Ident
}

/// Extract outline symbols (label, 1-based line) for the sidebar Outline
/// view and breadcrumbs. Deliberately simple line-pattern matching.
pub fn symbols(text: &str, lang: Language) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let t = raw.trim();
        let label = match lang {
            Language::Rust => rust_symbol(t),
            Language::Python if t.starts_with("def ") || t.starts_with("class ") => {
                let rest = t.trim_start_matches("def ").trim_start_matches("class ");
                Some(format!("{} {}", if t.starts_with("def") { "fn" } else { "class" }, first_word(rest)))
            }
            Language::JavaScript | Language::TypeScript
                if t.starts_with("function ") || t.starts_with("class ") =>
            {
                Some(t.to_string())
            }
            Language::Go if t.starts_with("func ") => Some(first_word(&t[5..]).to_string()),
            _ => None,
        };
        if let Some(label) = label {
            out.push((truncate_label(&label), i + 1));
        }
    }
    out
}

fn rust_symbol(t: &str) -> Option<String> {
    for kw in [
        "pub async fn ",
        "async fn ",
        "pub fn ",
        "fn ",
        "pub struct ",
        "struct ",
        "pub enum ",
        "enum ",
        "pub trait ",
        "trait ",
        "pub const ",
        "const ",
    ] {
        if let Some(rest) = t.strip_prefix(kw) {
            let kind = kw
                .trim()
                .trim_start_matches("pub ")
                .trim_start_matches("async ")
                .trim();
            let name = first_word(rest.trim_start_matches("async "));
            return Some(format!("{kind} {name}"));
        }
    }
    if let Some(rest) = t.strip_prefix("impl ") {
        return Some(format!("impl {}", first_word(rest)));
    }
    None
}

fn first_word(s: &str) -> &str {
    let end = s.find(|c: char| !(c.is_alphanumeric() || c == '_')).unwrap_or(s.len());
    &s[..end]
}

fn truncate_label(s: &str) -> String {
    if s.len() > 42 { format!("{}…", &s[..41]) } else { s.to_string() }
}

/// Build a colored [`LayoutJob`] for the whole document.
pub fn layout_job(text: &str, lang: Language, theme: &Theme, font_size: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    let base = TextFormat {
        font_id: egui::FontId::monospace(font_size),
        color: theme.text,
        ..Default::default()
    };
    let fmt = |tok: Tok| {
        let mut f = base.clone();
        f.color = match tok {
            Tok::Keyword => theme.keyword,
            Tok::Type => theme.typ,
            Tok::Str => theme.string,
            Tok::Comment => theme.comment,
            Tok::Number => theme.number,
            Tok::Macro => theme.macro_color,
            Tok::Annotation => theme.accent,
            Tok::Punct | Tok::Ident | Tok::Whitespace => theme.text,
        };
        f.italics = matches!(tok, Tok::Comment);
        f
    };

    let mut in_block: Option<String> = None;
    let mut lines = text.split('\n').peekable();
    while let Some(line) = lines.next() {
        // Keep the original line (possibly empty) so row alignment is exact.
        let show = if line.is_empty() { " " } else { line };
        for (tok, range) in tokenize_line(line, lang, &mut in_block) {
            job.append(&show[range], 0.0, fmt(tok));
        }
        if lines.peek().is_some() {
            job.append("\n", 0.0, base.clone());
        }
    }
    // Match egui's code-editor tab behavior expectations.
    job.wrap.max_width = f32::INFINITY;
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(line: &str, lang: Language) -> Vec<(Tok, String)> {
        let mut st = None;
        tokenize_line(line, lang, &mut st)
            .into_iter()
            .map(|(t, r)| (t, line[r].to_string()))
            .collect()
    }

    #[test]
    fn rust_keywords_types_and_macros() {
        let toks = kinds("let mut editor: String = vec![1]; // hi", Language::Rust);
        let get = |w: &str| toks.iter().find(|(_, s)| s == w).map(|(t, _)| *t);
        assert_eq!(get("let"), Some(Tok::Keyword));
        assert_eq!(get("mut"), Some(Tok::Keyword));
        assert_eq!(get("String"), Some(Tok::Type));
        assert_eq!(get("vec!"), Some(Tok::Macro));
        assert_eq!(get("1"), Some(Tok::Number));
        assert!(toks.iter().any(|(t, _)| *t == Tok::Comment));
    }

    #[test]
    fn strings_and_comments() {
        let toks = kinds(r#"let s = "hi"; # hello"#, Language::Python);
        assert!(toks.iter().any(|(t, _)| *t == Tok::Str));
        assert!(toks.iter().any(|(t, _)| *t == Tok::Comment));
    }

    #[test]
    fn block_comments_span_lines() {
        let mut st = None;
        let a = tokenize_line("/* start", Language::Rust, &mut st);
        assert!(st.is_some());
        assert!(a.iter().all(|(t, _)| *t == Tok::Comment));
        let b = tokenize_line("still comment */ let x = 1;", Language::Rust, &mut st);
        assert!(st.is_none());
        assert!(b.iter().any(|(t, _)| *t == Tok::Keyword));
    }
}
