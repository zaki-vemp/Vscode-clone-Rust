//! Center editor: VS Code-style tabs, breadcrumbs, line-number gutter,
//! current-line border highlight, GitLens inline blame, minimap,
//! side-by-side split view (same file, read-only), and local diff view.

use std::time::{SystemTime, UNIX_EPOCH};

use eframe::egui;

use crate::app::{menu_style_accent, OxideApp};
use crate::diff::{self, DiffTag};
use crate::highlight;
use crate::icons::{self, ci};
use crate::theme::Theme;

pub fn show(app: &mut OxideApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(
            egui::Frame::central_panel(&ctx.style())
                .fill(app.theme.bg)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ctx, |ui| {
            // A Git diff is independent of any editable file — render it
            // even with no editor open (e.g. a deleted file).
            if app.git_diff.is_some() {
                show_git_diff(app, ui);
                return;
            }
            if app.active.is_none() {
                show_watermark(app, ui);
                return;
            }
            show_tabs(app, ui);
            show_breadcrumbs(app, ui);
            if let Some(active) = app.active.clone() {
                if app.active_language() == crate::languages::Language::Json {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut app.json_view.tree, false, "Source");
                        ui.selectable_value(&mut app.json_view.tree, true, "JSON Tree");
                        if ui.button("Beautify JSON").clicked() {
                            app.format_json(false);
                        }
                        if ui.button("Minify JSON").clicked() {
                            app.format_json(true);
                        }
                    });
                    if app.json_view.tree {
                        if let Some(buffer) = app.workspace.as_ref().and_then(|w| w.get(&active)) {
                            app.json_view.show(
                                ui,
                                active.clone(),
                                buffer.revision(),
                                &buffer.text(),
                            );
                        }
                        return;
                    }
                }
                if app.preview.wants_preview(&active) {
                    let source = app
                        .workspace
                        .as_ref()
                        .and_then(|w| w.get(&active))
                        .map(|b| b.text());
                    if app.preview.show(ctx, ui, &active, source) {
                        app.preview.source_paths.insert(active.clone());
                        app.open_file(&active);
                    }
                    return;
                }
                if crate::preview::has_source(&active) && ui.button("Open Preview").clicked() {
                    app.preview.source_paths.remove(&active);
                    app.preview.invalidate();
                    return;
                }
            }
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

/// VS Code "watermark": a centered list of commands with key pills.
fn show_watermark(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let has_folder = app.workspace.is_some();
    let rows: &[(&str, &str)] = if has_folder {
        &[
            ("Show All Commands", "Ctrl+Shift+P"),
            ("Go to File", "Ctrl+P"),
            ("Find in Files", "Ctrl+Shift+F"),
            ("Toggle Terminal", "Ctrl+`"),
        ]
    } else {
        &[
            ("Open Folder…", ""),
            ("Show All Commands", "Ctrl+Shift+P"),
            ("Toggle Terminal", "Ctrl+`"),
        ]
    };
    let row_h = 26.0;
    let block_h = rows.len() as f32 * row_h;
    ui.add_space(((ui.available_height() - block_h) * 0.42).max(40.0));
    for (label, keys) in rows {
        ui.horizontal(|ui| {
            let label_g = ui.painter().layout_no_wrap(
                label.to_string(),
                egui::FontId::proportional(13.0),
                theme.dim,
            );
            let pill_w = key_pill_w(ui, keys);
            let total = label_g.size().x + 16.0 + pill_w;
            ui.add_space(((ui.available_width() - total) * 0.5).max(0.0));
            let (lr, lresp) =
                ui.allocate_exact_size(egui::vec2(label_g.size().x, row_h), egui::Sense::click());
            ui.painter().galley(
                egui::pos2(lr.left(), lr.center().y - label_g.size().y / 2.0),
                label_g,
                theme.dim,
            );
            ui.add_space(16.0);
            if !keys.is_empty() {
                key_pill(ui, keys, row_h);
            } else {
                ui.allocate_exact_size(egui::vec2(pill_w, row_h), egui::Sense::hover());
            }
            if lresp.clicked() {
                watermark_click(app, label);
            }
        });
    }
}

fn key_pill_w(ui: &mut egui::Ui, keys: &str) -> f32 {
    if keys.is_empty() {
        return 0.0;
    }
    keys.split('+')
        .map(|k| {
            let g = ui.painter().layout_no_wrap(
                k.trim().to_string(),
                egui::FontId::proportional(11.0),
                egui::Color32::WHITE,
            );
            g.size().x + 12.0
        })
        .sum::<f32>()
        + 4.0 * keys.matches('+').count() as f32
}

fn key_pill(ui: &mut egui::Ui, keys: &str, row_h: f32) {
    let n = keys.split('+').count();
    for (i, k) in keys.split('+').enumerate() {
        let k = k.trim();
        let g = ui.painter().layout_no_wrap(
            k.to_string(),
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgb(0xcc, 0xcc, 0xcc),
        );
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(g.size().x + 12.0, row_h - 8.0),
            egui::Sense::hover(),
        );
        let p = ui.painter_at(rect);
        p.rect_filled(
            rect,
            3.0,
            egui::Color32::from_rgba_unmultiplied(128, 128, 128, 43),
        );
        icons::rect_outline(
            &p,
            rect,
            egui::Color32::from_rgba_unmultiplied(51, 51, 51, 153),
            1.0,
            3.0,
        );
        p.galley(
            egui::pos2(rect.left() + 6.0, rect.center().y - g.size().y / 2.0),
            g,
            egui::Color32::from_rgb(0xcc, 0xcc, 0xcc),
        );
        if i + 1 < n {
            ui.add_space(4.0);
        }
    }
}

fn watermark_click(app: &mut OxideApp, label: &str) {
    match label {
        "Show All Commands" | "Go to File" => {
            app.palette_open = true;
            app.palette_query.clear();
            app.palette_selected = 0;
        }
        "Find in Files" => {
            app.activity = crate::app::Activity::Search;
            app.sidebar_open = true;
        }
        "Toggle Terminal" => {
            app.bottom_open = true;
            app.bottom_tab = crate::app::BottomTab::Terminal;
            app.pty.ensure_session();
        }
        "Open Folder…" => app.pick_and_open_folder(),
        _ => {}
    }
}

// ---------- Tabs + breadcrumbs ----------

fn show_tabs(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let tabs = app.tabs.clone();
    let (bar_rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 35.0), egui::Sense::hover());
    let bp = ui.painter_at(bar_rect);
    bp.rect_filled(bar_rect, 0.0, theme.tab_bg);
    bp.line_segment(
        [
            egui::pos2(bar_rect.left(), bar_rect.bottom() - 0.5),
            egui::pos2(bar_rect.right(), bar_rect.bottom() - 0.5),
        ],
        egui::Stroke::new(1.0, theme.border),
    );

    // Right-side controls region (split + ellipsis), then tabs scroll area.
    let (right_rect, tabs_rect) = {
        let w = 56.0;
        (
            egui::Rect::from_min_max(
                egui::pos2(bar_rect.right() - w, bar_rect.top()),
                bar_rect.max,
            ),
            egui::Rect::from_min_max(
                bar_rect.min,
                egui::pos2(bar_rect.right() - w, bar_rect.bottom()),
            ),
        )
    };

    let mut tabs_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(tabs_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    egui::ScrollArea::horizontal()
        .id_salt("tabs-scroll")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .auto_shrink([false, false])
        .show(&mut tabs_ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for tab in &tabs {
                    tab_widget(app, ui, tab, &theme);
                }
            });
        });

    let mut right_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(right_rect)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    right_ui.spacing_mut().item_spacing.x = 2.0;
    let er = icons::icon_button(&mut right_ui, ci::ELLIPSIS, "Editor Actions", &theme);
    er.context_menu(|ui| {
        menu_style_accent(ui, &theme);
        if ui
            .add(egui::Button::new("Close All").frame_when_inactive(false))
            .clicked()
        {
            app.close_all();
            ui.close();
        }
        if ui
            .add(egui::Button::new("Close Saved").frame_when_inactive(false))
            .clicked()
        {
            app.close_saved();
            ui.close();
        }
        if ui
            .add(egui::Button::new("Toggle Diff vs Saved").frame_when_inactive(false))
            .clicked()
        {
            app.diff_mode = !app.diff_mode;
            ui.close();
        }
    });
    if icons::icon_button(
        &mut right_ui,
        ci::SPLIT_HORIZONTAL,
        "Split Editor Right (Ctrl+\\)",
        &theme,
    )
    .clicked()
    {
        app.split_open = !app.split_open;
    }
}

fn tab_widget(app: &mut OxideApp, ui: &mut egui::Ui, tab: &std::path::Path, theme: &Theme) {
    let name = tab
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let dirty = app
        .workspace
        .as_ref()
        .and_then(|w| w.get(tab))
        .is_some_and(|b| b.is_dirty());
    let selected = app.active.as_deref() == Some(tab);

    let name_g =
        ui.painter()
            .layout_no_wrap(name.clone(), egui::FontId::proportional(13.0), theme.text);
    let w = 10.0 + 16.0 + 6.0 + name_g.size().x + 6.0 + 20.0 + 4.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 35.0), egui::Sense::click());
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        0.0,
        if selected {
            theme.tab_active
        } else {
            theme.tab_bg
        },
    );
    painter.line_segment(
        [
            egui::pos2(rect.right() - 0.5, rect.top() + 6.0),
            egui::pos2(rect.right() - 0.5, rect.bottom() - 6.0),
        ],
        egui::Stroke::new(1.0, theme.border),
    );
    if selected {
        painter.rect_filled(
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 1.0)),
            0.0,
            theme.accent,
        );
    }
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 18.0, rect.center().y),
        egui::vec2(14.0, 14.0),
    );
    icons::draw_file_icon(&painter, icon_rect, &name);
    ui.painter().galley(
        egui::pos2(
            icon_rect.right() + 6.0,
            rect.center().y - name_g.size().y / 2.0,
        ),
        name_g,
        if selected { theme.text } else { theme.dim },
    );

    // Close slot: dirty dot, X on hover.
    let close_center = egui::pos2(rect.right() - 14.0, rect.center().y);
    if dirty && !resp.hovered() {
        icons::codicon(&painter, close_center, ci::CIRCLE_FILLED, 10.0, theme.dim);
    } else {
        icons::codicon(&painter, close_center, ci::CLOSE, 14.0, theme.dim);
    }

    if resp.middle_clicked() {
        app.close_tab(tab);
        return;
    }
    if resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            if pos.x > rect.right() - 28.0 {
                app.close_tab(tab);
                return;
            }
        }
        app.active = Some(tab.to_path_buf());
        app.editor_for = None;
        app.diff_mode = false;
    }
    let tab_path = tab.to_path_buf();
    resp.context_menu(|ui| {
        menu_style_accent(ui, theme);
        if ui.button("Close").clicked() {
            app.close_tab(&tab_path);
            ui.close();
        }
        if ui.button("Close Others").clicked() {
            app.close_others(&tab_path);
            ui.close();
        }
        if ui.button("Close to the Right").clicked() {
            app.close_to_right(&tab_path);
            ui.close();
        }
        if ui.button("Close Saved").clicked() {
            app.close_saved();
            ui.close();
        }
        if ui.button("Close All").clicked() {
            app.close_all();
            ui.close();
        }
        ui.separator();
        if ui.button("Copy Path").clicked() {
            ui.ctx().copy_text(tab_path.to_string_lossy().into_owned());
            ui.close();
        }
        let rel = app
            .workspace
            .as_ref()
            .map(|w| w.rel_path(&tab_path))
            .unwrap_or_default();
        if ui.button("Copy Relative Path").clicked() {
            ui.ctx().copy_text(rel);
            ui.close();
        }
    });
}

fn show_breadcrumbs(app: &mut OxideApp, ui: &mut egui::Ui) {
    let Some(active) = app.active.clone() else {
        return;
    };
    let theme = app.theme.clone();
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
        .active_symbols()
        .into_iter()
        .rfind(|(_, line)| *line <= app.cursor.0);

    let (bc_rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::hover());
    let bp = ui.painter_at(bc_rect);
    bp.rect_filled(bc_rect, 0.0, theme.bg);
    let crumb_fg = theme.dim;

    let mut x = bc_rect.left() + 10.0;
    let mid = bc_rect.center().y;
    macro_rules! crumb {
        ($text:expr) => {{
            let g = bp.layout_no_wrap(
                $text.to_string(),
                egui::FontId::proportional(12.0),
                crumb_fg,
            );
            let w = g.size().x;
            bp.galley(egui::pos2(x, mid - g.size().y / 2.0), g, crumb_fg);
            x += w + 4.0;
        }};
    }
    macro_rules! sep {
        () => {{
            icons::codicon(
                &bp,
                egui::pos2(x + 4.0, mid),
                ci::CHEVRON_RIGHT,
                12.0,
                crumb_fg,
            );
            x += 12.0;
        }};
    }

    let segments: Vec<&str> = rel.split(['/', '\\']).filter(|s| !s.is_empty()).collect();
    for (i, seg) in segments.iter().enumerate() {
        let is_last = i + 1 == segments.len();
        if is_last {
            let ir = egui::Rect::from_center_size(egui::pos2(x + 8.0, mid), egui::vec2(14.0, 14.0));
            icons::draw_file_icon(&bp, ir, &file_name);
            x += 20.0;
            crumb!(seg);
        } else {
            crumb!(seg);
            sep!();
        }
    }

    if let Some((label, line)) = sym {
        sep!();
        let glyph = if label.starts_with("fn ") {
            ci::SYMBOL_METHOD
        } else if label.starts_with("struct ") || label.starts_with("enum ") {
            ci::SYMBOL_STRUCT
        } else {
            ci::SYMBOL_VARIABLE
        };
        icons::codicon(&bp, egui::pos2(x + 6.0, mid), glyph, 13.0, crumb_fg);
        x += 16.0;
        let g = bp.layout_no_wrap(label, egui::FontId::proportional(12.0), crumb_fg);
        let srect = egui::Rect::from_min_size(
            egui::pos2(x, bc_rect.top()),
            egui::vec2(g.size().x + 8.0, bc_rect.height()),
        );
        let sresp = ui.interact(srect, ui.id().with("crumb-sym"), egui::Sense::click());
        bp.galley(egui::pos2(x + 4.0, mid - g.size().y / 2.0), g, crumb_fg);
        if sresp.clicked() {
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

    let editor_id = egui::Id::new(("source-editor", &active));
    if let Some(snippet) = app.pending_snippet.take() {
        let range = egui::TextEdit::load_state(ctx, editor_id).and_then(|s| s.cursor.char_range());
        let range = range.map(|r| r.as_sorted_char_range()).unwrap_or(0..0);
        let byte = |index| {
            app.editor_text
                .char_indices()
                .nth(index)
                .map(|(offset, _)| offset)
                .unwrap_or(app.editor_text.len())
        };
        let bytes = byte(range.start)..byte(range.end);
        match crate::snippets::expand(&snippet.body, &active, &app.editor_text[bytes.clone()]) {
            Ok(expansion) => {
                app.editor_text.replace_range(bytes, &expansion.text);
                if let Some(buffer) = app.workspace.as_mut().and_then(|w| w.get_mut(&active)) {
                    buffer.set_text(&app.editor_text);
                }
                app.pending_selection = Some((
                    active.clone(),
                    range.start + expansion.selection.start..range.start + expansion.selection.end,
                ));
            }
            Err(error) => app.status(format!("Cannot insert snippet: {error}")),
        }
    }

    let font_size = app.font_size;
    crate::editor_find::show(app, ui);

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

    if app.gitlens_blame {
        refresh_blame(app, &active);
    }

    // Gutter width tracks the digit count of the last line number.
    let line_count = app
        .workspace
        .as_ref()
        .and_then(|w| w.get(&active))
        .map(|b| b.line_count())
        .unwrap_or(1)
        .max(1);
    let digits = line_count.to_string().len();
    let gutter_w = ui.fonts(|f| {
        f.layout_no_wrap(
            "9".repeat(digits),
            egui::FontId::monospace(font_size),
            theme.gutter,
        )
        .size()
        .x + 26.0
    });
    let total_h = ui.available_height();

    let mut text_output = None;
    let mut gutter_rect = egui::Rect::NOTHING;
    let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, _wrap: f32| {
        app.highlight_cache
            .layout(ui, buf.as_str(), lang, &theme, font_size)
    };
    let mut scroll = egui::ScrollArea::vertical().id_salt(("editor-main-scroll", &active));
    if let Some((target, line)) = &app.pending_goto {
        if target == &active {
            let row_height =
                ui.fonts(|fonts| fonts.row_height(&egui::FontId::monospace(font_size)));
            scroll = scroll
                .vertical_scroll_offset((line.saturating_sub(4) as f32 * row_height).max(0.0));
        }
    }
    scroll.auto_shrink([false, false]).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            let (gr, _) = ui.allocate_exact_size(
                egui::vec2(gutter_w, total_h.max(600.0)),
                egui::Sense::hover(),
            );
            gutter_rect = gr;
            let output = egui::TextEdit::multiline(&mut app.editor_text)
                .id(editor_id)
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
        if let Some(ws) = app.workspace.as_mut() {
            if let Some(buf) = ws.get_mut(&active) {
                buf.set_text(&app.editor_text);
            }
        }
    }
    // Auto Save (File > Auto Save): write the buffer when editing loses focus.
    if app.auto_save && output.response.lost_focus() {
        app.save_active();
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
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(0),
                    egui::text::CCursor::new(len),
                )));
            state.store(ctx, editor_id);
            need_focus = true;
        }
    }
    if let Some((ref target, line)) = app.pending_goto.clone() {
        if *target == active {
            let col = app.pending_col.take().unwrap_or(1);
            if let Some(ws) = app.workspace.as_ref() {
                if let Some(buf) = ws.get(&active) {
                    let offset =
                        buf.offset_from_line_col(line.saturating_sub(1), col.saturating_sub(1));
                    if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
                        let cc = egui::text::CCursor::new(offset);
                        state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::one(cc)));
                        state.store(ctx, editor_id);
                        need_focus = !app.find.open;
                    }
                    app.cursor = (line, col);
                    let caret = output
                        .galley
                        .pos_from_cursor(egui::text::CCursor::new(offset));
                    ui.scroll_to_rect(
                        caret.translate(output.galley_pos.to_vec2()),
                        Some(egui::Align::Center),
                    );
                }
            }
            app.pending_goto = None;
        }
    }
    if let Some((target, selection)) = app.pending_selection.take() {
        if target == active {
            if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::two(
                        egui::text::CCursor::new(selection.start),
                        egui::text::CCursor::new(selection.end),
                    )));
                state.store(ctx, editor_id);
                need_focus = !app.find.open;
            }
        } else {
            app.pending_selection = Some((target, selection));
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
        .filter(|_| app.minimap_enabled)
        .map(|b| b.lines().iter().take(65).cloned().collect())
        .unwrap_or_default();
    paint_gutter(app, ui, &output, gutter_rect, cursor_line, &diag_lines);
    crate::editor_find::paint(&app.find, ui, &output, theme.find_match);
    if app.minimap_enabled {
        paint_minimap(app, ui, &output, &lines_snapshot, &active);
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
    let Some(active) = app.active.clone() else {
        return;
    };
    let lang = app.active_language();
    let text = app
        .workspace
        .as_ref()
        .and_then(|w| w.get(&active))
        .map(|b| b.text())
        .unwrap_or_else(|| app.editor_text.clone());
    let job = highlight::layout_job(&text, lang, &theme, app.font_size);
    let galley = ui.fonts(|f| f.layout_job(job));
    egui::ScrollArea::vertical()
        .id_salt("split-scroll")
        .show(ui, |ui| {
            egui::Frame::default()
                .inner_margin(egui::Margin::symmetric(12, 4))
                .show(ui, |ui| {
                    ui.label(galley);
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

fn paint_gutter(
    app: &mut OxideApp,
    ui: &mut egui::Ui,
    output: &egui::widgets::text_edit::TextEditOutput,
    gutter_rect: egui::Rect,
    cursor_line: usize,
    diag_lines: &[(usize, crate::diagnostics::Severity)],
) {
    let theme = app.theme.clone();
    let painter = ui.painter_at(output.text_clip_rect.union(gutter_rect));
    painter.rect_filled(gutter_rect, 0.0, theme.bg);

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
            // VS Code renders the current line as a 2px outline, not a fill.
            let hl = egui::Rect::from_min_max(
                egui::pos2(gutter_rect.right(), row_top),
                egui::pos2(clip.right() - minimap_w, row_top + row_h),
            );
            painter.rect_stroke(
                hl,
                0.0,
                egui::Stroke::new(2.0, theme.line_hi),
                egui::StrokeKind::Inside,
            );
        }

        // Diagnostic squiggle marker in the gutter.
        if let Some((_, sev)) = diag_lines.iter().find(|(l, _)| *l == line_no) {
            icons::codicon(
                &painter,
                egui::pos2(gutter_rect.left() + 8.0, row_center),
                match sev {
                    crate::diagnostics::Severity::Error => ci::ERROR,
                    crate::diagnostics::Severity::Warning => ci::WARNING,
                    crate::diagnostics::Severity::Info => ci::INFO,
                },
                12.0,
                match sev {
                    crate::diagnostics::Severity::Error => theme.error,
                    crate::diagnostics::Severity::Warning => theme.warning,
                    crate::diagnostics::Severity::Info => theme.accent,
                },
            );
        }

        // Git gutter bar from the cached index baseline.
        if let Some(mark) = app
            .gutter_marks
            .as_ref()
            .and_then(|m| m.iter().find(|(l, _)| *l == line_no).map(|(_, c)| *c))
        {
            let c = match mark {
                'A' => egui::Color32::from_rgb(0x58, 0x7c, 0x0c),
                'D' => theme.git_deleted,
                _ => theme.git_modified,
            };
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(gutter_rect.right() - 2.0, row_top),
                    egui::pos2(gutter_rect.right(), row_top + row_h),
                ),
                0.0,
                c,
            );
        }
        let num_color = if is_cursor {
            theme.gutter_active
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

        // GitLens-style inline blame on the active line (real `git blame`).
        if app.gitlens_blame && is_cursor {
            if let Some(bl) = app.blame.as_ref().and_then(|(_, _, b)| b.get(line_no - 1)) {
                let blame_text = format!(
                    "{}, {} • {}",
                    bl.author,
                    rel_time(bl.time),
                    truncate(&bl.summary, 48)
                );
                let x = origin.x + prow.row.size.x + 16.0;
                if x + 160.0 < clip.right() - minimap_w {
                    painter.text(
                        egui::pos2(x, row_center),
                        egui::Align2::LEFT_CENTER,
                        blame_text,
                        egui::FontId::new(11.0, egui::FontFamily::Proportional),
                        theme.git_ignored_fg,
                    );
                }
            }
        }
    }
}

/// Map a diff of (index baseline, buffer) to per-line gutter markers:
/// 'A' added (green), 'M' modified hunk (blue), 'D' deleted (red edge on
/// the nearest surviving line).
pub fn gutter_markers(old: &[String], new: &[String]) -> Vec<(usize, char)> {
    let rows = crate::diff::diff_lines(old, new);
    let mut marks: Vec<(usize, char)> = Vec::new();
    let mut i = 0;
    while i < rows.len() {
        match rows[i].tag {
            crate::diff::DiffTag::Insert => {
                let mut j = i;
                let mut had_del = false;
                while j > 0 && rows[j - 1].tag == crate::diff::DiffTag::Delete {
                    j -= 1;
                    had_del = true;
                }
                let line = rows[i].new_no.unwrap_or(0);
                if had_del {
                    marks.push((line, 'M'));
                } else {
                    marks.push((line, 'A'));
                }
                i += 1;
            }
            crate::diff::DiffTag::Delete => {
                // Standalone deletion (not followed by insert): red edge on
                // the next surviving new-side line (or last line).
                let is_hunk_del = rows
                    .get(i + 1)
                    .is_some_and(|r| r.tag == crate::diff::DiffTag::Insert)
                    || i > 0 && rows[i - 1].tag == crate::diff::DiffTag::Insert;
                if !is_hunk_del {
                    let nearest = rows
                        .iter()
                        .skip(i)
                        .find_map(|r| r.new_no)
                        .or_else(|| rows.iter().rev().find_map(|r| r.new_no))
                        .unwrap_or(0);
                    marks.push((nearest, 'D'));
                }
                i += 1;
            }
            crate::diff::DiffTag::Equal => i += 1,
        }
    }
    marks
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

    // Viewport slider box on top of the minimap.
    let vp_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left() + 2.0, rect.top() + 6.0),
        egui::vec2(w - 4.0, 62.0),
    );
    painter.rect_filled(
        vp_rect,
        2.0,
        egui::Color32::from_rgba_unmultiplied(100, 100, 120, 45),
    );

    let resp = ui.interact(rect, ui.id().with("minimap"), egui::Sense::click_and_drag());
    if resp.clicked() || resp.dragged() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let frac = ((pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
            let line = ((frac * lines.len() as f32) as usize + 1).clamp(1, lines.len());
            app.pending_goto = Some((active.to_path_buf(), line));
        }
    }
}

/// Side-by-side Git diff: HEAD->index (staged) or index->worktree.
/// `split('\n')` keeps a trailing empty row so newline-only changes show;
/// a missing side yields zero lines, not one empty line.
fn split_lines_preserve(text: Option<&str>) -> Vec<String> {
    match text {
        None => Vec::new(),
        Some(t) => t.split('\n').map(|l| l.to_string()).collect(),
    }
}

fn show_git_diff(app: &mut OxideApp, ui: &mut egui::Ui) {
    let Some(d) = app.git_diff.clone() else {
        app.git_diff = None;
        return;
    };
    let theme = app.theme.clone();
    let (old_label, new_label) = if d.staged {
        ("< HEAD", "index >")
    } else {
        ("< index", "disk >")
    };
    let rows = crate::diff::diff_lines(
        &split_lines_preserve(d.old_text.as_deref()),
        &split_lines_preserve(d.new_text.as_deref()),
    );
    let rel = app
        .git
        .snapshot
        .as_ref()
        .and_then(|s| d.path.strip_prefix(&s.root).ok())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| d.path.clone());
    egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(rel.to_string_lossy().into_owned())
                        .color(theme.text)
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new(format!(" {old_label} / {new_label}"))
                        .color(theme.dim)
                        .size(11.0),
                );
                if let Some(err) = &d.error {
                    ui.label(
                        egui::RichText::new(format!(" — {err}"))
                            .color(theme.error)
                            .size(11.0),
                    );
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close Diff").clicked() {
                        app.git_diff = None;
                        app.git.cancel_diff();
                    }
                });
            });
        });
    ui.separator();
    if d.error.is_some() {
        return;
    }
    let font = egui::FontId::monospace(app.font_size);
    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                for (col, side) in [(0, true), (1, false)] {
                    cols[col].vertical(|ui| {
                        for row in &rows {
                            let (text, bg, fg, no) = match (row.tag, side) {
                                (crate::diff::DiffTag::Delete, true) => (
                                    row.text.clone(),
                                    Some(theme.removed),
                                    theme.removed_text,
                                    row.old_no,
                                ),
                                (crate::diff::DiffTag::Equal, _) => (
                                    row.text.clone(),
                                    None,
                                    theme.text,
                                    if side { row.old_no } else { row.new_no },
                                ),
                                (crate::diff::DiffTag::Insert, false) => (
                                    row.text.clone(),
                                    Some(theme.added),
                                    theme.added_text,
                                    row.new_no,
                                ),
                                _ => (String::new(), Some(theme.sidebar), theme.dim, None),
                            };
                            let no = no
                                .map(|n| format!("{:>4} ", n + 1))
                                .unwrap_or_else(|| "     ".to_string());
                            paint_diff_row(ui, &font, &theme, &no, &text, bg, fg);
                        }
                    });
                }
            });
        });
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
                        .background_color(theme.badge_bg)
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
    theme: &Theme,
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

#[cfg(test)]
mod tests {
    use super::gutter_markers;

    fn lines(s: &str) -> Vec<String> {
        if s.is_empty() {
            vec![]
        } else {
            s.split('\n').map(|l| l.to_string()).collect()
        }
    }

    #[test]
    fn markers_insert_delete_modify() {
        // pure insert
        let m = gutter_markers(&lines("a\nb\n"), &lines("a\nx\nb\n"));
        assert!(m.contains(&(1, 'A')));
        // pure modify (delete+insert same hunk)
        let m = gutter_markers(&lines("a\nb\n"), &lines("a\nB\n"));
        assert!(m.contains(&(1, 'M')) && !m.iter().any(|(_, c)| *c == 'A'));
        // pure delete -> red edge on nearest surviving line
        let m = gutter_markers(&lines("a\nb\nc\n"), &lines("a\nc\n"));
        assert!(m.iter().any(|(_, c)| *c == 'D'));
    }
}
