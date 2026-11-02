//! Bottom panel: PROBLEMS / OUTPUT / DEBUG CONSOLE / TERMINAL tabs with
//! VS Code panel chrome (text tabs, maximize, close) and Dark Modern
//! terminal colors.

use std::path::{Path, PathBuf};

use eframe::egui;

use crate::app::{BottomTab, OxideApp};
use crate::diagnostics::Severity;
use crate::icons::{self, ci};
use crate::terminal::TermKind;
use crate::theme::Theme;

/// Dark Modern terminal ANSI-ish accents for command output.
const TERM_SUCCESS: egui::Color32 = egui::Color32::from_rgb(0x0d, 0xbc, 0x79);
const TERM_ERROR: egui::Color32 = egui::Color32::from_rgb(0xcd, 0x31, 0x31);
const TERM_INFO: egui::Color32 = egui::Color32::from_rgb(0x24, 0x72, 0xc8);
const PROB_ERROR: egui::Color32 = egui::Color32::from_rgb(0xf1, 0x4c, 0x4c);
const PROB_WARNING: egui::Color32 = egui::Color32::from_rgb(0xcc, 0xa7, 0x00);
const PROB_INFO: egui::Color32 = egui::Color32::from_rgb(0x37, 0x94, 0xff);

pub fn show(app: &mut OxideApp, ctx: &egui::Context) {
    if !app.bottom_open {
        return;
    }
    let theme = app.theme.clone();
    let max_h = (ctx.screen_rect().height() - 160.0).max(200.0);
    let mut panel = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .fill(theme.panel)
                .inner_margin(egui::Margin::ZERO)
                .stroke(egui::Stroke::new(1.0, theme.border)),
        );
    panel = if app.panel_maximized {
        panel.exact_height(max_h)
    } else {
        panel
            .default_height(220.0)
            .min_height(130.0)
            .max_height(max_h)
    };
    panel.show(ctx, |ui| {
        // 35px tab header.
        let (header_rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 35.0), egui::Sense::hover());
        let painter = ui.painter_at(header_rect);
        painter.rect_filled(header_rect, 0.0, theme.panel);
        painter.line_segment(
            [
                egui::pos2(header_rect.left(), header_rect.bottom() - 0.5),
                egui::pos2(header_rect.right(), header_rect.bottom() - 0.5),
            ],
            egui::Stroke::new(1.0, theme.border),
        );

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(header_rect.shrink2(egui::vec2(16.0, 0.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.spacing_mut().item_spacing.x = 10.0;

        let tabs: &[(BottomTab, &str)] = &[
            (BottomTab::Problems, "PROBLEMS"),
            (BottomTab::Output, "OUTPUT"),
            (BottomTab::DebugConsole, "DEBUG CONSOLE"),
            (BottomTab::Terminal, "TERMINAL"),
        ];

        for &(tab, label) in tabs {
            let selected = app.bottom_tab == tab;
            let problems = if tab == BottomTab::Problems {
                app.diagnostics.len()
            } else {
                0
            };
            let text_g = child_ui.painter().layout_no_wrap(
                label.to_string(),
                egui::FontId::proportional(11.0),
                theme.text,
            );
            let label_w = text_g.size().x;
            let w = label_w + if problems > 0 { 22.0 } else { 0.0 };
            let (trect, resp) =
                child_ui.allocate_exact_size(egui::vec2(w, 34.0), egui::Sense::click());
            let tp = child_ui.painter_at(trect);
            let fg = if selected || resp.hovered() {
                theme.text
            } else {
                theme.dim
            };
            tp.galley(
                egui::pos2(trect.left(), trect.center().y - text_g.size().y / 2.0),
                text_g,
                fg,
            );
            if problems > 0 {
                let pill = egui::Rect::from_center_size(
                    egui::pos2(trect.right() - 11.0, trect.center().y),
                    egui::vec2(18.0, 15.0),
                );
                tp.rect_filled(pill, 8.0, theme.badge_bg);
                tp.text(
                    pill.center(),
                    egui::Align2::CENTER_CENTER,
                    problems.to_string(),
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
            }
            if selected {
                tp.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(trect.left(), trect.bottom() - 8.0),
                        egui::vec2(label_w, 1.0),
                    ),
                    0.0,
                    theme.accent,
                );
            }
            if resp.clicked() {
                app.bottom_tab = tab;
            }
        }

        if app.bottom_tab == BottomTab::Terminal && app.terminal.is_busy() {
            child_ui.spinner();
        }

        child_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(2.0);
            if icons::icon_button(ui, ci::CLOSE, "Close Panel (Ctrl+J)", &theme).clicked() {
                app.bottom_open = false;
            }
            let (glyph, tip) = if app.panel_maximized {
                (ci::CHEVRON_DOWN, "Restore Panel Size")
            } else {
                (ci::CHEVRON_UP, "Maximize Panel Size")
            };
            if icons::icon_button(ui, glyph, tip, &theme).clicked() {
                app.panel_maximized = !app.panel_maximized;
            }
            if app.bottom_tab == BottomTab::Terminal {
                if icons::icon_button(ui, ci::TRASH, "Kill Terminal", &theme).clicked() {
                    if let Some(id) = app.pty.panes.target() {
                        app.pty.kill(id);
                    }
                }
                if icons::icon_button(ui, ci::SPLIT_HORIZONTAL, "Split Terminal", &theme).clicked()
                {
                    app.pty.split_session();
                }
                if icons::icon_button(ui, ci::ADD, "New Terminal (Ctrl+Shift+`)", &theme).clicked()
                {
                    app.pty.new_session();
                }
                ui.menu_button(
                    egui::RichText::new(ci::CHEVRON_DOWN.to_string())
                        .family(egui::FontFamily::Name("codicon".into()))
                        .size(14.0),
                    |ui| {
                        let names: Vec<String> =
                            app.pty.profiles.iter().map(|p| p.name.clone()).collect();
                        for (i, name) in names.iter().enumerate() {
                            if ui.button(name).clicked() {
                                app.pty.new_session_with(i);
                                ui.close();
                            }
                        }
                    },
                );
            }
        });

        // Content body, VS Code-like side padding.
        egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(8, 0))
            .show(ui, |ui| match app.bottom_tab {
                BottomTab::Problems => show_problems(app, ui),
                BottomTab::Output => show_output(app, ui),
                BottomTab::DebugConsole => {}
                BottomTab::Terminal => show_terminal(app, ui),
            });
    });
}

fn vscode_terminal_theme(theme: &Theme) -> egui_term::TerminalTheme {
    let hex = |c: egui::Color32| format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b());
    egui_term::TerminalTheme::new(Box::new(egui_term::ColorPalette {
        foreground: hex(theme.terminal_fg),
        background: hex(theme.terminal_bg),
        black: "#000000".into(),
        red: "#CD3131".into(),
        green: "#0DBC79".into(),
        yellow: "#E5E510".into(),
        blue: "#2472C8".into(),
        magenta: "#BC3FBC".into(),
        cyan: "#11A8CD".into(),
        white: "#E5E5E5".into(),
        bright_black: "#666666".into(),
        bright_red: "#F14C4C".into(),
        bright_green: "#23D18B".into(),
        bright_yellow: "#F5F543".into(),
        bright_blue: "#3B8EEA".into(),
        bright_magenta: "#D670D6".into(),
        bright_cyan: "#29B8DB".into(),
        bright_white: "#E5E5E5".into(),
        bright_foreground: None,
        dim_foreground: "#9D9D9D".into(),
        dim_black: "#000000".into(),
        dim_red: "#8C3434".into(),
        dim_green: "#0A7B54".into(),
        dim_yellow: "#9B9B0C".into(),
        dim_blue: "#1B5599".into(),
        dim_magenta: "#8A2F8A".into(),
        dim_cyan: "#0C7A99".into(),
        dim_white: "#A8A8A8".into(),
    }))
}

fn show_terminal(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    app.pty.ensure_session();
    if !app.pty_hook_done {
        app.pty_hook_done = true;
        // Test hooks: deterministic split / post-kill states for screenshots.
        if std::env::var_os("OXIDE_SPLIT").is_some() {
            app.pty.split_session();
        }
        if std::env::var_os("OXIDE_KILL").is_some() {
            if let Some(id) = app.pty.panes.target() {
                app.pty.kill(id);
            }
        }
    }
    app.pty.poll_events();
    if let Some(err) = app.pty.last_error.clone() {
        ui.colored_label(theme.error, err);
        if ui.button("Retry").clicked() {
            app.pty.last_error = None;
            app.pty.new_session();
        }
    }
    if app.pty.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(24.0);
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("New Terminal").color(egui::Color32::WHITE),
                    )
                    .fill(theme.accent),
                )
                .clicked()
            {
                app.pty.new_session();
            }
        });
        return;
    }

    // Focus persists per pane: resolve clicks against last frame's rects.
    let modal = app.modal_open();

    let visible: Vec<u64> = [app.pty.panes.active, app.pty.panes.split]
        .into_iter()
        .flatten()
        .collect();
    let list_w = if app.pty.sessions().len() > 1 {
        130.0
    } else {
        0.0
    };
    let pane_w = (ui.available_width() - list_w) / visible.len().max(1) as f32;
    let pane_h = ui.available_height();
    app.term_pane_rects.clear();

    ui.horizontal_top(|ui| {
        for (i, id) in visible.iter().enumerate() {
            if i > 0 {
                ui.separator();
            }
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(pane_w - 2.0, pane_h), egui::Sense::hover());
            app.term_pane_rects.push((*id, rect));
            let focused = app.pty.panes.focused == Some(*id) && !modal;
            if let Some(session) = app.pty.get(*id) {
                let mut pane = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(rect)
                        .id_salt(("pty-pane", *id)),
                );
                let v = egui_term::TerminalView::new(&mut pane, &mut session.backend)
                    .set_theme(vscode_terminal_theme(&theme))
                    .set_font(egui_term::TerminalFont::new(egui_term::FontSettings {
                        font_type: egui::FontId::monospace(app.font_size),
                    }))
                    .set_focus(focused)
                    .set_size(rect.size());
                pane.add(v);
            }
        }
        if list_w > 0.0 {
            ui.vertical(|ui| {
                ui.set_max_width(list_w);
                let ids: Vec<(u64, String, bool)> = app
                    .pty
                    .sessions()
                    .iter()
                    .map(|s| (s.id, s.title.clone(), s.exited))
                    .collect();
                for (id, title, exited) in ids {
                    let sel = app.pty.panes.active == Some(id) || app.pty.panes.split == Some(id);
                    let (rect, resp) = ui
                        .allocate_exact_size(egui::vec2(list_w - 8.0, 20.0), egui::Sense::click());
                    let p = ui.painter_at(rect);
                    if sel {
                        p.rect_filled(rect, 0.0, theme.list_active);
                    } else if resp.hovered() {
                        p.rect_filled(rect, 0.0, theme.list_hover);
                    }
                    icons::codicon(
                        &p,
                        egui::pos2(rect.left() + 9.0, rect.center().y),
                        ci::TERMINAL,
                        12.0,
                        theme.dim,
                    );
                    p.text(
                        egui::pos2(rect.left() + 18.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        if exited {
                            format!("{title} (exited)")
                        } else {
                            title
                        },
                        egui::FontId::proportional(11.0),
                        theme.text,
                    );
                    if resp.clicked() {
                        app.pty.select(id);
                    }
                }
            });
        }
    });
}

fn show_problems(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    if app.diagnostics.is_empty() {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(if app.diag_busy {
                "Checking workspace…"
            } else {
                "No problems have been detected in the workspace."
            })
            .color(theme.dim)
            .size(13.0),
        );
        return;
    }
    // Group by file, preserving first-seen order.
    let mut groups: Vec<(String, Vec<crate::diagnostics::Diagnostic>)> = Vec::new();
    for d in &app.diagnostics {
        match groups.iter_mut().find(|(f, _)| f == &d.file) {
            Some((_, v)) => v.push(d.clone()),
            None => groups.push((d.file.clone(), vec![d.clone()])),
        }
    }
    egui::ScrollArea::vertical()
        .id_salt("problems-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            for (file, diags) in &groups {
                file_problem_group(app, ui, file, diags, &theme);
            }
        });
}

fn file_problem_group(
    app: &mut OxideApp,
    ui: &mut egui::Ui,
    file: &str,
    diags: &[crate::diagnostics::Diagnostic],
    theme: &Theme,
) {
    let id = ui.make_persistent_id(("problems-group", file));
    let mut open = ui.data_mut(|d| *d.get_persisted_mut_or(id, true));
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
    let p = ui.painter_at(rect);
    if resp.hovered() {
        p.rect_filled(rect, 0.0, theme.list_hover);
    }
    icons::codicon(
        &p,
        egui::pos2(rect.left() + 8.0, rect.center().y),
        if open {
            ci::CHEVRON_DOWN
        } else {
            ci::CHEVRON_RIGHT
        },
        14.0,
        theme.text,
    );
    let name = Path::new(file)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| file.to_string());
    let dir = Path::new(file)
        .parent()
        .map(|d| d.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ir = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 24.0, rect.center().y),
        egui::vec2(14.0, 14.0),
    );
    icons::draw_file_icon(&p, ir, &name);
    let name_g = p.text(
        egui::pos2(ir.right() + 6.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &name,
        egui::FontId::proportional(13.0),
        theme.text,
    );
    p.text(
        egui::pos2(name_g.right() + 6.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        dir,
        egui::FontId::proportional(11.0),
        theme.dim,
    );
    let pill = egui::Rect::from_center_size(
        egui::pos2(rect.right() - 12.0, rect.center().y),
        egui::vec2(18.0, 14.0),
    );
    p.rect_filled(pill, 7.0, theme.badge_bg);
    p.text(
        pill.center(),
        egui::Align2::CENTER_CENTER,
        diags.len().to_string(),
        egui::FontId::proportional(10.0),
        egui::Color32::WHITE,
    );
    if resp.clicked() {
        open = !open;
        ui.data_mut(|d| d.insert_persisted(id, open));
    }
    if !open {
        return;
    }
    for d in diags {
        problem_row(app, ui, d, theme);
    }
}

fn problem_row(
    app: &mut OxideApp,
    ui: &mut egui::Ui,
    d: &crate::diagnostics::Diagnostic,
    theme: &Theme,
) {
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
    let p = ui.painter_at(rect);
    if resp.hovered() {
        p.rect_filled(rect, 0.0, theme.list_hover);
    }
    let (glyph, color) = match d.severity {
        Severity::Error => (ci::ERROR, PROB_ERROR),
        Severity::Warning => (ci::WARNING, PROB_WARNING),
        Severity::Info => (ci::INFO, PROB_INFO),
    };
    icons::codicon(
        &p,
        egui::pos2(rect.left() + 26.0, rect.center().y),
        glyph,
        14.0,
        color,
    );
    let msg_g = p.text(
        egui::pos2(rect.left() + 36.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &d.message,
        egui::FontId::proportional(13.0),
        theme.text,
    );
    p.text(
        egui::pos2(msg_g.right() + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!(
            " rustc({}) [Ln {}, Col {}]",
            d.code.as_deref().unwrap_or("rustc"),
            d.line,
            d.col
        ),
        egui::FontId::proportional(11.0),
        theme.dim,
    );
    if resp.clicked() {
        let root = app.cwd();
        let abs: PathBuf = if Path::new(&d.file).is_absolute() {
            PathBuf::from(&d.file)
        } else {
            root.join(&d.file)
        };
        app.goto_location(&abs, d.line);
    }
}

fn show_output(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    egui::ScrollArea::vertical()
        .id_salt("output-scroll")
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            for line in &app.output_lines {
                ui.label(
                    egui::RichText::new(line)
                        .color(theme.text)
                        .monospace()
                        .size(11.5),
                );
            }
            // Noninteractive task output (cargo check/test/clippy…).
            for line in app.terminal.lines().iter() {
                let color = match line.kind {
                    TermKind::Error => TERM_ERROR,
                    TermKind::Success => TERM_SUCCESS,
                    TermKind::Info => TERM_INFO,
                    _ => theme.text,
                };
                ui.label(
                    egui::RichText::new(&line.text)
                        .color(color)
                        .monospace()
                        .size(11.5),
                );
            }
            if app.terminal.is_busy() {
                ui.spinner();
            }
        });
}
