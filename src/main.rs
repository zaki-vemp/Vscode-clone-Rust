//! OxideCode: a pure-Rust native desktop code editor.
//!
//! No JavaScript, no webviews, no Electron. The UI is rendered with `egui`
//! (via `eframe`), which is itself 100% Rust and builds for macOS, Linux
//! and Windows from this single codebase.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod buffer;
mod icons;
mod diagnostics;
mod diff;
mod highlight;
mod languages;
mod project;
mod terminal;
mod theme;
mod ui_bottom;
mod ui_editor;
mod ui_sidebar;

fn main() -> eframe::Result<()> {
    app::run()
}
