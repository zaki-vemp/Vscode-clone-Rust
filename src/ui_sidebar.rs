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
        .default_width(320.0)
        .min_width(260.0)
        .max_width(420.0)
        .frame(
            egui::Frame::side_top_panel(&ctx.style())
                .fill(theme.sidebar)
                .inner_margin(egui::Margin::ZERO)
                .stroke(egui::Stroke::new(1.0, theme.border)),
        )
        .show(ctx, |ui| {
            // A long path or branch name must never grow the whole sidebar.
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            ui.spacing_mut().interact_size.y = 28.0;
            match app.activity {
                Activity::Explorer => show_explorer(app, ui),
                Activity::Search => show_search(app, ui),
                Activity::Git => show_git(app, ui),
                Activity::Debug => show_debug(app, ui),
                Activity::Extensions => show_extensions(app, ui),
                Activity::Settings => show_settings(app, ui),
            }
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

/// Shared view title with a consistent toolbar inset.
fn sidebar_top_bar(
    ui: &mut egui::Ui,
    theme: &Theme,
    title: &str,
    right_actions: impl FnOnce(&mut egui::Ui),
) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 40.0), egui::Sense::hover());
    let mut bar = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_max(
                egui::pos2(rect.left() + 18.0, rect.top()),
                egui::pos2(rect.right() - 10.0, rect.bottom()),
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    bar.label(egui::RichText::new(title).color(theme.text).size(11.0));
    bar.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        right_actions(ui);
    });
}

fn clipped_text(
    painter: &egui::Painter,
    rect: egui::Rect,
    text: impl Into<String>,
    font: egui::FontId,
    color: egui::Color32,
) -> f32 {
    if rect.width() <= 0.0 {
        return 0.0;
    }
    let mut job = egui::text::LayoutJob::simple_singleline(text.into(), font, color);
    job.wrap.max_width = rect.width().max(0.0);
    job.wrap.max_rows = 1;
    job.wrap.overflow_character = Some('…');
    let galley = painter.layout_job(job);
    let width = galley.size().x;
    painter.with_clip_rect(rect).galley(
        egui::pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
        galley,
        color,
    );
    width
}

fn count_badge(painter: &egui::Painter, right: egui::Pos2, count: usize, theme: &Theme) {
    let galley = painter.layout_no_wrap(
        count.to_string(),
        egui::FontId::proportional(11.0),
        theme.text,
    );
    let rect = egui::Rect::from_center_size(
        egui::pos2(right.x - (galley.size().x + 12.0) / 2.0, right.y),
        egui::vec2(galley.size().x + 12.0, 18.0),
    );
    painter.rect_filled(rect, 8.0, theme.badge_bg);
    painter.galley(rect.center() - galley.size() / 2.0, galley, theme.text);
}

fn section_header(
    ui: &mut egui::Ui,
    theme: &Theme,
    label: &str,
    count: usize,
    default_open: bool,
    actions: impl FnOnce(&mut egui::Ui),
) -> bool {
    let id = ui.make_persistent_id(("sidebar-section", label));
    let mut open = ui.data_mut(|data| *data.get_persisted_mut_or(id, default_open));
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 30.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.hline(
        rect.x_range(),
        rect.top(),
        egui::Stroke::new(1.0, theme.border),
    );
    let mut toolbar = ui.new_child(
        egui::UiBuilder::new()
            .id_salt((label, "actions"))
            .max_rect(rect.shrink2(egui::vec2(8.0, 1.0)))
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    actions(&mut toolbar);
    let right = toolbar.min_rect().left().min(rect.right() - 12.0);
    let toggle = egui::Rect::from_min_max(rect.min, egui::pos2(right - 4.0, rect.bottom()));
    let response = ui.interact(toggle, id, egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::CollapsingHeader,
            ui.is_enabled(),
            open,
            label,
        )
    });
    if response.hovered() {
        painter.rect_filled(toggle, 0.0, theme.list_hover);
    }
    icons::codicon(
        &painter,
        egui::pos2(rect.left() + 14.0, rect.center().y),
        if open {
            ci::CHEVRON_DOWN
        } else {
            ci::CHEVRON_RIGHT
        },
        14.0,
        theme.dim,
    );
    clipped_text(
        &painter,
        egui::Rect::from_min_max(
            egui::pos2(rect.left() + 28.0, rect.top()),
            egui::pos2(right - 38.0, rect.bottom()),
        ),
        label,
        egui::FontId::proportional(12.0),
        theme.text,
    );
    count_badge(
        &painter,
        egui::pos2(right - 7.0, rect.center().y),
        count,
        theme,
    );
    if response.clicked() {
        open = !open;
        ui.data_mut(|data| data.insert_persisted(id, open));
    }
    open
}

fn empty_state(ui: &mut egui::Ui, theme: &Theme, glyph: char, title: &str, detail: &str) {
    egui::Frame::default()
        .inner_margin(egui::Margin::same(18))
        .show(ui, |ui| {
            ui.add_space(12.0);
            let (rect, _) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::hover());
            icons::codicon(
                ui.painter(),
                egui::pos2(rect.left() + 12.0, rect.center().y),
                glyph,
                24.0,
                theme.dim,
            );
            ui.add_space(6.0);
            ui.label(egui::RichText::new(title).size(13.0).color(theme.text));
            ui.add_space(3.0);
            ui.add(
                egui::Label::new(egui::RichText::new(detail).size(12.0).color(theme.dim)).wrap(),
            );
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
                    app.pick_and_open_folder();
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

fn search_query(
    ui: &mut egui::Ui,
    theme: &Theme,
    options: &mut crate::search::SearchOptions,
) -> (bool, bool) {
    let mut edited = false;
    let mut fire = false;
    let (row, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::hover());
    let field = egui::Rect::from_min_max(row.min, egui::pos2(row.right() - 36.0, row.bottom()));
    ui.painter().rect_filled(field, 2.0, theme.input_bg);
    let text_rect =
        egui::Rect::from_min_max(field.min, egui::pos2(field.right() - 72.0, field.bottom()));
    let query = ui.place(
        text_rect,
        egui::TextEdit::singleline(&mut options.query)
            .id_salt("workspace-search-query")
            .font(egui::FontId::proportional(13.0))
            .hint_text("Search")
            .frame(false)
            .margin(egui::Margin::symmetric(8, 7))
            .desired_width(text_rect.width()),
    );
    edited |= query.changed();
    fire |= query.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    icons::rect_outline(
        ui.painter(),
        field,
        if query.has_focus() {
            theme.focus_border
        } else {
            theme.input_border
        },
        1.0,
        2.0,
    );
    let mut toggles = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(egui::Rect::from_min_max(
                egui::pos2(field.right() - 70.0, field.top()),
                field.max,
            ))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    toggles.spacing_mut().item_spacing.x = 1.0;
    for (glyph, tip, flag) in [
        (
            ci::CASE_SENSITIVE,
            "Match case",
            &mut options.case_sensitive,
        ),
        (ci::WHOLE_WORD, "Match whole word", &mut options.whole_word),
        (ci::REGEX, "Use regular expression", &mut options.regex),
    ] {
        let (rect, response) =
            toggles.allocate_exact_size(egui::vec2(22.0, 24.0), egui::Sense::click());
        if *flag || response.hovered() {
            toggles.painter().rect_filled(
                rect,
                2.0,
                if *flag {
                    theme.selection
                } else {
                    theme.list_hover
                },
            );
        }
        icons::codicon(
            toggles.painter(),
            rect.center(),
            glyph,
            16.0,
            if *flag { theme.text } else { theme.dim },
        );
        if response.clicked() {
            *flag = !*flag;
            edited = true;
        }
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *flag, tip)
        });
        response.on_hover_text(tip);
    }
    let search_rect = egui::Rect::from_min_max(egui::pos2(row.right() - 30.0, row.top()), row.max);
    let search = ui.interact(
        search_rect,
        ui.id().with("search-submit"),
        egui::Sense::click(),
    );
    ui.painter().rect_filled(search_rect, 2.0, theme.accent);
    icons::codicon(
        ui.painter(),
        search_rect.center(),
        ci::SEARCH,
        17.0,
        egui::Color32::WHITE,
    );
    search.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), "Search")
    });
    if search.on_hover_text("Search (Enter)").clicked() {
        fire = true;
    }
    (edited, fire)
}

fn show_search(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let mut fire = false;
    let mut collapse = false;
    sidebar_top_bar(ui, &theme, "SEARCH", |ui| {
        if icons::icon_button(ui, ci::COLLAPSE_ALL, "Collapse results", &theme).clicked() {
            collapse = true;
        }
        if icons::icon_button(ui, ci::REFRESH, "Search again", &theme).clicked() {
            fire = true;
        }
    });
    let pending_id = ui.make_persistent_id("search-pending");
    let mut edited = false;
    egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            let (changed, submitted) = search_query(ui, &theme, &mut app.search_opts);
            edited |= changed;
            fire |= submitted;

            let replace_id = ui.make_persistent_id("search-replace-open");
            let mut replace_open =
                ui.data_mut(|data| *data.get_persisted_mut_or(replace_id, false));
            ui.horizontal(|ui| {
                let glyph = if replace_open {
                    ci::CHEVRON_DOWN
                } else {
                    ci::CHEVRON_RIGHT
                };
                if icons::icon_button(ui, glyph, "Toggle replace", &theme).clicked() {
                    replace_open = !replace_open;
                    ui.data_mut(|data| data.insert_persisted(replace_id, replace_open));
                }
                if replace_open {
                    let available = ui.available_width();
                    ui.add_sized(
                        [available - 30.0, 30.0],
                        egui::TextEdit::singleline(&mut app.replace_text)
                            .hint_text("Replace")
                            .margin(egui::Margin::symmetric(8, 6)),
                    );
                    let allowed = app.search_results.as_ref().is_some_and(|results| {
                        results.total_matches() > 0
                            && !results.truncated
                            && results.options == app.search_opts
                    }) && !app.search_running
                        && app.search_error.is_none()
                        && !edited;
                    ui.add_enabled_ui(allowed, |ui| {
                        if icons::icon_button(ui, ci::REPLACE_ALL, "Replace all…", &theme).clicked()
                        {
                            app.replace_confirm = app
                                .search_results
                                .clone()
                                .map(|results| (results, app.replace_text.clone()));
                        }
                    });
                } else if ui
                    .add(
                        egui::Label::new(
                            egui::RichText::new("Replace").size(12.0).color(theme.dim),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .clicked()
                {
                    ui.data_mut(|data| data.insert_persisted(replace_id, true));
                }
            });
            let filters_id = ui.id().with("search-filter-details");
            let mut filters_open = ui.data_mut(|data| {
                *data.get_persisted_mut_or(
                    filters_id,
                    !app.search_opts.include.is_empty() || !app.search_opts.exclude.is_empty(),
                )
            });
            ui.horizontal(|ui| {
                let glyph = if filters_open {
                    ci::CHEVRON_DOWN
                } else {
                    ci::CHEVRON_RIGHT
                };
                let toggle =
                    icons::icon_button(ui, glyph, "Toggle search details", &theme).clicked();
                let label = ui.add(
                    egui::Label::new(
                        egui::RichText::new("Search details")
                            .size(12.0)
                            .color(theme.dim),
                    )
                    .sense(egui::Sense::click()),
                );
                if toggle || label.clicked() {
                    filters_open = !filters_open;
                    ui.data_mut(|data| data.insert_persisted(filters_id, filters_open));
                }
            });
            if filters_open {
                for (label, value, hint) in [
                    (
                        "Files to include",
                        &mut app.search_opts.include,
                        "e.g. src/**, *.rs",
                    ),
                    (
                        "Files to exclude",
                        &mut app.search_opts.exclude,
                        "e.g. target/**",
                    ),
                ] {
                    ui.label(egui::RichText::new(label).size(11.0).color(theme.dim));
                    edited |= ui
                        .add_sized(
                            [ui.available_width(), 28.0],
                            egui::TextEdit::singleline(value)
                                .hint_text(hint)
                                .margin(egui::Margin::symmetric(8, 6)),
                        )
                        .changed();
                }
                edited |= ui
                    .checkbox(&mut app.search_opts.use_ignore_files, "Use ignore files")
                    .on_hover_text("Respect .gitignore and .ignore files")
                    .changed();
            }
        });

    if edited {
        ui.data_mut(|data| data.insert_temp(pending_id, std::time::Instant::now()));
    }
    if let Some(start) = ui.data(|data| data.get_temp::<std::time::Instant>(pending_id)) {
        if start.elapsed() >= std::time::Duration::from_millis(250) {
            fire = true;
        } else {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }
    }
    if fire {
        ui.data_mut(|data| data.remove::<std::time::Instant>(pending_id));
        app.run_search();
    }
    if let Some(error) = &app.search_error {
        egui::Frame::default()
            .inner_margin(egui::Margin::same(14))
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(egui::RichText::new(error).color(theme.error).size(12.0))
                        .wrap(),
                );
            });
        return;
    }
    if app.search_opts.query.is_empty() {
        empty_state(
            ui,
            &theme,
            ci::SEARCH,
            "Search your workspace",
            "Find text across your files. Type to search, or press Enter to refresh.",
        );
        return;
    }
    let Some(results) = &app.search_results else {
        empty_state(
            ui,
            &theme,
            ci::SEARCH,
            if app.search_running {
                "Searching…"
            } else {
                "Ready to search"
            },
            "Results are grouped by file. Select a match to open its location.",
        );
        return;
    };
    egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(14, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if app.search_running {
                    ui.spinner();
                }
                ui.label(
                    egui::RichText::new(format!(
                        "{} results in {} files",
                        results.total_matches(),
                        results.files.len()
                    ))
                    .size(12.0)
                    .color(theme.dim),
                );
            });
            if results.truncated {
                ui.label(
                    egui::RichText::new(format!(
                        "Limited to {} matches",
                        crate::search::MAX_MATCHES
                    ))
                    .size(11.0)
                    .color(theme.warning),
                );
            }
            if results.skipped > 0 {
                ui.label(
                    egui::RichText::new(format!(
                        "{} binary or large files skipped",
                        results.skipped
                    ))
                    .size(11.0)
                    .color(theme.dim),
                );
            }
        });
    if results.files.is_empty() && !app.search_running {
        empty_state(
            ui,
            &theme,
            ci::SEARCH,
            "No results found",
            "Try another query or adjust the file filters.",
        );
    }
    let mut goto = None;
    egui::ScrollArea::vertical()
        .id_salt("search-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for file in &results.files {
                let id = ui.make_persistent_id(("search-file", &file.path));
                let mut open = ui.data_mut(|data| *data.get_persisted_mut_or(id, true));
                if collapse {
                    open = false;
                    ui.data_mut(|data| data.insert_persisted(id, false));
                }
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 28.0),
                    egui::Sense::click(),
                );
                let painter = ui.painter_at(rect);
                if response.hovered() {
                    painter.rect_filled(rect, 0.0, theme.list_hover);
                }
                icons::codicon(
                    &painter,
                    egui::pos2(rect.left() + 14.0, rect.center().y),
                    if open {
                        ci::CHEVRON_DOWN
                    } else {
                        ci::CHEVRON_RIGHT
                    },
                    14.0,
                    theme.dim,
                );
                let name = file.path.file_name().unwrap_or_default().to_string_lossy();
                icons::draw_file_icon(
                    &painter,
                    egui::Rect::from_center_size(
                        egui::pos2(rect.left() + 34.0, rect.center().y),
                        egui::vec2(14.0, 14.0),
                    ),
                    &name,
                );
                let text_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.left() + 47.0, rect.top()),
                    egui::pos2(rect.right() - 40.0, rect.bottom()),
                );
                let width = clipped_text(
                    &painter,
                    text_rect,
                    name.to_string(),
                    egui::FontId::proportional(13.0),
                    theme.text,
                );
                if let Some(parent) = file.path.parent() {
                    clipped_text(
                        &painter,
                        egui::Rect::from_min_max(
                            egui::pos2(text_rect.left() + width + 7.0, rect.top()),
                            text_rect.max,
                        ),
                        parent.to_string_lossy(),
                        egui::FontId::proportional(11.0),
                        theme.dim,
                    );
                }
                count_badge(
                    &painter,
                    egui::pos2(rect.right() - 12.0, rect.center().y),
                    file.matches.len(),
                    &theme,
                );
                if response.clicked() {
                    open = !open;
                    ui.data_mut(|data| data.insert_persisted(id, open));
                }
                response.on_hover_text(file.path.display().to_string());
                if !open {
                    continue;
                }
                for found in &file.matches {
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 24.0),
                        egui::Sense::click(),
                    );
                    let painter = ui.painter_at(rect);
                    let selected = app
                        .active
                        .as_ref()
                        .is_some_and(|path| path.ends_with(&file.path))
                        && app.cursor.0 == found.line;
                    if selected || response.hovered() {
                        painter.rect_filled(
                            rect,
                            0.0,
                            if selected {
                                theme.list_active
                            } else {
                                theme.list_hover
                            },
                        );
                    }
                    clipped_text(
                        &painter,
                        egui::Rect::from_min_max(
                            egui::pos2(rect.left() + 28.0, rect.top()),
                            egui::pos2(rect.left() + 62.0, rect.bottom()),
                        ),
                        found.line.to_string(),
                        egui::FontId::monospace(11.0),
                        theme.dim,
                    );
                    let content = egui::Rect::from_min_max(
                        egui::pos2(rect.left() + 66.0, rect.top()),
                        egui::pos2(rect.right() - 12.0, rect.bottom()),
                    );
                    let job = search_preview(found, &theme);
                    let galley = painter.layout_job(job);
                    painter.with_clip_rect(content).galley(
                        egui::pos2(content.left(), content.center().y - galley.size().y / 2.0),
                        galley,
                        theme.text,
                    );
                    if response.clicked() {
                        goto = Some((file.path.clone(), found.line, found.column));
                    }
                    response.on_hover_ui(|ui| {
                        ui.label(format!(
                            "{}:{}:{}",
                            file.path.display(),
                            found.line,
                            found.column
                        ));
                        ui.add(egui::Label::new(search_preview(found, &theme)).wrap());
                    });
                }
            }
        });
    if let Some((relative, line, column)) = goto {
        app.goto_location_col(&app.cwd().join(relative), line, column);
    }
}

fn search_preview(found: &crate::search::SearchMatch, theme: &Theme) -> egui::text::LayoutJob {
    let text = &found.preview;
    let matched = &found.preview_match;
    let start = text[..matched.start]
        .char_indices()
        .rev()
        .nth(24)
        .map_or(0, |(index, _)| index);
    let end = text[start..]
        .char_indices()
        .nth(180)
        .map_or(text.len(), |(index, _)| start + index);
    let format = egui::TextFormat {
        font_id: egui::FontId::monospace(12.0),
        color: theme.text,
        ..Default::default()
    };
    let mut job = egui::text::LayoutJob::default();
    if start > 0 {
        job.append("…", 0.0, format.clone());
    }
    let highlight_start = matched.start.min(end);
    let highlight_end = matched.end.min(end);
    job.append(&text[start..highlight_start], 0.0, format.clone());
    job.append(
        &text[highlight_start..highlight_end],
        0.0,
        egui::TextFormat {
            background: theme.find_match,
            ..format.clone()
        },
    );
    job.append(&text[highlight_end..end], 0.0, format.clone());
    if end < text.len() {
        job.append("…", 0.0, format);
    }
    job
}

fn show_git(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    let snapshot = app.git.snapshot.clone();
    sidebar_top_bar(ui, &theme, "SOURCE CONTROL", |ui| {
        if let Some(snapshot) = &snapshot {
            let response =
                icons::icon_button(ui, ci::ELLIPSIS, "More source control actions", &theme);
            egui::Popup::menu(&response).show(|ui| {
                git_actions(app, ui, snapshot);
            });
        }
        if icons::icon_button(ui, ci::REFRESH, "Refresh source control", &theme).clicked() {
            if let Some(workspace) = &app.workspace {
                app.git.request_status(workspace.root.clone());
            }
        }
    });
    if snapshot.is_none() && !app.git.is_busy() && app.git.last_error.is_none() {
        if let Some(workspace) = &app.workspace {
            app.git.request_status(workspace.root.clone());
        }
    }
    let Some(snap) = snapshot else {
        empty_state(
            ui,
            &theme,
            ci::SOURCE_CONTROL,
            if app.git.is_busy() {
                "Loading repository…"
            } else {
                "No repository open"
            },
            "Open a folder containing a Git repository to review and commit changes.",
        );
        if let Some(error) = &app.git.last_error {
            ui.add(
                egui::Label::new(egui::RichText::new(error).color(theme.error).size(12.0)).wrap(),
            );
        }
        return;
    };
    let root = snap.root.clone();
    let branch = snap.branch.clone();
    let has_head = snap.has_head;
    let conflicts: Vec<_> = snap.conflicts().cloned().collect();
    let staged: Vec<_> = snap
        .staged()
        .filter(|entry| !entry.is_conflicted())
        .cloned()
        .collect();
    let changed: Vec<_> = snap
        .changed()
        .filter(|entry| !entry.is_conflicted())
        .cloned()
        .collect();
    egui::Frame::default()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.horizontal(|ui| {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(18.0, 20.0), egui::Sense::hover());
                icons::codicon(ui.painter(), rect.center(), ci::FOLDER, 16.0, theme.dim);
                let name = root.file_name().unwrap_or_default().to_string_lossy();
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(name)
                            .size(13.0)
                            .strong()
                            .color(theme.text),
                    )
                    .truncate(),
                )
                .on_hover_text(root.display().to_string());
                if app.git.is_busy() {
                    ui.add(egui::Spinner::new().size(12.0).color(theme.dim));
                }
            });
            let (rect, _) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 24.0), egui::Sense::hover());
            icons::codicon(
                ui.painter(),
                egui::pos2(rect.left() + 9.0, rect.center().y),
                ci::GIT_BRANCH,
                16.0,
                theme.dim,
            );
            clipped_text(
                ui.painter(),
                egui::Rect::from_min_max(
                    egui::pos2(rect.left() + 25.0, rect.top()),
                    egui::pos2(rect.right() - 30.0, rect.bottom()),
                ),
                &branch,
                egui::FontId::proportional(13.0),
                theme.text,
            );
            let mut sync_ui = ui.new_child(egui::UiBuilder::new().max_rect(
                egui::Rect::from_min_max(egui::pos2(rect.right() - 24.0, rect.top()), rect.max),
            ));
            sync_ui.add_enabled_ui(!app.git.is_busy(), |ui| {
                let publish = snap.upstream.is_none();
                if icons::icon_button(
                    ui,
                    ci::SYNC,
                    if publish {
                        "Publish branch to origin"
                    } else {
                        "Sync changes (pull, then push)"
                    },
                    &theme,
                )
                .clicked()
                {
                    if publish {
                        app.git.publish(&root, branch.clone());
                    } else {
                        app.git.sync(&root);
                    }
                }
            });
            let upstream = match &snap.upstream {
                Some(upstream) if snap.ahead == 0 && snap.behind == 0 => {
                    format!("{upstream} · Up to date")
                }
                Some(upstream) => format!("{upstream} · ↑{} ↓{}", snap.ahead, snap.behind),
                None => "No upstream configured".to_string(),
            };
            ui.add(
                egui::Label::new(egui::RichText::new(upstream).size(11.0).color(theme.dim))
                    .truncate(),
            );
            ui.add_space(2.0);
            let message = egui::Frame::default()
                .fill(theme.input_bg)
                .stroke(egui::Stroke::new(1.0, theme.input_border))
                .corner_radius(2.0)
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut app.scm_msg)
                            .font(egui::FontId::proportional(13.0))
                            .hint_text("Message")
                            .frame(false)
                            .margin(0.0)
                            .desired_width(f32::INFINITY)
                            .desired_rows(2),
                    );
                    ui.label(
                        egui::RichText::new("Ctrl+Enter to commit")
                            .size(11.0)
                            .color(theme.dim),
                    );
                    response
                });
            if message.inner.has_focus() {
                icons::rect_outline(
                    ui.painter(),
                    message.response.rect,
                    theme.focus_border,
                    1.0,
                    2.0,
                );
            }
            let can_commit =
                !staged.is_empty() && !app.scm_msg.trim().is_empty() && !app.git.is_busy();
            let can_commit_all =
                !changed.is_empty() && !app.scm_msg.trim().is_empty() && !app.git.is_busy();
            if message.inner.has_focus()
                && ui.input_mut(|input| {
                    input.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)
                })
                && can_commit
            {
                app.git.commit(&root, app.scm_msg.clone());
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.spacing_mut().interact_size.y = 32.0;
                if ui
                    .add_enabled(
                        can_commit,
                        egui::Button::new(
                            egui::RichText::new("Commit").color(egui::Color32::WHITE),
                        )
                        .fill(theme.accent)
                        .min_size(egui::vec2(ui.available_width() - 34.0, 32.0)),
                    )
                    .on_hover_text(if staged.is_empty() {
                        "Stage changes before committing"
                    } else {
                        "Commit staged changes (Ctrl+Enter)"
                    })
                    .clicked()
                {
                    app.git.commit(&root, app.scm_msg.clone());
                }
                ui.menu_button(
                    egui::RichText::new(ci::CHEVRON_DOWN.to_string())
                        .family(egui::FontFamily::Name("codicon".into())),
                    |ui| {
                        ui.set_min_width(220.0);
                        if ui
                            .add_enabled(can_commit, egui::Button::new("Commit"))
                            .clicked()
                        {
                            app.git.commit(&root, app.scm_msg.clone());
                            ui.close();
                        }
                        if ui.button("Commit (Amend)").clicked() {
                            app.git
                                .commit_opts(&root, app.scm_msg.clone(), true, false, false);
                            ui.close();
                        }
                        if ui.button("Commit (Sign Off)").clicked() {
                            app.git
                                .commit_opts(&root, app.scm_msg.clone(), false, false, true);
                            ui.close();
                        }
                        if ui
                            .add_enabled(can_commit_all, egui::Button::new("Commit All"))
                            .clicked()
                        {
                            app.git
                                .commit_opts(&root, app.scm_msg.clone(), false, true, false);
                            ui.close();
                        }
                        if ui.button("Commit All (Amend)").clicked() {
                            app.git
                                .commit_opts(&root, app.scm_msg.clone(), true, true, false);
                            ui.close();
                        }
                        if ui.button("Commit All (Sign Off)").clicked() {
                            app.git
                                .commit_opts(&root, app.scm_msg.clone(), false, true, true);
                            ui.close();
                        }
                    },
                );
            });
        });
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
            ui.spacing_mut().item_spacing.y = 0.0;
            if !conflicts.is_empty() {
                scm_group(app, ui, "Merge Changes", &conflicts, false, &root, has_head);
            }
            scm_group(app, ui, "Staged Changes", &staged, true, &root, has_head);
            scm_group(app, ui, "Changes", &changed, false, &root, has_head);
            if staged.is_empty() && changed.is_empty() && conflicts.is_empty() {
                empty_state(
                    ui,
                    &theme,
                    ci::CHECK,
                    "Working tree clean",
                    "Your changes will appear here as you edit files.",
                );
            }
            ui.add_space(8.0);
            if !snap.branches.is_empty()
                && section_header(ui, &theme, "Branches", snap.branches.len(), false, |ui| {
                    if icons::icon_button(ui, ci::ADD, "Create branch…", &theme).clicked() {
                        app.branch_input.clear();
                        app.show_branch_dialog = true;
                    }
                })
            {
                for name in &snap.branches {
                    let current = name == &branch;
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 28.0),
                        egui::Sense::hover(),
                    );
                    icons::codicon(
                        ui.painter(),
                        egui::pos2(rect.left() + 26.0, rect.center().y),
                        if current { ci::CHECK } else { ci::GIT_BRANCH },
                        14.0,
                        if current { theme.accent } else { theme.dim },
                    );
                    clipped_text(
                        ui.painter(),
                        egui::Rect::from_min_max(
                            egui::pos2(rect.left() + 44.0, rect.top()),
                            egui::pos2(rect.right() - 36.0, rect.bottom()),
                        ),
                        name,
                        egui::FontId::proportional(12.0),
                        theme.text,
                    );
                    if !current {
                        let mut actions = ui.new_child(egui::UiBuilder::new().max_rect(
                            egui::Rect::from_min_max(
                                egui::pos2(rect.right() - 32.0, rect.top()),
                                rect.max,
                            ),
                        ));
                        actions.menu_button("⋯", |ui| {
                            if ui.button("Checkout").clicked() {
                                app.git.checkout(&root, name.clone());
                                ui.close();
                            }
                            if ui.button("Merge into Current Branch").clicked() {
                                app.git.merge_branch(&root, name.clone());
                                ui.close();
                            }
                            if ui.button("Delete Branch…").clicked() {
                                app.confirm_git =
                                    Some(crate::app::GitConfirm::DeleteBranch(name.clone()));
                                ui.close();
                            }
                        });
                    }
                }
            }
            if !snap.stashes.is_empty()
                && section_header(ui, &theme, "Stashes", snap.stashes.len(), false, |_| {})
            {
                for stash in &snap.stashes {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 28.0),
                        egui::Sense::hover(),
                    );
                    clipped_text(
                        ui.painter(),
                        egui::Rect::from_min_max(
                            egui::pos2(rect.left() + 26.0, rect.top()),
                            egui::pos2(rect.right() - 36.0, rect.bottom()),
                        ),
                        format!("stash@{{{}}}: {}", stash.index, stash.message),
                        egui::FontId::proportional(12.0),
                        theme.text,
                    );
                    let mut actions =
                        ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_max(
                            egui::pos2(rect.right() - 32.0, rect.top()),
                            rect.max,
                        )));
                    actions
                        .menu_button("⋯", |ui| {
                            if ui.button("Apply Stash").clicked() {
                                app.git.stash_apply(&root, stash.index, false);
                                ui.close();
                            }
                            if ui.button("Pop Stash").clicked() {
                                app.git.stash_apply(&root, stash.index, true);
                                ui.close();
                            }
                            if ui.button("Drop Stash…").clicked() {
                                app.confirm_git =
                                    Some(crate::app::GitConfirm::DropStash(stash.index));
                                ui.close();
                            }
                        })
                        .response
                        .on_hover_text(&stash.message);
                }
            }
            if let Some(error) = &app.git.last_error {
                egui::Frame::default()
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(error).color(theme.error).size(12.0),
                            )
                            .wrap(),
                        );
                    });
            }
        });
}

fn git_actions(app: &mut OxideApp, ui: &mut egui::Ui, snap: &crate::git::GitSnapshot) {
    let root = snap.root.clone();
    let branch = snap.branch.clone();
    let has_head = snap.has_head;
    let staged: Vec<_> = snap
        .staged()
        .filter(|entry| !entry.is_conflicted())
        .cloned()
        .collect();
    let changed: Vec<_> = snap
        .changed()
        .filter(|entry| !entry.is_conflicted())
        .cloned()
        .collect();
    ui.set_min_width(240.0);
    ui.menu_button("Commit", |ui| {
        if ui.button("Commit").clicked() {
            app.git.commit(&root, app.scm_msg.clone());
            ui.close();
        }
        if ui.button("Commit (Amend)").clicked() {
            app.git
                .commit_opts(&root, app.scm_msg.clone(), true, false, false);
            ui.close();
        }
        if ui.button("Commit (Sign Off)").clicked() {
            app.git
                .commit_opts(&root, app.scm_msg.clone(), false, false, true);
            ui.close();
        }
        if ui.button("Commit All").clicked() {
            app.git
                .commit_opts(&root, app.scm_msg.clone(), false, true, false);
            ui.close();
        }
        if ui.button("Commit All (Amend)").clicked() {
            app.git
                .commit_opts(&root, app.scm_msg.clone(), true, true, false);
            ui.close();
        }
        if ui.button("Commit All (Sign Off)").clicked() {
            app.git
                .commit_opts(&root, app.scm_msg.clone(), false, true, true);
            ui.close();
        }
        ui.separator();
        if ui.button("Undo Last Commit").clicked() {
            app.confirm_git = Some(crate::app::GitConfirm::UndoCommit);
            ui.close();
        }
    });
    ui.menu_button("Changes", |ui| {
        if ui.button("Stage All Changes").clicked() {
            let all: Vec<_> = changed
                .iter()
                .flat_map(|e| {
                    let mut v = vec![e.path.clone()];
                    if let Some(o) = &e.original_path {
                        v.push(o.clone());
                    }
                    v
                })
                .collect();
            app.git.stage(&root, all);
            ui.close();
        }
        if ui.button("Unstage All Changes").clicked() {
            let all: Vec<_> = staged.iter().map(|e| e.path.clone()).collect();
            app.git.unstage(&root, all, has_head);
            ui.close();
        }
        if ui.button("Discard All Changes…").clicked() {
            app.confirm_git = Some(crate::app::GitConfirm::DiscardAll);
            ui.close();
        }
        if ui.button("Discard All Tracked Changes…").clicked() {
            app.confirm_git = Some(crate::app::GitConfirm::DiscardAllTracked);
            ui.close();
        }
        if ui.button("Clean All Untracked Files…").clicked() {
            app.confirm_git = Some(crate::app::GitConfirm::CleanAll);
            ui.close();
        }
    });
    ui.menu_button("Pull, Push", |ui| {
        if ui.button("Fetch").clicked() {
            app.git.fetch(&root);
            ui.close();
        }
        if ui.button("Fetch (Prune)").clicked() {
            app.git.fetch(&root);
            ui.close();
        }
        if ui.button("Pull").clicked() {
            app.git.pull(&root);
            ui.close();
        }
        if ui.button("Push").clicked() {
            app.git.push(&root);
            ui.close();
        }
        if ui.button("Publish Branch").clicked() {
            app.git.publish(&root, branch.clone());
            ui.close();
        }
        if ui.button("Sync").clicked() {
            app.git.sync(&root);
            ui.close();
        }
    });
    ui.menu_button("Branch", |ui| {
        if ui.button("Create Branch…").clicked() {
            app.branch_input.clear();
            app.show_branch_dialog = true;
            ui.close();
        }
        if ui.button("Create Branch From…").clicked() {
            app.branch_input.clear();
            app.show_branch_dialog = true;
            ui.close();
        }
        ui.separator();
        for b in &snap.branches {
            if b != &branch && ui.button(format!("Checkout to…  {b}")).clicked() {
                app.git.checkout(&root, b.clone());
                ui.close();
            }
        }
        ui.separator();
        for b in &snap.branches {
            if b != &branch && ui.button(format!("Merge Branch…  {b}")).clicked() {
                app.git.merge_branch(&root, b.clone());
                ui.close();
            }
        }
        ui.separator();
        for b in &snap.branches {
            if b != &branch && ui.button(format!("Delete Branch…  {b}")).clicked() {
                app.confirm_git = Some(crate::app::GitConfirm::DeleteBranch(b.clone()));
                ui.close();
            }
        }
    });
    ui.menu_button("Stash", |ui| {
        if ui.button("Stash Changes…").clicked() {
            app.stash_input.clear();
            app.show_stash_dialog = true;
            ui.close();
        }
        if ui.button("Stash Changes (Include Untracked)").clicked() {
            app.git.stash_push(&root, None, true, false, vec![]);
            ui.close();
        }
        if ui.button("Stash Staged Changes").clicked() {
            app.git.stash_push(&root, None, false, true, vec![]);
            ui.close();
        }
        ui.separator();
        if ui
            .add_enabled(
                !snap.stashes.is_empty(),
                egui::Button::new("Apply Latest Stash"),
            )
            .clicked()
        {
            app.git.stash_apply(&root, 0, false);
            ui.close();
        }
        if ui
            .add_enabled(
                !snap.stashes.is_empty(),
                egui::Button::new("Pop Latest Stash"),
            )
            .clicked()
        {
            app.git.stash_apply(&root, 0, true);
            ui.close();
        }
        if ui
            .add_enabled(
                !snap.stashes.is_empty(),
                egui::Button::new("Drop Latest Stash…"),
            )
            .clicked()
        {
            app.confirm_git = Some(crate::app::GitConfirm::DropStash(0));
            ui.close();
        }
    });
    ui.separator();
    if ui.button("Refresh").clicked() {
        app.git.request_status(root.clone());
        ui.close();
    }
    if ui.button("Open File (active)").clicked() {
        if let Some(a) = app.active.clone() {
            app.open_file(&a);
        }
        ui.close();
    }
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
    let open = section_header(ui, theme, label, entries.len(), true, |ui| {
        // VS Code group actions: Discard All / Clean, then Stage/Unstage All.
        if !staged {
            if entries.iter().any(|e| !e.is_untracked())
                && icons::icon_button(ui, ci::DISCARD, "Discard All Tracked Changes…", theme)
                    .clicked()
            {
                app.confirm_git = Some(crate::app::GitConfirm::DiscardAllTracked);
            }
            if entries.iter().any(|e| e.is_untracked())
                && icons::icon_button(ui, ci::TRASH, "Clean All Untracked Files…", theme).clicked()
            {
                app.confirm_git = Some(crate::app::GitConfirm::CleanAll);
            }
        }
        let (g, t) = if staged {
            (ci::DISCARD, "Unstage All Changes")
        } else {
            (ci::ADD, "Stage All Changes")
        };
        if icons::icon_button(ui, g, t, theme).clicked() {
            if staged {
                app.git.unstage(root, all.clone(), has_head);
            } else {
                app.git.stage(root, all.clone());
            }
        }
    });
    if !open {
        return;
    }
    for e in entries {
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::hover());
        let p = ui.painter_at(rect);
        let selected = app
            .git_diff
            .as_ref()
            .is_some_and(|diff| diff.path == e.path && diff.staged == staged);
        let hovered = ui.rect_contains_pointer(rect);
        if hovered || selected {
            p.rect_filled(
                rect,
                0.0,
                if selected {
                    theme.list_active
                } else {
                    theme.list_hover
                },
            );
        }
        let letter = if staged {
            e.index
        } else if e.is_untracked() {
            '?'
        } else {
            e.worktree
        };
        let color = match letter {
            'A' | '?' => theme.git_untracked_fg,
            'D' => theme.git_deleted_fg,
            _ => theme.git_modified_fg,
        };
        p.text(
            egui::pos2(rect.right() - 12.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            letter.to_string(),
            egui::FontId::monospace(11.0),
            color,
        );
        let action_width = if hovered || selected { 72.0 } else { 0.0 };
        let actions_left = rect.right() - 24.0 - action_width;
        let click_rect =
            egui::Rect::from_min_max(rect.min, egui::pos2(actions_left, rect.bottom()));
        let resp = ui.interact(
            click_rect,
            ui.id().with((label, &e.path)),
            egui::Sense::click(),
        );
        let name = e.path.file_name().unwrap_or_default().to_string_lossy();
        let icon = egui::Rect::from_center_size(
            egui::pos2(rect.left() + 28.0, rect.center().y),
            egui::vec2(14.0, 14.0),
        );
        icons::draw_file_icon(&p, icon, &name);
        let text = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 42.0, rect.top()),
            egui::pos2(actions_left - 5.0, rect.bottom()),
        );
        let width = clipped_text(
            &p,
            text,
            name.to_string(),
            egui::FontId::proportional(13.0),
            theme.text,
        );
        if let Some(parent) = e.path.parent() {
            clipped_text(
                &p,
                egui::Rect::from_min_max(
                    egui::pos2(text.left() + width + 7.0, text.top()),
                    text.max,
                ),
                parent.to_string_lossy(),
                egui::FontId::proportional(11.0),
                theme.dim,
            );
        }
        if action_width > 0.0 {
            let mut actions = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt((label, &e.path, "file-actions"))
                    .max_rect(egui::Rect::from_min_max(
                        egui::pos2(actions_left, rect.top()),
                        egui::pos2(rect.right() - 24.0, rect.bottom()),
                    ))
                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
            );
            actions.spacing_mut().item_spacing.x = 1.0;
            if icons::icon_button(&mut actions, ci::GO_TO_FILE, "Open file", theme).clicked() {
                app.open_file(&root.join(&e.path));
            }
            if icons::icon_button(&mut actions, ci::DISCARD, "Discard changes…", theme).clicked()
            {
                if staged {
                    app.confirm_git = Some(crate::app::GitConfirm::DiscardStaged(e.path.clone()));
                } else if e.is_untracked() {
                    app.confirm_git = Some(crate::app::GitConfirm::Clean(e.path.clone()));
                } else if !e.is_conflicted() {
                    app.confirm_discard = Some(e.path.clone());
                }
            }
            let mut paths = vec![e.path.clone()];
            if let Some(original) = &e.original_path {
                paths.push(original.clone());
            }
            if icons::icon_button(
                &mut actions,
                if staged { ci::DISCARD } else { ci::ADD },
                if staged {
                    "Unstage changes"
                } else {
                    "Stage changes"
                },
                theme,
            )
            .clicked()
            {
                if staged {
                    app.git.unstage(root, paths, has_head);
                } else {
                    app.git.stage(root, paths);
                }
            }
        }
        if resp.clicked() {
            app.git.request_diff(root, e, staged, has_head);
        }
        let resp = resp.on_hover_text(e.path.display().to_string());
        resp.context_menu(|ui| {
            ui.set_min_width(220.0);
            if ui.button("Open File").clicked() {
                app.open_file(&root.join(&e.path));
                ui.close();
            }
            if ui.button("Open Changes").clicked() {
                app.git.request_diff(root, e, staged, has_head);
                ui.close();
            }
            if ui.button("Copy Path").clicked() {
                ui.ctx()
                    .copy_text(root.join(&e.path).to_string_lossy().into_owned());
                ui.close();
            }
            if ui.button("Copy Relative Path").clicked() {
                ui.ctx().copy_text(e.path.to_string_lossy().into_owned());
                ui.close();
            }
            ui.separator();
            // Stage / Unstage (VS Code: Stage Changes / Unstage Changes)
            let mut paths = vec![e.path.clone()];
            if let Some(o) = &e.original_path {
                paths.push(o.clone());
            }
            if staged {
                if ui.button("Unstage Changes").clicked() {
                    app.git.unstage(root, paths.clone(), has_head);
                    ui.close();
                }
                if ui.button("Discard Changes…").clicked() {
                    app.confirm_git = Some(crate::app::GitConfirm::DiscardStaged(e.path.clone()));
                    ui.close();
                }
            } else if e.is_untracked() {
                if ui.button("Stage Changes").clicked() {
                    app.git.stage(root, paths.clone());
                    ui.close();
                }
                if ui.button("Clean (delete file)…").clicked() {
                    app.confirm_git = Some(crate::app::GitConfirm::Clean(e.path.clone()));
                    ui.close();
                }
            } else {
                if ui.button("Stage Changes").clicked() {
                    app.git.stage(root, paths.clone());
                    ui.close();
                }
                let can_discard =
                    !e.is_conflicted() && !e.path.to_string_lossy().contains("/.git/");
                ui.add_enabled_ui(can_discard, |ui| {
                    if ui.button("Discard Changes…").clicked() {
                        app.confirm_discard = Some(e.path.clone());
                        ui.close();
                    }
                });
            }
            ui.separator();
            if ui.button("Stash Changes (this file)…").clicked() {
                app.git.stash_push(root, None, false, false, paths.clone());
                ui.close();
            }
            if ui.button("Stash Changes…").clicked() {
                app.stash_input.clear();
                app.show_stash_dialog = true;
                ui.close();
            }
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
            ui.add(egui::Label::new(egui::RichText::new(
                "Run this Rust project with Cargo. Breakpoint debugging is not available yet.")
                .color(theme.dim).size(12.0)).wrap());
        });
}

// ---------- Extensions Marketplace ----------

fn show_extensions(app: &mut OxideApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    sidebar_top_bar(ui, &theme, "EXTENSIONS", |ui| {
        if icons::icon_button(ui, ci::ADD, "Load VSIX or extension folder…", &theme).clicked() {
            app.show_add_extension = true;
        }
    });
    egui::Frame::default().inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        ui.add_sized([ui.available_width(), 32.0], egui::TextEdit::singleline(&mut app.ext_search)
            .hint_text("Search loaded extensions").margin(egui::Margin::symmetric(8, 7)));
        if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Load VSIX or Folder…")).clicked() {
            app.show_add_extension = true;
        }
        if ui.add_sized([ui.available_width(), 30.0], egui::Button::new("Import from VS Code")).clicked() {
            app.import_vscode_extensions();
        }
        ui.add(egui::Label::new(egui::RichText::new("Color themes and basic snippets are supported. Code extensions need an extension host.")
            .size(12.0).color(theme.dim)).wrap());
        if let Some(error) = &app.extension_error {
            ui.add(egui::Label::new(egui::RichText::new(error).color(theme.error).size(12.0)).wrap());
        }
    });
    let mut selected = None;
    let mut remove = None;
    let query = app.ext_search.to_lowercase();
    egui::ScrollArea::vertical().id_salt("ext-scroll").auto_shrink([false, false]).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut visible = 0;
        for extension in &app.extensions {
            if !format!("{} {} {}", extension.display_name, extension.publisher, extension.description).to_lowercase().contains(&query) { continue; }
            visible += 1;
            ui.separator();
            egui::Frame::default().inner_margin(egui::Margin::same(14)).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.add(egui::Label::new(egui::RichText::new(&extension.display_name).strong().size(13.0).color(theme.text)).truncate());
                ui.add(egui::Label::new(egui::RichText::new(format!("{} · {}", extension.publisher, extension.version)).size(11.0).color(theme.dim)).truncate());
                ui.add(egui::Label::new(egui::RichText::new(&extension.description).size(12.0).color(theme.dim)).wrap());
                if !extension.snippets.is_empty() {
                    ui.label(egui::RichText::new(format!("{} snippets · Ctrl+Space", extension.snippets.len())).size(11.0).color(theme.dim));
                }
                for theme in &extension.themes {
                    if ui.selectable_label(app.extension_theme.as_ref() == Some(&theme.id), &theme.label).clicked() { selected = Some(theme.id.clone()); }
                }
                if extension.needs_host {
                    ui.add(egui::Label::new(egui::RichText::new("Code features unavailable").color(theme.warning).size(11.0)).wrap());
                }
                if ui.small_button("Remove").clicked() { remove = Some(extension.id.clone()); }
            });
        }
        if visible == 0 {
            empty_state(ui, &theme, ci::EXTENSIONS,
                if app.extensions.is_empty() { "Your extensions" } else { "No extensions found" },
                if app.extensions.is_empty() { "Load a local theme or snippet package, or import compatible packages from VS Code." }
                else { "Try another extension name or publisher." });
        }
    });
    if let Some(id) = selected {
        app.select_extension_theme(id);
    }
    if let Some(id) = remove {
        app.remove_extension(&id);
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
                    changed |= ui
                        .checkbox(
                            &mut app.auto_save,
                            "Auto Save (preference, File > Auto Save)",
                        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_input_keeps_sidebar_width_across_frames() {
        for width in [260.0, 320.0, 420.0] {
            let context = egui::Context::default();
            crate::theme::install_fonts(&context);
            let theme = ThemeKind::DarkModern.colors();
            let mut options = crate::search::SearchOptions {
                query: "a very long query 世界".repeat(20),
                ..Default::default()
            };
            for _ in 0..8 {
                let _ = context.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(960.0, 620.0),
                        )),
                        ..Default::default()
                    },
                    |context| {
                        let panel = egui::SidePanel::left("search-width")
                            .default_width(width)
                            .max_width(420.0)
                            .frame(egui::Frame::NONE)
                            .show(context, |ui| {
                                egui::Frame::default()
                                    .inner_margin(egui::Margin::symmetric(12, 8))
                                    .show(ui, |ui| {
                                        search_query(ui, &theme, &mut options);
                                    });
                            });
                        assert!(
                            (panel.response.rect.width() - width).abs() < 1.0,
                            "Search input must not enlarge its parent sidebar"
                        );
                    },
                );
            }
        }
    }

    #[test]
    fn long_unicode_search_preview_is_bounded_and_highlighted() {
        let options = crate::search::SearchOptions {
            query: "世界".into(),
            ..Default::default()
        };
        let matcher = crate::search::Matcher::new(&options).unwrap();
        let source = format!("{} 世界 {}", "ä".repeat(300), "é".repeat(300));
        let file =
            crate::search::search_text(Path::new("test.rs"), &source, &matcher, &mut 1).unwrap();
        let theme = ThemeKind::DarkModern.colors();
        let job = search_preview(&file.matches[0], &theme);
        assert!(job.text.chars().count() <= 182);
        assert!(job.text.contains("世界"));
        assert!(job
            .sections
            .iter()
            .any(|section| section.format.background == theme.find_match
                && &job.text[section.byte_range.clone()] == "世界"));
    }

    #[test]
    fn section_disclosure_and_toolbar_have_separate_hit_targets() {
        for width in [260.0, 320.0, 420.0] {
            let context = egui::Context::default();
            crate::theme::install_fonts(&context);
            let theme = ThemeKind::DarkModern.colors();
            let mut action_count = 0;
            let mut frame = |events| {
                let mut open = false;
                let mut action_center = egui::Pos2::ZERO;
                let mut actual_width = 0.0;
                let _ = context.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(960.0, 620.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |context| {
                        let panel = egui::SidePanel::left("test-sidebar")
                            .exact_width(width)
                            .frame(egui::Frame::NONE)
                            .show(context, |ui| {
                                open = section_header(ui, &theme, "Changes", 1234, true, |ui| {
                                    let response =
                                        icons::icon_button(ui, ci::ADD, "Stage all", &theme);
                                    action_center = response.rect.center();
                                    if response.clicked() {
                                        action_count += 1;
                                    }
                                });
                            });
                        actual_width = panel.response.rect.width();
                    },
                );
                (open, action_center, actual_width, action_count)
            };
            let (open, action_center, actual_width, _) = frame(vec![]);
            assert!(open);
            assert!((actual_width - width).abs() <= 1.0);
            let click = |position, pressed| {
                vec![
                    egui::Event::PointerMoved(position),
                    egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ]
            };
            frame(click(action_center, true));
            let (open, _, _, count) = frame(click(action_center, false));
            assert!(open, "Toolbar click must not collapse the group");
            assert_eq!(count, 1);
            let disclosure = egui::pos2(14.0, 15.0);
            frame(click(disclosure, true));
            let (open, _, _, count) = frame(click(disclosure, false));
            assert!(!open, "Disclosure should collapse the group");
            assert_eq!(count, 1);
        }
    }
}
