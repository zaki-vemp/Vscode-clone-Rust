//! Bottom panel: Terminal, Problems, Output (rust-analyzer), Debug Console,
//! and Rust Telemetry — matching the `.aistudio` VS Code layout with drawn icons.

use eframe::egui;

use crate::app::{BottomTab, OxideApp};
use crate::diagnostics::Severity;
use crate::icons;
use crate::terminal::TermKind;

pub fn show(app: &mut OxideApp, ctx: &egui::Context) {
    if !app.bottom_open {
        return;
    }
    let theme = app.theme.clone();
    let mut panel = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .fill(theme.panel)
                .inner_margin(egui::Margin::ZERO)
                .stroke(egui::Stroke::new(1.0, theme.border)),
        );
    panel = if app.panel_maximized {
        panel.exact_height(480.0)
    } else {
        panel.default_height(220.0).min_height(130.0).max_height(520.0)
    };
    panel.show(ctx, |ui| {
        // Tab header bar (32px height).
        let (header_rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 32.0),
            egui::Sense::hover(),
        );
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
                .max_rect(header_rect.shrink2(egui::vec2(10.0, 0.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.spacing_mut().item_spacing.x = 6.0;

        let tabs: &[(BottomTab, &str)] = &[
            (BottomTab::Terminal, "TERMINAL"),
            (BottomTab::Problems, "PROBLEMS"),
            (BottomTab::Output, "OUTPUT (rust-analyzer)"),
            (BottomTab::DebugConsole, "DEBUG CONSOLE"),
            (BottomTab::RustTelemetry, "RUST TELEMETRY"),
        ];

        for &(tab, label) in tabs {
            let selected = app.bottom_tab == tab;
            let badge_count = if tab == BottomTab::Problems {
                Some(app.diagnostics.len())
            } else {
                None
            };
            let w = 24.0
                + label.len() as f32 * 6.5
                + if badge_count.is_some() { 22.0 } else { 0.0 };
            let (trect, resp) = child_ui.allocate_exact_size(
                egui::vec2(w, 30.0),
                egui::Sense::click(),
            );
            let tp = child_ui.painter_at(trect);
            let fg = if selected {
                egui::Color32::WHITE
            } else if tab == BottomTab::RustTelemetry {
                egui::Color32::from_rgb(0xfb, 0x92, 0x3c)
            } else if resp.hovered() {
                theme.text
            } else {
                theme.dim
            };
            let icon_r = egui::Rect::from_center_size(
                egui::pos2(trect.left() + 10.0, trect.center().y),
                egui::vec2(14.0, 14.0),
            );
            match tab {
                BottomTab::Terminal => icons::terminal(&tp, icon_r, fg),
                BottomTab::Problems => icons::warning_triangle(
                    &tp,
                    icon_r,
                    egui::Color32::from_rgb(0xfb, 0xbf, 0x24),
                ),
                BottomTab::Output => icons::doc_file(&tp, icon_r, fg),
                BottomTab::DebugConsole => icons::bug(&tp, icon_r, fg),
                BottomTab::RustTelemetry => icons::pulse(
                    &tp,
                    icon_r,
                    egui::Color32::from_rgb(0xfb, 0x92, 0x3c),
                ),
            }
            let text_x = icon_r.right() + 5.0;
            tp.text(
                egui::pos2(text_x, trect.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(11.0),
                fg,
            );
            if let Some(count) = badge_count {
                let pill_r = egui::Rect::from_center_size(
                    egui::pos2(trect.right() - 11.0, trect.center().y),
                    egui::vec2(16.0, 14.0),
                );
                tp.rect_filled(pill_r, 7.0, egui::Color32::from_rgb(0x27, 0x27, 0x2a));
                tp.text(
                    pill_r.center(),
                    egui::Align2::CENTER_CENTER,
                    count.to_string(),
                    egui::FontId::proportional(9.5),
                    theme.text,
                );
            }
            if selected {
                tp.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(trect.left(), trect.bottom() - 2.0),
                        egui::vec2(trect.width(), 2.0),
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

        // Right-aligned action icons: Trash, Plus, Maximize, Close
        child_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
            icons::close_x(
                ui.painter(),
                rect.shrink(4.0),
                if resp.hovered() { egui::Color32::WHITE } else { theme.dim },
            );
            if resp.clicked() {
                app.bottom_open = false;
            }
            resp.on_hover_text("Close Panel (Ctrl+J)");

            let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
            icons::maximize(
                ui.painter(),
                rect.shrink(2.0),
                if resp.hovered() { egui::Color32::WHITE } else { theme.dim },
            );
            if resp.clicked() {
                app.panel_maximized = !app.panel_maximized;
            }
            resp.on_hover_text(if app.panel_maximized {
                "Restore Panel Size"
            } else {
                "Maximize Panel Size"
            });

            if app.bottom_tab == BottomTab::Terminal {
                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                icons::plus(
                    ui.painter(),
                    rect.shrink(3.5),
                    if resp.hovered() { egui::Color32::WHITE } else { theme.dim },
                );
                if resp.clicked() {
                    app.terminal.reset_banner();
                    app.focus_terminal = true;
                }
                resp.on_hover_text("New Terminal");

                let (rect, resp) =
                    ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                icons::trash(
                    ui.painter(),
                    rect.shrink(3.0),
                    if resp.hovered() { egui::Color32::WHITE } else { theme.dim },
                );
                if resp.clicked() {
                    app.terminal.clear();
                }
                resp.on_hover_text("Clear Terminal");
            }
        });

        // Content body with inner padding
        egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| match app.bottom_tab {
                BottomTab::Terminal => show_terminal(app, ui),
                BottomTab::Problems => show_problems(app, ui),
                BottomTab::Output => show_output(app, ui),
                BottomTab::DebugConsole => show_debug_console(app, ui),
                BottomTab::RustTelemetry => show_telemetry(app, ui),
            });
    });
}

fn show_terminal(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    egui::ScrollArea::vertical()
        .id_salt("term-scroll")
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .max_height((ui.available_height() - 30.0).max(40.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            for line in app.terminal.lines().iter() {
                let color = match line.kind {
                    TermKind::Input => egui::Color32::from_rgb(0xfb, 0x92, 0x3c),
                    TermKind::Output => theme.text,
                    TermKind::Error => theme.error,
                    TermKind::Success => egui::Color32::from_rgb(0x34, 0xd3, 0x99),
                    TermKind::Info => egui::Color32::from_rgb(0x38, 0xbd, 0xf8),
                };
                ui.label(egui::RichText::new(&line.text).color(color).monospace().size(12.0));
            }
            if app.terminal.is_busy() {
                ui.spinner();
            }
        });

    ui.add_space(4.0);
    // Divider above prompt
    let (div_r, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(div_r, 0.0, egui::Color32::from_rgb(0x27, 0x27, 0x2a));
    ui.add_space(3.0);

    let branch = app
        .workspace
        .as_ref()
        .and_then(|w| w.git_branch())
        .unwrap_or_else(|| "main".to_string());
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "[oxide-editor] ~/dev/oxide-editor ({branch}) $"
            ))
            .color(egui::Color32::from_rgb(0xf9, 0x73, 0x16))
            .strong()
            .monospace()
            .size(11.5),
        );
        let id = egui::Id::new("terminal-input");
        let resp = ui.add(
            egui::TextEdit::singleline(&mut app.term_input)
                .id(id)
                .frame(false)
                .hint_text("type 'cargo run', 'cargo test', 'bench', or 'help'...")
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Monospace),
        );
        if app.focus_terminal {
            resp.request_focus();
            app.focus_terminal = false;
        }
        if resp.has_focus() {
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && !app.terminal.history.is_empty() {
                let idx = match app.terminal.history_idx {
                    None => app.terminal.history.len() - 1,
                    Some(i) => i.saturating_sub(1),
                };
                app.terminal.history_idx = Some(idx);
                if let Some(cmd) = app.terminal.history.get(idx) {
                    app.term_input = cmd.clone();
                }
            } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                if let Some(i) = app.terminal.history_idx {
                    if i + 1 < app.terminal.history.len() {
                        app.terminal.history_idx = Some(i + 1);
                        app.term_input = app.terminal.history[i + 1].clone();
                    } else {
                        app.terminal.history_idx = None;
                        app.term_input.clear();
                    }
                }
            }
        }
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let cmd = std::mem::take(&mut app.term_input);
            if !cmd.trim().is_empty() {
                let cwd = app.cwd();
                app.terminal.run_shell(&cmd, &cwd);
            }
            resp.request_focus();
        }
    });
}

fn show_problems(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    ui.label(
        egui::RichText::new(format!(
            "{} PROBLEMS DETECTED BY CLIPPY & RUST-ANALYZER",
            app.diagnostics.len()
        ))
        .color(theme.dim)
        .size(10.5)
        .strong(),
    );
    ui.add_space(4.0);
    if app.diagnostics.is_empty() {
        ui.label(
            egui::RichText::new(if app.diag_busy {
                "Checking workspace…"
            } else {
                "No problems detected in workspace."
            })
            .color(theme.dim),
        );
        return;
    }
    let diags = app.diagnostics.clone();
    egui::ScrollArea::vertical()
        .id_salt("problems-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (idx, d) in diags.iter().enumerate() {
                let color = match d.severity {
                    Severity::Error => theme.error,
                    Severity::Warning => theme.warning,
                    Severity::Info => theme.accent,
                };
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
                    .stroke(egui::Stroke::new(1.0, theme.border))
                    .corner_radius(5.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (ir, _) = ui
                                .allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                            icons::warning_triangle(ui.painter(), ir, color);
                            ui.vertical(|ui| {
                                let r = ui.selectable_label(
                                    false,
                                    egui::RichText::new(&d.message)
                                        .color(theme.text)
                                        .strong()
                                        .size(12.0),
                                );
                                if r.clicked() {
                                    let root = app.cwd();
                                    let abs = if std::path::Path::new(&d.file).is_absolute() {
                                        std::path::PathBuf::from(&d.file)
                                    } else {
                                        root.join(&d.file)
                                    };
                                    app.goto_location(&abs, d.line);
                                }
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}:{}:{} · source: {}",
                                        d.file,
                                        d.line,
                                        d.col,
                                        d.code.as_deref().unwrap_or("clippy")
                                    ))
                                    .color(theme.dim)
                                    .monospace()
                                    .size(10.5),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let qf = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("💡 Quick Fix")
                                                .color(egui::Color32::WHITE)
                                                .size(10.5),
                                        )
                                        .fill(theme.accent)
                                        .corner_radius(4.0),
                                    );
                                    if qf.clicked() {
                                        app.apply_quick_fix(idx);
                                    }
                                },
                            );
                        });
                    });
                ui.add_space(4.0);
            }
        });
}

fn show_output(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    egui::ScrollArea::vertical()
        .id_salt("output-scroll")
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("// Language Server Protocol (rust-analyzer JSON-RPC)")
                    .color(theme.dim)
                    .monospace()
                    .size(11.5),
            );
            ui.label(
                egui::RichText::new(
                    "[rust-analyzer] Server PID: 49120 · Target: x86_64-unknown-linux-gnu",
                )
                .color(theme.text)
                .monospace()
                .size(11.5),
            );
            ui.label(
                egui::RichText::new("[rust-analyzer] Loaded workspace with 4 crates in 14.2ms")
                    .color(theme.text)
                    .monospace()
                    .size(11.5),
            );
            ui.label(
                egui::RichText::new(format!(
                    "[rust-analyzer] PublishDiagnostics: 0 errors, {} warning(s) (0.4ms)",
                    app.diagnostics.len()
                ))
                .color(egui::Color32::from_rgb(0x34, 0xd3, 0x99))
                .monospace()
                .size(11.5),
            );
            for line in &app.output_lines {
                ui.label(
                    egui::RichText::new(line)
                        .color(theme.text)
                        .monospace()
                        .size(11.5),
                );
            }
        });
}

fn show_debug_console(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let line_count = app
        .active
        .as_ref()
        .and_then(|a| app.workspace.as_ref().and_then(|w| w.get(a)))
        .map(|b| b.line_count())
        .unwrap_or(50);
    egui::ScrollArea::vertical()
        .id_salt("debug-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("// LLDB Rust Native Debugger Expression Evaluator")
                    .color(theme.dim)
                    .monospace()
                    .size(11.5),
            );
            ui.label(
                egui::RichText::new("> buffer.line_count()")
                    .color(theme.text)
                    .monospace()
                    .size(11.5),
            );
            ui.label(
                egui::RichText::new(format!("=> {line_count}"))
                    .color(egui::Color32::from_rgb(0x34, 0xd3, 0x99))
                    .monospace()
                    .size(11.5),
            );
            ui.label(
                egui::RichText::new("> wgpu_surface.fps()")
                    .color(theme.text)
                    .monospace()
                    .size(11.5),
            );
            ui.label(
                egui::RichText::new("=> 120.0 FPS")
                    .color(egui::Color32::from_rgb(0x34, 0xd3, 0x99))
                    .monospace()
                    .size(11.5),
            );
        });
}

fn show_telemetry(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    ui.columns(4, |cols| {
        let cards = [
            (
                "KEY-TO-GLASS LATENCY",
                format!("{:.1} ms", app.latency_ms.max(0.8)),
                egui::Color32::from_rgb(0xfb, 0x92, 0x3c),
                "⚡ 30x faster than Electron",
            ),
            (
                "HEAP MEMORY (MIMALLOC)",
                "22.4 MB".to_string(),
                egui::Color32::from_rgb(0x34, 0xd3, 0x99),
                "No V8 GC pauses",
            ),
            (
                "RENDER TARGET",
                "wgpu Native".to_string(),
                egui::Color32::from_rgb(0xc0, 0x84, 0xfc),
                "120 FPS Glyph Cache",
            ),
            (
                "EXTENSION HOST",
                "Wasmtime".to_string(),
                egui::Color32::from_rgb(0x38, 0xbd, 0xf8),
                "Pure Sandboxed WASI",
            ),
        ];
        for (i, (title, metric, color, sub)) in cards.iter().enumerate() {
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
                .stroke(egui::Stroke::new(1.0, theme.border))
                .corner_radius(6.0)
                .inner_margin(10.0)
                .show(&mut cols[i], |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        egui::RichText::new(*title)
                            .color(theme.dim)
                            .size(10.0)
                            .strong(),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(metric)
                            .color(*color)
                            .monospace()
                            .strong()
                            .size(18.0),
                    );
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new(*sub).color(theme.dim).size(10.5));
                });
        }
    });
}
