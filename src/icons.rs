//! VS Code codicons rendered from the bundled `@vscode/codicons` font
//! (src/assets/codicon.ttf, v0.0.45), plus a few painter helpers and the
//! colored per-language file chips.
//!
//! Font license: src/assets/codicon-LICENSE.txt.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind};

/// Codepoints from @vscode/codicons 0.0.45 `dist/codicon.css`.
/// (Some constants are reserved for upcoming phases.)
#[allow(dead_code)]
pub mod ci {
    pub const FILES: char = '\u{eaf0}';
    pub const SEARCH: char = '\u{ea6d}';
    pub const SOURCE_CONTROL: char = '\u{ea68}';
    pub const DEBUG_ALT: char = '\u{eb91}';
    pub const EXTENSIONS: char = '\u{eae6}';
    pub const ACCOUNT: char = '\u{eb99}';
    pub const SETTINGS_GEAR: char = '\u{eb51}';
    pub const NEW_FILE: char = '\u{ea7f}';
    pub const NEW_FOLDER: char = '\u{ea80}';
    pub const REFRESH: char = '\u{eb37}';
    pub const COLLAPSE_ALL: char = '\u{eac5}';
    pub const ELLIPSIS: char = '\u{ea7c}';
    pub const CHEVRON_RIGHT: char = '\u{eab6}';
    pub const CHEVRON_DOWN: char = '\u{eab4}';
    pub const CHEVRON_UP: char = '\u{eab7}';
    pub const CLOSE: char = '\u{ea76}';
    pub const ADD: char = '\u{ea60}';
    pub const TRASH: char = '\u{ea81}';
    /// Codicons has no `kill` glyph in 0.0.45; VS Code uses the trash icon.
    pub const KILL: char = TRASH;
    pub const SPLIT_HORIZONTAL: char = '\u{eb56}';
    pub const TERMINAL: char = '\u{ea85}';
    pub const TERMINAL_BASH: char = '\u{ebca}';
    pub const LAYOUT_SIDEBAR_LEFT: char = '\u{ebf3}';
    pub const LAYOUT_SIDEBAR_LEFT_OFF: char = '\u{ec02}';
    pub const LAYOUT_PANEL: char = '\u{ebf2}';
    pub const LAYOUT_PANEL_OFF: char = '\u{ec01}';
    pub const ARROW_LEFT: char = '\u{ea9b}';
    pub const ARROW_RIGHT: char = '\u{ea9c}';
    pub const GIT_BRANCH: char = '\u{ec6f}';
    pub const SYNC: char = '\u{ea77}';
    pub const ERROR: char = '\u{ea87}';
    pub const WARNING: char = '\u{ea6c}';
    pub const INFO: char = '\u{ea74}';
    pub const BELL: char = '\u{eaa2}';
    pub const CHECK: char = '\u{eab2}';
    pub const DISCARD: char = '\u{eae2}';
    pub const CIRCLE_FILLED: char = '\u{ea71}';
    pub const FOLDER: char = '\u{ea83}';
    pub const FOLDER_OPENED: char = '\u{eaf7}';
    pub const FILE: char = '\u{ea7b}';
    pub const FILE_CODE: char = '\u{eae9}';
    pub const MARKDOWN: char = '\u{eb1d}';
    pub const JSON: char = '\u{eb0f}';
    pub const LOCK: char = '\u{ea75}';
    pub const GEAR: char = '\u{eaf8}';
    pub const SYMBOL_METHOD: char = '\u{ea8c}';
    pub const SYMBOL_CLASS: char = '\u{eb5b}';
    pub const SYMBOL_STRUCT: char = '\u{ea91}';
    pub const SYMBOL_VARIABLE: char = '\u{ea88}';
    pub const CASE_SENSITIVE: char = '\u{eab1}';
    pub const WHOLE_WORD: char = '\u{eb7e}';
    pub const REGEX: char = '\u{eb38}';
    pub const REPLACE: char = '\u{eb3d}';
    pub const REPLACE_ALL: char = '\u{eb3c}';
    pub const CLEAR_ALL: char = '\u{eabf}';
    pub const GO_TO_FILE: char = '\u{ea94}';
    pub const DIFF: char = '\u{eae1}';
    pub const GIT_COMMIT: char = '\u{eafc}';
    pub const DEBUG_START: char = '\u{ead3}';
    pub const PLAY: char = '\u{eb2c}';
    pub const COPY: char = '\u{ebcc}';
    pub const EDIT: char = '\u{ea73}';
    pub const LIST_TREE: char = '\u{eb86}';
    pub const LIST_FLAT: char = '\u{eb84}';
    pub const FILTER: char = '\u{eaf1}';
    pub const CLOUD_DOWNLOAD: char = '\u{eac2}';
    pub const LIGHTBULB: char = '\u{ea61}';
}

pub fn rect_outline(p: &egui::Painter, r: Rect, c: Color32, w: f32, rounding: f32) {
    p.rect_stroke(r, rounding, Stroke::new(w, c), StrokeKind::Inside);
}

/// Paint a codicon glyph centered at `center`.
pub fn codicon(p: &egui::Painter, center: Pos2, glyph: char, size: f32, color: Color32) {
    p.text(
        center,
        egui::Align2::CENTER_CENTER,
        glyph.to_string(),
        egui::FontId::new(size, egui::FontFamily::Name("codicon".into())),
        color,
    );
}

/// A 22x22 codicon button (VS Code toolbar icon) with hover background.
pub fn icon_button(
    ui: &mut egui::Ui,
    glyph: char,
    tooltip: &str,
    theme: &crate::theme::Theme,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
    if resp.hovered() {
        ui.painter().rect_filled(rect, 2.0, theme.list_hover);
    }
    codicon(ui.painter(), rect.center(), glyph, 16.0, theme.text);
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tooltip)
    });
    resp.on_hover_text(tooltip)
}

/// Draw a file icon based on filename/language: codicons for well-known
/// types, colored letter chips for the rest (as in the file tree).
pub fn draw_file_icon(p: &egui::Painter, r: Rect, filename: &str) {
    let lower = filename.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or(&lower);
    let size = r.height() + 2.0;
    let c = r.center();
    if name == ".gitignore" {
        codicon(
            p,
            c,
            ci::GIT_BRANCH,
            size,
            Color32::from_rgb(0xf1, 0x4e, 0x32),
        );
    } else if name.ends_with(".md") {
        codicon(
            p,
            c,
            ci::MARKDOWN,
            size,
            Color32::from_rgb(0x51, 0x9a, 0xba),
        );
    } else if name.ends_with(".lock") {
        codicon(p, c, ci::LOCK, size, Color32::from_rgb(0x8c, 0x8c, 0x8c));
    } else if name.ends_with(".json") {
        codicon(p, c, ci::JSON, size, Color32::from_rgb(0xcb, 0xcb, 0x41));
    } else if name.ends_with(".toml") || name == ".cargo" || name.ends_with(".cfg") {
        codicon(p, c, ci::GEAR, size, Color32::from_rgb(0x6d, 0x80, 0x86));
    } else {
        let lang = crate::languages::Language::from_path(filename);
        if lang == crate::languages::Language::Plain {
            codicon(p, c, ci::FILE, size, Color32::from_rgb(0x8c, 0x8c, 0x8c));
        } else {
            let (bg, fg) = language_color(lang);
            file_chip(p, r, bg, lang.badge(), fg);
        }
    }
}

/// Colored file-type chip used for the programming languages.
pub fn file_chip(p: &egui::Painter, r: Rect, bg: Color32, label: &str, fg: Color32) {
    p.rect_filled(r, 3.0, bg);
    p.text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(8.5),
        fg,
    );
}

/// Per-language chip colors (dark bg, bright fg).
pub fn language_color(lang: crate::languages::Language) -> (Color32, Color32) {
    use crate::languages::Language as L;
    match lang {
        L::Rust => (
            Color32::from_rgb(84, 44, 34),
            Color32::from_rgb(255, 150, 110),
        ),
        L::Python => (
            Color32::from_rgb(38, 62, 92),
            Color32::from_rgb(140, 190, 255),
        ),
        L::JavaScript => (
            Color32::from_rgb(92, 82, 30),
            Color32::from_rgb(255, 225, 110),
        ),
        L::TypeScript => (
            Color32::from_rgb(30, 62, 104),
            Color32::from_rgb(120, 180, 255),
        ),
        L::Go => (
            Color32::from_rgb(28, 84, 96),
            Color32::from_rgb(110, 220, 240),
        ),
        L::C | L::Cpp => (
            Color32::from_rgb(44, 62, 104),
            Color32::from_rgb(150, 190, 255),
        ),
        L::CSharp => (
            Color32::from_rgb(70, 44, 104),
            Color32::from_rgb(200, 160, 255),
        ),
        L::Java => (
            Color32::from_rgb(96, 58, 30),
            Color32::from_rgb(255, 180, 110),
        ),
        L::Kotlin => (
            Color32::from_rgb(84, 48, 110),
            Color32::from_rgb(210, 160, 255),
        ),
        L::Swift => (
            Color32::from_rgb(100, 56, 32),
            Color32::from_rgb(255, 170, 110),
        ),
        L::Ruby => (
            Color32::from_rgb(96, 34, 40),
            Color32::from_rgb(255, 140, 140),
        ),
        L::Php => (
            Color32::from_rgb(56, 56, 100),
            Color32::from_rgb(170, 170, 255),
        ),
        L::Html => (
            Color32::from_rgb(100, 54, 30),
            Color32::from_rgb(255, 170, 110),
        ),
        L::Css => (
            Color32::from_rgb(30, 74, 110),
            Color32::from_rgb(120, 200, 255),
        ),
        L::Json => (
            Color32::from_rgb(90, 78, 34),
            Color32::from_rgb(240, 210, 120),
        ),
        L::Toml => (
            Color32::from_rgb(88, 60, 40),
            Color32::from_rgb(255, 190, 130),
        ),
        L::Yaml => (
            Color32::from_rgb(92, 40, 50),
            Color32::from_rgb(255, 150, 160),
        ),
        L::Markdown => (
            Color32::from_rgb(40, 70, 110),
            Color32::from_rgb(140, 200, 255),
        ),
        L::Shell => (
            Color32::from_rgb(40, 84, 54),
            Color32::from_rgb(140, 240, 170),
        ),
        L::Sql => (
            Color32::from_rgb(44, 78, 98),
            Color32::from_rgb(140, 210, 250),
        ),
        L::Xml => (
            Color32::from_rgb(96, 64, 34),
            Color32::from_rgb(255, 190, 120),
        ),
        L::Dockerfile => (
            Color32::from_rgb(30, 78, 104),
            Color32::from_rgb(120, 200, 255),
        ),
        L::Plain => (
            Color32::from_rgb(66, 70, 80),
            Color32::from_rgb(190, 195, 208),
        ),
    }
}
