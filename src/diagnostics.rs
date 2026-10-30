//! Diagnostics: parse real compiler output (`cargo check`, `cargo clippy`,
//! `rustc`) into Problems-panel entries.
//!
//! The primary path is `cargo check --message-format=json`, which gives
//! machine-readable spans. A human-format fallback keeps the panel useful
//! when JSON is unavailable.

/// Severity of a diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Severity {
    Error,
    #[default]
    Warning,
    Info,
}

/// One compiler diagnostic attached to a file position.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize, // 1-based
    pub col: usize,  // 1-based
    pub message: String,
    pub severity: Severity,
    pub code: Option<String>,
}

impl Diagnostic {
    #[allow(dead_code)]
    pub fn summary(&self) -> String {
        match &self.code {
            Some(c) => format!("{} [{}]", self.message, c),
            None => self.message.clone(),
        }
    }
}

/// Parse `cargo --message-format=json` output (one JSON object per line).
/// Only `compiler-message` artifacts with a primary span become diagnostics.
pub fn parse_cargo_json(output: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v.get("reason").and_then(|r| r.as_str()) != Some("compiler-message") {
            continue;
        }
        let Some(msg) = v.get("message") else {
            continue;
        };
        let text = msg
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string();
        if text.is_empty() {
            continue;
        }
        let level = msg
            .get("level")
            .and_then(|l| l.as_str())
            .unwrap_or("warning");
        let severity = match level {
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            _ => Severity::Info,
        };
        let code = msg
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());
        // First primary span wins.
        let mut span_info: Option<(String, usize, usize)> = None;
        if let Some(spans) = msg.get("spans").and_then(|s| s.as_array()) {
            for span in spans {
                let primary = span
                    .get("is_primary")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false);
                if !primary && span_info.is_some() {
                    continue;
                }
                let file = span.get("file_name").and_then(|f| f.as_str()).unwrap_or("");
                if file.is_empty() {
                    continue;
                }
                let l = span.get("line_start").and_then(|l| l.as_u64()).unwrap_or(1) as usize;
                let c = span
                    .get("column_start")
                    .and_then(|c| c.as_u64())
                    .unwrap_or(1) as usize;
                span_info = Some((file.to_string(), l.max(1), c.max(1)));
                if primary {
                    break;
                }
            }
        }
        if let Some((file, line_no, col)) = span_info {
            out.push(Diagnostic {
                file,
                line: line_no,
                col,
                message: first_line(&text),
                severity,
                code,
            });
        }
    }
    out
}

/// Parse human-readable `rustc`/`cargo` output (`error: ...`, `--> file:line:col`).
pub fn parse_human(output: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut pending: Option<(Severity, String, Option<String>)> = None;

    for line in output.lines() {
        let t = line.trim_start();
        let (sev, rest) = if let Some(r) = t.strip_prefix("error") {
            (Severity::Error, r)
        } else if let Some(r) = t.strip_prefix("warning") {
            (Severity::Warning, r)
        } else {
            (Severity::Info, "")
        };
        if !rest.is_empty() && (rest.starts_with(':') || rest.starts_with('[')) {
            let (msg, code) = split_code(rest);
            pending = Some((sev, msg, code));
            continue;
        }
        if let Some(arrow) = t.strip_prefix("--> ") {
            if let Some((sev, msg, code)) = pending.take() {
                if let Some((file, l, c)) = parse_location(arrow) {
                    out.push(Diagnostic {
                        file,
                        line: l,
                        col: c,
                        message: msg,
                        severity: sev,
                        code,
                    });
                }
            }
        }
    }
    out
}

fn split_code(rest: &str) -> (String, Option<String>) {
    // rest looks like ": message" or "[code]: message"
    let rest = rest.trim_start_matches(':').trim();
    if let Some(bracket) = rest.strip_prefix('[') {
        if let Some(end) = bracket.find(']') {
            let code = bracket[..end].to_string();
            let msg = bracket[end + 1..]
                .trim_start_matches(':')
                .trim()
                .to_string();
            return (first_line(&msg), Some(code));
        }
    }
    (first_line(rest), None)
}

fn parse_location(s: &str) -> Option<(String, usize, usize)> {
    // "src/main.rs:16:9" (trailing info after spaces ignored)
    let loc = s.split_whitespace().next()?;
    let parts: Vec<&str> = loc.rsplit(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let col: usize = parts[0].parse().ok()?;
    let line: usize = parts[1].parse().ok()?;
    let mut file_parts: Vec<&str> = parts[2..].to_vec();
    file_parts.reverse();
    Some((file_parts.join(":"), line.max(1), col.max(1)))
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_json_messages() {
        let json = r#"{"reason":"compiler-message","package_id":"x","target":{"kind":["lib"]},"message":{"message":"unused variable: `x`","level":"warning","spans":[{"file_name":"src/main.rs","line_start":7,"line_end":7,"column_start":9,"column_end":10,"is_primary":true}],"code":{"code":"unused_variables"}}}"#;
        let d = parse_cargo_json(json);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].file, "src/main.rs");
        assert_eq!((d[0].line, d[0].col), (7, 9));
        assert_eq!(d[0].code.as_deref(), Some("unused_variables"));
        assert_eq!(d[0].severity, Severity::Warning);
    }

    #[test]
    fn ignores_non_message_artifacts() {
        let json = "{\"reason\":\"build-finished\",\"success\":true}";
        assert!(parse_cargo_json(json).is_empty());
    }

    #[test]
    fn parses_human_format() {
        let text = "warning: unused variable: `x`\n --> src/main.rs:7:9\n";
        let d = parse_human(text);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].file, "src/main.rs");
        assert_eq!((d[0].line, d[0].col), (7, 9));
    }
}
