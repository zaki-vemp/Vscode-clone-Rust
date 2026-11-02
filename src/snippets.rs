//! Basic declarative VS Code snippets. No extension code is executed.
use std::collections::HashMap;
use std::ops::Range;
use std::path::Path;

use anyhow::{bail, Result};
use serde_json::Value;

#[derive(Clone)]
pub struct Snippet {
    pub label: String,
    pub prefix: String,
    pub language: String,
    pub body: String,
}

pub fn parse(value: &Value, language: &str) -> Result<Vec<Snippet>> {
    let object = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Snippet file must be an object"))?;
    let mut snippets = Vec::new();
    for (label, item) in object {
        let body = match &item["body"] {
            Value::String(s) => s.clone(),
            Value::Array(lines) => lines
                .iter()
                .map(|line| line.as_str().unwrap_or_default())
                .collect::<Vec<_>>()
                .join("\n"),
            _ => continue,
        };
        let prefixes: Vec<&str> = match &item["prefix"] {
            Value::String(s) => vec![s],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![""],
        };
        for prefix in prefixes {
            snippets.push(Snippet {
                label: label.clone(),
                prefix: prefix.into(),
                language: language.into(),
                body: body.clone(),
            });
        }
    }
    Ok(snippets)
}

pub fn language_id(path: &Path) -> String {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jsx" => "javascriptreact".into(),
        "tsx" => "typescriptreact".into(),
        "sh" | "bash" | "zsh" => "shellscript".into(),
        "cs" => "csharp".into(),
        "cpp" | "cc" | "h" | "hpp" => "cpp".into(),
        "lua" => "lua".into(),
        "ps1" | "psm1" | "psd1" => "powershell".into(),
        _ => crate::languages::Language::from_path(&path.to_string_lossy())
            .display()
            .to_lowercase(),
    }
}

pub struct Expansion {
    pub text: String,
    pub selection: Range<usize>,
}

/// Expand nested defaults, first choices, basic variables, and mirrored
/// placeholder defaults. Regex transforms are rejected instead of inserted as code.
pub fn expand(body: &str, path: &Path, selected: &str) -> Result<Expansion> {
    let mut parser = Expander {
        values: HashMap::new(),
        variables: HashMap::from([
            (
                "TM_FILENAME",
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            ),
            (
                "TM_FILENAME_BASE",
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            ),
            (
                "TM_DIRECTORY",
                path.parent()
                    .unwrap_or(Path::new(""))
                    .to_string_lossy()
                    .into_owned(),
            ),
            ("TM_FILEPATH", path.to_string_lossy().into_owned()),
            ("TM_SELECTED_TEXT", selected.into()),
        ]),
        stops: Vec::new(),
    };
    let text = parser.part(body, 0, 0)?;
    parser
        .stops
        .sort_by_key(|(id, _)| if *id == 0 { u32::MAX } else { *id });
    let end = text.chars().count();
    let selection = parser
        .stops
        .first()
        .map(|(_, range)| range.clone())
        .unwrap_or(end..end);
    Ok(Expansion { text, selection })
}

struct Expander<'a> {
    values: HashMap<u32, String>,
    variables: HashMap<&'a str, String>,
    stops: Vec<(u32, Range<usize>)>,
}
impl Expander<'_> {
    fn part(&mut self, body: &str, offset: usize, depth: usize) -> Result<String> {
        if depth > 16 {
            bail!("Snippet nesting exceeds 16 levels");
        }
        let mut out = String::new();
        let mut i = 0;
        while i < body.len() {
            let c = body[i..].chars().next().unwrap();
            if c == '\\' {
                let rest = &body[i + 1..];
                if let Some(next) = rest
                    .chars()
                    .next()
                    .filter(|c| matches!(c, '$' | '}' | '\\' | ',' | '|'))
                {
                    out.push(next);
                    i += 1 + next.len_utf8();
                    continue;
                }
            }
            if c != '$' {
                out.push(c);
                i += c.len_utf8();
                continue;
            }
            i += 1;
            let braced = body[i..].starts_with('{');
            if braced {
                i += 1;
            }
            let begin = i;
            while i < body.len()
                && (body.as_bytes()[i].is_ascii_alphanumeric() || body.as_bytes()[i] == b'_')
            {
                i += 1;
            }
            let name = &body[begin..i];
            if name.is_empty() {
                out.push('$');
                if braced {
                    out.push('{');
                }
                continue;
            }
            let mut default = None;
            if braced {
                match body.as_bytes().get(i) {
                    Some(b':') => {
                        i += 1;
                        let start = i;
                        let mut nesting = 0;
                        while i < body.len() {
                            match body.as_bytes()[i] {
                                b'\\' => {
                                    i = (i + 2).min(body.len());
                                    continue;
                                }
                                b'{' => nesting += 1,
                                b'}' if nesting == 0 => break,
                                b'}' => nesting -= 1,
                                _ => {}
                            }
                            i += 1;
                        }
                        if i == body.len() {
                            bail!("Unclosed snippet placeholder");
                        }
                        default = Some(self.part(
                            &body[start..i],
                            offset + out.chars().count(),
                            depth + 1,
                        )?);
                        i += 1;
                    }
                    Some(b'|') => {
                        i += 1;
                        let end = body[i..]
                            .find("|}")
                            .ok_or_else(|| anyhow::anyhow!("Unclosed snippet choice"))?
                            + i;
                        default = Some(
                            body[i..end]
                                .split(',')
                                .next()
                                .unwrap_or_default()
                                .to_string(),
                        );
                        i = end + 2;
                    }
                    Some(b'/') => {
                        bail!("This snippet uses regex transforms, which are not supported yet")
                    }
                    Some(b'}') => i += 1,
                    _ => bail!("Unsupported snippet placeholder syntax"),
                }
            }
            let start = offset + out.chars().count();
            if let Ok(id) = name.parse::<u32>() {
                let value = self
                    .values
                    .entry(id)
                    .or_insert_with(|| default.unwrap_or_default())
                    .clone();
                let end = start + value.chars().count();
                self.stops.push((id, start..end));
                out.push_str(&value);
            } else {
                out.push_str(
                    self.variables
                        .get(name)
                        .map(String::as_str)
                        .unwrap_or(default.as_deref().unwrap_or(name)),
                );
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expands_actual_react_snippet_patterns() {
        let expansion = expand(
            "class ${1:${TM_FILENAME_BASE}} {\n  $0\n}\nexport default $1;",
            Path::new("Widget.tsx"),
            "",
        )
        .unwrap();
        assert_eq!(
            expansion.text,
            "class Widget {\n  \n}\nexport default Widget;"
        );
        assert_eq!(expansion.selection, 6..12);
        assert_eq!(
            expand(
                "${1|one,two|} \\$ ${TM_SELECTED_TEXT}",
                Path::new("x.rs"),
                "中文"
            )
            .unwrap()
            .text,
            "one $ 中文"
        );
        assert!(expand("${1/foo/bar/}", Path::new("x"), "").is_err());
    }
}
