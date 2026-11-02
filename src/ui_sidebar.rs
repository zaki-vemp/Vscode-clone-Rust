//! Left activity rail + side bar: Explorer (lazy file tree + outline),
//! Search, Source Control (Git), Run & Debug, Extensions Marketplace,
//! and Settings — matching VS Code Dark Modern.

use std::path::Path;

use eframe::egui;

use crate::app::{menu_style_accent, Activity, ExplorerEdit, ExplorerEditKind, OxideApp};
use crate::icons::{self, ci};
use crate::project::FileEntry;
use crate::theme::{Theme, ThemeKind};

pub fn show(app: &mut OxideApp, ctx: &egui::Context) {
    let theme = app.theme.clone();

    // --- Activity rail (48px) ---
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
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            let modified = app
                .workspace
                .as_ref()
                .map(|w| {
                    app.tabs
                        .iter()
                        .filter(|t| w.get(t).is_some_and(|b| b.is_dirty()))
                        .count()
                })
                .unwrap_or(0);
            let changed = app
                .git
                .snapshot
                .as_ref()
                .map(|s| s.entries.len())
                .unwrap_or(0);
            let git_badge = if changed > 0 {
                changed.to_string()
            } else {
                String::new()
            };
            let _ = modified;

            ui.vertical(|ui| {
                rail_icon(
                    ui,
                    &theme,
                    Activity::Explorer,
                    app,
                    "",
                    ci::FILES,
                    "Explorer (Ctrl+Shift+E)",
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Search,
                    app,
                    "",
                    ci::SEARCH,
                    "Search (Ctrl+Shift+F)",
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Git,
                    app,
                    &git_badge,
                    ci::SOURCE_CONTROL,
                    "Source Control (Ctrl+Shift+G)",
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Debug,
                    app,
                    "",
                    ci::DEBUG_ALT,
                    "Run and Debug (Ctrl+Shift+D)",
                );
                rail_icon(
                    ui,
                    &theme,
                    Activity::Extensions,
                    app,
                    "",
                    ci::EXTENSIONS,
                    "Extensions (Ctrl+Shift+X)",
                );

                // Bottom cluster: account + manage gear
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(6.0);
                    rail_bottom(
                        ui,
                        ci::SETTINGS_GEAR,
                        "Manage",
                        |app| {
                            app.activity = Activity::Settings;
                            app.sidebar_open = true;
                        },
                        app,
                    );
                    rail_bottom(ui, ci::ACCOUNT, "Accounts", |_| {}, app);
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
            Activity::Settings => show_settings(app, ui),
        });
}

fn rail_icon(
    ui: &mut egui::Ui,
    theme: &Theme,
    tab: Activity,
    app: &mut OxideApp,
    badge: &str,
    glyph: char,
    tip: &str,
) {
    let active = app.sidebar_open && app.activity == tab;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::click());
    if active {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(rect.min, egui::vec2(2.0, rect.height())),
            0.0,
            theme.accent,
        );
    }
    let color = if active || resp.hovered() {
        theme.activity_fg
    } else {
        theme.activity_dim
    };
    icons::codicon(ui.painter(), rect.center(), glyph, 24.0, color);
    if !badge.is_empty() {
        let bc = egui::pos2(rect.right() - 9.0, rect.bottom() - 9.0);
        ui.painter().circle_filled(bc, 8.0, theme.accent);
        ui.painter().text(
            bc,
            egui::Align2::CENTER_CENTER,
            badge,
            egui::FontId::proportional(9.0),
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

fn rail_bottom(
    ui: &mut egui::Ui,
    glyph: char,
    tip: &str,
    on_click: impl FnOnce(&mut OxideApp),
    app: &mut OxideApp,
) {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::click());
    let color = if resp.hovered() {
        app.theme.activity_fg
    } else {
        app.theme.activity_dim
    };
    icons::codicon(ui.painter(), rect.center(), glyph, 24.0, color);
    if resp.clicked() {
        on_click(app);
    }
    resp.on_hover_text(tip);
}

/// View title row: 35px, 11px uppercase label, left padding 20px.
fn sidebar_top_bar(
    ui: &mut egui::Ui,
    theme: &Theme,
    title: &str,
    right_actions: impl FnOnce(&mut egui::Ui),
) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 35.0), egui::Sense::hover());
    let mut bar = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_max(
                egui::pos2(rect.left() + 20.0, rect.top()),
                rect.max,
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    bar.label(egui::RichText::new(title).color(theme.dim).size(11.0));
    bar.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.add_space(8.0);
        right_actions(ui);
    });
}

// ---------- Explorer ----------

fn show_explorer(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "EXPLORER", |_ui| {});

    if app.workspace.is_none() {
        egui::Frame::default()
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.add_space(12.0);
                ui.label(
                    egui::RichText::new("You have not yet opened a folder.")
                        .color(theme.dim)
                        .size(13.0),
                );
                ui.add_space(8.0);
                let btn = ui.add_sized(
                    [ui.available_width(), 28.0],
                    egui::Button::new(
                        egui::RichText::new("Open Folder")
                            .color(egui::Color32::WHITE)
                            .size(13.0),
                    )
                    .fill(theme.accent)
                    .corner_radius(2.0),
                );
                if btn.clicked() {
                    app.open_folder_input = app.cwd().to_string_lossy().into_owned();
                    app.show_open_folder = true;
                }
            });
        return;
    }

    let section_id = ui.make_persistent_id("explorer-section-open");
    let mut section_open = ui.data_mut(|d| *d.get_persisted_mut_or(section_id, true));
    let folder_name = app.workspace_name().to_uppercase();

    // 22px section header: chevron + FOLDER NAME, hover reveals 4 actions.
    let (hrect, hresp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
    let hp = ui.painter_at(hrect);
    let section_hover = hrect.contains(
        ui.ctx()
            .pointer_hover_pos()
            .unwrap_or(egui::pos2(-1.0, -1.0)),
    );
    icons::codicon(
        &hp,
        egui::pos2(hrect.left() + 10.0, hrect.center().y),
        if section_open {
            ci::CHEVRON_DOWN
        } else {
            ci::CHEVRON_RIGHT
        },
        14.0,
        theme.text,
    );
    hp.text(
        egui::pos2(hrect.left() + 20.0, hrect.center().y),
        egui::Align2::LEFT_CENTER,
        &folder_name,
        egui::FontId::new(11.0, egui::FontFamily::Name("bold".into())),
        theme.text,
    );

    // Hover actions as real widgets; being added after the header rect,
    // they win clicks that land on them.
    if section_hover {
        let mut actions = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(egui::Rect::from_min_max(
                    egui::pos2(hrect.right() - 92.0, hrect.top()),
                    hrect.max,
                ))
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        actions.spacing_mut().item_spacing.x = 0.0;
        if icons::icon_button(
            &mut actions,
            ci::COLLAPSE_ALL,
            "Collapse Folders in Explorer",
            &theme,
        )
        .clicked()
        {
            app.expanded_dirs.clear();
        }
        if icons::icon_button(&mut actions, ci::REFRESH, "Refresh Explorer", &theme).clicked() {
            if let Some(ws) = app.workspace.as_mut() {
                ws.refresh();
            }
        }
        if icons::icon_button(&mut actions, ci::NEW_FOLDER, "New Folder…", &theme).clicked() {
            start_edit(app, ExplorerEditKind::NewFolder);
        }
        if icons::icon_button(&mut actions, ci::NEW_FILE, "New File…", &theme).clicked() {
            start_edit(app, ExplorerEditKind::NewFile);
        }
    }
    if hresp.clicked() {
        section_open = !section_open;
        ui.data_mut(|d| d.insert_persisted(section_id, section_open));
    }

    if section_open {
        let root = app.cwd();
        let outline_h = 26.0;
        egui::ScrollArea::vertical()
            .id_salt("tree-scroll")
            .max_height((ui.available_height() - outline_h).max(120.0))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                render_dir(app, ui, &root, 0);
                // Empty-area context menu (rest of the tree space).
                let rest = ui.available_rect_before_wrap();
                if rest.height() > 4.0 {
                    let resp = ui.interact(
                        rest,
                        ui.id().with("explorer-empty-ctx"),
                        egui::Sense::click(),
                    );
                    explorer_context_menu(app, resp, None, &theme);
                }
            });
    }

    show_outline(app, ui);
}

fn start_edit(app: &mut OxideApp, kind: ExplorerEditKind) {
    let dir = app.explorer_target_dir();
    expand_to(app, &dir);
    app.explorer_edit = Some(ExplorerEdit {
        kind,
        target: dir,
        text: String::new(),
    });
}

/// Expand `dir` and its ancestors within the workspace root.
fn expand_to(app: &mut OxideApp, dir: &Path) {
    let root = app.cwd();
    let mut cur = Some(dir.to_path_buf());
    while let Some(d) = cur {
        if d.starts_with(&root) {
            app.expanded_dirs.insert(d.clone());
        }
        if d == root {
            break;
        }
        cur = d.parent().map(|p| p.to_path_buf());
    }
}

fn render_dir(app: &mut OxideApp, ui: &mut egui::Ui, dir: &Path, depth: usize) {
    // Inline "new item" edit row at the top of the target directory.
    if matches!(
        app.explorer_edit.as_ref().map(|e| (e.kind, &e.target)),
        Some((ExplorerEditKind::NewFile | ExplorerEditKind::NewFolder, t)) if t == dir
    ) {
        edit_row(app, ui, depth);
    }
    let entries = app
        .workspace
        .as_mut()
        .map(|ws| ws.children(dir))
        .unwrap_or_default();
    for entry in &entries {
        if matches!(
            app.explorer_edit.as_ref().map(|e| (e.kind, &e.target)),
            Some((ExplorerEditKind::Rename, t)) if t == &entry.path
        ) {
            edit_row(app, ui, depth);
            continue;
        }
        if entry.is_dir {
            folder_row(app, ui, entry, depth);
        } else {
            file_row(app, ui, entry, depth);
        }
    }
}

fn edit_row(app: &mut OxideApp, ui: &mut egui::Ui, depth: usize) {
    let indent = 8.0 + depth as f32 * 8.0;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::hover());
    let mut edit = app.explorer_edit.take().expect("edit row without edit");
    let inner = egui::Rect::from_min_max(
        egui::pos2(rect.left() + indent + 20.0, rect.top() + 1.0),
        egui::pos2(rect.right() - 4.0, rect.bottom() - 1.0),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let resp = child.add(
        egui::TextEdit::singleline(&mut edit.text)
            .desired_width(f32::INFINITY)
            .font(egui::FontId::proportional(13.0))
            .margin(egui::Margin::symmetric(4, 1)),
    );
    resp.request_focus();
    let commit = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let cancel = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape));
    if commit {
        app.explorer_edit = Some(edit);
        app.commit_explorer_edit();
    } else if cancel || (resp.lost_focus() && !ui.input(|i| i.key_pressed(egui::Key::Enter))) {
        app.explorer_edit = None;
    } else {
        app.explorer_edit = Some(edit);
    }
}

fn folder_row(app: &mut OxideApp, ui: &mut egui::Ui, entry: &FileEntry, depth: usize) {
    let theme = app.theme.clone();
    let is_open = app.expanded_dirs.contains(&entry.path);
    let indent = 8.0 + depth as f32 * 8.0;
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
    let painter = ui.painter_at(rect);
    if resp.hovered() {
        painter.rect_filled(rect, 0.0, theme.list_hover);
    }
    icons::codicon(
        &painter,
        egui::pos2(rect.left() + indent + 8.0, rect.center().y),
        if is_open {
            ci::CHEVRON_DOWN
        } else {
            ci::CHEVRON_RIGHT
        },
        14.0,
        theme.text,
    );
    icons::codicon(
        &painter,
        egui::pos2(rect.left() + indent + 24.0, rect.center().y),
        if is_open {
            ci::FOLDER_OPENED
        } else {
            ci::FOLDER
        },
        15.0,
        theme.dim,
    );
    painter.text(
        egui::pos2(rect.left() + indent + 34.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &entry.name,
        egui::FontId::proportional(13.0),
        theme.text,
    );
    if resp.clicked() {
        if is_open {
            app.expanded_dirs.remove(&entry.path);
        } else {
            app.expanded_dirs.insert(entry.path.clone());
        }
    }
    explorer_context_menu(app, resp, Some(entry), &theme);

    if is_open {
        render_dir(app, ui, &entry.path, depth + 1);
    }
}

fn file_row(app: &mut OxideApp, ui: &mut egui::Ui, entry: &FileEntry, depth: usize) {
    let theme = app.theme.clone();
    let selected = app.active.as_ref() == Some(&entry.path);
    let indent = 8.0 + depth as f32 * 8.0;
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
    let painter = ui.painter_at(rect);
    if selected {
        painter.rect_filled(rect, 0.0, theme.list_active);
    } else if resp.hovered() {
        painter.rect_filled(rect, 0.0, theme.list_hover);
    }

    let icon_r = egui::Rect::from_center_size(
        egui::pos2(rect.left() + indent + 24.0, rect.center().y),
        egui::vec2(14.0, 14.0),
    );
    icons::draw_file_icon(&painter, icon_r, &entry.name);
    // Explorer filename color reflects real git status.
    let name_color = app
        .git
        .snapshot
        .as_ref()
        .and_then(|s| s.entry(&entry.path))
        .map(|e| {
            if e.is_untracked() {
                theme.git_untracked_fg
            } else if e.worktree == 'D' || e.index == 'D' {
                theme.git_deleted_fg
            } else {
                theme.git_modified_fg
            }
        })
        .unwrap_or(theme.text);
    painter.text(
        egui::pos2(rect.left() + indent + 34.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &entry.name,
        egui::FontId::proportional(13.0),
        name_color,
    );
    if resp.clicked() {
        let p = entry.path.clone();
        app.open_file(&p);
    }
    explorer_context_menu(app, resp, Some(entry), &theme);
}

fn explorer_context_menu(
    app: &mut OxideApp,
    resp: egui::Response,
    entry: Option<&FileEntry>,
    theme: &Theme,
) {
    let entry = entry.cloned();
    resp.context_menu(|ui| {
        menu_style_accent(ui, theme);
        let base_dir = match &entry {
            Some(e) if e.is_dir => e.path.clone(),
            Some(e) => e
                .path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| app.cwd()),
            None => app.cwd(),
        };
        if menu_item(ui, "New File…", "").clicked() {
            expand_to(app, &base_dir);
            app.explorer_edit = Some(ExplorerEdit {
                kind: ExplorerEditKind::NewFile,
                target: base_dir.clone(),
                text: String::new(),
            });
            ui.close();
        }
        if menu_item(ui, "New Folder…", "").clicked() {
            expand_to(app, &base_dir);
            app.explorer_edit = Some(ExplorerEdit {
                kind: ExplorerEditKind::NewFolder,
                target: base_dir.clone(),
                text: String::new(),
            });
            ui.close();
        }
        if let Some(e) = &entry {
            ui.separator();
            if menu_item(ui, "Rename…", "F2").clicked() {
                let text = e.name.clone();
                app.explorer_edit = Some(ExplorerEdit {
                    kind: ExplorerEditKind::Rename,
                    target: e.path.clone(),
                    text,
                });
                ui.close();
            }
            if menu_item(ui, "Delete", "").clicked() {
                app.confirm_delete = Some(e.path.clone());
                ui.close();
            }
            ui.separator();
            if menu_item(ui, "Copy Path", "").clicked() {
                ui.ctx().copy_text(e.path.to_string_lossy().into_owned());
                ui.close();
            }
            let rel = app
                .workspace
                .as_ref()
                .map(|w| w.rel_path(&e.path))
                .unwrap_or_default();
            if menu_item(ui, "Copy Relative Path", "").clicked() {
                ui.ctx().copy_text(rel);
                ui.close();
            }
        }
    });
}

fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).size(13.0))
            .shortcut_text(egui::RichText::new(shortcut).size(12.0))
            .frame_when_inactive(false)
            .min_size(egui::vec2(ui.available_width(), 22.0)),
    )
}

fn show_outline(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let id = ui.make_persistent_id("outline-open");
    let mut open = ui.data_mut(|d| *d.get_persisted_mut_or(id, false));
    let (hrect, hresp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
    let hp = ui.painter_at(hrect);
    if hresp.hovered() {
        hp.rect_filled(hrect, 0.0, theme.list_hover);
    }
    icons::codicon(
        &hp,
        egui::pos2(hrect.left() + 10.0, hrect.center().y),
        if open {
            ci::CHEVRON_DOWN
        } else {
            ci::CHEVRON_RIGHT
        },
        14.0,
        theme.text,
    );
    hp.text(
        egui::pos2(hrect.left() + 20.0, hrect.center().y),
        egui::Align2::LEFT_CENTER,
        "OUTLINE",
        egui::FontId::new(11.0, egui::FontFamily::Name("bold".into())),
        theme.text,
    );
    if hresp.clicked() {
        open = !open;
        ui.data_mut(|d| d.insert_persisted(id, open));
    }
    if !open {
        return;
    }
    let syms = app.active_symbols();
    if syms.is_empty() {
        ui.label(
            egui::RichText::new("   No symbols found in document.")
                .color(theme.dim)
                .size(12.0),
        );
    }
    for (label, line) in syms {
        let glyph = if label.starts_with("fn ") {
            ci::SYMBOL_METHOD
        } else if label.starts_with("struct ") || label.starts_with("enum ") {
            ci::SYMBOL_STRUCT
        } else {
            ci::SYMBOL_VARIABLE
        };
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
        let p = ui.painter_at(rect);
        if resp.hovered() {
            p.rect_filled(rect, 0.0, theme.list_hover);
        }
        icons::codicon(
            &p,
            egui::pos2(rect.left() + 26.0, rect.center().y),
            glyph,
            14.0,
            theme.dim,
        );
        p.text(
            egui::pos2(rect.left() + 36.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &label,
            egui::FontId::proportional(13.0),
            theme.text,
        );
        if resp.clicked() {
            if let Some(active) = app.active.clone() {
                app.goto_location(&active, line);
            }
        }
    }
}

// ---------- Search ----------

fn show_search(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "SEARCH", |_ui| {});
    egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            // Query row with case/word/regex toggles inside the field.
            let q = ui.add(
                egui::TextEdit::singleline(&mut app.search_opts.query)
                    .hint_text("Search")
                    .desired_width(f32::INFINITY),
            );
            let mut fire = q.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            ui.horizontal(|ui| {
                for (flag, tip, on) in [
                    ("Aa", "Match Case", app.search_opts.case_sensitive),
                    ("ab|", "Match Whole Word", app.search_opts.whole_word),
                    (".*", "Use Regular Expression", app.search_opts.regex),
                ] {
                    let b = ui.add(
                        egui::Button::new(egui::RichText::new(flag).size(11.0))
                            .selected(on)
                            .frame(true),
                    );
                    if b.clicked() {
                        match tip {
                            "Match Case" => {
                                app.search_opts.case_sensitive = !app.search_opts.case_sensitive
                            }
                            "Match Whole Word" => {
                                app.search_opts.whole_word = !app.search_opts.whole_word
                            }
                            _ => app.search_opts.regex = !app.search_opts.regex,
                        }
                        fire = true;
                    }
                    b.on_hover_text(tip);
                }
            });
            ui.add(
                egui::TextEdit::singleline(&mut app.search_opts.include)
                    .hint_text("files to include (e.g. src/**,*.rs)")
                    .desired_width(f32::INFINITY),
            );
            ui.add(
                egui::TextEdit::singleline(&mut app.search_opts.exclude)
                    .hint_text("files to exclude")
                    .desired_width(f32::INFINITY),
            );
            ui.horizontal(|ui| {
                if ui.button("Search").clicked() || fire {
                    app.run_search();
                }
                ui.checkbox(&mut app.search_opts.use_ignore_files, "use ignore files");
                if app.search_running {
                    ui.spinner();
                }
            });
            ui.add_space(4.0);
            // Replace row + explicit confirm.
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.replace_text)
                        .hint_text("Replace")
                        .desired_width(ui.available_width() - 110.0),
                );
                let n = app
                    .search_results
                    .as_ref()
                    .map(|r| r.total_matches())
                    .unwrap_or(0);
                let blocked = app.search_running
                    || app.search_error.is_some()
                    || app.search_results.as_ref().is_some_and(|r| r.truncated);
                if ui
                    .add_enabled(n > 0 && !blocked, egui::Button::new("Replace All"))
                    .clicked()
                {
                    app.replace_confirm = app
                        .search_results
                        .clone()
                        .map(|r| (r, app.replace_text.clone()));
                }
            });
            ui.separator();

            let Some(res) = &app.search_results else {
                ui.label(
                    egui::RichText::new("Search files in the workspace.")
                        .color(theme.dim)
                        .size(12.0),
                );
                return;
            };
            if res.files.is_empty() && !app.search_running {
                ui.label(
                    egui::RichText::new("No results found.")
                        .color(theme.dim)
                        .size(12.0),
                );
            }
            if res.truncated {
                ui.label(
                    egui::RichText::new(format!(
                        "Result set limited to {} matches.",
                        crate::search::MAX_MATCHES
                    ))
                    .color(theme.dim)
                    .size(11.0),
                );
            }
            if res.skipped > 0 {
                ui.label(
                    egui::RichText::new(format!("{} files skipped (binary/large).", res.skipped))
                        .color(theme.dim)
                        .size(11.0),
                );
            }
            let files = res.files.clone();
            egui::ScrollArea::vertical()
                .id_salt("search-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    let mut goto: Option<(std::path::PathBuf, usize, usize)> = None;
                    for f in &files {
                        let id = ui.make_persistent_id(("search-file", &f.path));
                        let mut open = ui.data_mut(|d| *d.get_persisted_mut_or(id, true));
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 22.0),
                            egui::Sense::click(),
                        );
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
                        let name = f
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        p.text(
                            egui::pos2(rect.left() + 18.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            format!("{name} ({})", f.matches.len()),
                            egui::FontId::proportional(12.0),
                            theme.text,
                        );
                        if resp.clicked() {
                            open = !open;
                            ui.data_mut(|d| d.insert_persisted(id, open));
                        }
                        if !open {
                            continue;
                        }
                        for m in &f.matches {
                            let (r2, resp2) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 20.0),
                                egui::Sense::click(),
                            );
                            let p2 = ui.painter_at(r2);
                            if resp2.hovered() {
                                p2.rect_filled(r2, 0.0, theme.list_hover);
                            }
                            p2.text(
                                egui::pos2(r2.left() + 26.0, r2.center().y),
                                egui::Align2::LEFT_CENTER,
                                &m.preview,
                                egui::FontId::monospace(11.5),
                                theme.text,
                            );
                            p2.text(
                                egui::pos2(r2.right() - 8.0, r2.center().y),
                                egui::Align2::RIGHT_CENTER,
                                format!("{}:{}", m.line, m.column),
                                egui::FontId::monospace(11.0),
                                theme.dim,
                            );
                            if resp2.clicked() {
                                goto = Some((f.path.clone(), m.line, m.column));
                            }
                        }
                    }
                    if let Some((rel, line, col)) = goto {
                        let abs = app.cwd().join(&rel);
                        app.goto_location_col(&abs, line, col);
                    }
                });
        });
}

fn show_git(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    // Title row like the other views.
    let (tr, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 35.0), egui::Sense::hover());
    let tp = ui.painter_at(tr);
    tp.text(
        egui::pos2(tr.left() + 20.0, tr.center().y),
        egui::Align2::LEFT_CENTER,
        "SOURCE CONTROL",
        egui::FontId::new(11.0, egui::FontFamily::Name("bold".into())),
        theme.text,
    );
    icons::codicon(
        &tp,
        egui::pos2(tr.right() - 16.0, tr.center().y),
        ci::REFRESH,
        14.0,
        theme.dim,
    );
    // Clicking the refresh icon re-fetches status.
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(tr.right() - 16.0, tr.center().y),
        egui::vec2(24.0, tr.height()),
    );
    if ui
        .interact(icon_rect, ui.id().with("scm-refresh"), egui::Sense::click())
        .clicked()
    {
        if let Some(ws) = &app.workspace {
            app.git.request_status(ws.root.clone());
        }
    }

    if app.git.snapshot.is_none() && app.workspace.is_some() {
        if let Some(ws) = &app.workspace {
            app.git.request_status(ws.root.clone());
        }
    }
    let Some(snap) = app.git.snapshot.clone() else {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("No Git repository detected.")
                .color(theme.dim)
                .size(12.0),
        );
        return;
    };
    let root = snap.root.clone();
    let staged: Vec<crate::git::GitEntry> = snap.staged().cloned().collect();
    let changed: Vec<crate::git::GitEntry> = snap.changed().cloned().collect();
    let has_head = snap.has_head;

    // Multiline commit message + full-width button; Ctrl+Enter commits.
    let can_commit = !staged.is_empty() && !app.scm_msg.trim().is_empty() && !app.git.is_busy();
    let resp = ui.add(
        egui::TextEdit::multiline(&mut app.scm_msg)
            .hint_text("Message (Ctrl+Enter to commit on 'main')")
            .desired_width(f32::INFINITY)
            .desired_rows(2),
    );
    if resp.has_focus()
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter))
        && can_commit
    {
        app.git.commit(&root, app.scm_msg.clone());
    }
    ui.add_enabled_ui(can_commit, |ui| {
        if ui
            .add_sized(
                [ui.available_width(), 26.0],
                egui::Button::new(egui::RichText::new("Commit").color(egui::Color32::WHITE))
                    .fill(theme.accent),
            )
            .clicked()
        {
            app.git.commit(&root, app.scm_msg.clone());
        }
    });
    ui.add_space(6.0);

    // Screenshot hook: open a staged/worktree diff once per launch.
    if let Ok(which) = std::env::var("OXIDE_DIFF") {
        if app.git_diff.is_none()
            && !ui.data_mut(|d| *d.get_temp_mut_or(ui.id().with("oxdiff-done"), false))
        {
            let pick = if which == "staged" {
                staged.first()
            } else {
                changed.first()
            };
            if let Some(e) = pick {
                app.git.request_diff(&root, e, which == "staged", has_head);
                ui.data_mut(|d| d.insert_temp(ui.id().with("oxdiff-done"), true));
            }
        }
    }
    egui::ScrollArea::vertical()
        .id_salt("scm-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            scm_group(app, ui, "Staged Changes", &staged, true, &root, has_head);
            scm_group(app, ui, "Changes", &changed, false, &root, has_head);
            if staged.is_empty() && changed.is_empty() {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("No changes detected.")
                        .color(theme.dim)
                        .size(12.0),
                );
            }
        });
}

fn scm_group(
    app: &mut OxideApp,
    ui: &mut egui::Ui,
    label: &str,
    entries: &[crate::git::GitEntry],
    staged: bool,
    root: &std::path::Path,
    has_head: bool,
) {
    let theme = &app.theme.clone();
    if entries.is_empty() {
        return;
    }
    // Stage/unstage must include the rename ORIGINAL path too.
    let all: Vec<std::path::PathBuf> = entries
        .iter()
        .flat_map(|e| {
            e.original_path
                .clone()
                .map(|o| vec![e.path.clone(), o])
                .unwrap_or_else(|| vec![e.path.clone()])
        })
        .collect();
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{} ({})", label.to_uppercase(), entries.len()))
                .family(egui::FontFamily::Name("bold".into()))
                .size(11.0)
                .color(theme.text),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (g, t) = if staged {
                (ci::DISCARD, "Unstage All")
            } else {
                (ci::ADD, "Stage All")
            };
            if icons::icon_button(ui, g, t, theme).clicked() {
                if staged {
                    app.git.unstage(root, all.clone(), has_head);
                } else {
                    app.git.stage(root, all.clone());
                }
            }
        });
    });
    for e in entries {
        let (rect, resp) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
        let p = ui.painter_at(rect);
        if resp.hovered() {
            p.rect_filled(rect, 0.0, theme.list_hover);
        }
        let letter = if staged {
            e.index
        } else if e.is_untracked() {
            '?'
        } else {
            e.worktree
        };
        let color = match letter {
            'A' | '?' => theme.git_untracked_fg, // green
            'D' => theme.git_deleted_fg,         // red
            _ => theme.git_modified_fg,          // yellow
        };
        let name = e
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = e
            .path
            .parent()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        let ir = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 16.0, rect.center().y),
            egui::vec2(14.0, 14.0),
        );
        icons::draw_file_icon(&p, ir, &name);
        // status letter on the right, painted first so names never hide it
        p.text(
            egui::pos2(rect.right() - 10.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            letter.to_string(),
            egui::FontId::monospace(11.0),
            color,
        );
        let max_w = rect.right() - 22.0 - (ir.right() + 6.0);
        let mut name_s = name.clone();
        let g = p.layout_no_wrap(name_s.clone(), egui::FontId::proportional(13.0), theme.text);
        if g.size().x > max_w {
            let keep = (name.chars().count() as f32 * (max_w / g.size().x).max(0.1)) as usize;
            name_s = format!(
                "{}…",
                name.chars()
                    .take(keep.saturating_sub(1))
                    .collect::<String>()
            );
        }
        let ng = p.text(
            egui::pos2(ir.right() + 6.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &name_s,
            egui::FontId::proportional(13.0),
            theme.text,
        );
        p.text(
            egui::pos2(ng.right() + 6.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            dir,
            egui::FontId::proportional(11.0),
            theme.dim,
        );
        // Row click opens the real diff (HEAD->index or index->disk).
        if resp.clicked() {
            app.git.request_diff(root, e, staged, has_head);
        }
        resp.context_menu(|ui| {
            if ui.button("Open File").clicked() {
                app.open_file(&root.join(&e.path));
                ui.close();
            }
            if staged {
                if ui.button("Unstage").clicked() {
                    let mut paths = vec![e.path.clone()];
                    if let Some(o) = &e.original_path {
                        paths.push(o.clone());
                    }
                    app.git.unstage(root, paths, has_head);
                    ui.close();
                }
            } else if ui.button("Stage").clicked() {
                let mut paths = vec![e.path.clone()];
                if let Some(o) = &e.original_path {
                    paths.push(o.clone());
                }
                app.git.stage(root, paths);
                ui.close();
            }
            let can_discard = !staged
                && !e.is_untracked()
                && !e.is_conflicted()
                && !e.path.to_string_lossy().contains("/.git/");
            ui.add_enabled_ui(can_discard, |ui| {
                if ui.button("Discard Changes").clicked() {
                    app.confirm_discard = Some(e.path.clone());
                    ui.close();
                }
            });
        });
    }
}

fn show_debug(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "RUN AND DEBUG", |_ui| {});
    egui::Frame::default()
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.add_space(4.0);
            let run_btn = ui.add_sized(
                [ui.available_width(), 28.0],
                egui::Button::new(
                    egui::RichText::new("Run and Debug")
                        .color(egui::Color32::WHITE)
                        .size(13.0),
                )
                .fill(theme.accent)
                .corner_radius(2.0),
            );
            if run_btn.clicked() {
                app.run_cargo("run");
            }
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("To customize Run and Debug create a launch.json file.")
                    .color(theme.dim)
                    .size(12.0),
            );
        });
}

// ---------- Extensions Marketplace ----------

fn show_extensions(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "EXTENSIONS", |ui| {
        if icons::icon_button(ui, ci::ADD, "Load VSIX or Extension Folder…", &theme).clicked() {
            app.show_add_extension = true;
        }
    });
    egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.add(egui::TextEdit::singleline(&mut app.ext_search)
                .hint_text("Search Loaded Extensions")
                .desired_width(f32::INFINITY));
            ui.add_space(8.0);
            if ui.button("Load VSIX or Extension Folder…").clicked() {
                app.show_add_extension = true;
            }
            if ui.button("Import Compatible Packages from VS Code").clicked() {
                app.import_vscode_extensions();
            }
            ui.label(egui::RichText::new("JSON color themes and basic snippets supported. JavaScript extension features require an extension host.").color(theme.dim).size(12.0));
            if let Some(error) = &app.extension_error {
                ui.colored_label(theme.error, error);
            }
            ui.separator();
            let query = app.ext_search.to_lowercase();
            let mut selected = None;
            let mut remove = None;
            egui::ScrollArea::vertical().id_salt("ext-scroll").show(ui, |ui| {
                for ext in &app.extensions {
                    if !format!("{} {} {}", ext.display_name, ext.publisher, ext.description).to_lowercase().contains(&query) { continue; }
                    ui.label(egui::RichText::new(&ext.display_name).strong().color(theme.text));
                    ui.label(egui::RichText::new(format!("{} · {}", ext.publisher, ext.version)).size(11.0).color(theme.dim));
                    ui.label(egui::RichText::new(&ext.description).size(12.0).color(theme.dim));
                    if !ext.snippets.is_empty() { ui.label(format!("{} snippets · Ctrl+Space to insert", ext.snippets.len())); }
                    for t in &ext.themes {
                        if ui.selectable_label(app.extension_theme.as_ref() == Some(&t.id), &t.label).clicked() {
                            selected = Some(t.id.clone());
                        }
                    }
                    if ext.needs_host {
                        ui.label(egui::RichText::new("Supported data loaded. Code features are unavailable.").color(theme.warning).size(11.0));
                    }
                    if ui.small_button("Remove").clicked() { remove = Some(ext.id.clone()); }
                    ui.separator();
                }
                if app.extensions.is_empty() {
                    ui.label(egui::RichText::new("No extensions loaded. Choose a local .vsix file or a folder containing package.json.").color(theme.dim));
                }
            });
            if let Some(id) = selected { app.select_extension_theme(id); }
            if let Some(id) = remove { app.remove_extension(&id); }
        });
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
                            .size(11.0),
                    );
                    for (name, kind) in ThemeKind::all() {
                        if ui
                            .radio(
                                app.extension_theme.is_none() && app.theme_kind == *kind,
                                *name,
                            )
                            .clicked()
                        {
                            app.select_theme(*kind);
                            app.save_config();
                        }
                    }
                    let mut selected = None;
                    for ext in &app.extensions {
                        for t in &ext.themes {
                            if ui
                                .radio(app.extension_theme.as_ref() == Some(&t.id), &t.label)
                                .clicked()
                            {
                                selected = Some(t.id.clone());
                            }
                        }
                    }
                    if let Some(id) = selected {
                        app.select_extension_theme(id);
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label(egui::RichText::new("EDITOR").color(theme.dim).size(11.0));
                    let mut changed = ui
                        .checkbox(&mut app.gitlens_blame, "Git: Inline Blame")
                        .changed();
                    changed |= ui
                        .checkbox(&mut app.minimap_enabled, "Editor Minimap")
                        .changed();
                    changed |= ui
                        .checkbox(&mut app.format_on_save, "Format on Save (rustfmt)")
                        .changed();
                    changed |= ui
                        .checkbox(&mut app.check_on_save, "Check on Save (cargo check)")
                        .changed();
                    if changed {
                        app.save_config();
                    }
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
