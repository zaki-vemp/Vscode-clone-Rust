//! VS Code JSON/JSONC color themes, including inherited themes.
use anyhow::{anyhow, bail, Context, Result};
use eframe::egui::Color32;
use serde_json::Value;

use crate::theme::{Theme, ThemeKind};

/// JSONC permits comments and trailing commas. Blank them without changing
/// string contents, UTF-8, or error locations; serde handles JSON validation.
pub fn parse_jsonc(text: &str) -> Result<Value> {
    let mut bytes = text.trim_start_matches('\u{feff}').as_bytes().to_vec();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 2;
                    } else if bytes[i] == b'"' {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    bytes[i] = b' ';
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                bytes[i..i + 2].fill(b' ');
                i += 2;
                while i + 1 < bytes.len() && &bytes[i..i + 2] != b"*/" {
                    if bytes[i] != b'\n' && bytes[i] != b'\r' {
                        bytes[i] = b' ';
                    }
                    i += 1;
                }
                if i + 1 >= bytes.len() {
                    bail!("Unterminated JSONC comment");
                }
                bytes[i..i + 2].fill(b' ');
                i += 2;
            }
            _ => i += 1,
        }
    }
    i = 0;
    let mut quoted = false;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if quoted => {
                i += 2;
                continue;
            }
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                let next = bytes[i + 1..].iter().find(|b| !b.is_ascii_whitespace());
                if matches!(next, Some(b'}' | b']')) {
                    bytes[i] = b' ';
                }
            }
            _ => {}
        }
        i += 1;
    }
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn color(text: &str) -> Option<Color32> {
    let hex = text.strip_prefix('#')?;
    if !hex.is_ascii() {
        return None;
    }
    let hex = match hex.len() {
        3 | 4 => hex.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 | 8 => hex.to_string(),
        _ => return None,
    };
    let n = |i| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some(Color32::from_rgba_unmultiplied(
        n(0)?,
        n(2)?,
        n(4)?,
        if hex.len() == 8 { n(6)? } else { 255 },
    ))
}

/// Resolve package-relative includes on every platform; never escape the package.
pub fn relative_path(base: &str, relative: &str) -> Result<String> {
    let relative = relative.replace('\\', "/");
    if relative.starts_with('/') || relative.contains(':') {
        bail!("Theme paths must be relative");
    }
    let mut parts: Vec<&str> = base
        .rsplit_once('/')
        .map(|(p, _)| p.split('/').collect())
        .unwrap_or_default();
    for part in relative.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts
                    .pop()
                    .ok_or_else(|| anyhow!("Theme path escapes the extension"))?;
            }
            p => parts.push(p),
        }
    }
    Ok(parts.join("/"))
}

pub fn load(
    path: &str,
    light: bool,
    read: &mut dyn FnMut(&str) -> Result<String>,
) -> Result<Theme> {
    let mut theme = ThemeKind::DarkModern.base_colors();
    theme.dark = !light;
    if light {
        theme.bg = Color32::WHITE;
        theme.text = Color32::from_gray(59);
        theme.panel = Color32::WHITE;
        theme.sidebar = Color32::from_gray(248);
        theme.activity = theme.sidebar;
        theme.titlebar = theme.sidebar;
        theme.tab_bg = theme.sidebar;
        theme.tab_active = theme.bg;
        theme.status = theme.sidebar;
        theme.status_fg = theme.text;
        theme.dim = Color32::from_gray(97);
        theme.border = Color32::from_gray(229);
        theme.input_bg = Color32::WHITE;
        theme.input_border = Color32::from_gray(206);
        theme.widget_bg = theme.sidebar;
        theme.widget_border = theme.input_border;
        theme.list_hover = Color32::from_gray(232);
        theme.list_active = Color32::from_gray(228);
        theme.list_focus = Color32::from_rgb(200, 225, 250);
        theme.selection = Color32::from_rgb(173, 214, 255);
        theme.line_hi = Color32::from_gray(238);
        theme.gutter_active = theme.text;
        theme.find_match = Color32::from_rgb(234, 192, 120);
        theme.comment = Color32::from_rgb(0, 128, 0);
        theme.keyword = Color32::from_rgb(0, 0, 255);
        theme.string = Color32::from_rgb(163, 21, 21);
        theme.number = Color32::from_rgb(9, 134, 88);
        theme.typ = Color32::from_rgb(38, 127, 153);
        theme.activity_fg = theme.text;
        theme.activity_dim = theme.dim;
        theme.terminal_bg = theme.panel;
        theme.terminal_fg = theme.text;
        theme.git_untracked_fg = Color32::from_rgb(40, 122, 40);
        theme.git_modified_fg = Color32::from_rgb(137, 85, 3);
    }
    load_into(&mut theme, path, read, &mut Vec::new())?;
    Ok(theme)
}

fn load_into(
    theme: &mut Theme,
    path: &str,
    read: &mut dyn FnMut(&str) -> Result<String>,
    stack: &mut Vec<String>,
) -> Result<()> {
    if stack.len() >= 16 || stack.iter().any(|p| p == path) {
        bail!("Cyclic or excessively deep theme include: {path}");
    }
    stack.push(path.to_string());
    let value = parse_jsonc(&read(path)?).with_context(|| format!("Invalid theme {path}"))?;
    if !value.is_object() {
        bail!("A color theme must be a JSON object");
    }
    if let Some(include) = value.get("include").and_then(Value::as_str) {
        load_into(theme, &relative_path(path, include)?, read, stack)?;
    }
    if value.get("tokenColors").is_some_and(Value::is_string) {
        bail!("Legacy .tmTheme token files are not supported; use a JSON color theme");
    }
    apply(theme, &value);
    stack.pop();
    Ok(())
}

fn apply(t: &mut Theme, v: &Value) {
    if let Some(name) = v["name"].as_str() {
        t.name = name.to_string();
    }
    if let Some(colors) = v["colors"].as_object() {
        macro_rules! map {
            ($($field:ident => $key:literal),* $(,)?) => { $(
                if let Some(c) = colors.get($key).and_then(Value::as_str).and_then(color) { t.$field = c; }
            )* };
        }
        map! {
            activity_fg => "activityBar.foreground", activity_dim => "activityBar.inactiveForeground",
            terminal_bg => "terminal.background", terminal_fg => "terminal.foreground",
            bg => "editor.background", text => "editor.foreground",
            panel => "panel.background", sidebar => "sideBar.background",
            activity => "activityBar.background", titlebar => "titleBar.activeBackground",
            tab_bg => "editorGroupHeader.tabsBackground", tab_active => "tab.activeBackground",
            status => "statusBar.background", status_fg => "statusBar.foreground",
            border => "panel.border", dim => "descriptionForeground",
            gutter => "editorLineNumber.foreground", gutter_active => "editorLineNumber.activeForeground",
            accent => "button.background", focus_border => "focusBorder",
            line_hi => "editor.lineHighlightBorder", input_bg => "input.background",
            input_border => "input.border", list_hover => "list.hoverBackground",
            list_active => "list.inactiveSelectionBackground", list_focus => "list.activeSelectionBackground",
            widget_bg => "editorWidget.background", widget_border => "widget.border",
            badge_bg => "badge.background", selection => "editor.selectionBackground",
            error => "errorForeground", warning => "editorWarning.foreground",
            git_added => "editorGutter.addedBackground", git_modified => "editorGutter.modifiedBackground",
            git_deleted => "editorGutter.deletedBackground", git_untracked_fg => "gitDecoration.untrackedResourceForeground",
            git_modified_fg => "gitDecoration.modifiedResourceForeground", git_deleted_fg => "gitDecoration.deletedResourceForeground",
            git_ignored_fg => "gitDecoration.ignoredResourceForeground", find_match => "editor.findMatchBackground",
            added => "diffEditor.insertedLineBackground", removed => "diffEditor.removedLineBackground",
        }
    }
    // ponytail: map broad TextMate scopes onto the native scanner's token
    // classes. Full per-language scope matching needs a TextMate grammar engine.
    if let Some(tokens) = v["tokenColors"].as_array() {
        for token in tokens {
            let scopes: Vec<&str> = match &token["scope"] {
                Value::String(s) => s.split(',').map(str::trim).collect(),
                Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
                _ => Vec::new(),
            };
            for scope in scopes {
                let field = match scope {
                    "keyword" | "storage" | "storage.type" | "storage.modifier" => {
                        Some(&mut t.keyword)
                    }
                    "entity.name.type" | "support.type" | "support.class" | "entity.name.class" => {
                        Some(&mut t.typ)
                    }
                    "string" => Some(&mut t.string),
                    "comment" => {
                        if let Some(style) = token["settings"]["fontStyle"].as_str() {
                            t.comment_italics = style.split_whitespace().any(|s| s == "italic");
                        }
                        Some(&mut t.comment)
                    }
                    "constant.numeric" => Some(&mut t.number),
                    "entity.name.function" | "support.function" => Some(&mut t.macro_color),
                    _ => None,
                };
                if let (Some(field), Some(c)) = (
                    field,
                    token["settings"]["foreground"].as_str().and_then(color),
                ) {
                    *field = c;
                }
            }
        }
    }
}

pub fn builtin(file: &str, light: bool) -> Theme {
    load(file, light, &mut |path| {
        let source = match path {
            "dark_modern.json" => include_str!("assets/vscode-themes/dark_modern.json"),
            "dark_plus.json" => include_str!("assets/vscode-themes/dark_plus.json"),
            "dark_vs.json" => include_str!("assets/vscode-themes/dark_vs.json"),
            "light_modern.json" => include_str!("assets/vscode-themes/light_modern.json"),
            "light_plus.json" => include_str!("assets/vscode-themes/light_plus.json"),
            "light_vs.json" => include_str!("assets/vscode-themes/light_vs.json"),
            "hc_black.json" => include_str!("assets/vscode-themes/hc_black.json"),
            "hc_light.json" => include_str!("assets/vscode-themes/hc_light.json"),
            _ => bail!("Unknown bundled theme {path}"),
        };
        Ok(source.to_string())
    })
    .expect("bundled VS Code themes are validated by tests")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jsonc_preserves_strings_and_validates_comments() {
        let v = parse_jsonc(
            "{ /* hi */ \"url\": \"https://x/\\\"/*中文\", // comment\n \"items\": [1,], }",
        )
        .unwrap();
        assert_eq!(v["url"], "https://x/\"/*中文");
        assert_eq!(v["items"][0], 1);
        assert!(parse_jsonc("{/* open").is_err());
    }
    #[test]
    fn colors_and_paths_are_bounded() {
        assert_eq!(color("#f08"), color("#ff0088"));
        assert_eq!(color("#ff008880").unwrap().a(), 128);
        assert!(color("#ééé").is_none());
        assert!(relative_path("themes/a.json", "../../secret").is_err());
        assert!(relative_path("a.json", "C:\\secret").is_err());
        assert_eq!(
            relative_path("themes/sub/a.json", "../base.json").unwrap(),
            "themes/base.json"
        );
    }
    #[test]
    fn bundled_themes_resolve_with_official_colors() {
        for (file, light) in [
            ("dark_modern.json", false),
            ("dark_plus.json", false),
            ("light_modern.json", true),
            ("light_plus.json", true),
            ("hc_black.json", false),
            ("hc_light.json", true),
        ] {
            let t = builtin(file, light);
            assert_eq!(t.dark, !light);
            assert_ne!(t.bg, t.text);
        }
        let dark = builtin("dark_modern.json", false);
        assert_eq!(dark.bg, color("#1f1f1f").unwrap());
        assert_eq!(dark.keyword, color("#569cd6").unwrap());
        assert_eq!(builtin("light_modern.json", true).bg, Color32::WHITE);
    }
    #[test]
    fn includes_override_and_cycles_fail() {
        let mut read = |path: &str| {
            Ok(match path {
            "base.json" => r##"{"colors":{"editor.background":"#123456"},"tokenColors":[{"scope":"comment","settings":{"foreground":"#abcdef"}}]}"##,
            _ => r##"{"include":"base.json","colors":{"editor.foreground":"#654321"}}"##,
        }.to_string())
        };
        let t = load("theme.json", false, &mut read).unwrap();
        assert_eq!(t.bg, color("#123456").unwrap());
        assert_eq!(t.comment, color("#abcdef").unwrap());
        assert!(load("cycle.json", false, &mut |_| Ok(
            r#"{"include":"cycle.json"}"#.into()
        ))
        .is_err());
    }
}
