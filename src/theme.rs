//! Application color themes matching the `.aistudio` design specification.
//! All rendering goes through these colors; there is no CSS and no web technology.

use eframe::egui;

/// Identifies one of the built-in color themes.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ThemeKind {
    #[default]
    RustOxide,
    DarkModern,
    TokyoNight,
    Catppuccin,
    Monokai,
}

impl ThemeKind {
    pub fn all() -> &'static [(&'static str, ThemeKind)] {
        &[
            ("Rust Oxide Dark", ThemeKind::RustOxide),
            ("VS Code Dark Modern", ThemeKind::DarkModern),
            ("Tokyo Night", ThemeKind::TokyoNight),
            ("Catppuccin Mocha", ThemeKind::Catppuccin),
            ("Monokai Pro", ThemeKind::Monokai),
        ]
    }

    pub fn name(self) -> &'static str {
        Self::all()
            .iter()
            .find(|(_, k)| *k == self)
            .map(|(n, _)| *n)
            .unwrap_or("Rust Oxide Dark")
    }

    pub fn colors(self) -> Theme {
        match self {
            // Exact palette from .aistudio `rust-oxide`:
            // windowBg: #15141b, titleBarBg: #111015, activityBarBg: #0e0d12,
            // sidebarBg: #15141b, editorBg: #17161f, statusBarBg: #ea580c,
            // tabActiveBg: #17161f, tabInactiveBg: #111015, panelBg: #111015,
            // border: #2a2736, accent: #ea580c, textPrimary: #e4e4e7, textMuted: #71717a
            ThemeKind::RustOxide => Theme {
                name: "Rust Oxide Dark",
                bg: rgb(0x17, 0x16, 0x1f),
                panel: rgb(0x11, 0x10, 0x15),
                sidebar: rgb(0x15, 0x14, 0x1b),
                activity: rgb(0x0e, 0x0d, 0x12),
                titlebar: rgb(0x11, 0x10, 0x15),
                tab_bg: rgb(0x11, 0x10, 0x15),
                tab_active: rgb(0x17, 0x16, 0x1f),
                status: rgb(0xea, 0x58, 0x0c),
                status_fg: rgb(0xff, 0xff, 0xff),
                pill: rgb(0xea, 0x58, 0x0c),
                border: rgb(0x2a, 0x27, 0x36),
                text: rgb(0xe4, 0xe4, 0xe7),
                dim: rgb(0xa1, 0xa1, 0xaa),
                gutter: rgb(0x52, 0x52, 0x5b),
                accent: rgb(0xea, 0x58, 0x0c),
                line_hi: rgba(0x27, 0x27, 0x2a, 110),
                keyword: rgb(0xc6, 0x78, 0xdd),
                typ: rgb(0xe5, 0xc0, 0x7b),
                string: rgb(0x98, 0xc3, 0x79),
                comment: rgb(0x10, 0xb9, 0x81),
                number: rgb(0xd1, 0x9a, 0x66),
                macro_color: rgb(0x61, 0xaf, 0xef),
                added: rgb(35, 80, 52),
                added_text: rgb(52, 211, 153),
                removed: rgb(96, 40, 46),
                removed_text: rgb(248, 113, 113),
                error: rgb(248, 113, 113),
                warning: rgb(251, 191, 36),
            },
            // VS Code Dark Modern
            ThemeKind::DarkModern => Theme {
                name: "VS Code Dark Modern",
                bg: rgb(0x1f, 0x1f, 0x1f),
                panel: rgb(0x18, 0x18, 0x18),
                sidebar: rgb(0x18, 0x18, 0x18),
                activity: rgb(0x18, 0x18, 0x18),
                titlebar: rgb(0x1f, 0x1f, 0x1f),
                tab_bg: rgb(0x18, 0x18, 0x18),
                tab_active: rgb(0x1f, 0x1f, 0x1f),
                status: rgb(0x00, 0x7a, 0xcc),
                status_fg: rgb(0xff, 0xff, 0xff),
                pill: rgb(0x00, 0x7a, 0xcc),
                border: rgb(0x2b, 0x2b, 0x2b),
                text: rgb(0xcc, 0xcc, 0xcc),
                dim: rgb(0x85, 0x85, 0x85),
                gutter: rgb(0x6e, 0x76, 0x81),
                accent: rgb(0x00, 0x7a, 0xcc),
                line_hi: rgba(255, 255, 255, 12),
                keyword: rgb(86, 156, 214),
                typ: rgb(78, 201, 176),
                string: rgb(206, 145, 120),
                comment: rgb(106, 153, 85),
                number: rgb(181, 206, 168),
                macro_color: rgb(220, 220, 170),
                added: rgb(24, 83, 45),
                added_text: rgb(115, 220, 140),
                removed: rgb(100, 40, 40),
                removed_text: rgb(240, 140, 140),
                error: rgb(244, 71, 71),
                warning: rgb(220, 200, 120),
            },
            ThemeKind::TokyoNight => Theme {
                name: "Tokyo Night",
                bg: rgb(0x1a, 0x1b, 0x26),
                panel: rgb(0x16, 0x16, 0x1e),
                sidebar: rgb(0x1a, 0x1b, 0x26),
                activity: rgb(0x16, 0x16, 0x1e),
                titlebar: rgb(0x16, 0x16, 0x1e),
                tab_bg: rgb(0x16, 0x16, 0x1e),
                tab_active: rgb(0x1a, 0x1b, 0x26),
                status: rgb(0x7a, 0xa2, 0xf7),
                status_fg: rgb(0x15, 0x16, 0x1e),
                pill: rgb(0x7a, 0xa2, 0xf7),
                border: rgb(0x29, 0x2e, 0x42),
                text: rgb(0xc0, 0xca, 0xf5),
                dim: rgb(0x56, 0x5f, 0x89),
                gutter: rgb(0x3b, 0x42, 0x61),
                accent: rgb(0x7a, 0xa2, 0xf7),
                line_hi: rgba(255, 255, 255, 12),
                keyword: rgb(187, 154, 247),
                typ: rgb(125, 207, 255),
                string: rgb(158, 206, 106),
                comment: rgb(86, 95, 137),
                number: rgb(255, 158, 100),
                macro_color: rgb(125, 207, 255),
                added: rgb(35, 80, 60),
                added_text: rgb(140, 230, 160),
                removed: rgb(95, 45, 60),
                removed_text: rgb(247, 140, 160),
                error: rgb(247, 118, 142),
                warning: rgb(224, 175, 104),
            },
            ThemeKind::Catppuccin => Theme {
                name: "Catppuccin Mocha",
                bg: rgb(0x1e, 0x1e, 0x2e),
                panel: rgb(0x18, 0x18, 0x25),
                sidebar: rgb(0x18, 0x18, 0x25),
                activity: rgb(0x11, 0x11, 0x1b),
                titlebar: rgb(0x18, 0x18, 0x25),
                tab_bg: rgb(0x18, 0x18, 0x25),
                tab_active: rgb(0x1e, 0x1e, 0x2e),
                status: rgb(0xcb, 0xa6, 0xf7),
                status_fg: rgb(0x11, 0x11, 0x1b),
                pill: rgb(0xcb, 0xa6, 0xf7),
                border: rgb(0x31, 0x32, 0x44),
                text: rgb(0xcd, 0xd6, 0xf4),
                dim: rgb(0x6c, 0x70, 0x86),
                gutter: rgb(0x45, 0x47, 0x5a),
                accent: rgb(0xcb, 0xa6, 0xf7),
                line_hi: rgba(255, 255, 255, 12),
                keyword: rgb(203, 166, 247),
                typ: rgb(249, 226, 175),
                string: rgb(166, 227, 161),
                comment: rgb(108, 112, 134),
                number: rgb(250, 179, 135),
                macro_color: rgb(137, 180, 250),
                added: rgb(40, 85, 60),
                added_text: rgb(150, 230, 160),
                removed: rgb(100, 50, 60),
                removed_text: rgb(243, 139, 168),
                error: rgb(243, 139, 168),
                warning: rgb(249, 226, 175),
            },
            ThemeKind::Monokai => Theme {
                name: "Monokai Pro",
                bg: rgb(0x27, 0x28, 0x22),
                panel: rgb(0x1d, 0x1d, 0x1d),
                sidebar: rgb(0x22, 0x22, 0x22),
                activity: rgb(0x19, 0x19, 0x19),
                titlebar: rgb(0x1d, 0x1d, 0x1d),
                tab_bg: rgb(0x1d, 0x1d, 0x1d),
                tab_active: rgb(0x27, 0x28, 0x22),
                status: rgb(0xff, 0xd8, 0x66),
                status_fg: rgb(0x22, 0x22, 0x22),
                pill: rgb(0xff, 0xd8, 0x66),
                border: rgb(0x38, 0x38, 0x30),
                text: rgb(0xf8, 0xf8, 0xf2),
                dim: rgb(0x75, 0x71, 0x5e),
                gutter: rgb(0x49, 0x48, 0x3e),
                accent: rgb(0xff, 0xd8, 0x66),
                line_hi: rgba(255, 255, 255, 12),
                keyword: rgb(249, 38, 114),
                typ: rgb(102, 217, 239),
                string: rgb(230, 219, 116),
                comment: rgb(117, 113, 94),
                number: rgb(174, 129, 255),
                macro_color: rgb(166, 226, 46),
                added: rgb(50, 90, 55),
                added_text: rgb(170, 230, 150),
                removed: rgb(105, 50, 55),
                removed_text: rgb(249, 150, 150),
                error: rgb(249, 38, 114),
                warning: rgb(255, 216, 102),
            },
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

/// Concrete RGB values for every UI surface and syntax token.
#[derive(Clone, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub bg: egui::Color32,
    pub panel: egui::Color32,
    pub sidebar: egui::Color32,
    pub activity: egui::Color32,
    pub titlebar: egui::Color32,
    pub tab_bg: egui::Color32,
    pub tab_active: egui::Color32,
    pub status: egui::Color32,
    pub status_fg: egui::Color32,
    #[allow(dead_code)]
    pub pill: egui::Color32,
    pub border: egui::Color32,
    pub text: egui::Color32,
    pub dim: egui::Color32,
    pub gutter: egui::Color32,
    pub accent: egui::Color32,
    pub line_hi: egui::Color32,
    pub keyword: egui::Color32,
    pub typ: egui::Color32,
    pub string: egui::Color32,
    pub comment: egui::Color32,
    pub number: egui::Color32,
    pub macro_color: egui::Color32,
    pub added: egui::Color32,
    pub added_text: egui::Color32,
    pub removed: egui::Color32,
    pub removed_text: egui::Color32,
    pub error: egui::Color32,
    pub warning: egui::Color32,
}

/// Apply the theme to egui's global visuals plus the editor font size.
pub fn apply(ctx: &egui::Context, theme: &Theme, font_size: f32) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = theme.panel;
    visuals.window_fill = theme.sidebar;
    visuals.window_stroke = egui::Stroke::new(1.0, theme.border);
    visuals.extreme_bg_color = theme.bg;
    visuals.code_bg_color = theme.bg;
    visuals.faint_bg_color = theme.sidebar;
    visuals.widgets.noninteractive.bg_fill = theme.panel;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, theme.text);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, theme.border);
    visuals.widgets.inactive.bg_fill = theme.sidebar;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme.text);
    visuals.widgets.inactive.weak_bg_fill = theme.sidebar;
    visuals.widgets.hovered.bg_fill = theme.border;
    visuals.widgets.hovered.weak_bg_fill = theme.border;
    visuals.widgets.active.bg_fill = theme.accent;
    visuals.widgets.open.bg_fill = theme.sidebar;
    visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(
        theme.accent.r(),
        theme.accent.g(),
        theme.accent.b(),
        90,
    );
    visuals.selection.stroke = egui::Stroke::new(1.0, theme.text);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: egui::Color32::from_black_alpha(150),
    };
    visuals.popup_shadow = visuals.window_shadow;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(font_size));
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(font_size.clamp(12.0, 14.0)),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::proportional((font_size - 1.5).max(10.0)),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::proportional(font_size.clamp(12.0, 13.5)),
    );
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(16.0));
    // Slim VS Code-like scrollbars.
    style.spacing.scroll.bar_width = 8.0;
    style.spacing.scroll.handle_min_length = 36.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_outer_margin = 0.0;
    style.spacing.scroll.floating = true;
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 3.0);
    style.spacing.window_margin = egui::Margin::same(12);
    ctx.set_style(style);
}
