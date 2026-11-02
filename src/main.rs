//! OxideCode: a pure-Rust native desktop code editor.
//!
//! No JavaScript, no webviews, no Electron. The UI is rendered with `egui`
//! (via `eframe`), which is itself 100% Rust and builds for macOS, Linux
//! and Windows from this single codebase.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod buffer;
mod diagnostics;
mod diff;
mod editor_find;
mod extensions;
mod file_dialog;
mod git;
mod highlight;
mod icons;
mod json_view;
mod languages;
mod preview;
mod process;
mod project;
mod pty_terminal;
mod search;
mod snippets;
mod terminal;
mod theme;
mod ui_bottom;
mod ui_editor;
mod ui_sidebar;
mod vscode_theme;

fn main() -> eframe::Result<()> {
    let mut paths = Vec::new();
    let mut positional = false;
    let mut check_extensions = false;
    let mut check_previews = false;
    for argument in std::env::args_os().skip(1) {
        if !positional {
            match argument.to_str() {
                Some("--") => {
                    positional = true;
                    continue;
                }
                Some("--help" | "-h") => {
                    println!("OxideCode {}\nUsage: oxide-code [FOLDER] [FILE ...]\n\n  -h, --help               Show this help\n  -V, --version            Show the version\n  --check-extension PATH   Validate supported extension contributions\n  --check-preview PATH     Validate a file preview without opening the UI", env!("CARGO_PKG_VERSION"));
                    return Ok(());
                }
                Some("--version" | "-V") => {
                    println!("OxideCode {}", env!("CARGO_PKG_VERSION"));
                    return Ok(());
                }
                Some("--check-extension") => {
                    check_extensions = true;
                    continue;
                }
                Some("--check-preview") => {
                    check_previews = true;
                    continue;
                }
                Some(value) if value.starts_with('-') => {
                    eprintln!("Unknown option: {value}");
                    std::process::exit(2);
                }
                _ => {}
            }
        }
        match std::path::PathBuf::from(argument).canonicalize() {
            Ok(path) => paths.push(path),
            Err(error) => {
                eprintln!("Cannot open path: {error}");
                std::process::exit(1);
            }
        }
    }
    if check_previews {
        let mut failed = paths.is_empty();
        for path in &paths {
            match preview::check(path) {
                Ok(description) => println!("{}: {description}", path.display()),
                Err(error) => {
                    eprintln!("{}: {error:#}", path.display());
                    failed = true;
                }
            }
        }
        std::process::exit(i32::from(failed));
    }
    if check_extensions {
        let mut failed = paths.is_empty();
        for path in paths {
            match extensions::load(&path) {
                Ok(extension) => println!(
                    "{}: {} themes, {} snippets{}",
                    extension.id,
                    extension.themes.len(),
                    extension.snippets.len(),
                    if extension.needs_host {
                        "; code features require a host"
                    } else {
                        ""
                    }
                ),
                Err(error) => {
                    eprintln!("{}: {error:#}", path.display());
                    failed = true;
                }
            }
        }
        std::process::exit(i32::from(failed));
    }
    app::run(paths)
}
