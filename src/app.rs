//! Application shell: state, menus, status bar, command palette, and
//! file/project dialogs — laid out like VS Code's title/menu/status bars.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Instant;

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::diagnostics::{self, Diagnostic, Severity};
use crate::git::GitState;
use crate::icons::{self, ci};
use crate::languages::Language;
use crate::project::Workspace;
use crate::pty_terminal::{click_focus, TerminalManager};
use crate::terminal::Terminal;
use crate::theme::{self, Theme, ThemeKind};

/// Left Activity Bar selection.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Activity {
    #[default]
    Explorer,
    Search,
    Git,
    Debug,
    Extensions,
    Settings,
}

/// Bottom panel tab.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BottomTab {
    Problems,
    Output,
    DebugConsole,
    #[default]
    Terminal,
}

/// Inline explorer text edit (new file / new folder / rename input row).
#[derive(Clone)]
pub struct ExplorerEdit {
    pub kind: ExplorerEditKind,
    /// Directory for New*/the entry itself for Rename.
    pub target: PathBuf,
    pub text: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExplorerEditKind {
    NewFile,
    NewFolder,
    Rename,
}

#[derive(Serialize, Deserialize, Default)]
struct SavedConfig {
    theme: Option<String>,
    font_size: Option<f32>,
    tab_size: Option<usize>,
    format_on_save: Option<bool>,
    last_folder: Option<String>,
    #[serde(default)]
    extensions: Vec<PathBuf>,
    extension_theme: Option<String>,
    check_on_save: Option<bool>,
    minimap_enabled: Option<bool>,
    gitlens_blame: Option<bool>,
    #[serde(default)]
    recent_folders: Vec<String>,
    auto_save: Option<bool>,
}

/// Destructive / branch Git confirmations (VS Code-style modal prompts).
#[derive(Clone, Debug, PartialEq)]
pub enum GitConfirm {
    DiscardAll,
    DiscardAllTracked,
    CleanAll,
    DiscardStaged(std::path::PathBuf),
    Clean(std::path::PathBuf),
    DropStash(usize),
    DeleteBranch(String),
    UndoCommit,
}

#[derive(Clone)]
enum UnsavedAction {
    CloseTabs(Vec<PathBuf>),
    OpenFolder(PathBuf),
    CloseFolder,
    Quit,
}

type OutlineSymbols = Vec<(String, usize)>;

pub struct OxideApp {
    pub workspace: Option<Workspace>,
    pub tabs: Vec<PathBuf>,
    pub active: Option<PathBuf>,
    pub editor_text: String,
    pub editor_for: Option<PathBuf>,
    pub highlight_cache: crate::highlight::HighlightCache,
    pub preview: crate::preview::Preview,
    pub json_view: crate::json_view::JsonView,
    pub find: crate::editor_find::Find,
    symbol_cache: Option<(PathBuf, u64, OutlineSymbols)>,
    unsaved_action: Option<UnsavedAction>,
    quit_confirmed: bool,
    capture: Option<(PathBuf, Instant, bool)>,
    pub cursor: (usize, usize),
    pub activity: Activity,
    pub sidebar_open: bool,
    pub bottom_open: bool,
    pub bottom_tab: BottomTab,
    pub split_open: bool,
    pub diff_mode: bool,
    pub palette_open: bool,
    pub palette_query: String,
    pub palette_selected: usize,
    pub extensions: Vec<crate::extensions::Extension>,
    pub extension_theme: Option<String>,
    pub show_add_extension: bool,
    pub extension_path: String,
    pub extension_error: Option<String>,
    pub show_snippets: bool,
    pub snippet_query: String,
    pub pending_snippet: Option<crate::snippets::Snippet>,
    pub ext_search: String,
    pub gitlens_blame: bool,
    pub minimap_enabled: bool,
    pub expanded_dirs: HashSet<PathBuf>,
    pub explorer_edit: Option<ExplorerEdit>,
    pub terminal: Terminal,
    pub pty: TerminalManager,
    pub git: GitState,
    pub git_refresh: std::time::Instant,
    pub git_diff: Option<crate::git::GitDiff>,
    pub git_index: std::collections::HashMap<std::path::PathBuf, Option<String>>,
    git_index_pending: HashSet<PathBuf>,
    index_active: Option<PathBuf>,
    pub gutter_marks: Option<Vec<(usize, char)>>,
    pub gutter_source: Option<(PathBuf, Option<String>, u64)>,
    pub scm_msg: String,
    pub confirm_discard: Option<std::path::PathBuf>,
    pub confirm_git: Option<GitConfirm>,
    pub recent_folders: Vec<PathBuf>,
    pub auto_save: bool,
    pub show_stash_dialog: bool,
    pub stash_input: String,
    pub stash_include_untracked: bool,
    pub stash_keep_index: bool,
    pub show_branch_dialog: bool,
    pub branch_input: String,
    pub term_pane_rects: Vec<(u64, egui::Rect)>,
    pub pty_hook_done: bool,
    pub output_lines: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
    diag_rx: Option<Receiver<Vec<Diagnostic>>>,
    pub diag_busy: bool,
    pub theme_kind: ThemeKind,
    pub theme: Theme,
    applied_style: Option<(ThemeKind, u32, Option<String>)>,
    pub font_size: f32,
    pub tab_size: usize,
    pub format_on_save: bool,
    pub check_on_save: bool,
    pub show_open_folder: bool,
    pub open_folder_input: String,
    pub show_new_project: bool,
    pub new_proj_name: String,
    pub new_proj_lib: bool,
    pub show_about: bool,
    pub search_opts: crate::search::SearchOptions,
    pub search_results: Option<crate::search::SearchResults>,
    pub search_running: bool,
    pub search_gen: u64,
    search_cancel: Arc<AtomicBool>,
    search_rx:
        Option<std::sync::mpsc::Receiver<(u64, Result<crate::search::SearchResults, String>)>>,
    pub replace_confirm: Option<(crate::search::SearchResults, String)>,
    pub search_error: Option<String>,
    pub replace_text: String,
    pub file_list: Vec<PathBuf>,
    file_list_rx: Option<std::sync::mpsc::Receiver<Vec<PathBuf>>>,
    pub pending_col: Option<usize>,
    pub pending_selection: Option<(PathBuf, std::ops::Range<usize>)>,
    pub status_msg: String,
    status_until: Option<Instant>,
    pub pending_goto: Option<(PathBuf, usize)>,
    pub confirm_delete: Option<PathBuf>,
    pub select_all_requested: bool,
    pub show_goto: bool,
    pub goto_input: String,
    pub panel_maximized: bool,
    pub blame: Option<(PathBuf, usize, Vec<crate::project::BlameLine>)>,
    git_branch: Option<String>,
    git_branch_at: Instant,
    last_title: String,
    ctx: egui::Context,
}

impl OxideApp {
    pub fn modal_open(&self) -> bool {
        self.palette_open
            || self.show_about
            || self.show_goto
            || self.confirm_delete.is_some()
            || self.confirm_discard.is_some()
            || self.confirm_git.is_some()
            || self.show_stash_dialog
            || self.show_branch_dialog
            || self.show_open_folder
            || self.show_new_project
            || self.show_add_extension
            || self.show_snippets
            || self.unsaved_action.is_some()
            || self.find.open
    }

    fn selected_theme_colors(&self) -> Theme {
        self.extensions
            .iter()
            .flat_map(|e| &e.themes)
            .find(|t| self.extension_theme.as_ref() == Some(&t.id))
            .map(|t| t.theme.clone())
            .unwrap_or_else(|| self.theme_kind.colors())
    }

    pub fn select_theme(&mut self, kind: ThemeKind) {
        self.theme_kind = kind;
        self.extension_theme = None;
        self.save_config();
    }

    pub fn select_extension_theme(&mut self, id: String) {
        self.extension_theme = Some(id);
        self.save_config();
    }

    pub fn load_extension(&mut self, path: &Path) {
        match crate::extensions::load(path) {
            Ok(extension) => {
                let id = extension.themes.first().map(|t| t.id.clone());
                self.extensions.retain(|e| e.id != extension.id);
                self.extensions.push(extension);
                self.extension_error = None;
                if id.is_some() {
                    self.extension_theme = id;
                }
                self.applied_style = None;
                self.save_config();
                self.show_add_extension = false;
            }
            Err(error) => self.extension_error = Some(format!("{error:#}")),
        }
    }

    pub fn remove_extension(&mut self, id: &str) {
        if self
            .extension_theme
            .as_ref()
            .is_some_and(|t| t.starts_with(&format!("{id}/")))
        {
            self.extension_theme = None;
        }
        self.extensions.retain(|e| e.id != id);
        self.save_config();
    }

    pub fn import_vscode_extensions(&mut self) {
        let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))
        else {
            return;
        };
        let mut loaded = 0;
        for folder in [
            ".vscode/extensions",
            ".vscode-oss/extensions",
            ".vscode-insiders/extensions",
        ] {
            let Ok(entries) = std::fs::read_dir(PathBuf::from(&home).join(folder)) else {
                continue;
            };
            for entry in entries.flatten() {
                let Ok(extension) = crate::extensions::load(&entry.path()) else {
                    continue;
                };
                if !self
                    .extensions
                    .iter()
                    .any(|e| e.id.eq_ignore_ascii_case(&extension.id))
                {
                    self.extensions.push(extension);
                    loaded += 1;
                }
            }
        }
        self.save_config();
        self.status(format!(
            "Loaded {loaded} compatible theme/snippet packages from VS Code"
        ));
    }

    pub fn new(cc: &eframe::CreationContext<'_>, paths: &[PathBuf]) -> Self {
        theme::install_fonts(&cc.egui_ctx);

        let mut app = Self {
            workspace: None,
            tabs: Vec::new(),
            active: None,
            editor_text: String::new(),
            editor_for: None,
            highlight_cache: crate::highlight::HighlightCache::default(),
            preview: crate::preview::Preview::default(),
            json_view: crate::json_view::JsonView::default(),
            find: crate::editor_find::Find::default(),
            symbol_cache: None,
            unsaved_action: None,
            quit_confirmed: false,
            capture: std::env::var_os("OXIDE_CAPTURE")
                .map(|path| (PathBuf::from(path), Instant::now(), false)),
            cursor: (1, 1),
            activity: Activity::Explorer,
            sidebar_open: true,
            bottom_open: true,
            bottom_tab: BottomTab::Terminal,
            split_open: false,
            diff_mode: false,
            palette_open: false,
            palette_query: String::new(),
            palette_selected: 0,
            extensions: Vec::new(),
            extension_theme: None,
            show_add_extension: false,
            extension_path: String::new(),
            extension_error: None,
            show_snippets: false,
            snippet_query: String::new(),
            pending_snippet: None,
            ext_search: String::new(),
            gitlens_blame: false,
            minimap_enabled: true,
            expanded_dirs: HashSet::new(),
            explorer_edit: None,
            terminal: Terminal::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
            pty: TerminalManager::new(
                cc.egui_ctx.clone(),
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            ),
            term_pane_rects: Vec::new(),
            pty_hook_done: false,
            git: GitState::new(cc.egui_ctx.clone()),
            git_refresh: std::time::Instant::now(),
            git_diff: None,
            git_index: std::collections::HashMap::new(),
            git_index_pending: HashSet::new(),
            index_active: None,
            gutter_marks: None,
            gutter_source: None,
            scm_msg: String::new(),
            confirm_discard: None,
            confirm_git: None,
            recent_folders: Vec::new(),
            auto_save: false,
            show_stash_dialog: false,
            stash_input: String::new(),
            stash_include_untracked: false,
            stash_keep_index: false,
            show_branch_dialog: false,
            branch_input: String::new(),
            output_lines: Vec::new(),
            diagnostics: Vec::new(),
            diag_rx: None,
            diag_busy: false,
            theme_kind: ThemeKind::DarkModern,
            theme: ThemeKind::DarkModern.colors(),
            applied_style: None,
            font_size: 14.0,
            tab_size: 4,
            format_on_save: true,
            check_on_save: true,
            show_open_folder: false,
            open_folder_input: String::new(),
            show_new_project: false,
            new_proj_name: String::new(),
            new_proj_lib: false,
            show_about: false,
            search_opts: crate::search::SearchOptions::default(),
            search_results: None,
            search_running: false,
            search_gen: 0,
            search_cancel: Arc::new(AtomicBool::new(false)),
            search_rx: None,
            replace_confirm: None,
            search_error: None,
            replace_text: String::new(),
            file_list: Vec::new(),
            file_list_rx: None,
            pending_col: None,
            pending_selection: None,
            status_msg: String::new(),
            status_until: None,
            pending_goto: None,
            confirm_delete: None,
            select_all_requested: false,
            show_goto: false,
            goto_input: String::new(),
            panel_maximized: false,
            blame: None,
            git_branch: None,
            git_branch_at: Instant::now() - std::time::Duration::from_secs(60),
            last_title: String::new(),
            ctx: cc.egui_ctx.clone(),
        };
        app.load_config();
        if std::env::var_os("OXIDE_IMPORT_VSCODE").is_some() {
            app.import_vscode_extensions();
        }
        app.theme = app.selected_theme_colors();

        let start_dir = paths
            .iter()
            .find(|p| p.is_dir())
            .cloned()
            .or_else(|| {
                paths
                    .first()
                    .and_then(|p| p.parent())
                    .map(Path::to_path_buf)
            })
            .or_else(|| {
                (!app.open_folder_input.is_empty() && Path::new(&app.open_folder_input).is_dir())
                    .then(|| PathBuf::from(&app.open_folder_input))
            })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        app.open_folder(&start_dir);
        for path in paths.iter().filter(|p| p.is_file()) {
            app.open_file(path);
        }
        if std::env::var_os("OXIDE_JSON_TREE").is_some() {
            app.json_view.tree = true;
        }
        if std::env::var_os("OXIDE_HIDE_PANEL").is_some() {
            app.bottom_open = false;
        }

        // Test/screenshot hooks: OXIDE_OPEN=rel/file, OXIDE_EXPAND=rel/dir,…,
        // OXIDE_BOTTOM=problems|output|debugconsole|terminal.
        if let Ok(f) = std::env::var("OXIDE_OPEN") {
            let p = app.cwd().join(&f);
            app.open_file(&p);
        }
        if let Ok(dirs) = std::env::var("OXIDE_EXPAND") {
            for d in dirs.split(',').filter(|d| !d.is_empty()) {
                let mut cur = Some(app.cwd().join(d));
                while let Some(dir) = cur {
                    app.expanded_dirs.insert(dir.clone());
                    if dir == app.cwd() {
                        break;
                    }
                    cur = dir.parent().map(|p| p.to_path_buf());
                }
            }
        }
        if let Ok(v) = std::env::var("OXIDE_VIEW") {
            app.activity = match v.to_ascii_lowercase().as_str() {
                "git" | "scm" => Activity::Git,
                "search" => Activity::Search,
                "debug" => Activity::Debug,
                "extensions" => Activity::Extensions,
                "settings" => Activity::Settings,
                _ => Activity::Explorer,
            };
            app.sidebar_open = true;
        }
        if let Ok(tab) = std::env::var("OXIDE_BOTTOM") {
            app.bottom_open = true;
            app.bottom_tab = match tab.to_ascii_lowercase().as_str() {
                "problems" => BottomTab::Problems,
                "output" => BottomTab::Output,
                "debugconsole" => BottomTab::DebugConsole,
                _ => BottomTab::Terminal,
            };
        }
        if let Ok(query) = std::env::var("OXIDE_SEARCH") {
            app.search_opts.query = query;
            app.search_opts.include = std::env::var("OXIDE_SEARCH_INCLUDE").unwrap_or_default();
            app.search_opts.regex = std::env::var_os("OXIDE_SEARCH_REGEX").is_some();
            app.run_search();
        }

        theme::apply(&cc.egui_ctx, &app.theme, app.font_size);
        app.applied_style = Some((
            app.theme_kind,
            app.font_size.to_bits(),
            app.extension_theme.clone(),
        ));
        app
    }

    // ---------- workspace / file actions ----------

    pub fn cwd(&self) -> PathBuf {
        self.workspace
            .as_ref()
            .map(|w| w.root.clone())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Workspace folder display name (for the title bar / command center).
    pub fn workspace_name(&self) -> String {
        self.cwd()
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.cwd().to_string_lossy().into_owned())
    }

    pub fn open_folder(&mut self, path: &Path) {
        if self
            .workspace
            .as_ref()
            .is_some_and(|w| !w.dirty_paths().is_empty())
        {
            self.unsaved_action = Some(UnsavedAction::OpenFolder(path.to_path_buf()));
            return;
        }
        self.open_folder_unchecked(path);
    }

    fn open_folder_unchecked(&mut self, path: &Path) {
        match Workspace::open_folder(path) {
            Ok(ws) => {
                self.terminal.set_cwd(ws.root.clone());
                self.pty.set_cwd(ws.root.clone());
                self.git.reset(ws.root.clone());
                self.git_diff = None;
                self.git_index.clear();
                self.git_index_pending.clear();
                self.index_active = None;
                self.gutter_source = None;
                self.gutter_marks = None;
                self.symbol_cache = None;
                self.highlight_cache = crate::highlight::HighlightCache::default();
                self.preview.clear();
                self.search_cancel.store(true, Ordering::Relaxed);
                self.search_rx = None;
                self.search_running = false;
                self.search_results = None;
                self.file_list_rx = None;
                self.file_list.clear();
                self.confirm_discard = None;
                self.workspace = Some(ws);
                self.refresh_file_list();
                self.tabs.clear();
                self.active = None;
                self.editor_for = None;
                self.expanded_dirs.clear();
                self.explorer_edit = None;
                self.git_branch = None;
                self.git_branch_at = Instant::now() - std::time::Duration::from_secs(60);
                self.open_folder_input = self.cwd().to_string_lossy().into_owned();
                let root = self.cwd();
                self.recent_folders.retain(|p| p != &root);
                self.recent_folders.insert(0, root);
                self.recent_folders.truncate(10);
                self.save_config();
                self.status(format!("Opened {}", self.cwd().display()));
            }
            Err(e) => self.status(format!("Cannot open folder: {e}")),
        }
    }

    /// File > Open Folder… — native OS chooser (Finder / GTK / KDE via
    /// zenity/kdialog, WinForms). Falls back to the path-entry dialog
    /// (which itself has a Browse… button) when no picker exists.
    /// Cancelling the native dialog does nothing, like VS Code.
    pub fn pick_and_open_folder(&mut self) {
        use crate::file_dialog::PickOutcome;
        match crate::file_dialog::pick_folder(Some(&self.cwd())) {
            PickOutcome::Picked(p) => self.open_folder(&p),
            PickOutcome::Cancelled => {}
            PickOutcome::Unavailable => {
                self.open_folder_input = self.cwd().to_string_lossy().into_owned();
                self.show_open_folder = true;
            }
        }
    }

    pub fn pick_and_open_file(&mut self) {
        use crate::file_dialog::PickOutcome;
        match crate::file_dialog::pick_file(Some(&self.cwd())) {
            PickOutcome::Picked(p) => self.open_file(&p),
            PickOutcome::Cancelled => {}
            PickOutcome::Unavailable => {
                self.status("No native file picker found (install zenity or kdialog)".into());
            }
        }
    }

    /// Save As… — native save dialog, copies the buffer to the new path.
    pub fn save_as_active(&mut self) {
        use crate::file_dialog::PickOutcome;
        let Some(active) = self.active.clone() else {
            return;
        };
        let name = active
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "untitled.txt".into());
        let dir = active.parent().map(|p| p.to_path_buf());
        match crate::file_dialog::pick_save_file(dir.as_deref(), &name) {
            PickOutcome::Picked(target) => {
                // Flush the editor into the buffer, then write to target.
                if self.editor_for.as_ref() == Some(&active) {
                    if let Some(ws) = self.workspace.as_mut() {
                        if let Some(buf) = ws.get_mut(&active) {
                            buf.set_text(&self.editor_text.clone());
                        }
                    }
                }
                let text = self
                    .workspace
                    .as_ref()
                    .and_then(|w| w.get(&active))
                    .map(|b| b.text());
                match text {
                    Some(text) => match std::fs::write(&target, text) {
                        Ok(()) => {
                            self.status(format!("Saved as {}", target.display()));
                            self.open_file(&target);
                        }
                        Err(e) => self.status(format!("Save As failed: {e}")),
                    },
                    None => self.status("Save As: no editable buffer".into()),
                }
            }
            PickOutcome::Cancelled => {}
            PickOutcome::Unavailable => {
                self.status("No native save dialog found (install zenity or kdialog)".into())
            }
        }
    }

    /// Revert File — reload the active file from disk, dropping unsaved edits.
    pub fn revert_active_file(&mut self) {
        let Some(active) = self.active.clone() else {
            return;
        };
        match std::fs::read_to_string(&active) {
            Ok(text) => {
                if let Some(ws) = self.workspace.as_mut() {
                    if let Some(buf) = ws.get_mut(&active) {
                        buf.set_text(&text);
                        buf.mark_saved();
                    }
                }
                if self.active.as_ref() == Some(&active) {
                    self.editor_text = text;
                    self.editor_for = Some(active);
                }
                self.status("File reverted".into());
            }
            Err(e) => self.status(format!("Revert failed: {e}")),
        }
    }

    /// Close Folder (Ctrl+K F) — VS Code returns to the empty window.
    pub fn close_folder(&mut self) {
        if self
            .workspace
            .as_ref()
            .is_some_and(|w| !w.dirty_paths().is_empty())
        {
            self.unsaved_action = Some(UnsavedAction::CloseFolder);
            return;
        }
        self.finish_close_folder();
    }

    fn finish_close_folder(&mut self) {
        self.workspace = None;
        self.tabs.clear();
        self.active = None;
        self.editor_for = None;
        self.editor_text.clear();
        self.git.snapshot = None;
        self.git_diff = None;
        self.expanded_dirs.clear();
        self.status("Folder closed".into());
    }

    /// New Window / Duplicate Workspace — spawn a fresh process.
    pub fn new_window(&mut self, folder: Option<PathBuf>) {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("oxide-code"));
        let mut cmd = std::process::Command::new(exe);
        if let Some(f) = folder {
            cmd.arg(f);
        }
        match cmd.spawn() {
            Ok(_) => self.status("Opened new window".into()),
            Err(e) => self.status(format!("New Window failed: {e}")),
        }
    }

    /// Save Workspace As… — writes a `.code-workspace` JSON file.
    pub fn save_workspace_as(&mut self) {
        use crate::file_dialog::PickOutcome;
        let root = self.cwd();
        let name = format!(
            "{}.code-workspace",
            root.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "workspace".into())
        );
        match crate::file_dialog::pick_save_file(Some(&root), &name) {
            PickOutcome::Picked(target) => {
                let body = serde_json::json!({
                    "folders": [{ "path": root.to_string_lossy() }],
                    "settings": {}
                });
                match std::fs::write(
                    &target,
                    serde_json::to_string_pretty(&body).unwrap_or_default(),
                ) {
                    Ok(()) => self.status(format!("Workspace saved to {}", target.display())),
                    Err(e) => self.status(format!("Save Workspace failed: {e}")),
                }
            }
            PickOutcome::Cancelled => {}
            PickOutcome::Unavailable => self.status("No native save dialog found".into()),
        }
    }

    /// Reload every open buffer from disk (after Discard All / stash / pull…).
    pub fn reload_all_buffers(&mut self) {
        let tabs = self.tabs.clone();
        for path in tabs {
            if let Ok(text) = std::fs::read_to_string(&path) {
                if let Some(ws) = self.workspace.as_mut() {
                    if let Some(buf) = ws.get_mut(&path) {
                        buf.set_text(&text);
                        buf.mark_saved();
                    }
                }
                if self.active.as_ref() == Some(&path) {
                    self.editor_text = text;
                    self.editor_for = Some(path);
                }
            }
        }
    }

    pub fn open_file(&mut self, path: &Path) {
        self.git_diff = None;
        self.git.cancel_diff();
        let path = self.canonical_tab_path(path);
        if self.workspace.is_none() {
            if let Some(parent) = path.parent() {
                self.open_folder(parent);
            }
        }
        let Some(root) = self.workspace.as_ref().map(|w| w.root.clone()) else {
            return;
        };
        if self.preview.wants_preview(&path) {
            if !self.tabs.contains(&path) {
                self.tabs.push(path.clone());
            }
            self.active = Some(path);
            self.editor_for = None;
            self.diff_mode = false;
            return;
        }
        let rel = path
            .strip_prefix(&root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path.clone());
        if self
            .workspace
            .as_mut()
            .is_some_and(|ws| ws.buffer(&rel).is_ok())
        {
            if !self.tabs.contains(&path) {
                self.tabs.push(path.clone());
            }
            self.active = Some(path);
            self.editor_for = None;
            self.diff_mode = false;
        } else {
            let display = rel.to_string_lossy().into_owned();
            self.status(format!("Cannot read {display}"));
        }
    }

    fn canonical_tab_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            return path.to_path_buf();
        }
        if let Some(ws) = self.workspace.as_ref() {
            return ws.root.join(path);
        }
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }

    pub fn active_language(&self) -> Language {
        self.active
            .as_ref()
            .map(|p| Language::from_path(&p.to_string_lossy()))
            .unwrap_or(Language::Plain)
    }

    pub fn active_symbols(&mut self) -> OutlineSymbols {
        let Some(active) = &self.active else {
            return Vec::new();
        };
        let Some(buffer) = self.workspace.as_ref().and_then(|w| w.get(active)) else {
            return Vec::new();
        };
        if !self
            .symbol_cache
            .as_ref()
            .is_some_and(|(path, revision, _)| path == active && *revision == buffer.revision())
        {
            self.symbol_cache = Some((
                active.clone(),
                buffer.revision(),
                crate::highlight::symbols(&buffer.text(), self.active_language()),
            ));
        }
        self.symbol_cache.as_ref().unwrap().2.clone()
    }

    pub fn save_active(&mut self) {
        let Some(active) = self.active.clone() else {
            return;
        };
        if self
            .workspace
            .as_ref()
            .and_then(|w| w.get(&active))
            .is_none()
        {
            self.status("This is a read-only file preview. Open in System App to edit it.".into());
            return;
        }
        if self.editor_for.as_ref() == Some(&active) {
            if let Some(ws) = self.workspace.as_mut() {
                if let Some(buf) = ws.get_mut(&active) {
                    buf.set_text(&self.editor_text);
                }
            }
        }
        let result = self.workspace.as_mut().map(|ws| ws.save(&active));
        match result {
            Some(Ok(())) => {
                let rel = self
                    .workspace
                    .as_ref()
                    .map(|w| w.rel_path(&active))
                    .unwrap_or_default();
                self.status(format!("Saved {rel}"));
                let lang = Language::from_path(&active.to_string_lossy());
                if self.format_on_save && lang == Language::Rust {
                    self.run_rustfmt(&active);
                }
                if self.check_on_save {
                    self.refresh_diagnostics(false);
                }
            }
            Some(Err(e)) => self.status(format!("Save failed: {e}")),
            None => {}
        }
    }

    pub fn format_json(&mut self, compact: bool) {
        let Some(active) = &self.active else {
            return;
        };
        let Some(buffer) = self.workspace.as_ref().and_then(|w| w.get(active)) else {
            return;
        };
        let text = if self.editor_for.as_ref() == Some(active) {
            self.editor_text.clone()
        } else {
            buffer.text()
        };
        match crate::json_view::format(&text, self.tab_size, compact) {
            Ok(formatted) => {
                if let Some(buffer) = self.workspace.as_mut().and_then(|w| w.get_mut(active)) {
                    buffer.set_text(&formatted);
                }
                self.editor_text = formatted;
                self.editor_for = Some(active.clone());
                self.status(
                    if compact {
                        "JSON minified"
                    } else {
                        "JSON formatted"
                    }
                    .into(),
                );
            }
            Err(error) => self.status(format!("JSON formatting failed: {error}")),
        }
    }

    pub fn format_document(&mut self) {
        if self.active_language() == Language::Json {
            self.format_json(false);
            return;
        }
        if self.active_language() != Language::Rust {
            self.status("No document formatter is available for this language yet.".into());
            return;
        }
        let Some(active) = self.active.clone() else {
            return;
        };
        let text = self
            .workspace
            .as_ref()
            .and_then(|w| w.get(&active))
            .map(|b| b.text())
            .unwrap_or_default();
        let result = (|| -> std::io::Result<String> {
            use std::io::Write;
            use std::process::{Command, Stdio};
            let mut child = Command::new("rustfmt")
                .args(["--edition", "2021", "--emit", "stdout"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            let mut stdin = child.stdin.take().expect("piped stdin");
            let writer = std::thread::spawn(move || stdin.write_all(text.as_bytes()));
            let output = child.wait_with_output()?;
            writer
                .join()
                .map_err(|_| std::io::Error::other("Formatter input worker stopped"))??;
            if !output.status.success() {
                return Err(std::io::Error::other(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                ));
            }
            String::from_utf8(output.stdout).map_err(std::io::Error::other)
        })();
        match result {
            Ok(text) => {
                if let Some(buffer) = self.workspace.as_mut().and_then(|w| w.get_mut(&active)) {
                    buffer.set_text(&text);
                }
                self.editor_text = text;
                self.editor_for = Some(active);
                self.status("Document formatted with rustfmt".into());
            }
            Err(error) => self.status(format!("Formatting failed: {error}")),
        }
    }

    pub fn save_all(&mut self) {
        if self.editor_for.as_ref() == self.active.as_ref() {
            if let (Some(ws), Some(active)) = (self.workspace.as_mut(), self.active.clone()) {
                if let Some(buf) = ws.get_mut(&active) {
                    buf.set_text(&self.editor_text);
                }
            }
        }
        if let Some(ws) = self.workspace.as_mut() {
            match ws.save_all() {
                Ok(n) => self.status(format!("Saved {n} file(s)")),
                Err(e) => self.status(format!("Save all failed: {e}")),
            }
        }
    }

    pub fn close_tab(&mut self, path: &Path) {
        self.request_close_tabs(vec![path.to_path_buf()]);
    }

    pub fn close_others(&mut self, keep: &Path) {
        self.request_close_tabs(self.tabs.iter().filter(|p| *p != keep).cloned().collect());
    }

    pub fn close_to_right(&mut self, path: &Path) {
        if let Some(i) = self.tabs.iter().position(|t| t == path) {
            self.request_close_tabs(self.tabs[i + 1..].to_vec());
        }
    }

    pub fn close_all(&mut self) {
        self.request_close_tabs(self.tabs.clone());
    }

    fn request_close_tabs(&mut self, paths: Vec<PathBuf>) {
        if self
            .workspace
            .as_ref()
            .is_some_and(|w| paths.iter().any(|p| w.get(p).is_some_and(|b| b.is_dirty())))
        {
            self.unsaved_action = Some(UnsavedAction::CloseTabs(paths));
        } else {
            self.finish_unsaved_action(UnsavedAction::CloseTabs(paths));
        }
    }

    fn finish_unsaved_action(&mut self, action: UnsavedAction) {
        match action {
            UnsavedAction::CloseTabs(paths) => {
                self.tabs.retain(|p| !paths.contains(p));
                if let Some(ws) = &mut self.workspace {
                    for path in paths {
                        ws.close_buffer(&path);
                        self.git_index.remove(&path);
                    }
                }
                if self.active.as_ref().is_some_and(|p| !self.tabs.contains(p)) {
                    self.active = self.tabs.last().cloned();
                    self.editor_for = None;
                    self.editor_text.clear();
                    self.highlight_cache = crate::highlight::HighlightCache::default();
                    self.symbol_cache = None;
                }
            }
            UnsavedAction::OpenFolder(path) => self.open_folder_unchecked(&path),
            UnsavedAction::CloseFolder => self.finish_close_folder(),
            UnsavedAction::Quit => {
                self.quit_confirmed = true;
                self.ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    pub fn close_saved(&mut self) {
        let paths = self
            .tabs
            .iter()
            .filter(|p| {
                !self
                    .workspace
                    .as_ref()
                    .and_then(|w| w.get(p))
                    .is_some_and(|b| b.is_dirty())
            })
            .cloned()
            .collect();
        self.request_close_tabs(paths);
    }

    fn run_rustfmt(&mut self, file: &Path) {
        let (ok, _) = Terminal::run_blocking(
            "rustfmt",
            &["--edition", "2021", &file.to_string_lossy()],
            &self.cwd(),
        );
        if ok {
            if let Some(ws) = self.workspace.as_mut() {
                if let Ok(text) = std::fs::read_to_string(file) {
                    if let Some(buf) = ws.get_mut(file) {
                        buf.set_text(&text);
                        buf.mark_saved();
                    }
                    if self.active.as_ref() == Some(&file.to_path_buf()) {
                        self.editor_text = text;
                    }
                }
            }
        }
    }

    // ---------- explorer ----------

    /// Directory that explorer "New File/Folder" actions target: the folder
    /// containing the active file, else the workspace root.
    pub fn explorer_target_dir(&self) -> PathBuf {
        if let (Some(ws), Some(active)) = (&self.workspace, &self.active) {
            if let Some(parent) = active.parent() {
                if parent.starts_with(&ws.root) {
                    return parent.to_path_buf();
                }
            }
        }
        self.cwd()
    }

    pub fn commit_explorer_edit(&mut self) {
        let Some(edit) = self.explorer_edit.take() else {
            return;
        };
        let name = edit.text.trim().to_string();
        if name.is_empty() {
            return;
        }
        let Some(ws) = self.workspace.as_mut() else {
            return;
        };
        match edit.kind {
            ExplorerEditKind::NewFile => match ws.new_file(&edit.target, &name) {
                Ok(path) => {
                    self.open_file(&path);
                    self.status(format!("Created {name}"));
                }
                Err(e) => {
                    self.status(format!("Cannot create file: {e}"));
                    self.explorer_edit = Some(edit);
                }
            },
            ExplorerEditKind::NewFolder => match ws.new_folder(&edit.target, &name) {
                Ok(path) => {
                    self.expanded_dirs.insert(path);
                    self.status(format!("Created {name}"));
                }
                Err(e) => {
                    self.status(format!("Cannot create folder: {e}"));
                    self.explorer_edit = Some(edit);
                }
            },
            ExplorerEditKind::Rename => {
                let to = edit
                    .target
                    .parent()
                    .map(|p| p.join(&name))
                    .unwrap_or_else(|| PathBuf::from(&name));
                if to == edit.target {
                    return;
                }
                match ws.rename(&edit.target, &to) {
                    Ok(()) => {
                        self.remap_open_paths(&edit.target, &to);
                        self.status(format!("Renamed to {name}"));
                    }
                    Err(e) => {
                        self.status(format!("Cannot rename: {e}"));
                        self.explorer_edit = Some(edit);
                    }
                }
            }
        }
    }

    /// Update tabs/active/editor paths after a file or folder rename.
    fn remap_open_paths(&mut self, from: &Path, to: &Path) {
        let remap = |p: &PathBuf| -> PathBuf {
            p.strip_prefix(from)
                .map(|s| to.join(s))
                .unwrap_or_else(|_| p.clone())
        };
        for t in &mut self.tabs {
            *t = remap(t);
        }
        self.active = self.active.as_ref().map(&remap);
        self.editor_for = self.editor_for.as_ref().map(&remap);
        if let Some((p, _, _)) = &mut self.blame {
            *p = remap(p);
        }
        for d in self.expanded_dirs.clone() {
            self.expanded_dirs.remove(&d);
            self.expanded_dirs.insert(remap(&d));
        }
    }

    // ---------- cargo tasks ----------

    pub fn run_cargo(&mut self, task: &str) {
        let cwd = self.cwd();
        self.bottom_open = true;
        self.bottom_tab = BottomTab::Output;
        match task {
            "check" => {
                self.terminal
                    .run_program("cargo", &["check", "--message-format=short"], &cwd);
                self.refresh_diagnostics(false);
            }
            "clippy" => {
                self.terminal
                    .run_program("cargo", &["clippy", "--message-format=short"], &cwd);
                self.refresh_diagnostics(true);
            }
            "test" => {
                self.terminal.run_program("cargo", &["test"], &cwd);
            }
            "run" => {
                self.terminal.run_program("cargo", &["run"], &cwd);
            }
            "build" => {
                self.terminal.run_program("cargo", &["build"], &cwd);
            }
            "fmt" => {
                self.terminal.run_program("cargo", &["fmt", "--all"], &cwd);
            }
            _ => {}
        }
    }

    pub fn refresh_diagnostics(&mut self, clippy: bool) {
        if self.diag_busy {
            return;
        }
        let cwd = self.cwd();
        let (tx, rx) = std::sync::mpsc::channel();
        self.diag_rx = Some(rx);
        self.diag_busy = true;
        std::thread::spawn(move || {
            let args: &[&str] = if clippy {
                &["clippy", "--message-format=json", "--quiet"]
            } else {
                &["check", "--message-format=json", "--quiet"]
            };
            let (_, text) = Terminal::run_blocking("cargo", args, &cwd);
            let mut diags = diagnostics::parse_cargo_json(&text);
            if diags.is_empty() {
                diags = diagnostics::parse_human(&text);
            }
            let _ = tx.send(diags);
        });
    }

    pub fn poll_diagnostics(&mut self) {
        if let Some(rx) = self.diag_rx.take() {
            match rx.try_recv() {
                Ok(diags) => {
                    if !diags.is_empty() {
                        self.diagnostics = diags;
                    }
                    self.diag_busy = false;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => self.diag_rx = Some(rx),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.diag_busy = false,
            }
        }
    }

    /// Palette helpers: open with a seeded query ("", ">", ":", "@", or a
    /// file name).
    pub fn open_palette(&mut self, query: &str) {
        self.palette_open = true;
        self.palette_query = query.to_string();
        self.palette_selected = 0;
    }

    /// goto_location with a column; column is in *characters*.
    pub fn goto_location_col(&mut self, path: &Path, line: usize, col: usize) {
        self.pending_col = Some(col.max(1));
        self.goto_location(path, line);
    }

    /// Kick an async workspace search (generation-tagged; stale results
    /// are ignored). Buffers are snapshotted so unsaved edits count.
    pub fn run_search(&mut self) {
        let Some(ws) = &self.workspace else { return };
        self.search_error = None;
        self.search_gen += 1;
        let gen = self.search_gen;
        let (tx, rx) = std::sync::mpsc::channel();
        self.search_rx = Some(rx);
        self.search_running = true;
        let root = ws.root.clone();
        let opts = self.search_opts.clone();
        self.search_cancel.store(true, Ordering::Relaxed);
        self.search_cancel = Arc::new(AtomicBool::new(false));
        let cancelled = self.search_cancel.clone();
        let ctx = self.ctx.clone();
        // Every cached buffer, not only open tabs — a replaced file
        // must stay searchable/editable even untabbed.
        let buffers: Vec<(PathBuf, String)> = ws.buffer_snapshots();
        std::thread::spawn(move || {
            let _ = tx.send((
                gen,
                crate::search::search_cancellable(&root, &opts, &buffers, &cancelled),
            ));
            ctx.request_repaint();
        });
    }

    /// Background workspace file list for quick-open.
    pub fn refresh_file_list(&mut self) {
        if self.file_list_rx.is_some() {
            return;
        }
        let Some(ws) = &self.workspace else { return };
        let (tx, rx) = std::sync::mpsc::channel();
        self.file_list_rx = Some(rx);
        let root = ws.root.clone();
        let ctx = self.ctx.clone();
        std::thread::spawn(move || {
            let mut w = ignore::WalkBuilder::new(&root);
            w.follow_links(false)
                .hidden(false)
                .git_ignore(true)
                .git_exclude(true)
                .ignore(true)
                .filter_entry(|e| e.file_name() != ".git");
            let mut v = Vec::new();
            for e in w.build().flatten() {
                if e.path().is_file() {
                    if let Ok(r) = e.path().strip_prefix(&root) {
                        v.push(r.to_path_buf());
                    }
                }
            }
            let _ = tx.send(v);
            ctx.request_repaint();
        });
    }

    pub fn goto_location(&mut self, path: &Path, line: usize) {
        self.open_file(path);
        let path = self.canonical_tab_path(path);
        self.pending_goto = Some((path, line.max(1)));
    }

    // ---------- status + config ----------

    pub fn status(&mut self, msg: String) {
        self.status_msg = msg;
        self.status_until = Some(Instant::now() + std::time::Duration::from_secs(5));
    }

    /// Git branch of the workspace, refreshed at most every 2 seconds.
    pub fn git_branch_cached(&mut self) -> Option<String> {
        self.git
            .snapshot
            .as_ref()
            .map(|s| s.branch.clone())
            .or_else(|| self.git_branch.clone())
    }

    fn config_path() -> Option<PathBuf> {
        if let Some(path) = std::env::var_os("OXIDE_CONFIG_DIR") {
            return Some(PathBuf::from(path).join("config.json"));
        }
        #[cfg(target_os = "macos")]
        let base = std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library/Application Support"))?;
        #[cfg(target_os = "windows")]
        let base = std::env::var("APPDATA").ok().map(PathBuf::from)?;
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .filter(|p| !p.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("oxide-code").join("config.json"))
    }

    fn load_config(&mut self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let Ok(cfg) = serde_json::from_str::<SavedConfig>(&text) else {
            return;
        };
        if let Some(name) = cfg.theme {
            if let Some((_, kind)) = ThemeKind::all().iter().find(|(n, _)| *n == name) {
                self.theme_kind = *kind;
            }
        }
        if let Some(f) = cfg.font_size {
            self.font_size = f.clamp(9.0, 24.0);
        }
        if let Some(t) = cfg.tab_size {
            self.tab_size = t.clamp(2, 8);
        }
        if let Some(f) = cfg.format_on_save {
            self.format_on_save = f;
        }
        if let Some(value) = cfg.check_on_save {
            self.check_on_save = value;
        }
        if let Some(value) = cfg.minimap_enabled {
            self.minimap_enabled = value;
        }
        if let Some(value) = cfg.gitlens_blame {
            self.gitlens_blame = value;
        }
        for path in cfg.extensions {
            match crate::extensions::load(&path) {
                Ok(extension) => self.extensions.push(extension),
                Err(error) => self.extension_error = Some(format!("{}: {error:#}", path.display())),
            }
        }
        self.extension_theme = cfg.extension_theme;
        if let Some(v) = cfg.auto_save {
            self.auto_save = v;
        }
        self.recent_folders = cfg
            .recent_folders
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
            .take(10)
            .collect();
        if let Some(folder) = cfg.last_folder {
            self.open_folder_input = folder;
        }
    }

    pub fn save_config(&self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let cfg = SavedConfig {
            theme: Some(self.theme_kind.name().to_string()),
            font_size: Some(self.font_size),
            tab_size: Some(self.tab_size),
            format_on_save: Some(self.format_on_save),
            last_folder: Some(self.cwd().to_string_lossy().into_owned()),
            extensions: self.extensions.iter().map(|e| e.path.clone()).collect(),
            extension_theme: self.extension_theme.clone(),
            check_on_save: Some(self.check_on_save),
            minimap_enabled: Some(self.minimap_enabled),
            gitlens_blame: Some(self.gitlens_blame),
            recent_folders: self
                .recent_folders
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
            auto_save: Some(self.auto_save),
        };
        if let Ok(text) = serde_json::to_string_pretty(&cfg) {
            let _ = std::fs::write(path, text);
        }
    }

    fn ensure_style(&mut self, ctx: &egui::Context) {
        let key = (
            self.theme_kind,
            self.font_size.to_bits(),
            self.extension_theme.clone(),
        );
        if self.applied_style.as_ref() != Some(&key) {
            self.theme = self.selected_theme_colors();
            theme::apply(ctx, &self.theme, self.font_size);
            self.applied_style = Some(key);
        }
    }

    /// `"<file>[ ●] - <folder> - OxideCode"` — sent to the OS only on change.
    fn update_title(&mut self, ctx: &egui::Context) {
        let folder = self.workspace_name();
        let title = match self.active.clone() {
            Some(a) => {
                let name = a
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let dirty = self
                    .workspace
                    .as_ref()
                    .and_then(|w| w.get(&a))
                    .is_some_and(|b| b.is_dirty());
                format!(
                    "{name}{} - {folder} - OxideCode",
                    if dirty { " ●" } else { "" }
                )
            }
            None => format!("{folder} - OxideCode"),
        };
        if title != self.last_title {
            self.last_title = title.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        use egui::{Key, Modifiers};
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::N)) {
            self.run_palette_item("New File…");
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::Tab)) {
            if let Some(index) = self
                .tabs
                .iter()
                .position(|path| Some(path) == self.active.as_ref())
            {
                self.active =
                    Some(self.tabs[(index + self.tabs.len() - 1) % self.tabs.len()].clone());
                self.editor_for = None;
            }
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Tab)) {
            if let Some(index) = self
                .tabs
                .iter()
                .position(|path| Some(path) == self.active.as_ref())
            {
                self.active = Some(self.tabs[(index + 1) % self.tabs.len()].clone());
                self.editor_for = None;
            }
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::F)) {
            self.find.open = true;
            self.find.focus = true;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::H)) {
            self.find.open = true;
            self.find.focus = true;
            self.find.replace_open = true;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::SHIFT, Key::F3)) {
            self.find.open = true;
            self.find.next = -1;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F3)) {
            self.find.open = true;
            self.find.next = 1;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::G)) {
            self.show_goto = true;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Comma)) {
            self.activity = Activity::Settings;
            self.sidebar_open = true;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::B)) {
            self.run_cargo("check");
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, Key::Space)) {
            self.show_snippets = true;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::ALT | Modifiers::SHIFT, Key::F)) {
            self.format_document();
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::P)) {
            self.open_palette(">");
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::P)) {
            self.open_palette("");
            self.refresh_file_list();
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::B)) {
            self.sidebar_open = !self.sidebar_open;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::J)) {
            self.bottom_open = !self.bottom_open;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Backslash)) {
            self.split_open = !self.split_open;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::O)) {
            self.pick_and_open_file();
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::S)) {
            self.save_as_active();
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::N)) {
            self.new_window(None);
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Q)) {
            if self
                .workspace
                .as_ref()
                .is_some_and(|w| !w.dirty_paths().is_empty())
            {
                self.unsaved_action = Some(UnsavedAction::Quit);
            } else {
                self.quit_confirmed = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::S)) {
            self.save_active();
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::W)) {
            if let Some(a) = self.active.clone() {
                self.close_tab(&a);
            }
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Backtick)) {
            if self.bottom_open && self.bottom_tab == BottomTab::Terminal {
                self.bottom_open = false;
            } else {
                self.bottom_open = true;
                self.bottom_tab = BottomTab::Terminal;
                self.pty.ensure_session();
                self.pty.panes.focused = self.pty.panes.active;
            }
        } else if ctx
            .input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Backtick))
        {
            self.pty.new_session();
            self.bottom_open = true;
            self.bottom_tab = BottomTab::Terminal;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::E)) {
            self.activity = Activity::Explorer;
            self.sidebar_open = true;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::F)) {
            self.activity = Activity::Search;
            self.sidebar_open = true;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::G)) {
            self.activity = Activity::Git;
            self.sidebar_open = true;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::X)) {
            self.activity = Activity::Extensions;
            self.sidebar_open = true;
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F5)) {
            self.run_cargo("run");
        }
    }
}

/// Command palette entries: (category, title, shortcut).
const PALETTE_ITEMS: &[(&str, &str, &str)] = &[
    ("File", "New File…", "Ctrl+N"),
    ("File", "New Folder…", ""),
    ("File", "New Rust Project…", ""),
    ("File", "Open File…", "Ctrl+O"),
    ("File", "Open Folder…", "Ctrl+K Ctrl+O"),
    ("File", "Save As…", "Ctrl+Shift+S"),
    ("Git", "Git: Stash Changes…", ""),
    ("Git", "Git: Fetch", ""),
    ("Git", "Git: Pull", ""),
    ("Git", "Git: Push", ""),
    ("Git", "Git: Sync", ""),
    ("Git", "Git: Create Branch…", ""),
    ("Git", "Git: Undo Last Commit", ""),
    ("Cargo", "Cargo: Check", "Ctrl+Shift+B"),
    ("Cargo", "Cargo: Build", ""),
    ("Cargo", "Cargo: Run", ""),
    ("Cargo", "Cargo: Test", ""),
    ("Cargo", "Cargo: Clippy", ""),
    ("Cargo", "Format Document (rustfmt)", "Shift+Alt+F"),
    ("JSON", "JSON: Format Document", "Shift+Alt+F"),
    ("JSON", "JSON: Minify Document", ""),
    ("JSON", "JSON: Toggle Tree View", ""),
    ("Snippets", "Snippets: Insert Snippet…", "Ctrl+Space"),
    (
        "Extensions",
        "Extensions: Import Compatible Packages from VS Code",
        "",
    ),
    ("View", "Split Editor (Side-by-Side)", "Ctrl+\\"),
    ("View", "Toggle Minimap", ""),
    ("View", "Toggle Diff vs Saved", ""),
    ("View", "Git: Toggle Inline Blame", ""),
    (
        "Theme",
        "Preferences: Color Theme -> VS Code Dark Modern",
        "",
    ),
    ("Preferences", "Preferences: Color Theme", ""),
    (
        "Extensions",
        "Extensions: Load VSIX or Extension Folder…",
        "",
    ),
    (
        "Theme",
        "Preferences: Color Theme -> VS Code Light Modern",
        "",
    ),
    ("Theme", "Preferences: Color Theme -> VS Code Dark+", ""),
    ("Theme", "Preferences: Color Theme -> VS Code Light+", ""),
    (
        "Theme",
        "Preferences: Color Theme -> VS Code Dark High Contrast",
        "",
    ),
    (
        "Theme",
        "Preferences: Color Theme -> VS Code Light High Contrast",
        "",
    ),
    ("Theme", "Preferences: Color Theme -> Tokyo Night", ""),
    ("Theme", "Preferences: Color Theme -> Catppuccin Mocha", ""),
    ("Theme", "Preferences: Color Theme -> Monokai", ""),
];

impl OxideApp {
    fn run_palette_item(&mut self, title: &str) {
        match title {
            "Snippets: Insert Snippet…" => self.show_snippets = true,
            "Extensions: Import Compatible Packages from VS Code" => {
                self.import_vscode_extensions()
            }
            "JSON: Format Document" => self.format_json(false),
            "JSON: Minify Document" => self.format_json(true),
            "JSON: Toggle Tree View" => self.json_view.tree = !self.json_view.tree,
            "Preferences: Color Theme" => {
                self.activity = Activity::Settings;
                self.sidebar_open = true;
            }
            "Extensions: Load VSIX or Extension Folder…" => {
                self.show_add_extension = true;
            }
            "Preferences: Color Theme -> VS Code Light Modern" => {
                self.select_theme(ThemeKind::LightModern)
            }
            "Preferences: Color Theme -> VS Code Dark+" => self.select_theme(ThemeKind::DarkPlus),
            "Preferences: Color Theme -> VS Code Light+" => self.select_theme(ThemeKind::LightPlus),
            "Preferences: Color Theme -> VS Code Dark High Contrast" => {
                self.select_theme(ThemeKind::HighContrastDark)
            }
            "Preferences: Color Theme -> VS Code Light High Contrast" => {
                self.select_theme(ThemeKind::HighContrastLight)
            }
            "Format Document (rustfmt)" => self.format_document(),
            "Cargo: Check" => self.run_cargo("check"),
            "Cargo: Build" => self.run_cargo("build"),
            "Cargo: Run" => self.run_cargo("run"),
            "Cargo: Test" => self.run_cargo("test"),
            "Cargo: Clippy" => self.run_cargo("clippy"),
            "Split Editor (Side-by-Side)" => self.split_open = !self.split_open,
            "Toggle Minimap" => self.minimap_enabled = !self.minimap_enabled,
            "Toggle Diff vs Saved" => self.diff_mode = !self.diff_mode,
            "Git: Toggle Inline Blame" => self.gitlens_blame = !self.gitlens_blame,
            "Preferences: Color Theme -> VS Code Dark Modern" => {
                self.select_theme(ThemeKind::DarkModern);
                self.save_config();
            }
            "Preferences: Color Theme -> Tokyo Night" => {
                self.select_theme(ThemeKind::TokyoNight);
                self.save_config();
            }
            "Preferences: Color Theme -> Catppuccin Mocha" => {
                self.select_theme(ThemeKind::Catppuccin);
                self.save_config();
            }
            "Preferences: Color Theme -> Monokai" => {
                self.select_theme(ThemeKind::Monokai);
                self.save_config();
            }
            "New File…" => {
                let dir = self.explorer_target_dir();
                self.explorer_edit = Some(ExplorerEdit {
                    kind: ExplorerEditKind::NewFile,
                    target: dir.clone(),
                    text: String::new(),
                });
                self.expanded_dirs.insert(dir);
                self.activity = Activity::Explorer;
                self.sidebar_open = true;
            }
            "New Folder…" => {
                let dir = self.explorer_target_dir();
                self.explorer_edit = Some(ExplorerEdit {
                    kind: ExplorerEditKind::NewFolder,
                    target: dir.clone(),
                    text: String::new(),
                });
                self.expanded_dirs.insert(dir);
                self.activity = Activity::Explorer;
                self.sidebar_open = true;
            }
            "New Rust Project…" => {
                self.new_proj_name.clear();
                self.show_new_project = true;
            }
            "Open Folder…" => self.pick_and_open_folder(),
            "Open File…" => self.pick_and_open_file(),
            "Save As…" => self.save_as_active(),
            "Git: Stash Changes…" => {
                self.stash_input.clear();
                self.show_stash_dialog = true;
                self.activity = Activity::Git;
                self.sidebar_open = true;
            }
            "Git: Fetch" => {
                if let Some(s) = &self.git.snapshot {
                    let r = s.root.clone();
                    self.git.fetch(&r);
                }
            }
            "Git: Pull" => {
                if let Some(s) = &self.git.snapshot {
                    let r = s.root.clone();
                    self.git.pull(&r);
                }
            }
            "Git: Push" => {
                if let Some(s) = &self.git.snapshot {
                    let r = s.root.clone();
                    self.git.push(&r);
                }
            }
            "Git: Sync" => {
                if let Some(s) = &self.git.snapshot {
                    let r = s.root.clone();
                    self.git.sync(&r);
                }
            }
            "Git: Create Branch…" => {
                self.branch_input.clear();
                self.show_branch_dialog = true;
            }
            "Git: Undo Last Commit" => self.confirm_git = Some(GitConfirm::UndoCommit),
            _ => {}
        }
        self.palette_open = false;
    }

    /// VS Code title bar (35px): menu row, centered command-center box,
    /// right-side layout toggles. The OS window chrome stays native.
    fn show_menu(&mut self, ctx: &egui::Context) {
        let theme = self.theme.clone();
        egui::TopBottomPanel::top("title_bar")
            .exact_height(35.0)
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(theme.titlebar)
                    .inner_margin(egui::Margin::symmetric(8, 0))
                    .stroke(egui::Stroke::new(1.0, theme.border)),
            )
            .show(ctx, |ui| {
                let drag = ui.interact(
                    ui.max_rect(),
                    ui.id().with("title-drag"),
                    egui::Sense::drag(),
                );
                if drag.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                ui.horizontal_centered(|ui| {
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                    ui.visuals_mut().widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = theme.list_hover;
                    ui.style_mut().visuals.widgets.hovered.corner_radius =
                        egui::CornerRadius::same(4);
                    ui.style_mut().spacing.item_spacing.x = 2.0;

                    // Menus — File mirrors the VS Code File menu (screenshot).
                    ui.menu_button("File", |ui| {
                        menu_style_accent(ui, &theme);
                        ui.set_min_width(280.0);
                        if menu_item(ui, "New Text File", "Ctrl+N").clicked() {
                            let dir = self.explorer_target_dir();
                            self.explorer_edit = Some(ExplorerEdit {
                                kind: ExplorerEditKind::NewFile,
                                target: dir,
                                text: String::new(),
                            });
                            self.activity = Activity::Explorer;
                            self.sidebar_open = true;
                            ui.close();
                        }
                        if menu_item(ui, "New File…", "Ctrl+Alt+N").clicked() {
                            let dir = self.explorer_target_dir();
                            self.explorer_edit = Some(ExplorerEdit {
                                kind: ExplorerEditKind::NewFile,
                                target: dir,
                                text: String::new(),
                            });
                            self.activity = Activity::Explorer;
                            self.sidebar_open = true;
                            ui.close();
                        }
                        if menu_item(ui, "New Window", "Ctrl+Shift+N").clicked() {
                            self.new_window(None);
                            ui.close();
                        }
                        ui.menu_button("New Window with Profile", |ui| {
                            menu_style_accent(ui, &theme);
                            if menu_item(ui, "Default Profile", "").clicked() {
                                self.new_window(None);
                                ui.close();
                            }
                        });
                        if menu_item(ui, "New Folder…", "").clicked() {
                            let dir = self.explorer_target_dir();
                            self.explorer_edit = Some(ExplorerEdit {
                                kind: ExplorerEditKind::NewFolder,
                                target: dir,
                                text: String::new(),
                            });
                            ui.close();
                        }
                        if menu_item(ui, "New Rust Project…", "").clicked() {
                            self.new_proj_name.clear();
                            self.show_new_project = true;
                            ui.close();
                        }
                        ui.separator();
                        if menu_item(ui, "Open File…", "Ctrl+O").clicked() {
                            self.pick_and_open_file();
                            ui.close();
                        }
                        if menu_item(ui, "Open Folder…", "Ctrl+K Ctrl+O").clicked() {
                            self.pick_and_open_folder();
                            ui.close();
                        }
                        if menu_item(ui, "Open Workspace from File…", "").clicked() {
                            self.pick_and_open_file();
                            ui.close();
                        }
                        ui.menu_button("Open Recent", |ui| {
                            menu_style_accent(ui, &theme);
                            ui.set_min_width(280.0);
                            if self.recent_folders.is_empty() {
                                ui.add_enabled(false, egui::Button::new("(no recent folders)"));
                            }
                            let recents = self.recent_folders.clone();
                            for p in recents {
                                if menu_item(ui, &p.to_string_lossy(), "").clicked() {
                                    self.open_folder(&p);
                                    ui.close();
                                }
                            }
                            ui.separator();
                            if menu_item(ui, "Clear Recently Opened", "").clicked() {
                                self.recent_folders.clear();
                                self.save_config();
                                ui.close();
                            }
                        });
                        ui.separator();
                        if menu_item(ui, "Add Folder to Workspace…", "").clicked() {
                            self.pick_and_open_folder();
                            ui.close();
                        }
                        if menu_item(ui, "Save Workspace As…", "").clicked() {
                            self.save_workspace_as();
                            ui.close();
                        }
                        if menu_item(ui, "Duplicate Workspace", "").clicked() {
                            let cwd = self.cwd();
                            self.new_window(Some(cwd));
                            ui.close();
                        }
                        ui.separator();
                        let has_active = self.active.is_some();
                        ui.add_enabled_ui(has_active, |ui| {
                            if menu_item(ui, "Save", "Ctrl+S").clicked() {
                                self.save_active();
                                ui.close();
                            }
                        });
                        ui.add_enabled_ui(has_active, |ui| {
                            if menu_item(ui, "Save As…", "Ctrl+Shift+S").clicked() {
                                self.save_as_active();
                                ui.close();
                            }
                        });
                        if menu_item(ui, "Save All", "").clicked() {
                            self.save_all();
                            ui.close();
                        }
                        ui.separator();
                        ui.menu_button("Share", |ui| {
                            menu_style_accent(ui, &theme);
                            if menu_item(ui, "Copy Workspace Path", "").clicked() {
                                ui.ctx()
                                    .copy_text(self.cwd().to_string_lossy().into_owned());
                                ui.close();
                            }
                            if let Some(active) = self.active.clone() {
                                if menu_item(ui, "Copy File Path", "").clicked() {
                                    ui.ctx().copy_text(active.to_string_lossy().into_owned());
                                    ui.close();
                                }
                            }
                        });
                        ui.menu_button("Auto Save", |ui| {
                            menu_style_accent(ui, &theme);
                            if ui.selectable_label(!self.auto_save, "Off").clicked() {
                                self.auto_save = false;
                                self.save_config();
                                ui.close();
                            }
                            if ui
                                .selectable_label(
                                    self.auto_save,
                                    "On (save dirty buffers when the editor loses focus)",
                                )
                                .clicked()
                            {
                                self.auto_save = true;
                                self.save_config();
                                ui.close();
                            }
                        });
                        ui.menu_button("Preferences", |ui| {
                            menu_style_accent(ui, &theme);
                            if menu_item(ui, "Settings", "Ctrl+,").clicked() {
                                self.activity = Activity::Settings;
                                self.sidebar_open = true;
                                ui.close();
                            }
                            ui.menu_button("Color Theme", |ui| {
                                menu_style_accent(ui, &theme);
                                for (name, kind) in ThemeKind::all() {
                                    if ui
                                        .selectable_label(
                                            self.extension_theme.is_none()
                                                && self.theme_kind == *kind,
                                            *name,
                                        )
                                        .clicked()
                                    {
                                        self.select_theme(*kind);
                                        ui.close();
                                    }
                                }
                            });
                            if menu_item(ui, "Toggle Minimap", "").clicked() {
                                self.minimap_enabled = !self.minimap_enabled;
                                self.save_config();
                                ui.close();
                            }
                        });
                        ui.separator();
                        ui.add_enabled_ui(has_active, |ui| {
                            if menu_item(ui, "Revert File", "").clicked() {
                                self.revert_active_file();
                                ui.close();
                            }
                        });
                        ui.add_enabled_ui(has_active, |ui| {
                            if menu_item(ui, "Close Editor", "Ctrl+W").clicked() {
                                if let Some(a) = self.active.clone() {
                                    self.close_tab(&a);
                                }
                                ui.close();
                            }
                        });
                        if menu_item(ui, "Close Folder", "Ctrl+K F").clicked() {
                            self.close_folder();
                            ui.close();
                        }
                        if menu_item(ui, "Close Window", "Alt+F4").clicked() {
                            if self
                                .workspace
                                .as_ref()
                                .is_some_and(|w| !w.dirty_paths().is_empty())
                            {
                                self.unsaved_action = Some(UnsavedAction::Quit);
                            } else {
                                self.quit_confirmed = true;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            ui.close();
                        }
                        ui.separator();
                        if menu_item(ui, "Exit", "Ctrl+Q").clicked() {
                            if self
                                .workspace
                                .as_ref()
                                .is_some_and(|w| !w.dirty_paths().is_empty())
                            {
                                self.unsaved_action = Some(UnsavedAction::Quit);
                            } else {
                                self.quit_confirmed = true;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            ui.close();
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        menu_style_accent(ui, &theme);
                        if menu_item(ui, "Find", "Ctrl+F").clicked() {
                            self.find.open = true;
                            self.find.focus = true;
                            ui.close();
                        }
                        if menu_item(ui, "Replace", "Ctrl+H").clicked() {
                            self.find.open = true;
                            self.find.focus = true;
                            self.find.replace_open = true;
                            ui.close();
                        }
                        if menu_item(ui, "Format Document", "Shift+Alt+F").clicked() {
                            self.format_document();
                            ui.close();
                        }
                        if menu_item(ui, "Find in Files", "Ctrl+Shift+F").clicked() {
                            self.activity = Activity::Search;
                            self.sidebar_open = true;
                            ui.close();
                        }
                        if let Some(active) = self.active.clone() {
                            if menu_item(ui, "Copy Path", "").clicked() {
                                ui.ctx().copy_text(active.to_string_lossy().into_owned());
                                ui.close();
                            }
                            let rel = self
                                .workspace
                                .as_ref()
                                .map(|w| w.rel_path(&active))
                                .unwrap_or_default();
                            if menu_item(ui, "Copy Relative Path", "").clicked() {
                                ui.ctx().copy_text(rel);
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Selection", |ui| {
                        menu_style_accent(ui, &theme);
                        if menu_item(ui, "Select All", "Ctrl+A").clicked() {
                            self.select_all_requested = true;
                            ui.close();
                        }
                    });
                    ui.menu_button("View", |ui| {
                        menu_style_accent(ui, &theme);
                        if menu_item(ui, "Command Palette…", "Ctrl+Shift+P").clicked() {
                            self.palette_open = true;
                            self.palette_query.clear();
                            self.palette_selected = 0;
                            ui.close();
                        }
                        ui.separator();
                        if menu_item(ui, "Explorer", "Ctrl+Shift+E").clicked() {
                            self.activity = Activity::Explorer;
                            self.sidebar_open = true;
                            ui.close();
                        }
                        if menu_item(ui, "Search", "Ctrl+Shift+F").clicked() {
                            self.activity = Activity::Search;
                            self.sidebar_open = true;
                            ui.close();
                        }
                        if menu_item(ui, "Source Control", "Ctrl+Shift+G").clicked() {
                            self.activity = Activity::Git;
                            self.sidebar_open = true;
                            ui.close();
                        }
                        if menu_item(ui, "Run", "").clicked() {
                            self.activity = Activity::Debug;
                            self.sidebar_open = true;
                            ui.close();
                        }
                        if menu_item(ui, "Extensions", "Ctrl+Shift+X").clicked() {
                            self.activity = Activity::Extensions;
                            self.sidebar_open = true;
                            ui.close();
                        }
                        ui.separator();
                        if menu_item(ui, "Toggle Primary Side Bar", "Ctrl+B").clicked() {
                            self.sidebar_open = !self.sidebar_open;
                            ui.close();
                        }
                        if menu_item(ui, "Toggle Panel", "Ctrl+J").clicked() {
                            self.bottom_open = !self.bottom_open;
                            ui.close();
                        }
                        if menu_item(ui, "Split Editor Right", "Ctrl+\\").clicked() {
                            self.split_open = !self.split_open;
                            ui.close();
                        }
                        if menu_item(ui, "Toggle Diff vs Saved", "").clicked() {
                            self.diff_mode = !self.diff_mode;
                            ui.close();
                        }
                        ui.separator();
                        for (name, kind) in ThemeKind::all() {
                            if ui
                                .selectable_label(
                                    self.extension_theme.is_none() && self.theme_kind == *kind,
                                    *name,
                                )
                                .clicked()
                            {
                                self.select_theme(*kind);
                                self.save_config();
                                ui.close();
                            }
                        }
                    });
                    ui.menu_button("Go", |ui| {
                        menu_style_accent(ui, &theme);
                        if menu_item(ui, "Go to File…", "Ctrl+P").clicked() {
                            self.palette_open = true;
                            self.palette_query.clear();
                            self.palette_selected = 0;
                            ui.close();
                        }
                        if menu_item(ui, "Go to Line/Column…", "Ctrl+G").clicked() {
                            self.goto_input.clear();
                            self.show_goto = true;
                            ui.close();
                        }
                    });
                    ui.menu_button("Run", |ui| {
                        menu_style_accent(ui, &theme);
                        if menu_item(ui, "Start Debugging", "F5").clicked() {
                            self.run_cargo("run");
                            ui.close();
                        }
                        if menu_item(ui, "Run Without Debugging", "Ctrl+F5").clicked() {
                            self.run_cargo("run");
                            ui.close();
                        }
                        ui.separator();
                        if menu_item(ui, "Run cargo check", "").clicked() {
                            self.run_cargo("check");
                            ui.close();
                        }
                        if menu_item(ui, "Run cargo test", "").clicked() {
                            self.run_cargo("test");
                            ui.close();
                        }
                        if menu_item(ui, "Run cargo clippy", "").clicked() {
                            self.run_cargo("clippy");
                            ui.close();
                        }
                    });
                    ui.menu_button("Terminal", |ui| {
                        menu_style_accent(ui, &theme);
                        if menu_item(ui, "New Terminal", "Ctrl+`").clicked() {
                            self.bottom_open = true;
                            self.bottom_tab = BottomTab::Terminal;
                            self.pty.new_session();
                            ui.close();
                        }
                        if menu_item(ui, "Split Terminal", "").clicked() {
                            self.bottom_open = true;
                            self.bottom_tab = BottomTab::Terminal;
                            self.pty.split_session();
                            ui.close();
                        }
                        if menu_item(ui, "Kill Terminal", "").clicked() {
                            if let Some(id) = self.pty.panes.target() {
                                self.pty.kill(id);
                            }
                            ui.close();
                        }
                        if menu_item(ui, "Run Build Task…", "Ctrl+Shift+B").clicked() {
                            self.run_cargo("build");
                            ui.close();
                        }
                        ui.separator();
                        if menu_item(ui, "Clear Terminal", "").clicked() {
                            if let Some(id) = self.pty.panes.target() {
                                self.pty.send(id, b"\x0c");
                            }
                            ui.close();
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        menu_style_accent(ui, &theme);
                        if menu_item(ui, "About", "").clicked() {
                            self.show_about = true;
                            ui.close();
                        }
                    });

                    // Center command center: arrows + workspace search box.
                    let avail = ui.available_width();
                    let right_reserve = 60.0;
                    let box_w = (avail * 0.38)
                        .clamp(240.0, 600.0)
                        .min(avail - right_reserve);
                    let pad = ((avail - right_reserve - box_w - 44.0) * 0.5).max(4.0);
                    ui.add_space(pad);

                    let arrow_c = theme.gutter;
                    let (ar, _) =
                        ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                    icons::codicon(ui.painter(), ar.center(), ci::ARROW_LEFT, 14.0, arrow_c);
                    let (ar, _) =
                        ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                    icons::codicon(ui.painter(), ar.center(), ci::ARROW_RIGHT, 14.0, arrow_c);
                    ui.add_space(6.0);

                    let folder = self.workspace_name();
                    let (srect, sresp) =
                        ui.allocate_exact_size(egui::vec2(box_w, 22.0), egui::Sense::click());
                    let sp = ui.painter_at(srect);
                    sp.rect_filled(
                        srect,
                        6.0,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 13),
                    );
                    icons::rect_outline(
                        &sp,
                        srect,
                        if sresp.hovered() {
                            theme.border
                        } else {
                            egui::Color32::from_rgba_unmultiplied(204, 204, 204, 51)
                        },
                        1.0,
                        6.0,
                    );
                    let search_g =
                        sp.layout_no_wrap(folder, egui::FontId::proportional(12.0), theme.dim);
                    let text_w = search_g.size().x;
                    let mut cx = srect.center().x - (text_w + 18.0) / 2.0;
                    icons::codicon(
                        &sp,
                        egui::pos2(cx + 8.0, srect.center().y),
                        ci::SEARCH,
                        13.0,
                        theme.dim,
                    );
                    cx += 18.0;
                    sp.galley(
                        egui::pos2(cx, srect.center().y - search_g.size().y / 2.0),
                        search_g,
                        theme.dim,
                    );
                    if sresp.clicked() {
                        self.palette_open = true;
                        self.palette_query.clear();
                        self.palette_selected = 0;
                    }
                    sresp.on_hover_text("Search files, symbols, commands (Ctrl+P)");

                    // Right: layout toggles.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);
                        let g = if self.bottom_open {
                            ci::LAYOUT_PANEL
                        } else {
                            ci::LAYOUT_PANEL_OFF
                        };
                        if icons::icon_button(ui, g, "Toggle Panel (Ctrl+J)", &theme).clicked() {
                            self.bottom_open = !self.bottom_open;
                        }
                        let g = if self.sidebar_open {
                            ci::LAYOUT_SIDEBAR_LEFT
                        } else {
                            ci::LAYOUT_SIDEBAR_LEFT_OFF
                        };
                        if icons::icon_button(ui, g, "Toggle Primary Side Bar (Ctrl+B)", &theme)
                            .clicked()
                        {
                            self.sidebar_open = !self.sidebar_open;
                        }
                    });
                });
            });
    }

    /// VS Code status bar (22px): branch, problem counts, status message on
    /// the left; cursor position / indentation / encoding / EOL / language /
    /// notifications on the right.
    fn show_status(&mut self, ctx: &egui::Context) {
        let theme = self.theme.clone();
        let (line, col) = self.cursor;
        let lang = self.active_language();
        let errors = self
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warnings = self
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();
        let branch = self.git_branch_cached();
        let msg = if self.status_until.is_some_and(|u| Instant::now() < u) {
            self.status_msg.clone()
        } else {
            String::new()
        };

        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(22.0)
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(theme.status)
                    .inner_margin(egui::Margin::symmetric(10, 0))
                    .stroke(egui::Stroke::new(1.0, theme.border)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.horizontal_centered(|ui| {
                    if let Some(branch) = &branch {
                        if status_icon_text(ui, &theme, ci::GIT_BRANCH, branch, "").clicked() {
                            self.activity = Activity::Git;
                            self.sidebar_open = true;
                        }
                    }
                    let problems = format!("{errors}  {warnings}");
                    let pw = 16.0 + text_w(ui, &problems, 12.0) + 16.0 + 8.0;
                    let (rect, resp) =
                        ui.allocate_exact_size(egui::vec2(pw, 18.0), egui::Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(
                            rect,
                            0.0,
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 31),
                        );
                    }
                    let p = ui.painter_at(rect);
                    let mid = rect.center().y;
                    icons::codicon(
                        &p,
                        egui::pos2(rect.left() + 10.0, mid),
                        ci::ERROR,
                        12.0,
                        theme.status_fg,
                    );
                    p.text(
                        egui::pos2(rect.left() + 19.0, mid),
                        egui::Align2::LEFT_CENTER,
                        errors.to_string(),
                        egui::FontId::proportional(12.0),
                        theme.status_fg,
                    );
                    icons::codicon(
                        &p,
                        egui::pos2(rect.left() + 34.0, mid),
                        ci::WARNING,
                        12.0,
                        theme.status_fg,
                    );
                    p.text(
                        egui::pos2(rect.left() + 43.0, mid),
                        egui::Align2::LEFT_CENTER,
                        warnings.to_string(),
                        egui::FontId::proportional(12.0),
                        theme.status_fg,
                    );
                    if resp.clicked() {
                        self.bottom_open = true;
                        self.bottom_tab = BottomTab::Problems;
                    }
                    resp.on_hover_text("Problems");

                    if !msg.is_empty() {
                        ui.label(egui::RichText::new(msg).color(theme.status_fg).size(12.0));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        let (br, bresp) =
                            ui.allocate_exact_size(egui::vec2(20.0, 18.0), egui::Sense::click());
                        if bresp.hovered() {
                            ui.painter().rect_filled(
                                br,
                                0.0,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 31),
                            );
                        }
                        icons::codicon(ui.painter(), br.center(), ci::BELL, 13.0, theme.status_fg);
                        bresp.on_hover_text("Notifications");

                        status_text(ui, &theme, lang.display(), "");
                        status_text(ui, &theme, "LF", "");
                        status_text(ui, &theme, "UTF-8", "");
                        status_text(ui, &theme, format!("Spaces: {}", self.tab_size), "");
                        if status_text(
                            ui,
                            &theme,
                            format!("Ln {line}, Col {col}"),
                            "Go to Line/Column…",
                        )
                        .clicked()
                        {
                            self.goto_input.clear();
                            self.show_goto = true;
                        }
                    });
                });
            });
    }

    /// Command palette modal anchored at the top, VS Code quick-pick style.
    fn show_palette(&mut self, ctx: &egui::Context) {
        if !self.palette_open {
            return;
        }
        let theme = self.theme.clone();
        let screen = ctx.screen_rect();
        egui::Area::new(egui::Id::new("palette-backdrop"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (r, resp) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
                ui.painter()
                    .rect_filled(r, 0.0, egui::Color32::from_black_alpha(140));
                if resp.clicked() {
                    self.palette_open = false;
                }
            });

        egui::Window::new("command_palette_modal")
            .title_bar(false)
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 48.0])
            .resizable(false)
            .fixed_size([560.0, 420.0])
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(theme.widget_bg)
                    .stroke(egui::Stroke::new(1.0, theme.widget_border))
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::same(6)),
            )
            .show(ctx, |ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.palette_query)
                        .hint_text("file name, >command, :line, @symbol…")
                        .desired_width(f32::INFINITY),
                );
                resp.request_focus();
                if resp.changed() {
                    self.palette_selected = 0;
                }
                ui.add_space(4.0);

                use crate::search::{fuzzy_score, QuickMode};
                let mode = crate::search::parse_query(self.palette_query.trim());
                // Rows: (label, sub, action)
                enum Act {
                    Cmd(&'static str),
                    Open(PathBuf),
                    OpenAt(PathBuf, usize, Option<usize>),
                    Goto(usize, Option<usize>),
                    Sym(usize),
                }
                let mut rows: Vec<(String, String, Act)> = Vec::new();
                match &mode {
                    QuickMode::Commands(q) => {
                        let ql = q.to_lowercase();
                        for (cat, title, shortcut) in PALETTE_ITEMS.iter().filter(|(c, t, _)| {
                            ql.is_empty()
                                || c.to_lowercase().contains(&ql)
                                || t.to_lowercase().contains(&ql)
                        }) {
                            rows.push((
                                format!("{cat}: {title}"),
                                shortcut.to_string(),
                                Act::Cmd(title),
                            ));
                        }
                    }
                    QuickMode::Goto { line, col } => {
                        rows.push((
                            format!("Go to line {line}"),
                            String::new(),
                            Act::Goto(*line, *col),
                        ));
                    }
                    QuickMode::Symbols(q) => {
                        if let Some(active) = self.active.clone() {
                            let lang = self.active_language();
                            let text = self
                                .workspace
                                .as_ref()
                                .and_then(|w| w.get(&active))
                                .map(|b| b.text())
                                .unwrap_or_default();
                            let ql = q.to_lowercase();
                            for (label, line) in crate::highlight::symbols(&text, lang)
                                .into_iter()
                                .filter(|(l, _)| ql.is_empty() || l.to_lowercase().contains(&ql))
                            {
                                rows.push((label, String::new(), Act::Sym(line)));
                            }
                        }
                    }
                    QuickMode::FileAt { path, line, col } => {
                        let mut v: Vec<(i64, PathBuf)> = self
                            .file_list
                            .iter()
                            .filter_map(|p| {
                                fuzzy_score(path, &p.to_string_lossy()).map(|s| (s, p.clone()))
                            })
                            .collect();
                        v.sort_by_key(|(s, _)| -*s);
                        let (l, c) = (*line, *col);
                        for (_, p) in v.into_iter().take(30) {
                            let disp = p.to_string_lossy().into_owned();
                            rows.push((
                                disp.clone(),
                                l.map(|n| format!(":{n}")).unwrap_or_default(),
                                Act::OpenAt(p, l.unwrap_or(1), c),
                            ));
                        }
                    }
                    QuickMode::Files(q) => {
                        let mut v: Vec<(i64, PathBuf)> = self
                            .file_list
                            .iter()
                            .filter_map(|p| {
                                fuzzy_score(q, &p.to_string_lossy()).map(|s| (s, p.clone()))
                            })
                            .collect();
                        v.sort_by_key(|(s, _)| -*s);
                        for (_, p) in v.into_iter().take(30) {
                            rows.push((
                                p.to_string_lossy().into_owned(),
                                String::new(),
                                Act::Open(p),
                            ));
                        }
                    }
                }

                if !rows.is_empty() {
                    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        self.palette_selected = (self.palette_selected + 1) % rows.len();
                    } else if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        self.palette_selected =
                            (self.palette_selected + rows.len() - 1) % rows.len();
                    } else if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let (_, _, act) = &rows[self.palette_selected.min(rows.len() - 1)];
                        match act {
                            Act::Cmd(t) => self.run_palette_item(t),
                            Act::Open(p) => {
                                let abs = self.cwd().join(p);
                                self.open_file(&abs);
                            }
                            Act::OpenAt(p, l, c) => {
                                let abs = self.cwd().join(p);
                                self.goto_location_col(&abs, *l, c.unwrap_or(1));
                            }
                            Act::Goto(l, c) => {
                                if let Some(active) = self.active.clone() {
                                    self.goto_location_col(&active, *l, c.unwrap_or(1));
                                }
                            }
                            Act::Sym(line) => {
                                if let Some(active) = self.active.clone() {
                                    self.goto_location(&active, *line);
                                }
                            }
                        }
                        self.palette_open = false;
                        return;
                    }
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.palette_open = false;
                    return;
                }

                egui::ScrollArea::vertical()
                    .id_salt("palette-scroll")
                    .max_height(360.0)
                    .show(ui, |ui| {
                        for (idx, (label, sub, _)) in rows.iter().enumerate() {
                            let is_sel = idx == self.palette_selected;
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 24.0),
                                egui::Sense::click(),
                            );
                            if resp.hovered() {
                                self.palette_selected = idx;
                            }
                            let p = ui.painter_at(rect);
                            if is_sel {
                                p.rect_filled(rect, 4.0, theme.list_focus);
                                if resp.clicked()
                                    || (is_sel && ctx.input(|i| i.key_pressed(egui::Key::Enter)))
                                {
                                }
                            }
                            p.text(
                                egui::pos2(rect.left() + 8.0, rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                label,
                                egui::FontId::proportional(12.0),
                                theme.text,
                            );
                            if !sub.is_empty() {
                                p.text(
                                    egui::pos2(rect.right() - 8.0, rect.center().y),
                                    egui::Align2::RIGHT_CENTER,
                                    sub,
                                    egui::FontId::proportional(11.0),
                                    theme.dim,
                                );
                            }
                            if resp.clicked() {
                                self.palette_selected = idx;
                                // synthesize Enter
                                match &rows[idx].2 {
                                    Act::Cmd(t) => self.run_palette_item(t),
                                    Act::Open(p) => {
                                        let abs = self.cwd().join(p);
                                        self.open_file(&abs);
                                    }
                                    Act::OpenAt(p, l, c) => {
                                        let abs = self.cwd().join(p);
                                        self.goto_location_col(&abs, *l, c.unwrap_or(1));
                                    }
                                    Act::Goto(l, c) => {
                                        if let Some(active) = self.active.clone() {
                                            self.goto_location_col(&active, *l, c.unwrap_or(1));
                                        }
                                    }
                                    Act::Sym(line) => {
                                        if let Some(active) = self.active.clone() {
                                            self.goto_location(&active, *line);
                                        }
                                    }
                                }
                                self.palette_open = false;
                            }
                        }
                    });
            });
    }

    fn show_dialogs(&mut self, ctx: &egui::Context) {
        let theme = self.theme.clone();
        if self.show_snippets {
            let mut open = true;
            egui::Window::new("Insert Snippet").open(&mut open).collapsible(false).default_width(550.0)
                .show(ctx, |ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.snippet_query).hint_text("Search snippet name or prefix…").desired_width(f32::INFINITY));
                    let language = self.active.as_deref().map(crate::snippets::language_id).unwrap_or_default();
                    let query = self.snippet_query.to_lowercase();
                    ui.label(format!("Language: {language}"));
                    egui::ScrollArea::vertical().max_height(450.0).show(ui, |ui| {
                        for ext in &self.extensions {
                            for snippet in &ext.snippets {
                                if snippet.language != language || !format!("{} {}", snippet.label, snippet.prefix).to_lowercase().contains(&query) { continue; }
                                if ui.button(format!("{}   {}", snippet.prefix, snippet.label)).on_hover_text(&snippet.body).clicked() {
                                    self.pending_snippet = Some(snippet.clone()); self.show_snippets = false;
                                }
                            }
                        }
                    });
                    ui.label("Load a snippet extension in Extensions, or import compatible packages from VS Code.");
                });
            if !open {
                self.show_snippets = false;
            }
        }
        if let Some(action) = self.unsaved_action.clone() {
            let dirty: Vec<PathBuf> = self
                .workspace
                .as_ref()
                .map(|w| w.dirty_paths())
                .unwrap_or_default()
                .into_iter()
                .filter(|p| match &action {
                    UnsavedAction::CloseTabs(paths) => paths.contains(p),
                    _ => true,
                })
                .collect();
            let mut open = true;
            egui::Window::new("Save Changes?")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(format!("Save changes to {} file(s)?", dirty.len()));
                    for path in &dirty {
                        ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                    }
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Save", &theme).clicked() {
                            let result = self
                                .workspace
                                .as_mut()
                                .map(|w| dirty.iter().try_for_each(|p| w.save(p)))
                                .unwrap_or(Ok(()));
                            match result {
                                Ok(()) => {
                                    self.unsaved_action = None;
                                    self.finish_unsaved_action(action.clone());
                                }
                                Err(error) => self.status(format!("Save failed: {error}")),
                            }
                        }
                        if secondary_button(ui, "Don't Save", &theme).clicked() {
                            self.unsaved_action = None;
                            self.finish_unsaved_action(action.clone());
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.unsaved_action = None;
                        }
                    });
                });
            if !open {
                self.unsaved_action = None;
            }
        }
        if self.show_add_extension {
            let mut open = true;
            egui::Window::new("Load VS Code Extension")
                .open(&mut open).collapsible(false).resizable(false).default_width(480.0)
                .show(ctx, |ui| {
                    ui.label("VSIX file or extension folder:");
                    let input = ui.add(egui::TextEdit::singleline(&mut self.extension_path).desired_width(f32::INFINITY));
                    ui.label("Loads JSON color themes directly from the package. Keep the package at this path for future sessions.");
                    if let Some(error) = &self.extension_error { ui.colored_label(theme.error, error); }
                    if primary_button(ui, "Load Extension", &theme).clicked()
                        || (input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        let path = PathBuf::from(self.extension_path.trim());
                        self.load_extension(&path);
                    }
                });
            if !open {
                self.show_add_extension = false;
            }
        }
        if self.show_open_folder {
            let mut open = true;
            egui::Window::new("Open Folder")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(520.0)
                .show(ctx, |ui| {
                    ui.label("Select a folder to open:");
                    ui.add_space(4.0);
                    if primary_button(ui, "Browse… (native folder chooser)", &theme).clicked() {
                        self.show_open_folder = false;
                        self.pick_and_open_folder();
                        return;
                    }
                    ui.add_space(6.0);
                    ui.separator();
                    ui.label(
                        egui::RichText::new("Recent folders")
                            .color(theme.dim)
                            .size(11.0),
                    );
                    let recents = self.recent_folders.clone();
                    for p in recents.iter().take(6) {
                        if ui.button(p.to_string_lossy().as_ref()).clicked() {
                            let p = p.clone();
                            self.show_open_folder = false;
                            self.open_folder(&p);
                            return;
                        }
                    }
                    ui.add_space(4.0);
                    ui.separator();
                    ui.label("Or type / paste a folder path:");
                    let resp = ui.text_edit_singleline(&mut self.open_folder_input);
                    let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Open", &theme).clicked() || enter {
                            let p = PathBuf::from(self.open_folder_input.trim());
                            self.show_open_folder = false;
                            self.open_folder(&p);
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.show_open_folder = false;
                        }
                    });
                    if !crate::file_dialog::native_available() {
                        ui.label(
                            egui::RichText::new(
                                "Tip: install zenity or kdialog for the native chooser.",
                            )
                            .color(theme.dim)
                            .size(11.0),
                        );
                    }
                });
            if !open {
                self.show_open_folder = false;
            }
        }

        // Stash dialog (VS Code: Stash Changes…)
        if self.show_stash_dialog {
            let mut open = true;
            egui::Window::new("Stash Changes")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(440.0)
                .show(ctx, |ui| {
                    ui.label("Stash message (optional):");
                    ui.text_edit_singleline(&mut self.stash_input);
                    ui.checkbox(&mut self.stash_include_untracked, "Include untracked files");
                    ui.checkbox(
                        &mut self.stash_keep_index,
                        "Keep staged changes in the working tree (--keep-index)",
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Stash", &theme).clicked() {
                            if let Some(snap) = &self.git.snapshot {
                                let root = snap.root.clone();
                                let msg = if self.stash_input.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.stash_input.clone())
                                };
                                self.git.stash_push(
                                    &root,
                                    msg,
                                    self.stash_include_untracked,
                                    self.stash_keep_index,
                                    vec![],
                                );
                            }
                            self.show_stash_dialog = false;
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.show_stash_dialog = false;
                        }
                    });
                });
            if !open {
                self.show_stash_dialog = false;
            }
        }

        // Create Branch dialog (VS Code: Create Branch…)
        if self.show_branch_dialog {
            let mut open = true;
            egui::Window::new("Create Branch")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(440.0)
                .show(ctx, |ui| {
                    ui.label("New branch name:");
                    let resp = ui.text_edit_singleline(&mut self.branch_input);
                    resp.request_focus();
                    let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if (primary_button(ui, "Create and Checkout", &theme).clicked() || enter)
                            && !self.branch_input.trim().is_empty()
                        {
                            if let Some(snap) = &self.git.snapshot {
                                let root = snap.root.clone();
                                self.git.create_branch(
                                    &root,
                                    self.branch_input.trim().to_string(),
                                    true,
                                );
                            }
                            self.show_branch_dialog = false;
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.show_branch_dialog = false;
                        }
                    });
                });
            if !open {
                self.show_branch_dialog = false;
            }
        }

        // Git confirmations (discard all / clean / drop stash / delete branch / undo)
        if let Some(confirm) = self.confirm_git.clone() {
            use GitConfirm as G;
            let (title, body) = match &confirm {
                G::DiscardAll => (
                    "Discard All Changes",
                    "Discard all staged and unstaged tracked changes? Untracked files are kept."
                        .to_string(),
                ),
                G::DiscardAllTracked => (
                    "Discard All Tracked Changes",
                    "Discard all unstaged tracked changes? Staged changes are kept.".to_string(),
                ),
                G::CleanAll => (
                    "Clean All Untracked Files",
                    "Permanently delete all untracked files? This cannot be undone.".to_string(),
                ),
                G::DiscardStaged(p) => (
                    "Discard Staged Changes",
                    format!("Discard staged + worktree changes to '{}'?", p.display()),
                ),
                G::Clean(p) => (
                    "Clean Untracked File",
                    format!("Permanently delete untracked file '{}'?", p.display()),
                ),
                G::DropStash(i) => (
                    "Drop Stash",
                    format!("Drop stash@{{{i}}}? This cannot be undone."),
                ),
                G::DeleteBranch(b) => ("Delete Branch", format!("Delete branch '{b}'?")),
                G::UndoCommit => (
                    "Undo Last Commit",
                    "Undo the last commit? Changes stay staged (--soft).".to_string(),
                ),
            };
            let mut open = true;
            egui::Window::new(title)
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(body);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Confirm", &theme).clicked() {
                            if let Some(snap) = self.git.snapshot.clone() {
                                let root = snap.root.clone();
                                match &confirm {
                                    G::DiscardAll => {
                                        // Tracked only: restore worktree + unstage tracked.
                                        let paths: Vec<PathBuf> = snap
                                            .entries
                                            .iter()
                                            .filter(|e| !e.is_untracked())
                                            .flat_map(|e| {
                                                let mut v = vec![e.path.clone()];
                                                if let Some(o) = &e.original_path {
                                                    v.push(o.clone());
                                                }
                                                v
                                            })
                                            .collect();
                                        self.git.unstage(&root, paths.clone(), snap.has_head);
                                        self.git.discard_all(&root, paths);
                                    }
                                    G::DiscardAllTracked => {
                                        let paths: Vec<PathBuf> = snap
                                            .changed()
                                            .filter(|e| !e.is_untracked())
                                            .map(|e| e.path.clone())
                                            .collect();
                                        self.git.discard_all(&root, paths);
                                    }
                                    G::CleanAll => {
                                        let paths: Vec<PathBuf> =
                                            snap.untracked().map(|e| e.path.clone()).collect();
                                        self.git.clean(&root, paths);
                                    }
                                    G::DiscardStaged(p) => {
                                        self.git.discard_staged(&root, p.clone(), snap.has_head)
                                    }
                                    G::Clean(p) => self.git.clean(&root, vec![p.clone()]),
                                    G::DropStash(i) => self.git.stash_drop(&root, *i),
                                    G::DeleteBranch(b) => {
                                        self.git.delete_branch(&root, b.clone(), false)
                                    }
                                    G::UndoCommit => self.git.undo_commit(&root),
                                }
                            }
                            self.confirm_git = None;
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.confirm_git = None;
                        }
                    });
                });
            if !open {
                self.confirm_git = None;
            }
        }

        if self.show_new_project {
            let mut open = true;
            egui::Window::new("New Rust Project")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Project name:");
                    ui.text_edit_singleline(&mut self.new_proj_name);
                    ui.checkbox(
                        &mut self.new_proj_lib,
                        "Library crate (lib.rs instead of main.rs)",
                    );
                    ui.label(
                        egui::RichText::new(format!("Location: {}", self.cwd().display()))
                            .color(theme.dim)
                            .small(),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Create", &theme).clicked()
                            && !self.new_proj_name.trim().is_empty()
                        {
                            let name = self.new_proj_name.trim().to_string();
                            let cwd = self.cwd();
                            match Workspace::new_rust_project(&cwd, &name, self.new_proj_lib) {
                                Ok(root) => {
                                    self.open_folder(&root);
                                    let main = if self.new_proj_lib {
                                        "src/lib.rs"
                                    } else {
                                        "src/main.rs"
                                    };
                                    let mp = root.join(main);
                                    self.open_file(&mp);
                                    self.status(format!("Created Rust project {name}"));
                                }
                                Err(e) => self.status(format!("Cannot create project: {e}")),
                            }
                            self.show_new_project = false;
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.show_new_project = false;
                        }
                    });
                });
            if !open {
                self.show_new_project = false;
            }
        }

        if let Some(target) = self.confirm_delete.clone() {
            let mut open = true;
            let name = target
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            egui::Window::new("Delete")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to permanently delete '{name}'?"
                    ));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Delete", &theme).clicked() {
                            if let Some(ws) = self.workspace.as_mut() {
                                match ws.delete(&target) {
                                    Ok(()) => {
                                        self.close_tab(&target);
                                        self.tabs.retain(|t| !t.starts_with(&target));
                                        if self
                                            .active
                                            .as_ref()
                                            .is_some_and(|a| a.starts_with(&target) || a == &target)
                                        {
                                            self.active = self.tabs.last().cloned();
                                            self.editor_for = None;
                                        }
                                        self.status(format!("Deleted {name}"));
                                    }
                                    Err(e) => self.status(format!("Delete failed: {e}")),
                                }
                            }
                            self.confirm_delete = None;
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.confirm_delete = None;
                        }
                    });
                });
            if !open {
                self.confirm_delete = None;
            }
        }

        // Discard-confirmation dialog (E): names the file, explains the loss.
        if self.confirm_discard.is_some() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.confirm_discard = None;
        }
        if let Some(target) = self.confirm_discard.clone() {
            let mut open = true;
            let name = target
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            egui::Window::new("Discard Changes")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Discard all changes to '{name}'?"));
                    ui.label(
                        egui::RichText::new(
                            "Both the on-disk edits and any unsaved buffer content will be lost.",
                        )
                        .color(theme.dim)
                        .size(11.5),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Discard", &theme).clicked() {
                            if let Some(snap) = &self.git.snapshot {
                                self.git.discard(&snap.root.clone(), target.clone());
                            }
                            self.confirm_discard = None;
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.confirm_discard = None;
                        }
                    });
                });
            if !open {
                self.confirm_discard = None;
            }
        }

        // Replace-all explicit preview-count confirmation.
        if let Some((res, _repl)) = self.replace_confirm.clone() {
            let mut open = true;
            let n = res.total_matches();
            let f = res.files.len();
            egui::Window::new("Replace All")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Replace {n} occurrences in {f} files?"));
                    ui.label(
                        egui::RichText::new(
                            "Changes apply to editor buffers only — save to write to disk.",
                        )
                        .color(theme.dim)
                        .size(11.5),
                    );
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Replace", &theme).clicked() {
                            match crate::search::Matcher::new(&res.options).map(|m| {
                                self.workspace.as_mut().map(|ws| {
                                    ws.apply_replacements(&res, &m, &_repl, res.options.regex)
                                })
                            }) {
                                Ok(Some(Ok((n, paths)))) => {
                                    for p in &paths {
                                        if !self.tabs.contains(p) {
                                            self.tabs.push(p.clone());
                                        }
                                    }
                                    if let Some(active) = self.active.clone() {
                                        if paths.contains(&active) {
                                            if let Some(ws) = self.workspace.as_ref() {
                                                if let Some(b) = ws.get(&active) {
                                                    self.editor_text = b.text();
                                                }
                                            }
                                            self.editor_for = None;
                                        }
                                    }
                                    self.status(format!("Replaced {n}"));
                                }
                                Ok(Some(Err(e))) | Err(e) => self.status(format!("replace: {e}")),
                                _ => {}
                            }
                            self.replace_confirm = None;
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.replace_confirm = None;
                        }
                    });
                });
            if !open {
                self.replace_confirm = None;
            }
        }

        if self.show_goto {
            let mut open = true;
            egui::Window::new("Go to Line/Column")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Line number:");
                    let resp = ui.text_edit_singleline(&mut self.goto_input);
                    resp.request_focus();
                    let submit = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Go", &theme).clicked() || submit {
                            if let Ok(n) = self.goto_input.trim().parse::<usize>() {
                                if let Some(active) = self.active.clone() {
                                    self.pending_goto = Some((active, n.max(1)));
                                }
                            }
                            self.show_goto = false;
                        }
                        if secondary_button(ui, "Cancel", &theme).clicked() {
                            self.show_goto = false;
                        }
                    });
                });
            if !open {
                self.show_goto = false;
            }
        }

        if self.show_about {
            let mut open = true;
            egui::Window::new("About")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading(egui::RichText::new("OxideCode").size(16.0));
                    ui.label("A pure-Rust native desktop clone of VS Code.");
                    ui.label(
                        egui::RichText::new(format!("Version: 0.1.0 · Theme: {}", self.theme.name))
                            .color(theme.dim)
                            .small(),
                    );
                });
            if !open {
                self.show_about = false;
            }
        }
    }
}

/// Menu popup styling: transparent rows, accent hover with white text.
pub(crate) fn menu_style_accent(ui: &mut egui::Ui, theme: &Theme) {
    let v = &mut ui.style_mut().visuals;
    v.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    v.widgets.hovered.weak_bg_fill = theme.accent;
    v.widgets.hovered.bg_fill = theme.accent;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    v.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    v.widgets.active.weak_bg_fill = theme.accent;
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
}

fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: &str) -> egui::Response {
    let btn = egui::Button::new(egui::RichText::new(label).size(13.0))
        .shortcut_text(egui::RichText::new(shortcut).size(12.0))
        .frame_when_inactive(false)
        .min_size(egui::vec2(ui.available_width(), 22.0));
    ui.add(btn)
}

fn text_w(ui: &egui::Ui, s: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(
            s.to_string(),
            egui::FontId::proportional(size),
            egui::Color32::WHITE,
        )
        .size()
        .x
}

fn status_text(
    ui: &mut egui::Ui,
    theme: &Theme,
    text: impl Into<String>,
    tip: &str,
) -> egui::Response {
    let text = text.into();
    let g = ui
        .painter()
        .layout_no_wrap(text, egui::FontId::proportional(12.0), theme.status_fg);
    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(g.size().x + 10.0, 18.0), egui::Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 31),
        );
    }
    ui.painter().galley(
        egui::pos2(rect.left() + 5.0, rect.center().y - g.size().y / 2.0),
        g,
        theme.status_fg,
    );
    if !tip.is_empty() {
        resp.clone().on_hover_text(tip);
    }
    resp
}

fn status_icon_text(
    ui: &mut egui::Ui,
    theme: &Theme,
    glyph: char,
    text: &str,
    tip: &str,
) -> egui::Response {
    let g = ui.painter().layout_no_wrap(
        text.to_string(),
        egui::FontId::proportional(12.0),
        theme.status_fg,
    );
    let w = 14.0 + 4.0 + g.size().x + 10.0;
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 18.0), egui::Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(
            rect,
            0.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 31),
        );
    }
    let p = ui.painter_at(rect);
    icons::codicon(
        &p,
        egui::pos2(rect.left() + 10.0, rect.center().y),
        glyph,
        12.0,
        theme.status_fg,
    );
    p.galley(
        egui::pos2(rect.left() + 18.0, rect.center().y - g.size().y / 2.0),
        g,
        theme.status_fg,
    );
    if !tip.is_empty() {
        resp.clone().on_hover_text(tip);
    }
    resp
}

fn primary_button(ui: &mut egui::Ui, label: &str, theme: &Theme) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .color(egui::Color32::WHITE)
                .size(13.0),
        )
        .fill(theme.accent)
        .corner_radius(2.0),
    )
}

fn secondary_button(ui: &mut egui::Ui, label: &str, theme: &Theme) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(theme.text).size(13.0))
            .fill(theme.input_bg)
            .corner_radius(2.0),
    )
}

impl eframe::App for OxideApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_style(ctx);
        if !self.quit_confirmed
            && ctx.input(|i| i.viewport().close_requested())
            && self
                .workspace
                .as_ref()
                .is_some_and(|w| !w.dirty_paths().is_empty())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.unsaved_action = Some(UnsavedAction::Quit);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !self.palette_open {
            self.show_open_folder = false;
            self.show_new_project = false;
            self.show_add_extension = false;
            self.confirm_delete = None;
            self.show_about = false;
            self.show_goto = false;
            self.explorer_edit = None;
            self.unsaved_action = None;
            self.show_snippets = false;
            self.find.open = false;
        }
        self.handle_shortcuts(ctx);
        self.terminal.poll();
        self.pty.poll_events();
        if let Some(rx) = &self.search_rx {
            if let Ok((gen, res)) = rx.try_recv() {
                self.search_running = false;
                self.search_rx = None;
                if gen == self.search_gen {
                    match res {
                        Ok(r) => self.search_results = Some(r),
                        Err(e) => {
                            self.search_error = Some(e);
                            self.search_results = None;
                        }
                    }
                }
            }
        }
        if let Some(rx) = &self.file_list_rx {
            if let Ok(v) = rx.try_recv() {
                self.file_list = v;
                self.file_list_rx = None;
            }
        }
        // Once-per-frame terminal focus resolution BEFORE any header/menu
        // actions so deliberate focus changes made this frame survive.
        if self.bottom_open
            && self.bottom_tab == BottomTab::Terminal
            && ctx.input(|i| i.pointer.primary_clicked())
        {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                match click_focus(&self.term_pane_rects, pos) {
                    Ok(id) => self.pty.focus_pane(id),
                    Err(()) => self.pty.unfocus(),
                }
            }
        }
        // Git: refresh status on a timer and after the worker finishes.
        if self.workspace.is_some() {
            if self.git_refresh.elapsed() > std::time::Duration::from_secs(2) {
                if let Some(ws) = &self.workspace {
                    self.git.request_status(ws.root.clone());
                }
                self.git_refresh = std::time::Instant::now();
            }
            ctx.request_repaint_after(std::time::Duration::from_secs(2));
        }
        while let Some(res) = self.git.poll() {
            match res {
                crate::git::GitResult::Snapshot(s) => match *s {
                    Ok(snap) => {
                        self.git.last_error = None;
                        self.git_branch = Some(snap.branch.clone());
                        if let Some(active) = self.active.clone() {
                            if let Ok(rel) = active.strip_prefix(&snap.root) {
                                if self.git_index_pending.insert(active.clone()) {
                                    self.git.request_index(&snap.root, rel.to_path_buf());
                                }
                            }
                        }
                        self.git.snapshot = Some(snap);
                    }
                    Err(e) => {
                        self.git.last_error = Some(e);
                        self.git.snapshot = None;
                    }
                },
                crate::git::GitResult::Action { kind, ok, message } => {
                    use crate::git::ActionKind;
                    if !ok {
                        self.status(format!("git: {message}"));
                    } else {
                        match &kind {
                            // Clear the box only if the submitted message
                            // is still what's typed there.
                            ActionKind::Commit { message } => {
                                if self.scm_msg == *message {
                                    self.scm_msg.clear();
                                }
                            }
                            // Successful discard reloads only that buffer.
                            ActionKind::Discard { path } => {
                                let abs = self
                                    .git
                                    .snapshot
                                    .as_ref()
                                    .map(|s| s.root.join(path))
                                    .unwrap_or_else(|| path.clone());
                                if let (Some(ws), Ok(text)) =
                                    (self.workspace.as_mut(), std::fs::read_to_string(&abs))
                                {
                                    if let Some(buf) = ws.get_mut(&abs) {
                                        buf.set_text(&text);
                                        buf.mark_saved();
                                    }
                                    if self.active.as_ref() == Some(&abs) {
                                        self.editor_text = text;
                                        self.editor_for = Some(abs);
                                    }
                                }
                            }
                            // Anything that rewrites the worktree/index
                            // outside the editor: resync all open buffers.
                            ActionKind::DiscardAll
                            | ActionKind::DiscardStaged { .. }
                            | ActionKind::Clean { .. }
                            | ActionKind::CleanAll
                            | ActionKind::StashPush
                            | ActionKind::StashApply { .. }
                            | ActionKind::Pull
                            | ActionKind::Sync
                            | ActionKind::Checkout { .. }
                            | ActionKind::MergeBranch { .. }
                            | ActionKind::UndoCommit => {
                                if !message.is_empty() {
                                    self.status(message.clone());
                                }
                                self.reload_all_buffers();
                            }
                            _ => {
                                if !message.is_empty()
                                    && matches!(
                                        kind,
                                        ActionKind::Fetch
                                            | ActionKind::Push
                                            | ActionKind::Publish
                                            | ActionKind::CreateBranch { .. }
                                    )
                                {
                                    self.status(message.clone());
                                }
                            }
                        }
                    }
                    if let Some(ws) = &self.workspace {
                        self.git.request_status(ws.root.clone());
                    }
                }
                crate::git::GitResult::Diff { diff, .. } => {
                    self.git_diff = diff.ok();
                }
                crate::git::GitResult::Index { path, text } => {
                    self.git_index_pending.remove(&path);
                    self.git_index.insert(path, text);
                }
            }
        }
        if self.index_active != self.active {
            self.index_active = self.active.clone();
            if let (Some(active), Some(snap)) = (&self.active, &self.git.snapshot) {
                if let Ok(rel) = active.strip_prefix(&snap.root) {
                    if self.git_index_pending.insert(active.clone()) {
                        let root = snap.root.clone();
                        self.git.request_index(&root, rel.to_path_buf());
                    }
                }
            }
        }
        // A revision catches same-length edits, and the path prevents stale
        // markers when switching between equally sized files.
        if let Some(active) = &self.active {
            if let Some(buf) = self.workspace.as_ref().and_then(|w| w.get(active)) {
                let baseline = self.git_index.get(active).and_then(Option::as_ref);
                if !self
                    .gutter_source
                    .as_ref()
                    .is_some_and(|(path, base, revision)| {
                        path == active && base.as_ref() == baseline && *revision == buf.revision()
                    })
                {
                    self.gutter_marks = baseline.map(|base| {
                        let old: Vec<String> = base.split('\n').map(str::to_string).collect();
                        crate::ui_editor::gutter_markers(&old, buf.lines())
                    });
                    self.gutter_source = Some((active.clone(), baseline.cloned(), buf.revision()));
                }
            }
        }
        self.poll_diagnostics();
        self.update_title(ctx);

        self.show_menu(ctx);
        self.show_status(ctx);
        crate::ui_sidebar::show(self, ctx);
        crate::ui_bottom::show(self, ctx);
        crate::ui_editor::show(self, ctx);
        self.show_palette(ctx);
        self.show_dialogs(ctx);

        // Opt-in UI regression captures use the app's framebuffer, never
        // the rest of the desktop. Production sessions have no capture timer.
        if let Some((path, start, requested)) = &mut self.capture {
            if let Some(image) = ctx.input(|i| {
                i.events.iter().find_map(|event| match event {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            }) {
                let rgba: Vec<u8> = image
                    .pixels
                    .iter()
                    .flat_map(|pixel| pixel.to_srgba_unmultiplied())
                    .collect();
                if let Err(error) = image::save_buffer(
                    &*path,
                    &rgba,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                ) {
                    eprintln!("Capture failed: {error}");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                self.quit_confirmed = true;
            } else if !*requested && start.elapsed() > std::time::Duration::from_secs(2) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                *requested = true;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        if self.terminal.is_busy() || self.diag_busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

pub fn run(paths: Vec<PathBuf>) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 840.0])
            .with_min_inner_size([960.0, 620.0])
            .with_title("OxideCode"),
        ..Default::default()
    };
    eframe::run_native(
        "oxide-code",
        options,
        Box::new(move |cc| Ok(Box::new(OxideApp::new(cc, &paths)) as Box<dyn eframe::App>)),
    )
}
