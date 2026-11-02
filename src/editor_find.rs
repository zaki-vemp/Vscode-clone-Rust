//! In-editor find/replace, sharing the workspace search matcher.
use crate::app::OxideApp;
use crate::search::{Matcher, SearchOptions};
use eframe::egui;
use std::ops::Range;
use std::path::PathBuf;

#[derive(Default)]
pub struct Find {
    pub open: bool,
    pub focus: bool,
    pub replace_open: bool,
    pub options: SearchOptions,
    pub replacement: String,
    pub next: i32,
    pub matches: Vec<Range<usize>>,
    selected: usize,
    char_matches: Vec<Range<usize>>,
    key: Option<(PathBuf, u64, SearchOptions)>,
    error: Option<String>,
}

pub fn show(app: &mut OxideApp, ui: &mut egui::Ui) {
    if !app.find.open {
        return;
    }
    let Some(active) = app.active.clone() else {
        return;
    };
    let mut movement = std::mem::take(&mut app.find.next);
    let mut replace = false;
    let mut replace_all = false;
    egui::Frame::default()
        .fill(app.theme.widget_bg)
        .inner_margin(egui::Margin::same(6))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let query = ui.add(
                    egui::TextEdit::singleline(&mut app.find.options.query)
                        .hint_text("Find")
                        .desired_width(240.0),
                );
                if app.find.focus {
                    query.request_focus();
                    app.find.focus = false;
                }
                ui.toggle_value(&mut app.find.options.case_sensitive, "Aa")
                    .on_hover_text("Match Case");
                ui.toggle_value(&mut app.find.options.whole_word, "ab")
                    .on_hover_text("Whole Word");
                ui.toggle_value(&mut app.find.options.regex, ".*")
                    .on_hover_text("Regular Expression");
                let count = app.find.matches.len();
                ui.label(if count == 0 {
                    "No results".into()
                } else {
                    format!(
                        "{} of {}{}",
                        app.find.selected + 1,
                        count,
                        if count == 10_000 { "+" } else { "" }
                    )
                });
                if ui.button("↑").clicked() {
                    movement = -1;
                }
                if ui.button("↓").clicked() {
                    movement = 1;
                }
                ui.toggle_value(&mut app.find.replace_open, "Replace");
                if ui.button("×").clicked() {
                    app.find.open = false;
                }
                if query.has_focus()
                    && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
                {
                    movement = 1;
                }
                if query.has_focus()
                    && ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Enter))
                {
                    movement = -1;
                }
            });
            if app.find.replace_open {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.find.replacement)
                            .hint_text("Replace")
                            .desired_width(240.0),
                    );
                    replace = ui.button("Replace").clicked();
                    replace_all = ui.button("Replace All").clicked();
                });
            }
            if let Some(error) = &app.find.error {
                ui.colored_label(app.theme.error, error);
            }
        });
    let revision = app
        .workspace
        .as_ref()
        .and_then(|w| w.get(&active))
        .map(|b| b.revision())
        .unwrap_or(0);
    let key = (active.clone(), revision, app.find.options.clone());
    let changed = app.find.key.as_ref() != Some(&key);
    if changed {
        app.find.matches.clear();
        app.find.error = None;
        app.find.selected = 0;
        if !app.find.options.query.is_empty() {
            match Matcher::new(&app.find.options) {
                Ok(matcher) => {
                    app.find.matches = matcher.find_iter(&app.editor_text).take(10_000).collect()
                }
                Err(error) => app.find.error = Some(error),
            }
        }
        app.find.key = Some(key);
        app.find.char_matches.clear();
        let mut byte_offset = 0;
        let mut char_offset = 0;
        for range in &app.find.matches {
            let start = char_offset + app.editor_text[byte_offset..range.start].chars().count();
            let end = start + app.editor_text[range.clone()].chars().count();
            app.find.char_matches.push(start..end);
            byte_offset = range.end;
            char_offset = end;
        }
    }
    if app.find.matches.is_empty() {
        return;
    }
    if movement != 0 {
        app.find.selected = (app.find.selected as i32 + movement)
            .rem_euclid(app.find.matches.len() as i32) as usize;
    }
    if replace || replace_all {
        match replace_matches(
            &app.editor_text,
            &app.find.options,
            &app.find.replacement,
            if replace_all {
                None
            } else {
                Some(app.find.selected)
            },
        ) {
            Ok(text) => {
                app.editor_text = text;
                if let Some(buffer) = app.workspace.as_mut().and_then(|w| w.get_mut(&active)) {
                    buffer.set_text(&app.editor_text);
                }
                app.find.key = None;
            }
            Err(error) => app.find.error = Some(error),
        }
    } else if changed || movement != 0 {
        let range = app.find.matches[app.find.selected].clone();
        let start = app.editor_text[..range.start].chars().count();
        let end = start + app.editor_text[range.clone()].chars().count();
        let line = app.editor_text[..range.start]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1;
        app.pending_selection = Some((active.clone(), start..end));
        app.pending_goto = Some((active, line));
    }
}

fn replace_matches(
    text: &str,
    options: &SearchOptions,
    replacement: &str,
    selected: Option<usize>,
) -> Result<String, String> {
    let matcher = Matcher::new(options)?;
    let mut result = String::new();
    let mut previous = 0;
    for (index, range) in matcher.find_iter(text).enumerate() {
        if selected.is_some_and(|selected| selected != index) {
            continue;
        }
        result.push_str(&text[previous..range.start]);
        result.push_str(&matcher.replacement_text(text, &range, replacement, options.regex)?);
        previous = range.end;
    }
    result.push_str(&text[previous..]);
    Ok(result)
}

pub fn paint(
    find: &Find,
    ui: &egui::Ui,
    output: &egui::text_edit::TextEditOutput,
    color: egui::Color32,
) {
    if !find.open {
        return;
    }
    let painter = ui.painter_at(output.text_clip_rect);
    // Paint only visible matches, so the cost follows the viewport.
    for range in &find.char_matches {
        let a = output
            .galley
            .pos_from_cursor(egui::text::CCursor::new(range.start))
            .translate(output.galley_pos.to_vec2());
        if a.top() > output.text_clip_rect.bottom() {
            break;
        }
        let b = output
            .galley
            .pos_from_cursor(egui::text::CCursor::new(range.end))
            .translate(output.galley_pos.to_vec2());
        if a.bottom() < output.text_clip_rect.top() || a.top() > output.text_clip_rect.bottom() {
            continue;
        }
        if (a.top() - b.top()).abs() < 1.0 {
            painter.rect_stroke(
                egui::Rect::from_min_max(
                    a.min,
                    egui::pos2(b.right().max(a.left() + 2.0), a.bottom()),
                ),
                0.0,
                egui::Stroke::new(1.0, color),
                egui::StrokeKind::Inside,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replaces_unicode_and_regex_captures_without_touching_other_matches() {
        let mut options = SearchOptions {
            query: "(猫)([0-9])".into(),
            regex: true,
            ..Default::default()
        };
        assert_eq!(
            replace_matches("猫1 猫2", &options, "$2:$1", None).unwrap(),
            "1:猫 2:猫"
        );
        assert_eq!(
            replace_matches("猫1 猫2", &options, "dog", Some(1)).unwrap(),
            "猫1 dog"
        );
        options.query = "[".into();
        assert!(replace_matches("unchanged", &options, "x", None).is_err());
    }
}
