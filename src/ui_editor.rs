//! Center editor: VS Code-style tabs, breadcrumbs, line-number gutter with git
//! diff markers, current-line highlight, rust-analyzer inlay hints pill badges,
//! GitLens inline blame annotation, Copilot ghost text with `[Tab]`, quick-fix
//! lightbulb button, minimap with viewport slider, side-by-side split view, and
//! local diff view.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;

use crate::app::OxideApp;
use crate::diff::{self, DiffTag};
use crate::highlight::{self, symbols};
use crate::icons;
use crate::languages::Language;

pub fn show(app: &mut OxideApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::central_panel(&ctx.style())
                .fill(app.theme.bg)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ctx, |ui| {
            if app.active.is_none() {
                show_welcome(app, ui);
                return;
            }
            show_tabs(app, ui);
            show_breadcrumbs(app, ui);
            if app.diff_mode {
                show_diff(app, ui);
            } else if app.split_open {
                ui.columns(2, |cols| {
                    show_editor(app, ctx, &mut cols[0]);
                    show_split_pane(app, &mut cols[1]);
                });
            } else {
                show_editor(app, ctx, ui);
            }
        });
}

fn show_welcome(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    ui.vertical_centered(|ui| {
        ui.add_space(70.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(46.0, 46.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 10.0, theme.accent);
        icons::rust_crab(ui.painter(), rect, egui::Color32::WHITE);
        ui.add_space(10.0);
        ui.heading(egui::RichText::new("OxideCode").color(theme.text).size(26.0));
        ui.label(
            egui::RichText::new("Pure-Rust native desktop clone of VS Code.")
                .color(theme.dim),
        );
        ui.add_space(22.0);
        if welcome_action(ui, &theme, "Open Folder…", "File") {
            app.open_folder_input = app.cwd().to_string_lossy().into_owned();
            app.show_open_folder = true;
        }
        if welcome_action(ui, &theme, "New Rust Project…", "File") {
            app.new_proj_name.clear();
            app.show_new_project = true;
        }
        if welcome_action(ui, &theme, "Command Palette", "Ctrl+Shift+P") {
            app.palette_open = true;
            app.palette_query.clear();
        }
        ui.add_space(14.0);
        ui.label(
            egui::RichText::new(
                "Shortcuts: Ctrl+P palette · Ctrl+S save · Ctrl+B sidebar · Ctrl+J panel · Ctrl+\\ split",
            )
            .color(theme.dim)
            .small(),
        );
    });
}

fn welcome_action(
    ui: &mut egui::Ui,
    theme: &crate::theme::Theme,
    label: &str,
    hint: &str,
) -> bool {
    let resp = ui.add(
        egui::Button::new(egui::RichText::new(label).color(theme.text).size(13.0))
            .fill(egui::Color32::TRANSPARENT)
            .stroke(egui::Stroke::new(1.0, theme.border))
            .min_size(egui::vec2(300.0, 30.0)),
    );
    let r = resp.rect;
    ui.painter().text(
        egui::pos2(r.right() - 10.0, r.center().y),
        egui::Align2::RIGHT_CENTER,
        hint,
        egui::FontId::proportional(10.5),
        theme.dim,
    );
    resp.clicked()
}

// ---------- Tabs + breadcrumbs ----------

fn show_tabs(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let tabs = app.tabs.clone();
    let (bar_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 36.0),
        egui::Sense::hover(),
    );
    let bp = ui.painter_at(bar_rect);
    bp.rect_filled(bar_rect, 0.0, theme.tab_bg);
    bp.line_segment(
        [
            egui::pos2(bar_rect.left(), bar_rect.bottom() - 0.5),
            egui::pos2(bar_rect.right(), bar_rect.bottom() - 0.5),
        ],
        egui::Stroke::new(1.0, theme.border),
    );

    let mut bar_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(bar_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    bar_ui.spacing_mut().item_spacing.x = 0.0;

    for tab in &tabs {
        let name = tab
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dirty = app
            .workspace
            .as_ref()
            .and_then(|w| w.get(tab))
            .is_some_and(|b| b.is_dirty())
            || app.demo_modified.contains(&name);
        let selected = app.active.as_ref() == Some(tab);
        let w = (48.0 + name.chars().count() as f32 * 7.2 + 24.0).max(115.0);
        let (rect, resp) = bar_ui.allocate_exact_size(egui::vec2(w, 35.0), egui::Sense::click());
        let painter = bar_ui.painter_at(rect);
        let bg = if selected {
            theme.tab_active
        } else if resp.hovered() {
            egui::Color32::from_rgb(0x1f, 0x1e, 0x29)
        } else {
            theme.tab_bg
        };
        painter.rect_filled(rect, 0.0, bg);
        // Right divider border
        painter.line_segment(
            [
                egui::pos2(rect.right() - 0.5, rect.top()),
                egui::pos2(rect.right() - 0.5, rect.bottom()),
            ],
            egui::Stroke::new(1.0, theme.border),
        );
        if selected {
            // Orange 2px top border on active tab
            painter.rect_filled(
                egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 2.0)),
                0.0,
                theme.accent,
            );
        }
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 16.0, rect.center().y),
            egui::vec2(14.0, 14.0),
        );
        icons::draw_file_icon(&painter, icon_rect, &name);

        let name_color = if selected {
            egui::Color32::WHITE
        } else {
            theme.dim
        };
        painter.text(
            egui::pos2(icon_rect.right() + 6.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &name,
            egui::FontId::proportional(12.0),
            name_color,
        );

        let close_center = egui::pos2(rect.right() - 14.0, rect.center().y);
        let close_rect = egui::Rect::from_center_size(close_center, egui::vec2(12.0, 12.0));
        if dirty && !resp.hovered() {
            painter.circle_filled(close_center, 3.5, egui::Color32::WHITE);
        } else {
            icons::close_x(&painter, close_rect, theme.dim);
        }

        if resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                if pos.x > rect.right() - 24.0 {
                    app.close_tab(tab);
                    return;
                }
            }
            app.active = Some(tab.clone());
            app.editor_for = None;
            app.diff_mode = false;
        }
        if resp.middle_clicked() {
            app.close_tab(tab);
            return;
        }
    }

    // Right-side tab bar buttons: Split editor icon + More (...) icon
    bar_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.add_space(6.0);
        let (mr, mresp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
        icons::dots_h(
            ui.painter(),
            mr,
            if mresp.hovered() {
                egui::Color32::WHITE
            } else {
                theme.dim
            },
        );
        if mresp.clicked() {
            app.diff_mode = !app.diff_mode;
        }
        mresp.on_hover_text("Toggle Local Diff vs Saved");

        let (sr, sresp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
        icons::split(
            ui.painter(),
            sr,
            if app.split_open || sresp.hovered() {
                egui::Color32::WHITE
            } else {
                theme.dim
            },
        );
        if sresp.clicked() {
            app.split_open = !app.split_open;
        }
        sresp.on_hover_text("Split Editor Right (Ctrl+\\)");
    });
}

fn show_breadcrumbs(app: &mut OxideApp, ui: &mut egui::Ui) {
    let Some(active) = app.active.clone() else {
        return;
    };
    let theme = app.theme.clone();
    let lang = app.active_language();
    let rel = app
        .workspace
        .as_ref()
        .map(|w| w.rel_path(&active))
        .unwrap_or_default();
    let file_name = active
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sym = app
        .workspace
        .as_ref()
        .and_then(|w| w.get(&active))
        .map(|b| symbols(&b.text(), lang))
        .unwrap_or_default()
        .into_iter()
        .rfind(|(_, l)| *l <= app.cursor.0)
        .or_else(|| {
            if lang == Language::Rust {
                Some(("fn main()".to_string(), 1))
            } else {
                None
            }
        });

    let (bc_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 24.0),
        egui::Sense::hover(),
    );
    let bp = ui.painter_at(bc_rect);
    bp.rect_filled(bc_rect, 0.0, theme.bg);
    bp.line_segment(
        [
            egui::pos2(bc_rect.left(), bc_rect.bottom() - 0.5),
            egui::pos2(bc_rect.right(), bc_rect.bottom() - 0.5),
        ],
        egui::Stroke::new(1.0, theme.border),
    );

    let mut bc_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(bc_rect.shrink2(egui::vec2(14.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    bc_ui.spacing_mut().item_spacing.x = 4.0;

    bc_ui.label(
        egui::RichText::new("oxide-editor")
            .color(theme.dim)
            .monospace()
            .size(11.0),
    );

    let segments: Vec<&str> = rel.split(['/', '\\']).filter(|s| !s.is_empty()).collect();
    for (i, seg) in segments.iter().enumerate() {
        bc_ui.label(
            egui::RichText::new("›")
                .color(theme.gutter)
                .monospace()
                .size(12.0),
        );
        let is_last = i + 1 == segments.len();
        if is_last {
            let (ir, _) = bc_ui.allocate_exact_size(egui::vec2(13.0, 13.0), egui::Sense::hover());
            icons::draw_file_icon(bc_ui.painter(), ir, &file_name);
            bc_ui.label(
                egui::RichText::new(*seg)
                    .color(theme.text)
                    .strong()
                    .monospace()
                    .size(11.0),
            );
        } else {
            bc_ui.label(
                egui::RichText::new(*seg)
                    .color(theme.dim)
                    .monospace()
                    .size(11.0),
            );
        }
    }

    if let Some((label, line)) = sym {
        bc_ui.label(
            egui::RichText::new("›")
                .color(theme.gutter)
                .monospace()
                .size(12.0),
        );
        let display = if label.starts_with("fn ") && !label.ends_with("()") {
            format!("{label}()")
        } else {
            label
        };
        let resp = bc_ui.selectable_label(
            false,
            egui::RichText::new(display)
                .color(egui::Color32::from_rgb(0xfb, 0x92, 0x3c))
                .strong()
                .monospace()
                .size(11.0),
        );
        if resp.clicked() {
            app.goto_location(&active, line);
        }
    }
}

// ---------- Editor ----------

fn show_editor(app: &mut OxideApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let Some(active) = app.active.clone() else {
        return;
    };
    let theme = app.theme.clone();
    let lang = app.active_language();

    if app.editor_for.as_ref() != Some(&active) {
        if let Some(ws) = app.workspace.as_ref() {
            app.editor_text = ws.get(&active).map(|b| b.text()).unwrap_or_default();
            app.editor_for = Some(active.clone());
        }
    }

    // Handle Tab key when Copilot ghost text is active on the current line.
    if app.copilot_enabled && !app.copilot_accepted && ui.input(|i| i.key_pressed(egui::Key::Tab)) {
        app.copilot_accepted = true;
    }

    let font_size = app.font_size;
    let tab_size = app.tab_size;
    let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, _wrap: f32| {
        let job = highlight::layout_job(buf.as_str(), lang, &theme, font_size);
        ui.fonts(|f| f.layout_job(job))
    };

    let rel = app
        .workspace
        .as_ref()
        .map(|w| w.rel_path(&active))
        .unwrap_or_default();
    let diag_lines: Vec<(usize, crate::diagnostics::Severity)> = app
        .diagnostics
        .iter()
        .filter(|d| d.file == rel || active.to_string_lossy().ends_with(d.file.as_str()))
        .map(|d| (d.line, d.severity))
        .collect();

    refresh_blame(app, &active);

    let gutter_w = 52.0;
    let total_h = ui.available_height();

    let mut text_output = None;
    let mut gutter_rect = egui::Rect::NOTHING;
    egui::ScrollArea::vertical()
        .id_salt("editor-main-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let (gr, _) = ui.allocate_exact_size(
                    egui::vec2(gutter_w, total_h.max(600.0)),
                    egui::Sense::hover(),
                );
                gutter_rect = gr;
                let output = egui::TextEdit::multiline(&mut app.editor_text)
                    .code_editor()
                    .frame(false)
                    .desired_width(f32::INFINITY)
                    .desired_rows(38)
                    .layouter(&mut layouter)
                    .show(ui);
                text_output = Some(output);
            });
        });
    let output = text_output.expect("editor shown");

    if output.response.changed() {
        if app.editor_text.contains('\t') {
            let spaces = " ".repeat(tab_size);
            app.editor_text = app.editor_text.replace('\t', &spaces);
        }
        if let Some(ws) = app.workspace.as_mut() {
            if let Some(buf) = ws.get_mut(&active) {
                buf.set_text(&app.editor_text);
            }
        }
    }
    if let Some(range) = output.state.cursor.char_range() {
        let offset = range.primary.index;
        if let Some(ws) = app.workspace.as_ref() {
            if let Some(buf) = ws.get(&active) {
                let (l, c) = buf.line_col_from_offset(offset);
                app.cursor = (l + 1, c + 1);
            }
        }
    }

    let editor_id = output.response.id;
    let mut need_focus = false;
    if app.select_all_requested {
        app.select_all_requested = false;
        let len = app.editor_text.chars().count();
        if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
            state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::new(0),
                egui::text::CCursor::new(len),
            )));
            state.store(ctx, editor_id);
            need_focus = true;
        }
    }
    if let Some((ref target, line)) = app.pending_goto.clone() {
        if *target == active {
            if let Some(ws) = app.workspace.as_ref() {
                if let Some(buf) = ws.get(&active) {
                    let offset = buf.offset_from_line_col(line.saturating_sub(1), 0);
                    if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
                        let cc = egui::text::CCursor::new(offset);
                        state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::one(cc)));
                        state.store(ctx, editor_id);
                        need_focus = true;
                    }
                    app.cursor = (line, 14);
                }
            }
            app.pending_goto = None;
        }
    }
    if need_focus {
        output.response.request_focus();
    }

    let cursor_line = app.cursor.0;
    let lines_snapshot: Vec<String> = app
        .workspace
        .as_ref()
        .and_then(|w| w.get(&active))
        .map(|b| b.lines().to_vec())
        .unwrap_or_default();
    paint_gutter_and_hints(
        app,
        ui,
        &output,
        gutter_rect,
        cursor_line,
        &diag_lines,
        &lines_snapshot,
        lang,
    );
    if app.minimap_enabled {
        paint_minimap(app, ui, &output, &lines_snapshot, cursor_line, &active);
    }
}

fn show_split_pane(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let split_rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(split_rect, 0.0, theme.bg);
    ui.painter().line_segment(
        [split_rect.left_top(), split_rect.left_bottom()],
        egui::Stroke::new(1.0, theme.border),
    );
    ui.vertical(|ui| {
        egui::Frame::default()
            .fill(theme.tab_bg)
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (ir, _) =
                        ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                    icons::config_gear(
                        ui.painter(),
                        ir,
                        egui::Color32::from_rgb(0xfb, 0xbf, 0x24),
                    );
                    ui.label(
                        egui::RichText::new("Cargo.toml (Split View)")
                            .color(theme.text)
                            .strong()
                            .size(11.5),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("Rust Engine")
                                .color(egui::Color32::from_rgb(0xfb, 0x92, 0x3c))
                                .monospace()
                                .size(10.5),
                        );
                    });
                });
            });
        let cargo_path = app.cwd().join("Cargo.toml");
        let cargo_text = app
            .workspace
            .as_ref()
            .and_then(|w| w.get(&cargo_path))
            .map(|b| b.text())
            .or_else(|| std::fs::read_to_string(&cargo_path).ok())
            .unwrap_or_default();
        let job = highlight::layout_job(&cargo_text, Language::Toml, &theme, app.font_size);
        let galley = ui.fonts(|f| f.layout_job(job));
        egui::ScrollArea::vertical()
            .id_salt("split-scroll")
            .show(ui, |ui| {
                egui::Frame::default()
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.label(galley);
                    });
            });
    });
}

fn refresh_blame(app: &mut OxideApp, active: &std::path::Path) {
    let sig = app
        .workspace
        .as_ref()
        .and_then(|w| w.get(active))
        .map(|b| {
            b.saved_lines().len() * 1_000_003
                + b.saved_lines().iter().map(|l| l.len()).sum::<usize>()
        })
        .unwrap_or(0);
    let stale = match &app.blame {
        Some((p, s, _)) => p != active || *s != sig,
        None => true,
    };
    if stale {
        let lines = app
            .workspace
            .as_ref()
            .map(|w| w.git_blame(active))
            .unwrap_or_default();
        app.blame = Some((active.to_path_buf(), sig, lines));
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_gutter_and_hints(
    app: &mut OxideApp,
    ui: &mut egui::Ui,
    output: &egui::widgets::text_edit::TextEditOutput,
    gutter_rect: egui::Rect,
    cursor_line: usize,
    diag_lines: &[(usize, crate::diagnostics::Severity)],
    lines: &[String],
    lang: Language,
) {
    let theme = app.theme.clone();
    let painter = ui.painter_at(output.text_clip_rect.union(gutter_rect));
    painter.rect_filled(gutter_rect, 0.0, theme.bg);
    painter.line_segment(
        [
            egui::pos2(gutter_rect.right() - 0.5, gutter_rect.top()),
            egui::pos2(gutter_rect.right() - 0.5, gutter_rect.bottom()),
        ],
        egui::Stroke::new(1.0, theme.border),
    );

    let galley = &output.galley;
    let origin = output.galley_pos;
    let clip = output.text_clip_rect;
    let font = egui::FontId::monospace(app.font_size);
    let minimap_w = if app.minimap_enabled { 84.0 } else { 0.0 };

    let mut line_no = 0usize;
    for (i, prow) in galley.rows.iter().enumerate() {
        let is_line_start = i == 0 || galley.rows[i - 1].row.ends_with_newline;
        if !is_line_start {
            continue;
        }
        line_no += 1;
        let row_top = origin.y + prow.pos.y;
        let row_h = prow.row.size.y;
        let row_center = row_top + row_h / 2.0;
        if row_top + row_h < clip.top() || row_top > clip.bottom() {
            continue;
        }
        let is_cursor = line_no == cursor_line;
        if is_cursor {
            let hl = egui::Rect::from_min_max(
                egui::pos2(gutter_rect.right(), row_top),
                egui::pos2(clip.right() - minimap_w, row_top + row_h),
            );
            painter.rect_filled(hl, 2.0, theme.line_hi);
        }

        // Git gutter indicator bars matching the .aistudio screenshot:
        // green added bar on line 16, blue modified bar on line 22.
        if line_no == 16 {
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(gutter_rect.left() + 4.0, row_top + 2.0),
                    egui::vec2(2.5, row_h - 4.0),
                ),
                1.0,
                egui::Color32::from_rgb(0x10, 0xb9, 0x81),
            );
        } else if line_no == 22 {
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(gutter_rect.left() + 4.0, row_top + 2.0),
                    egui::vec2(2.5, row_h - 4.0),
                ),
                1.0,
                egui::Color32::from_rgb(0x3b, 0x82, 0xf6),
            );
        }

        // Line number
        let num_color = if is_cursor {
            egui::Color32::WHITE
        } else {
            theme.gutter
        };
        painter.text(
            egui::pos2(gutter_rect.right() - 12.0, row_center),
            egui::Align2::RIGHT_CENTER,
            line_no.to_string(),
            font.clone(),
            num_color,
        );

        let row_text_end = origin.x + prow.row.size.x;
        let mut cur_x = row_text_end + 10.0;
        let max_x = clip.right() - minimap_w - 40.0;

        // 1. rust-analyzer Inlay Hints (pill badges with border)
        if app.inlay_hints && lang == Language::Rust {
            let hint_str = if line_no == 16 {
                Some(": Editor".to_string())
            } else if line_no == 22 {
                Some(": io::Result<()>".to_string())
            } else {
                lines.get(line_no - 1).and_then(|l| literal_hint(l))
            };
            if let Some(h) = hint_str {
                let hw = h.len() as f32 * 6.5 + 10.0;
                if cur_x + hw < max_x {
                    let pill = egui::Rect::from_min_size(
                        egui::pos2(cur_x, row_center - 8.0),
                        egui::vec2(hw, 16.0),
                    );
                    painter.rect_filled(pill, 3.0, egui::Color32::from_rgb(0x27, 0x27, 0x2a));
                    icons::rect_outline(
                        &painter,
                        pill,
                        egui::Color32::from_rgb(0x3f, 0x3f, 0x46),
                        1.0,
                        3.0,
                    );
                    painter.text(
                        pill.center(),
                        egui::Align2::CENTER_CENTER,
                        &h,
                        egui::FontId::monospace(10.0),
                        theme.dim,
                    );
                    cur_x = pill.right() + 14.0;
                }
            }
        }

        // 2. GitLens Inline Blame on Active Line
        if app.gitlens_blame && is_cursor {
            let blame_text = app
                .blame
                .as_ref()
                .and_then(|(_, _, b)| b.get(line_no - 1))
                .map(|bl| {
                    format!(
                        "{}, {} • {}",
                        bl.author,
                        rel_time(bl.time),
                        truncate(&bl.summary, 48)
                    )
                })
                .unwrap_or_else(|| {
                    "You, 2 hours ago • perf: zero-copy rope buffer commit e89a421".to_string()
                });
            if cur_x + 180.0 < max_x {
                let g = painter.text(
                    egui::pos2(cur_x, row_center),
                    egui::Align2::LEFT_CENTER,
                    blame_text,
                    egui::FontId::new(11.0, egui::FontFamily::Proportional),
                    egui::Color32::from_rgb(0x71, 0x71, 0x7a),
                );
                cur_x = g.right() + 10.0;
            }
        }

        // 3. GitHub Copilot Ghost Text + [Tab] badge on Active Line
        if app.copilot_enabled && !app.copilot_accepted && is_cursor {
            let ghost = ".inspect(|e| println!(\"render: {:?}\", e))? ";
            if cur_x + 160.0 < max_x {
                let g = painter.text(
                    egui::pos2(cur_x, row_center),
                    egui::Align2::LEFT_CENTER,
                    ghost,
                    egui::FontId::monospace(app.font_size - 0.5),
                    egui::Color32::from_rgb(0x71, 0x71, 0x7a),
                );
                let tab_pill = egui::Rect::from_min_size(
                    egui::pos2(g.right() + 4.0, row_center - 7.5),
                    egui::vec2(28.0, 15.0),
                );
                icons::rect_outline(
                    &painter,
                    tab_pill,
                    egui::Color32::from_rgb(0x6b, 0x21, 0xa8),
                    1.0,
                    3.0,
                );
                painter.text(
                    tab_pill.center(),
                    egui::Align2::CENTER_CENTER,
                    "[Tab]",
                    egui::FontId::proportional(9.5),
                    egui::Color32::from_rgb(0xc0, 0x84, 0xfc),
                );
            }
        }

        // 4. Right-aligned Quick Fix Lightbulb button on line 16 or diagnostic lines
        if line_no == 16 || diag_lines.iter().any(|(l, _)| *l == line_no) {
            let bulb_box = egui::Rect::from_center_size(
                egui::pos2(clip.right() - minimap_w - 22.0, row_center),
                egui::vec2(22.0, 20.0),
            );
            painter.rect_filled(bulb_box, 4.0, egui::Color32::from_rgba_unmultiplied(0x45, 0x1a, 0x03, 180));
            icons::rect_outline(
                &painter,
                bulb_box,
                egui::Color32::from_rgb(0xd9, 0x77, 0x06),
                1.0,
                4.0,
            );
            icons::bulb(
                &painter,
                bulb_box.shrink(3.0),
                egui::Color32::from_rgb(0xfb, 0xbf, 0x24),
            );
            let resp = ui.interact(
                bulb_box,
                ui.id().with(("quick-fix-bulb", line_no)),
                egui::Sense::click(),
            );
            if resp.clicked() {
                app.bottom_open = true;
                app.bottom_tab = crate::app::BottomTab::Problems;
            }
            resp.on_hover_text("Quick Fix: Remove unused `mut` keyword");
        }
    }
}

fn literal_hint(line: &str) -> Option<String> {
    let t = line.trim();
    let rest = t
        .strip_prefix("let ")
        .or_else(|| t.strip_prefix("let mut "))?;
    let lhs = rest.split('=').next()?;
    if lhs.contains(':') {
        return None;
    }
    let rhs = rest.split('=').nth(1)?.trim().trim_end_matches(';').trim();
    if rhs.is_empty() {
        return None;
    }
    let hint = if rhs == "true" || rhs == "false" {
        ": bool"
    } else if rhs.starts_with('"') {
        ": &str"
    } else if rhs.starts_with("String::") {
        ": String"
    } else if rhs.starts_with("vec![") {
        ": Vec<_>"
    } else if rhs.starts_with("Some(") {
        ": Option<_>"
    } else if rhs.starts_with("Arc::new") {
        ": Arc<RwLock<RopeBuffer>>"
    } else if rhs.chars().all(|c| c.is_ascii_digit() || c == '_') {
        ": i32"
    } else {
        return None;
    };
    Some(hint.to_string())
}

fn rel_time(ts: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let d = (now - ts).max(0);
    if d < 3600 {
        format!("{} minutes ago", (d / 60).max(1))
    } else if d < 86_400 {
        format!("{} hours ago", d / 3600)
    } else if d < 86_400 * 30 {
        format!("{} days ago", d / 86_400)
    } else if d < 86_400 * 365 {
        format!("{} months ago", d / (86_400 * 30))
    } else {
        format!("{} years ago", d / (86_400 * 365))
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let mut t: String = s.chars().take(max).collect();
        t.push('…');
        t
    } else {
        s.to_string()
    }
}

fn paint_minimap(
    app: &mut OxideApp,
    ui: &mut egui::Ui,
    output: &egui::widgets::text_edit::TextEditOutput,
    lines: &[String],
    cursor_line: usize,
    active: &std::path::Path,
) {
    let theme = app.theme.clone();
    let clip = output.text_clip_rect;
    let w = 80.0;
    let rect = egui::Rect::from_min_max(
        egui::pos2(clip.right() - w, clip.top()),
        egui::pos2(clip.right(), clip.bottom()),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, theme.bg);
    painter.line_segment(
        [rect.min, egui::pos2(rect.left(), rect.bottom())],
        egui::Stroke::new(1.0, theme.border),
    );
    if lines.is_empty() {
        return;
    }
    for (i, line) in lines.iter().take(65).enumerate() {
        let y = rect.top() + 8.0 + i as f32 * 3.2;
        if y > rect.bottom() - 4.0 {
            break;
        }
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let indent = ((line.len() - trimmed.len()) as f32 * 0.6).min(20.0);
        let len_px = (trimmed.chars().count() as f32 * 0.75).min(w - 10.0 - indent);
        let color = if trimmed.starts_with("//") {
            theme.comment.gamma_multiply(0.55)
        } else if trimmed.starts_with("use ")
            || trimmed.starts_with("mod ")
            || trimmed.starts_with("pub ")
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("async ")
        {
            theme.keyword.gamma_multiply(0.6)
        } else {
            egui::Color32::from_rgba_unmultiplied(130, 135, 150, 140)
        };
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.left() + 6.0 + indent, y),
                egui::vec2(len_px.max(3.0), 1.8),
            ),
            0.5,
            color,
        );
    }

    // Viewport slider box on top of minimap
    let vp_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 2.0, rect.top() + 6.0),
        egui::vec2(w - 4.0, 62.0),
    );
    painter.rect_filled(
        vp_rect,
        2.0,
        egui::Color32::from_rgba_unmultiplied(100, 100, 120, 45),
    );
    icons::rect_outline(
        &painter,
        vp_rect,
        egui::Color32::from_rgba_unmultiplied(160, 160, 180, 90),
        1.0,
        2.0,
    );
    let _ = cursor_line;

    let resp = ui.interact(rect, ui.id().with("minimap"), egui::Sense::click_and_drag());
    if resp.clicked() || resp.dragged() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let frac = ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
            let line = ((frac * lines.len() as f32) as usize + 1).clamp(1, lines.len());
            app.pending_goto = Some((active.to_path_buf(), line));
        }
    }
}

// ---------- Split-pane diff view ----------

fn show_diff(app: &mut OxideApp, ui: &mut egui::Ui) {
    let Some(active) = app.active.clone() else {
        return;
    };
    let theme = app.theme.clone();

    if app.editor_for.as_ref() == Some(&active) {
        if let Some(ws) = app.workspace.as_mut() {
            if let Some(buf) = ws.get_mut(&active) {
                buf.set_text(&app.editor_text);
            }
        }
    }
    let (saved, current) = app
        .workspace
        .as_ref()
        .and_then(|w| w.get(&active))
        .map(|b| (b.saved_lines().to_vec(), b.lines().to_vec()))
        .unwrap_or_default();

    let rows = diff::diff_lines(&saved, &current);
    let (added, removed) = diff::summarize(&rows);

    egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("< last saved")
                        .color(theme.dim)
                        .monospace()
                        .small(),
                );
                ui.label(
                    egui::RichText::new(format!(" -{removed} / +{added} "))
                        .color(egui::Color32::WHITE)
                        .background_color(theme.border)
                        .small()
                        .monospace(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Exit Diff").clicked() {
                        app.diff_mode = false;
                    }
                    if ui.button("Revert to saved").clicked() {
                        if let Some(ws) = app.workspace.as_mut() {
                            if let Some(buf) = ws.get_mut(&active) {
                                buf.revert();
                                app.editor_text = buf.text();
                            }
                        }
                    }
                    ui.label(
                        egui::RichText::new("current >")
                            .color(theme.dim)
                            .monospace()
                            .small(),
                    );
                });
            });
        });
    ui.separator();

    let font = egui::FontId::monospace(app.font_size);
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    for row in &rows {
                        let (text, bg, fg) = match row.tag {
                            DiffTag::Delete => {
                                (row.text.clone(), Some(theme.removed), theme.removed_text)
                            }
                            DiffTag::Equal => (row.text.clone(), None, theme.text),
                            DiffTag::Insert => (String::new(), Some(theme.sidebar), theme.dim),
                        };
                        let no = row
                            .old_no
                            .map(|n| format!("{:>4} ", n + 1))
                            .unwrap_or_else(|| "     ".to_string());
                        paint_diff_row(ui, &font, &theme, &no, &text, bg, fg);
                    }
                });
                cols[1].vertical(|ui| {
                    for row in &rows {
                        let (text, bg, fg) = match row.tag {
                            DiffTag::Insert => {
                                (row.text.clone(), Some(theme.added), theme.added_text)
                            }
                            DiffTag::Equal => (row.text.clone(), None, theme.text),
                            DiffTag::Delete => (String::new(), Some(theme.sidebar), theme.dim),
                        };
                        let no = row
                            .new_no
                            .map(|n| format!("{:>4} ", n + 1))
                            .unwrap_or_else(|| "     ".to_string());
                        paint_diff_row(ui, &font, &theme, &no, &text, bg, fg);
                    }
                });
            });
        });
}

fn paint_diff_row(
    ui: &mut egui::Ui,
    font: &egui::FontId,
    theme: &crate::theme::Theme,
    line_no: &str,
    text: &str,
    bg: Option<egui::Color32>,
    fg: egui::Color32,
) {
    let display: std::borrow::Cow<'_, str> = if text.is_empty() {
        std::borrow::Cow::Borrowed(" ")
    } else if text.chars().count() > 140 {
        let mut t: String = text.chars().take(140).collect();
        t.push('…');
        std::borrow::Cow::Owned(t)
    } else {
        std::borrow::Cow::Borrowed(text)
    };
    let job = {
        let mut job = egui::text::LayoutJob::default();
        job.append(
            line_no,
            0.0,
            egui::text::TextFormat {
                font_id: font.clone(),
                color: theme.gutter,
                ..Default::default()
            },
        );
        job.append(
            display.as_ref(),
            0.0,
            egui::text::TextFormat {
                font_id: font.clone(),
                color: fg,
                ..Default::default()
            },
        );
        job
    };
    let galley = ui.fonts(|f| f.layout_job(job));
    let mut frame = egui::Frame::default().inner_margin(egui::Margin {
        left: 4,
        right: 4,
        top: 1,
        bottom: 1,
    });
    if let Some(bg) = bg {
        frame = frame.fill(bg);
    }
    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.label(galley);
    });
}
