//! Left activity rail + side bar: Explorer (tree + outline), Search,
//! Source Control (Git), Run & Debug, Extensions Marketplace, Rust Profiler,
//! Design Mockups, and Settings — matching the `.aistudio` VS Code layout.

use eframe::egui;

use crate::app::{Activity, ExtensionFilter, OxideApp};
use crate::highlight;
use crate::icons;
use crate::languages::Language;
use crate::project::FileEntry;
use crate::theme::ThemeKind;

pub fn show(app: &mut OxideApp, ctx: &egui::Context) {
    let theme = app.theme.clone();

    // --- Activity rail (48px), drawn codicon-style icons ---
    egui::SidePanel::left("activity_rail")
        .exact_width(48.0)
        .resizable(false)
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .fill(theme.activity)
                .inner_margin(egui::Margin::ZERO)
                .stroke(egui::Stroke::new(1.0, theme.border)),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
            let modified = app
                .workspace
                .as_ref()
                .map(|w| {
                    app.tabs
                        .iter()
                        .filter(|t| w.get(t).is_some_and(|b| b.is_dirty()))
                        .count()
                })
                .unwrap_or(0)
                .max(app.demo_modified.len());
            let git_badge = if modified > 0 {
                modified.to_string()
            } else {
                String::new()
            };

            ui.vertical(|ui| {
                ui.add_space(4.0);
                rail_icon(
                    ui,
                    &theme,
                    Activity::Explorer,
                    app,
                    "",
                    None,
                    "Explorer (Ctrl+Shift+E)",
                    icons::files,
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Search,
                    app,
                    "",
                    None,
                    "Search (Ctrl+Shift+F)",
                    icons::search,
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Git,
                    app,
                    &git_badge,
                    None,
                    "Source Control: Git (Ctrl+Shift+G)",
                    icons::git,
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Debug,
                    app,
                    "",
                    None,
                    "Run and Debug (Ctrl+Shift+D)",
                    icons::play_circle,
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Extensions,
                    app,
                    "Rust",
                    None,
                    "Extensions: Marketplace (Ctrl+Shift+X)",
                    icons::extensions,
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Profiler,
                    app,
                    "",
                    Some(egui::Color32::from_rgb(0xfb, 0x92, 0x3c)),
                    "Rust Performance Engine & Latency Profiler",
                    icons::pulse,
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Mockups,
                    app,
                    "2",
                    Some(egui::Color32::from_rgb(0xfc, 0xd3, 0x4d)),
                    "Design Mockups & Architecture Visuals",
                    icons::image_icon,
                );

                // Bottom cluster: account + settings gear
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(6.0);
                    let (rect, resp) =
                        ui.allocate_exact_size(egui::vec2(48.0, 42.0), egui::Sense::click());
                    let active = app.activity == Activity::Settings;
                    let c = if active || resp.hovered() {
                        egui::Color32::WHITE
                    } else {
                        theme.dim
                    };
                    icons::gear(ui.painter(), rect.shrink2(egui::vec2(15.0, 12.0)), c);
                    if resp.clicked() {
                        app.activity = Activity::Settings;
                        app.sidebar_open = true;
                    }
                    resp.on_hover_text("Manage Settings and Themes");

                    let (rect, resp) =
                        ui.allocate_exact_size(egui::vec2(48.0, 42.0), egui::Sense::click());
                    let c = if resp.hovered() {
                        egui::Color32::WHITE
                    } else {
                        theme.dim
                    };
                    icons::account(ui.painter(), rect.shrink2(egui::vec2(15.0, 12.0)), c);
                    if resp.clicked() {
                        app.profiler_open = true;
                    }
                    resp.on_hover_text("Rust Cloud Sync (Logged in as Oxide Developer)");
                });
            });
        });

    if !app.sidebar_open {
        return;
    }

    egui::SidePanel::left("side_bar")
        .default_width(256.0)
        .min_width(210.0)
        .max_width(420.0)
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .fill(theme.sidebar)
                .inner_margin(egui::Margin::ZERO)
                .stroke(egui::Stroke::new(1.0, theme.border)),
        )
        .show(ctx, |ui| match app.activity {
            Activity::Explorer => show_explorer(app, ui),
            Activity::Search => show_search(app, ui),
            Activity::Git => show_git(app, ui),
            Activity::Debug => show_debug(app, ui),
            Activity::Extensions => show_extensions(app, ui),
            Activity::Profiler => show_profiler_sidebar(app, ui),
            Activity::Mockups => show_mockups_sidebar(app, ctx, ui),
            Activity::Settings => show_settings(app, ui),
        });
}

#[allow(clippy::too_many_arguments)]
fn rail_icon(
    ui: &mut egui::Ui,
    theme: &crate::theme::Theme,
    tab: Activity,
    app: &mut OxideApp,
    badge: &str,
    tint: Option<egui::Color32>,
    tip: &str,
    draw: impl FnOnce(&egui::Painter, egui::Rect, egui::Color32),
) {
    let active = app.sidebar_open && app.activity == tab;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(48.0, 44.0), egui::Sense::click());
    if resp.hovered() && !active {
        ui.painter().rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
        );
    }
    if active {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.left(), rect.top() + 4.0),
                egui::vec2(2.5, rect.height() - 8.0),
            ),
            1.0,
            theme.accent,
        );
    }
    let color = if active {
        tint.unwrap_or(egui::Color32::WHITE)
    } else if resp.hovered() {
        tint.unwrap_or(theme.text)
    } else {
        tint.unwrap_or(theme.dim)
    };
    draw(ui.painter(), rect.shrink2(egui::vec2(13.0, 11.0)), color);
    if !badge.is_empty() {
        icons::text_badge(
            ui.painter(),
            rect,
            badge,
            theme.accent,
            egui::Color32::WHITE,
        );
    }
    if resp.clicked() {
        if app.activity == tab && app.sidebar_open {
            app.sidebar_open = false;
        } else {
            app.activity = tab;
            app.sidebar_open = true;
        }
    }
    resp.on_hover_text(tip);
}

fn sidebar_top_bar(
    ui: &mut egui::Ui,
    theme: &crate::theme::Theme,
    title: &str,
    right_actions: impl FnOnce(&mut egui::Ui),
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 36.0),
        egui::Sense::hover(),
    );
    ui.painter().line_segment(
        [
            egui::pos2(rect.left(), rect.bottom() - 0.5),
            egui::pos2(rect.right(), rect.bottom() - 0.5),
        ],
        egui::Stroke::new(1.0, theme.border),
    );
    let mut bar = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(14.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    bar.label(
        egui::RichText::new(title)
            .color(theme.dim)
            .size(11.0)
            .strong(),
    );
    bar.with_layout(egui::Layout::right_to_left(egui::Align::Center), right_actions);
}

fn icon_button(
    ui: &mut egui::Ui,
    tip: &str,
    draw: impl FnOnce(&egui::Painter, egui::Rect, egui::Color32),
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
    let color = if resp.hovered() {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_rgb(0xa1, 0xa1, 0xaa)
    };
    draw(ui.painter(), rect.shrink(3.5), color);
    let clicked = resp.clicked();
    resp.on_hover_text(tip);
    clicked
}

// ---------- Explorer ----------

fn show_explorer(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "EXPLORER", |ui| {
        if icon_button(ui, "Refresh Workspace (cargo check)", icons::refresh) {
            if let Some(ws) = app.workspace.as_mut() {
                ws.refresh();
            }
            app.run_cargo("check");
        }
        if icon_button(ui, "New File…", icons::file_plus) {
            app.new_file_name.clear();
            app.show_new_file = true;
        }
    });

    egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(0, 6))
        .show(ui, |ui| {
            if app.workspace.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("No folder open.").color(theme.dim));
                    if ui.button("Open Folder…").clicked() {
                        app.open_folder_input = app.cwd().to_string_lossy().into_owned();
                        app.show_open_folder = true;
                    }
                });
                return;
            }

            // Section header: v OXIDE-EDITOR (RUST WORKSPACE)
            egui::Frame::default()
                .inner_margin(egui::Margin::symmetric(10, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (r, _) =
                            ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                        icons::chevron(ui.painter(), r, theme.dim, true);
                        ui.label(
                            egui::RichText::new("OXIDE-EDITOR (RUST WORKSPACE)")
                                .color(theme.dim)
                                .strong()
                                .size(10.5),
                        );
                    });
                });

            let tree = app
                .workspace
                .as_ref()
                .map(|w| w.tree.clone())
                .unwrap_or_default();
            let outline_h = 125.0;
            egui::ScrollArea::vertical()
                .id_salt("tree-scroll")
                .max_height((ui.available_height() - outline_h).max(120.0))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    show_entries(app, ui, &tree, 0);
                });

            show_outline(app, ui);
        });
}

fn show_entries(app: &mut OxideApp, ui: &mut egui::Ui, entries: &[FileEntry], depth: usize) {
    for entry in entries {
        if entry.is_dir {
            folder_row(app, ui, entry, depth);
        } else {
            file_row(app, ui, entry, depth);
        }
    }
}

fn folder_row(app: &mut OxideApp, ui: &mut egui::Ui, entry: &FileEntry, depth: usize) {
    let theme = app.theme.clone();
    let id = ui.make_persistent_id(("dir_open", &entry.path));
    let mut is_open = ui.data_mut(|d| *d.get_persisted_mut_or(id, true));

    let indent = 12.0 + depth as f32 * 12.0;
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 24.0),
        egui::Sense::click(),
    );
    let painter = ui.painter_at(rect);
    if resp.hovered() {
        painter.rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
        );
    }
    let chev_r = egui::Rect::from_center_size(
        egui::pos2(rect.left() + indent + 5.0, rect.center().y),
        egui::vec2(12.0, 12.0),
    );
    icons::chevron(&painter, chev_r, theme.dim, is_open);
    painter.text(
        egui::pos2(chev_r.right() + 4.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &entry.name,
        egui::FontId::proportional(12.0),
        theme.text,
    );
    if entry.name == "src" {
        painter.text(
            egui::pos2(rect.right() - 12.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            "Rust source",
            egui::FontId::proportional(10.0),
            egui::Color32::from_rgb(0x71, 0x71, 0x7a),
        );
    }
    if resp.clicked() {
        is_open = !is_open;
        ui.data_mut(|d| d.insert_persisted(id, is_open));
    }
    resp.context_menu(|ui| {
        if ui.button("New File…").clicked() {
            app.new_file_name.clear();
            app.show_new_file = true;
            ui.close();
        }
        if ui.button("Delete Folder").clicked() {
            app.confirm_delete = Some(entry.path.clone());
            ui.close();
        }
    });

    if is_open {
        show_entries(app, ui, &entry.children, depth + 1);
    }
}

fn file_row(app: &mut OxideApp, ui: &mut egui::Ui, entry: &FileEntry, depth: usize) {
    let theme = app.theme.clone();
    let dirty = app
        .workspace
        .as_ref()
        .and_then(|w| w.get(&entry.path))
        .is_some_and(|b| b.is_dirty())
        || app.demo_modified.contains(&entry.name);
    let selected = app.active.as_ref() == Some(&entry.path);

    let indent = 14.0 + depth as f32 * 14.0;
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 24.0),
        egui::Sense::click(),
    );
    let painter = ui.painter_at(rect);
    if selected {
        painter.rect_filled(
            rect.shrink2(egui::vec2(6.0, 0.0)),
            3.0,
            egui::Color32::from_rgba_unmultiplied(0xea, 0x58, 0x0c, 52),
        );
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.left() + 6.0, rect.top() + 1.0),
                egui::vec2(2.0, rect.height() - 2.0),
            ),
            1.0,
            theme.accent,
        );
    } else if resp.hovered() {
        painter.rect_filled(
            rect.shrink2(egui::vec2(6.0, 0.0)),
            3.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
        );
    }

    let icon_r = egui::Rect::from_center_size(
        egui::pos2(rect.left() + indent + 8.0, rect.center().y),
        egui::vec2(14.0, 14.0),
    );
    icons::draw_file_icon(&painter, icon_r, &entry.name);

    let name_color = if selected {
        egui::Color32::from_rgb(0xfe, 0xd7, 0xaa)
    } else {
        theme.text
    };
    painter.text(
        egui::pos2(icon_r.right() + 6.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &entry.name,
        egui::FontId::proportional(12.0),
        name_color,
    );
    if dirty {
        painter.text(
            egui::pos2(rect.right() - 14.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            "M",
            egui::FontId::proportional(10.5),
            egui::Color32::from_rgb(0xfb, 0xbf, 0x24),
        );
    }
    if resp.clicked() {
        let p = entry.path.clone();
        app.open_file(&p);
    }
    resp.context_menu(|ui| {
        if ui.button("Delete File").clicked() {
            app.confirm_delete = Some(entry.path.clone());
            ui.close();
        }
    });
}

fn show_outline(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    ui.add_space(8.0);
    let (div_r, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(div_r, 0.0, theme.border);
    ui.add_space(4.0);

    egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(10, 2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                icons::chevron(ui.painter(), r, theme.dim, false);
                ui.label(
                    egui::RichText::new("OUTLINE (RUST-ANALYZER)")
                        .color(theme.dim)
                        .strong()
                        .size(10.5),
                );
            });
            ui.add_space(2.0);
            let mut syms: Vec<(String, usize)> = app
                .active
                .as_ref()
                .and_then(|a| {
                    let lang = Language::from_path(&a.to_string_lossy());
                    app.workspace
                        .as_ref()
                        .and_then(|w| w.get(a))
                        .map(|b| highlight::symbols(&b.text(), lang))
                })
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| {
                    vec![
                        ("struct EditorState".to_string(), 16),
                        ("fn main() -> Result<()>".to_string(), 24),
                        ("struct RopeBuffer".to_string(), 1),
                    ]
                });
            if syms.len() == 2 && syms.iter().any(|(s, _)| s.contains("EditorState")) {
                syms.push(("struct RopeBuffer".to_string(), 12));
            }
            for (label, line) in syms.iter().take(6) {
                let display = if label == "async fn main" || label == "fn main" {
                    "fn main() -> Result<()>"
                } else {
                    label.as_str()
                };
                let resp = ui.selectable_label(
                    false,
                    egui::RichText::new(format!("    {display}"))
                        .color(egui::Color32::from_rgb(0x71, 0x71, 0x7a))
                        .size(11.0),
                );
                if resp.clicked() {
                    if let Some(active) = app.active.clone() {
                        app.goto_location(&active, *line);
                    }
                }
            }
        });
}

// ---------- Search ----------

fn show_search(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "SEARCH", |_ui| {});
    egui::Frame::default()
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            let resp = ui.add(
                egui::TextEdit::singleline(&mut app.search_query)
                    .hint_text("Search code across files...")
                    .desired_width(f32::INFINITY),
            );
            if resp.changed() {
                if let Some(ws) = app.workspace.as_ref() {
                    app.search_hits = ws.search(app.search_query.trim(), 200);
                }
            }
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(if app.search_query.trim().is_empty() {
                    "Type a query to search".to_string()
                } else {
                    format!("{} result(s) found", app.search_hits.len())
                })
                .color(theme.dim)
                .small(),
            );
            ui.separator();
            let hits = app.search_hits.clone();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for hit in hits {
                        let rel = app
                            .workspace
                            .as_ref()
                            .map(|w| w.rel_path(&hit.path))
                            .unwrap_or_else(|| hit.path.to_string_lossy().into_owned());
                        ui.label(
                            egui::RichText::new(format!("{rel}:{}", hit.line))
                                .color(theme.accent)
                                .size(11.5)
                                .monospace(),
                        );
                        let resp = ui.selectable_label(
                            false,
                            egui::RichText::new(&hit.preview)
                                .color(theme.text)
                                .size(11.5)
                                .monospace(),
                        );
                        if resp.clicked() {
                            app.goto_location(&hit.path, hit.line);
                        }
                    }
                });
        });
}

// ---------- Source Control (Git) ----------

fn show_git(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "SOURCE CONTROL", |ui| {
        if icon_button(ui, "Refresh Git Status", icons::refresh) {
            let cwd = app.cwd();
            app.bottom_open = true;
            app.bottom_tab = crate::app::BottomTab::Terminal;
            app.terminal.run_shell("git status", &cwd);
        }
    });
    egui::Frame::default()
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut app.git_commit_msg)
                    .hint_text("Commit message (Ctrl+Enter to commit)...")
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(6.0);
            let commit_btn = ui.add_sized(
                [ui.available_width(), 28.0],
                egui::Button::new(
                    egui::RichText::new("Commit & Push")
                        .color(egui::Color32::WHITE)
                        .strong()
                        .size(12.0),
                )
                .fill(theme.accent)
                .corner_radius(4.0),
            );
            if commit_btn.clicked() {
                let cwd = app.cwd();
                app.bottom_open = true;
                app.bottom_tab = crate::app::BottomTab::Terminal;
                if app.git_commit_msg.trim().is_empty() {
                    app.terminal.run_shell("git status", &cwd);
                } else {
                    let msg = std::mem::take(&mut app.git_commit_msg);
                    app.terminal
                        .run_shell(&format!("git commit -m \"{}\"", msg.trim()), &cwd);
                }
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("STAGED CHANGES (1)")
                        .color(theme.dim)
                        .size(10.5)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("git add").color(theme.dim).size(10.0));
                });
            });
            git_file_item(app, ui, "Cargo.toml", "A", egui::Color32::from_rgb(0x34, 0xd3, 0x99));

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("CHANGES (2)")
                        .color(theme.dim)
                        .size(10.5)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .link(
                            egui::RichText::new("Stage All")
                                .color(egui::Color32::from_rgb(0xfb, 0x92, 0x3c))
                                .size(10.5),
                        )
                        .clicked()
                    {
                        let cwd = app.cwd();
                        app.bottom_open = true;
                        app.bottom_tab = crate::app::BottomTab::Terminal;
                        app.terminal.run_shell("git status", &cwd);
                    }
                });
            });
            git_file_item(app, ui, "src/main.rs", "M", egui::Color32::from_rgb(0xfb, 0xbf, 0x24));
            git_file_item(app, ui, "Cargo.toml", "M", egui::Color32::from_rgb(0xfb, 0xbf, 0x24));

            ui.add_space(14.0);
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(0x0c, 0x4a, 0x6e),
                ))
                .corner_radius(5.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("🌱 GitLens Rust Extension Active")
                            .color(egui::Color32::from_rgb(0x38, 0xbd, 0xf8))
                            .strong()
                            .size(11.0),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Zero-copy libgit2 bindings provide instant commit blame and line authorship directly in the editor.",
                        )
                        .color(theme.dim)
                        .size(10.0),
                    );
                });
        });
}

fn git_file_item(
    app: &mut OxideApp,
    ui: &mut egui::Ui,
    rel: &str,
    status: &str,
    status_color: egui::Color32,
) {
    let theme = app.theme.clone();
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 24.0),
        egui::Sense::click(),
    );
    let p = ui.painter_at(rect);
    if resp.hovered() {
        p.rect_filled(rect, 3.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10));
    }
    let ir = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        egui::vec2(13.0, 13.0),
    );
    icons::draw_file_icon(&p, ir, rel);
    p.text(
        egui::pos2(ir.right() + 6.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        rel,
        egui::FontId::proportional(11.5),
        theme.text,
    );
    p.text(
        egui::pos2(rect.right() - 6.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        status,
        egui::FontId::proportional(10.5),
        status_color,
    );
    if resp.clicked() {
        let target = app.cwd().join(rel);
        if target.exists() {
            app.open_file(&target);
        }
    }
}

// ---------- Run & Debug ----------

fn show_debug(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "RUN AND DEBUG", |_ui| {});
    egui::Frame::default()
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            let run_btn = ui.add_sized(
                [ui.available_width(), 28.0],
                egui::Button::new(
                    egui::RichText::new("▶ Launch Oxide Native (F5)")
                        .color(egui::Color32::WHITE)
                        .strong()
                        .size(12.0),
                )
                .fill(egui::Color32::from_rgb(0x05, 0x96, 0x69))
                .corner_radius(4.0),
            );
            if run_btn.clicked() {
                app.run_cargo("run");
            }
            ui.add_space(8.0);
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
                .stroke(egui::Stroke::new(1.0, theme.border))
                .corner_radius(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        egui::RichText::new("Debug Configuration:")
                            .color(theme.text)
                            .strong()
                            .monospace()
                            .size(11.0),
                    );
                    ui.label(
                        egui::RichText::new("target/debug/oxide-code")
                            .color(egui::Color32::from_rgb(0xfb, 0x92, 0x3c))
                            .monospace()
                            .size(11.0),
                    );
                    ui.label(
                        egui::RichText::new("Debugger: LLDB Rust Native v18")
                            .color(theme.dim)
                            .monospace()
                            .size(10.0),
                    );
                });
            ui.add_space(10.0);
            ui.separator();
            ui.label(
                egui::RichText::new("CALL STACK")
                    .color(theme.dim)
                    .strong()
                    .size(10.5),
            );
            ui.label(
                egui::RichText::new("oxide_core::main() at src/main.rs:24")
                    .color(theme.dim)
                    .monospace()
                    .size(11.0),
            );
            ui.label(
                egui::RichText::new("tokio::runtime::park()")
                    .color(theme.dim)
                    .monospace()
                    .size(11.0),
            );
            ui.add_space(8.0);
            ui.separator();
            ui.label(
                egui::RichText::new("BREAKPOINTS")
                    .color(theme.dim)
                    .strong()
                    .size(10.5),
            );
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(r.center(), 4.0, theme.error);
                ui.label(
                    egui::RichText::new("main.rs:24 (main loop entry)")
                        .color(theme.text)
                        .size(11.0),
                );
            });
        });
}

// ---------- Extensions Marketplace ----------

fn show_extensions(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "EXTENSIONS", |_ui| {});
    egui::Frame::default()
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.ext_search)
                    .hint_text("Search VS Code Extensions in Marketplace...")
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(6.0);

            // Segmented filter: All | Installed | Recommended
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
                .stroke(egui::Stroke::new(1.0, theme.border))
                .corner_radius(4.0)
                .inner_margin(2.0)
                .show(ui, |ui| {
                    ui.columns(3, |cols| {
                        for (i, (flt, label)) in [
                            (ExtensionFilter::All, "All"),
                            (ExtensionFilter::Installed, "Installed"),
                            (ExtensionFilter::Recommended, "Recommended"),
                        ]
                        .iter()
                        .enumerate()
                        {
                            let sel = app.ext_filter == *flt;
                            let fill = if sel {
                                egui::Color32::from_rgb(0x27, 0x27, 0x2a)
                            } else {
                                egui::Color32::TRANSPARENT
                            };
                            let fg = if sel { egui::Color32::WHITE } else { theme.dim };
                            if cols[i]
                                .add_sized(
                                    [cols[i].available_width(), 22.0],
                                    egui::Button::new(
                                        egui::RichText::new(*label).color(fg).strong().size(10.5),
                                    )
                                    .fill(fill)
                                    .stroke(egui::Stroke::NONE)
                                    .corner_radius(3.0),
                                )
                                .clicked()
                            {
                                app.ext_filter = *flt;
                            }
                        }
                    });
                });

            ui.add_space(8.0);
            let q = app.ext_search.to_lowercase();
            let filter = app.ext_filter;
            egui::ScrollArea::vertical()
                .id_salt("ext-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for i in 0..app.extensions.len() {
                        let ext = app.extensions[i].clone();
                        if !q.is_empty()
                            && !ext.display_name.to_lowercase().contains(&q)
                            && !ext.description.to_lowercase().contains(&q)
                        {
                            continue;
                        }
                        if filter == ExtensionFilter::Installed && !ext.installed {
                            continue;
                        }
                        if filter == ExtensionFilter::Recommended && ext.installed {
                            continue;
                        }

                        egui::Frame::default()
                            .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
                            .stroke(egui::Stroke::new(1.0, theme.border))
                            .corner_radius(6.0)
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    let (ir, _) = ui.allocate_exact_size(
                                        egui::vec2(32.0, 32.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(ir, 5.0, ext.icon_bg);
                                    draw_ext_symbol(ui.painter(), ir, ext.id);

                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            let max_chars = if ext.wasm_runtime { 18 } else { 24 };
                                            let title = if ext.display_name.chars().count()
                                                > max_chars
                                            {
                                                let mut s: String = ext
                                                    .display_name
                                                    .chars()
                                                    .take(max_chars)
                                                    .collect();
                                                s.push('…');
                                                s
                                            } else {
                                                ext.display_name.to_string()
                                            };
                                            ui.label(
                                                egui::RichText::new(title)
                                                    .color(egui::Color32::WHITE)
                                                    .strong()
                                                    .size(12.0),
                                            );
                                            if ext.wasm_runtime {
                                                let (wr, _) = ui.allocate_exact_size(
                                                    egui::vec2(36.0, 15.0),
                                                    egui::Sense::hover(),
                                                );
                                                ui.painter().rect_filled(
                                                    wr,
                                                    3.0,
                                                    egui::Color32::from_rgb(0x3b, 0x07, 0x64),
                                                );
                                                icons::rect_outline(
                                                    ui.painter(),
                                                    wr,
                                                    egui::Color32::from_rgb(0x6b, 0x21, 0xa8),
                                                    1.0,
                                                    3.0,
                                                );
                                                ui.painter().text(
                                                    wr.center(),
                                                    egui::Align2::CENTER_CENTER,
                                                    "WASM",
                                                    egui::FontId::monospace(8.5),
                                                    egui::Color32::from_rgb(0xd8, 0xb4, 0xfe),
                                                );
                                            }
                                        });
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} · {}",
                                                ext.publisher, ext.version
                                            ))
                                            .color(egui::Color32::from_rgb(0x71, 0x71, 0x7a))
                                            .size(10.0),
                                        );
                                    });
                                });

                                ui.add_space(4.0);
                                let desc = if ext.description.chars().count() > 74 {
                                    let mut d: String =
                                        ext.description.chars().take(74).collect();
                                    d.push_str("...");
                                    d
                                } else {
                                    ext.description.to_string()
                                };
                                ui.label(egui::RichText::new(desc).color(theme.dim).size(10.5));
                                ui.add_space(4.0);

                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "★ {:.1} · {}",
                                            ext.rating, ext.downloads
                                        ))
                                        .color(egui::Color32::from_rgb(0x71, 0x71, 0x7a))
                                        .size(10.0),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ext.installed {
                                                let un_btn = ui.add(
                                                    egui::Button::new(
                                                        egui::RichText::new("Uninstall")
                                                            .color(egui::Color32::from_rgb(
                                                                0xfc, 0xa5, 0xa5,
                                                            ))
                                                            .size(10.0),
                                                    )
                                                    .fill(egui::Color32::from_rgb(
                                                        0x45, 0x0a, 0x0a,
                                                    ))
                                                    .stroke(egui::Stroke::new(
                                                        1.0,
                                                        egui::Color32::from_rgb(0x7f, 0x1d, 0x1d),
                                                    ))
                                                    .corner_radius(3.0),
                                                );
                                                if un_btn.clicked() {
                                                    app.extensions[i].installed = false;
                                                    app.extensions[i].enabled = false;
                                                }

                                                let dis_label = if ext.enabled {
                                                    "Disable"
                                                } else {
                                                    "Enable"
                                                };
                                                let dis_btn = ui.add(
                                                    egui::Button::new(
                                                        egui::RichText::new(dis_label)
                                                            .color(theme.text)
                                                            .size(10.0),
                                                    )
                                                    .fill(egui::Color32::from_rgb(
                                                        0x27, 0x27, 0x2a,
                                                    ))
                                                    .stroke(egui::Stroke::NONE)
                                                    .corner_radius(3.0),
                                                );
                                                if dis_btn.clicked() {
                                                    app.extensions[i].enabled = !ext.enabled;
                                                    if ext.id == "rust-analyzer" {
                                                        app.inlay_hints = app.extensions[i].enabled;
                                                    } else if ext.id == "gitlens" {
                                                        app.gitlens_blame =
                                                            app.extensions[i].enabled;
                                                    } else if ext.id == "copilot" {
                                                        app.copilot_enabled =
                                                            app.extensions[i].enabled;
                                                    }
                                                }
                                            } else {
                                                let inst = ui.add(
                                                    egui::Button::new(
                                                        egui::RichText::new("Install (Rust Wasm)")
                                                            .color(egui::Color32::WHITE)
                                                            .size(10.0),
                                                    )
                                                    .fill(theme.accent)
                                                    .corner_radius(3.0),
                                                );
                                                if inst.clicked() {
                                                    app.extensions[i].installed = true;
                                                    app.extensions[i].enabled = true;
                                                }
                                            }
                                        },
                                    );
                                });
                            });
                        ui.add_space(6.0);
                    }
                });
        });
}

fn draw_ext_symbol(p: &egui::Painter, r: egui::Rect, id: &str) {
    match id {
        "rust-analyzer" => icons::rust_crab(p, r, egui::Color32::from_rgb(0x43, 0x14, 0x07)),
        "gitlens" => icons::git(p, r.shrink(6.0), egui::Color32::from_rgb(0xa7, 0xf3, 0xd0)),
        "copilot" => icons::sparkles(p, r.shrink(6.0), egui::Color32::from_rgb(0x93, 0xc5, 0xfd)),
        "prettier" => icons::zap(p, r.shrink(6.0), egui::Color32::from_rgb(0xfd, 0xe0, 0x47)),
        "tailwind" => icons::layers(p, r.shrink(6.0), egui::Color32::WHITE),
        _ => icons::extensions(p, r.shrink(6.0), egui::Color32::WHITE),
    }
}

// ---------- Rust Profiler Sidebar ----------

fn show_profiler_sidebar(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "RUST ENGINE PROFILER", |ui| {
        if icon_button(ui, "Run Microbenchmark", |p, r, _c| {
            icons::zap(p, r, egui::Color32::from_rgb(0xfb, 0x92, 0x3c))
        }) {
            let cwd = app.cwd();
            app.bottom_open = true;
            app.bottom_tab = crate::app::BottomTab::Terminal;
            app.terminal.run_shell("bench", &cwd);
        }
    });
    egui::Frame::default()
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(0xea, 0x58, 0x0c, 90),
                ))
                .corner_radius(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("⚡ Rust Native Performance Core")
                            .color(egui::Color32::from_rgb(0xfb, 0x92, 0x3c))
                            .strong()
                            .size(12.0),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(
                            "Zero JavaScript garbage collection. Custom Ropey text buffers paired with wgpu GPU glyph streaming for 120 FPS latency.",
                        )
                        .color(theme.dim)
                        .size(11.0),
                    );
                });

            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("TYPING LATENCY (LOWER IS BETTER)")
                    .color(theme.dim)
                    .strong()
                    .size(10.5),
            );
            ui.add_space(4.0);
            latency_bar(
                ui,
                "OxideCode (Pure Rust)",
                "0.8 ms",
                0.05,
                egui::Color32::from_rgb(0xf9, 0x73, 0x16),
            );
            latency_bar(
                ui,
                "Zed Editor (Rust)",
                "2.1 ms",
                0.12,
                egui::Color32::from_rgb(0x71, 0x71, 0x7a),
            );
            latency_bar(
                ui,
                "Standard VS Code (Electron)",
                "24.5 ms",
                1.0,
                egui::Color32::from_rgb(0xef, 0x44, 0x44),
            );

            ui.add_space(12.0);
            ui.label(
                egui::RichText::new("RAM CONSUMPTION (IDLE)")
                    .color(theme.dim)
                    .strong()
                    .size(10.5),
            );
            ui.add_space(4.0);
            ui.columns(2, |cols| {
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(0x06, 0x4e, 0x3b),
                    ))
                    .corner_radius(5.0)
                    .inner_margin(8.0)
                    .show(&mut cols[0], |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("22 MB")
                                    .color(egui::Color32::from_rgb(0x34, 0xd3, 0x99))
                                    .strong()
                                    .monospace()
                                    .size(15.0),
                            );
                            ui.label(
                                egui::RichText::new("OxideCode Rust")
                                    .color(theme.dim)
                                    .size(10.0),
                            );
                        });
                    });
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(0x7f, 0x1d, 0x1d),
                    ))
                    .corner_radius(5.0)
                    .inner_margin(8.0)
                    .show(&mut cols[1], |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("480 MB")
                                    .color(egui::Color32::from_rgb(0xf8, 0x71, 0x71))
                                    .strong()
                                    .monospace()
                                    .size(15.0),
                            );
                            ui.label(
                                egui::RichText::new("Electron VS Code")
                                    .color(theme.dim)
                                    .size(10.0),
                            );
                        });
                    });
            });

            ui.add_space(12.0);
            if ui
                .add_sized(
                    [ui.available_width(), 28.0],
                    egui::Button::new(
                        egui::RichText::new("⚡ Open Full Profiler Modal")
                            .color(egui::Color32::WHITE)
                            .strong()
                            .size(11.5),
                    )
                    .fill(theme.accent)
                    .corner_radius(4.0),
                )
                .clicked()
            {
                app.profiler_open = true;
            }
            ui.add_space(6.0);
            if ui
                .add_sized(
                    [ui.available_width(), 26.0],
                    egui::Button::new(
                        egui::RichText::new("Run Live Benchmark in Terminal")
                            .color(theme.text)
                            .size(11.0),
                    )
                    .fill(egui::Color32::from_rgb(0x27, 0x27, 0x2a))
                    .corner_radius(4.0),
                )
                .clicked()
            {
                let cwd = app.cwd();
                app.bottom_open = true;
                app.bottom_tab = crate::app::BottomTab::Terminal;
                app.terminal.run_shell("bench", &cwd);
            }
        });
}

fn latency_bar(
    ui: &mut egui::Ui,
    name: &str,
    val: &str,
    frac: f32,
    color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(name).color(egui::Color32::WHITE).size(11.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(val).color(color).monospace().size(11.0));
        });
    });
    let (bar_r, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 10.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(bar_r, 3.0, egui::Color32::from_rgb(0x27, 0x27, 0x2a));
    let fill_w = (bar_r.width() * frac).max(6.0);
    ui.painter().rect_filled(
        egui::Rect::from_min_size(bar_r.min, egui::vec2(fill_w, bar_r.height())),
        3.0,
        color,
    );
    ui.add_space(4.0);
}

// ---------- Design Mockups Sidebar ----------

fn show_mockups_sidebar(app: &mut OxideApp, ctx: &egui::Context, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    app.ensure_mockup_textures(ctx);
    sidebar_top_bar(ui, &theme, "DESIGN MOCKUPS", |_ui| {});
    egui::Frame::default()
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(
                    "High-resolution generated desktop mockups created for this repository:",
                )
                .color(theme.dim)
                .size(11.0),
            );
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .id_salt("mockups-side-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    mockup_card(
                        app,
                        ui,
                        0,
                        "1. Desktop IDE Shell",
                        "Native macOS dark theme, 0.9ms latency HUD, Rust syntax highlighting, and integrated Cargo terminal.",
                    );
                    ui.add_space(10.0);
                    mockup_card(
                        app,
                        ui,
                        1,
                        "2. Extensions & Profiler",
                        "Extensions Marketplace running rust-analyzer, GitLens, Copilot with live side-by-side benchmark graph.",
                    );
                });
        });
}

fn mockup_card(app: &mut OxideApp, ui: &mut egui::Ui, idx: usize, title: &str, desc: &str) {
    let theme = app.theme.clone();
    let frame_resp = egui::Frame::default()
        .fill(egui::Color32::from_rgb(0x18, 0x18, 0x1b))
        .stroke(egui::Stroke::new(1.0, theme.border))
        .corner_radius(6.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            let tex = if idx == 0 {
                app.mockup_tex_1.as_ref()
            } else {
                app.mockup_tex_2.as_ref()
            };
            if let Some(tex) = tex {
                let w = ui.available_width();
                let h = w * 0.56;
                let (img_r, _) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::hover());
                ui.painter().image(
                    tex.id(),
                    img_r,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .color(egui::Color32::WHITE)
                        .strong()
                        .size(12.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("↗").color(theme.dim).size(12.0));
                });
            });
            ui.label(egui::RichText::new(desc).color(theme.dim).size(10.5));
        });
    let resp = ui.interact(
        frame_resp.response.rect,
        ui.id().with(("mockup-card-click", idx)),
        egui::Sense::click(),
    );
    if resp.clicked() {
        app.selected_mockup = idx;
        app.mockup_modal_open = true;
    }
}

// ---------- Settings ----------

fn show_settings(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "SETTINGS", |_ui| {});
    egui::Frame::default()
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("COLOR THEME")
                            .color(theme.dim)
                            .strong()
                            .size(10.5),
                    );
                    for (name, kind) in ThemeKind::all() {
                        if ui.radio(app.theme_kind == *kind, *name).clicked() {
                            app.theme_kind = *kind;
                            app.save_config();
                        }
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label(
                        egui::RichText::new("EDITOR FEATURES")
                            .color(theme.dim)
                            .strong()
                            .size(10.5),
                    );
                    ui.checkbox(&mut app.inlay_hints, "rust-analyzer Inlay Hints");
                    ui.checkbox(&mut app.gitlens_blame, "GitLens Inline Blame");
                    ui.checkbox(&mut app.copilot_enabled, "GitHub Copilot Ghost Text");
                    ui.checkbox(&mut app.minimap_enabled, "Editor Minimap");
                    ui.checkbox(&mut app.format_on_save, "Format Rust on save (rustfmt)");
                    ui.checkbox(&mut app.check_on_save, "Typecheck on save (cargo check)");
                    ui.horizontal(|ui| {
                        ui.label("Font size");
                        if ui
                            .add(egui::Slider::new(&mut app.font_size, 9.0..=24.0))
                            .changed()
                        {
                            app.save_config();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Tab size");
                        if ui
                            .add(egui::Slider::new(&mut app.tab_size, 2..=8))
                            .changed()
                        {
                            app.save_config();
                        }
                    });
                });
        });
}
