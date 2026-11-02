//! Application color themes. All rendering goes through these colors;
//! Official VS Code themes are loaded from vendored upstream JSONC sources.

use eframe::egui;

/// Identifies one of the built-in color themes.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ThemeKind {
    #[default]
    DarkModern,
    LightModern,
    DarkPlus,
    LightPlus,
    HighContrastDark,
    HighContrastLight,
    TokyoNight,
    Catppuccin,
    Monokai,
}

impl ThemeKind {
    pub fn all() -> &'static [(&'static str, ThemeKind)] {
        &[
            ("VS Code Dark Modern", ThemeKind::DarkModern),
            ("VS Code Light Modern", ThemeKind::LightModern),
            ("VS Code Dark+", ThemeKind::DarkPlus),
            ("VS Code Light+", ThemeKind::LightPlus),
            ("VS Code Dark High Contrast", ThemeKind::HighContrastDark),
            ("VS Code Light High Contrast", ThemeKind::HighContrastLight),
            ("Tokyo Night", ThemeKind::TokyoNight),
            ("Catppuccin Mocha", ThemeKind::Catppuccin),
            ("Monokai", ThemeKind::Monokai),
        ]
    }

    pub fn name(self) -> &'static str {
        Self::all()
            .iter()
            .find(|(_, k)| *k == self)
            .map(|(n, _)| *n)
            .unwrap_or("VS Code Dark Modern")
    }

    pub fn colors(self) -> Theme {
        let official = match self {
            Self::DarkModern => Some(("dark_modern.json", false)),
            Self::LightModern => Some(("light_modern.json", true)),
            Self::DarkPlus => Some(("dark_plus.json", false)),
            Self::LightPlus => Some(("light_plus.json", true)),
            Self::HighContrastDark => Some(("hc_black.json", false)),
            Self::HighContrastLight => Some(("hc_light.json", true)),
            _ => None,
        };
        if let Some((file, light)) = official {
            let mut theme = crate::vscode_theme::builtin(file, light);
            theme.name = self.name().to_string();
            theme
        } else {
            self.base_colors()
        }
    }

    pub(crate) fn base_colors(self) -> Theme {
        match self {
            ThemeKind::DarkModern => Theme {
                name: "VS Code Dark Modern".into(),
                dark: true,
                comment_italics: false,
                activity_fg: rgb(0xd7, 0xd7, 0xd7),
                activity_dim: rgb(0x86, 0x86, 0x86),
                terminal_bg: rgb(0x18, 0x18, 0x18),
                terminal_fg: rgb(0xcc, 0xcc, 0xcc),
                bg: rgb(0x1f, 0x1f, 0x1f),
                panel: rgb(0x18, 0x18, 0x18),
                sidebar: rgb(0x18, 0x18, 0x18),
                activity: rgb(0x18, 0x18, 0x18),
                titlebar: rgb(0x18, 0x18, 0x18),
                tab_bg: rgb(0x18, 0x18, 0x18),
                tab_active: rgb(0x1f, 0x1f, 0x1f),
                status: rgb(0x18, 0x18, 0x18),
                status_fg: rgb(0xcc, 0xcc, 0xcc),
                border: rgb(0x2b, 0x2b, 0x2b),
                text: rgb(0xcc, 0xcc, 0xcc),
                dim: rgb(0x9d, 0x9d, 0x9d),
                gutter: rgb(0x6e, 0x76, 0x81),
                gutter_active: rgb(0xcc, 0xcc, 0xcc),
                accent: rgb(0x00, 0x78, 0xd4),
                focus_border: rgb(0x00, 0x78, 0xd4),
                line_hi: rgb(0x28, 0x28, 0x28),
                input_bg: rgb(0x31, 0x31, 0x31),
                input_border: rgb(0x3c, 0x3c, 0x3c),
                list_hover: rgb(0x2a, 0x2d, 0x2e),
                list_active: rgb(0x37, 0x37, 0x3d),
                list_focus: rgb(0x04, 0x39, 0x5e),
                widget_bg: rgb(0x20, 0x20, 0x20),
                widget_border: rgb(0x45, 0x45, 0x45),
                badge_bg: rgb(0x61, 0x61, 0x61),
                selection: rgb(0x26, 0x4f, 0x78),
                error: rgb(0xf8, 0x51, 0x49),
                warning: rgb(0xcc, 0xa7, 0x00),
                git_added: rgb(0x2e, 0xa0, 0x43),
                git_modified: rgb(0x00, 0x78, 0xd4),
                git_deleted: rgb(0xf8, 0x51, 0x49),
                git_untracked_fg: rgb(0x73, 0xc9, 0x91),
                git_modified_fg: rgb(0xe2, 0xc0, 0x8d),
                git_deleted_fg: rgb(0xc7, 0x4e, 0x39),
                git_ignored_fg: rgb(0x8c, 0x8c, 0x8c),
                find_match: rgb(0x9e, 0x6a, 0x03),
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
            },
            ThemeKind::TokyoNight => Theme {
                name: "Tokyo Night (native approximation)".into(),
                bg: rgb(0x1a, 0x1b, 0x26),
                panel: rgb(0x16, 0x16, 0x1e),
                sidebar: rgb(0x1a, 0x1b, 0x26),
                activity: rgb(0x16, 0x16, 0x1e),
                titlebar: rgb(0x16, 0x16, 0x1e),
                tab_bg: rgb(0x16, 0x16, 0x1e),
                tab_active: rgb(0x1a, 0x1b, 0x26),
                status: rgb(0x7a, 0xa2, 0xf7),
                status_fg: rgb(0x15, 0x16, 0x1e),
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
                ..ThemeKind::DarkModern.base_colors()
            },
            ThemeKind::Catppuccin => Theme {
                name: "Catppuccin Mocha (native approximation)".into(),
                bg: rgb(0x1e, 0x1e, 0x2e),
                panel: rgb(0x18, 0x18, 0x25),
                sidebar: rgb(0x18, 0x18, 0x25),
                activity: rgb(0x11, 0x11, 0x1b),
                titlebar: rgb(0x18, 0x18, 0x25),
                tab_bg: rgb(0x18, 0x18, 0x25),
                tab_active: rgb(0x1e, 0x1e, 0x2e),
                status: rgb(0xcb, 0xa6, 0xf7),
                status_fg: rgb(0x11, 0x11, 0x1b),
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
                ..ThemeKind::DarkModern.base_colors()
            },
            ThemeKind::Monokai => Theme {
                name: "Monokai (native approximation)".into(),
                bg: rgb(0x27, 0x28, 0x22),
                panel: rgb(0x1d, 0x1d, 0x1d),
                sidebar: rgb(0x22, 0x22, 0x22),
                activity: rgb(0x19, 0x19, 0x19),
                titlebar: rgb(0x1d, 0x1d, 0x1d),
                tab_bg: rgb(0x1d, 0x1d, 0x1d),
                tab_active: rgb(0x27, 0x28, 0x22),
                status: rgb(0xff, 0xd8, 0x66),
                status_fg: rgb(0x22, 0x22, 0x22),
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
                ..ThemeKind::DarkModern.base_colors()
            },
            _ => ThemeKind::DarkModern.base_colors(),
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
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    pub dark: bool,
    pub comment_italics: bool,
    pub activity_fg: egui::Color32,
    pub activity_dim: egui::Color32,
    pub terminal_bg: egui::Color32,
    pub terminal_fg: egui::Color32,
    pub bg: egui::Color32,
    pub panel: egui::Color32,
    pub sidebar: egui::Color32,
    pub activity: egui::Color32,
    pub titlebar: egui::Color32,
    pub tab_bg: egui::Color32,
    pub tab_active: egui::Color32,
    pub status: egui::Color32,
    pub status_fg: egui::Color32,
    pub border: egui::Color32,
    pub text: egui::Color32,
    pub dim: egui::Color32,
    pub gutter: egui::Color32,
    pub gutter_active: egui::Color32,
    pub accent: egui::Color32,
    pub focus_border: egui::Color32,
    pub line_hi: egui::Color32,
    pub input_bg: egui::Color32,
    pub input_border: egui::Color32,
    pub list_hover: egui::Color32,
    pub list_active: egui::Color32,
    pub list_focus: egui::Color32,
    pub widget_bg: egui::Color32,
    pub widget_border: egui::Color32,
    pub badge_bg: egui::Color32,
    pub selection: egui::Color32,
    pub error: egui::Color32,
    pub warning: egui::Color32,
    // Git gutter + SCM decorations, consumed by phases 3–4.
    #[allow(dead_code)]
    pub git_added: egui::Color32,
    #[allow(dead_code)]
    pub git_modified: egui::Color32,
    #[allow(dead_code)]
    pub git_deleted: egui::Color32,
    #[allow(dead_code)]
    pub git_untracked_fg: egui::Color32,
    pub git_modified_fg: egui::Color32,
    #[allow(dead_code)]
    pub git_deleted_fg: egui::Color32,
    pub git_ignored_fg: egui::Color32,
    // Editor find-highlight color, consumed by phase 4.
    #[allow(dead_code)]
    pub find_match: egui::Color32,
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
}

/// Register the codicon font plus any system UI/monospace fonts that exist
/// on this platform. Missing files are silently skipped; egui defaults
/// remain as the fallback glyphs.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "codicon".to_owned(),
        egui::FontData::from_static(include_bytes!("assets/codicon.ttf")).into(),
    );
    fonts.families.insert(
        egui::FontFamily::Name("codicon".into()),
        vec!["codicon".to_owned()],
    );

    #[cfg(target_os = "linux")]
    const UI_FONTS: &[&str] = &[
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];
    #[cfg(target_os = "macos")]
    const UI_FONTS: &[&str] = &[
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ];
    #[cfg(target_os = "windows")]
    const UI_FONTS: &[&str] = &["C:\\Windows\\Fonts\\segoeui.ttf"];

    #[cfg(target_os = "linux")]
    const MONO_FONTS: &[&str] = &["/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"];
    #[cfg(target_os = "macos")]
    const MONO_FONTS: &[&str] = &["/System/Library/Fonts/Menlo.ttc"];
    #[cfg(target_os = "windows")]
    const MONO_FONTS: &[&str] = &["C:\\Windows\\Fonts\\consola.ttf"];

    #[cfg(target_os = "linux")]
    const BOLD_FONTS: &[&str] = &[
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf",
        "/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    ];
    #[cfg(target_os = "macos")]
    const BOLD_FONTS: &[&str] = &[];
    #[cfg(target_os = "windows")]
    const BOLD_FONTS: &[&str] = &["C:\\Windows\\Fonts\\segoeuib.ttf"];

    let load = |fonts: &mut egui::FontDefinitions, paths: &[&str]| -> Vec<String> {
        let mut names = Vec::new();
        for path in paths {
            if let Ok(bytes) = std::fs::read(path) {
                let name = format!("file:{path}");
                fonts
                    .font_data
                    .insert(name.clone(), egui::FontData::from_owned(bytes).into());
                names.push(name);
            }
        }
        names
    };

    let ui_fonts = load(&mut fonts, UI_FONTS);
    let mono_fonts = load(&mut fonts, MONO_FONTS);
    let bold_fonts = load(&mut fonts, BOLD_FONTS);

    let insert_front = |family: &mut Vec<String>, names: &[String]| {
        for (i, n) in names.iter().enumerate() {
            family.insert(i.min(family.len()), n.clone());
        }
    };
    if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        insert_front(prop, &ui_fonts);
    }
    if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        insert_front(mono, &mono_fonts);
    }
    // `bold` must always resolve: egui panics on undefined families.
    let mut bold = bold_fonts;
    bold.extend(
        fonts.families[&egui::FontFamily::Proportional]
            .iter()
            .cloned(),
    );
    fonts
        .families
        .insert(egui::FontFamily::Name("bold".into()), bold);

    ctx.set_fonts(fonts);
}

/// Apply the theme to egui's global visuals plus the editor font size.
pub fn apply(ctx: &egui::Context, theme: &Theme, font_size: f32) {
    let mut visuals = if theme.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = theme.panel;
    visuals.window_fill = theme.widget_bg;
    visuals.window_stroke = egui::Stroke::new(1.0, theme.widget_border);
    visuals.window_corner_radius = egui::CornerRadius::same(5);
    visuals.menu_corner_radius = egui::CornerRadius::same(5);
    visuals.extreme_bg_color = theme.input_bg;
    visuals.code_bg_color = theme.bg;
    visuals.faint_bg_color = theme.widget_bg;
    visuals.widgets.noninteractive.bg_fill = theme.panel;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, theme.text);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, theme.input_border);
    visuals.widgets.inactive.bg_fill = theme.input_bg;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme.text);
    visuals.widgets.inactive.weak_bg_fill = theme.input_bg;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, theme.input_border);
    visuals.widgets.hovered.bg_fill = theme.list_hover;
    visuals.widgets.hovered.weak_bg_fill = theme.list_hover;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, theme.input_border);
    visuals.widgets.active.bg_fill = theme.accent;
    visuals.widgets.open.bg_fill = theme.widget_bg;
    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.corner_radius = egui::CornerRadius::same(2);
    }
    visuals.selection.bg_fill = theme.selection;
    // egui also uses selection.stroke as selectable-label foreground; keep
    // it readable on both light and dark selection backgrounds.
    visuals.selection.stroke = egui::Stroke::new(1.5, theme.text);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: egui::Color32::from_black_alpha(150),
    };
    visuals.popup_shadow = visuals.window_shadow;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::monospace(font_size),
    );
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, egui::FontId::proportional(11.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(16.0));
    // Slim VS Code-like scrollbars.
    style.spacing.scroll.bar_width = 10.0;
    style.spacing.scroll.handle_min_length = 36.0;
    style.spacing.scroll.bar_inner_margin = 0.0;
    style.spacing.scroll.bar_outer_margin = 0.0;
    style.spacing.scroll.floating = true;
    style.spacing.scroll.dormant_handle_opacity = 0.4;
    style.spacing.scroll.active_handle_opacity = 0.55;
    style.spacing.scroll.interact_handle_opacity = 0.8;
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 3.0);
    style.spacing.window_margin = egui::Margin::same(12);
    ctx.set_style(style);
}
