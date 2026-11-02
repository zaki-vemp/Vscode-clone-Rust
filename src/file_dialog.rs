//! Native OS file/folder chooser — no typing paths by hand.
//!
//! Linux: `zenity` / `kdialog` / `yad` (the GTK/KDE dialogs VS Code
//! reaches through the XDG portal), macOS: Finder via `osascript`
//! (`choose folder` / `choose file`), Windows: WinForms via PowerShell.
//! When no native tool exists the caller falls back to the in-app
//! path-entry dialog, which also offers a Browse… button.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PickOutcome {
    Picked(PathBuf),
    /// User dismissed the native dialog.
    Cancelled,
    /// No native picker tool is installed / it failed to launch.
    Unavailable,
}

fn outcome_from(output: std::io::Result<std::process::Output>) -> PickOutcome {
    match output {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            // osascript prints an `alias …:` prefix handled by caller parsers;
            // generic tools print a plain path.
            if s.is_empty() {
                PickOutcome::Cancelled
            } else {
                PickOutcome::Picked(PathBuf::from(s))
            }
        }
        Ok(_) => PickOutcome::Cancelled,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => PickOutcome::Unavailable,
        Err(_) => PickOutcome::Unavailable,
    }
}

fn run(cmd: &mut Command) -> PickOutcome {
    outcome_from(crate::process::capture(
        cmd,
        crate::process::Limits {
            timeout: std::time::Duration::from_secs(600),
            stdout: 64 * 1024,
            stderr: 64 * 1024,
        },
    ))
}

/// Escape a string for an AppleScript double-quoted literal.
#[allow(dead_code)] // used on macOS / in tests
pub fn applescript_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Escape a string for a PowerShell single-quoted literal.
#[allow(dead_code)] // used on Windows / in tests
pub fn powershell_escape(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(target_os = "macos")]
fn macos(script: &str) -> PickOutcome {
    match run(Command::new("osascript").args(["-e", script])) {
        PickOutcome::Picked(p) => {
            // `choose folder` returns `alias …:` unless POSIX path is asked.
            PickOutcome::Picked(p)
        }
        other => other,
    }
}

pub fn pick_folder(start: Option<&Path>) -> PickOutcome {
    #[cfg(target_os = "macos")]
    {
        let loc = start
            .map(|p| {
                format!(
                    " default location POSIX file \"{}\"",
                    applescript_escape(&p.to_string_lossy())
                )
            })
            .unwrap_or_default();
        return macos(&format!(
            "POSIX path of (choose folder with prompt \"Open Folder\"{loc})"
        ));
    }
    #[cfg(target_os = "windows")]
    {
        let init = start
            .map(|p| {
                format!(
                    "$d.SelectedPath='{}';",
                    powershell_escape(&p.to_string_lossy())
                )
            })
            .unwrap_or_default();
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.FolderBrowserDialog; {init} if($d.ShowDialog() -eq 'OK'){{[Console]::Out.Write($d.SelectedPath)}}"
        );
        return run(Command::new("powershell").args(["-NoProfile", "-STA", "-Command", &script]));
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pick_folder_linux(start)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn pick_folder_linux(start: Option<&Path>) -> PickOutcome {
    let start_s = start.map(|p| p.to_string_lossy().into_owned());
    // zenity (GNOME/GTK)
    let mut cmd = Command::new("zenity");
    cmd.args(["--file-selection", "--directory", "--title=Open Folder"]);
    if let Some(s) = &start_s {
        cmd.arg(format!("--filename={s}/"));
    }
    match run(&mut cmd) {
        PickOutcome::Unavailable => {}
        other => return other,
    }
    // kdialog (KDE)
    let mut cmd = Command::new("kdialog");
    cmd.args(["--getexistingdirectory"]);
    if let Some(s) = &start_s {
        cmd.arg(s);
    }
    cmd.arg("--title").arg("Open Folder");
    match run(&mut cmd) {
        PickOutcome::Unavailable => {}
        other => return other,
    }
    // yad
    let mut cmd = Command::new("yad");
    cmd.args(["--file-selection", "--directory", "--title=Open Folder"]);
    if let Some(s) = &start_s {
        cmd.arg(format!("--filename={s}/"));
    }
    run(&mut cmd)
}

pub fn pick_file(start: Option<&Path>) -> PickOutcome {
    #[cfg(target_os = "macos")]
    {
        let loc = start
            .filter(|p| p.is_dir())
            .map(|p| {
                format!(
                    " default location POSIX file \"{}\"",
                    applescript_escape(&p.to_string_lossy())
                )
            })
            .unwrap_or_default();
        return macos(&format!(
            "POSIX path of (choose file with prompt \"Open File\"{loc})"
        ));
    }
    #[cfg(target_os = "windows")]
    {
        let init = start
            .filter(|p| p.is_dir())
            .map(|p| {
                format!(
                    "$d.InitialDirectory='{}';",
                    powershell_escape(&p.to_string_lossy())
                )
            })
            .unwrap_or_default();
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.OpenFileDialog; {init} if($d.ShowDialog() -eq 'OK'){{[Console]::Out.Write($d.FileName)}}"
        );
        return run(Command::new("powershell").args(["-NoProfile", "-STA", "-Command", &script]));
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let start_s = start.map(|p| p.to_string_lossy().into_owned());
        let mut cmd = Command::new("zenity");
        cmd.args(["--file-selection", "--title=Open File"]);
        if let Some(s) = &start_s {
            cmd.arg(format!("--filename={s}"));
        }
        match run(&mut cmd) {
            PickOutcome::Unavailable => {}
            other => return other,
        }
        let mut cmd = Command::new("kdialog");
        cmd.args(["--getopenfilename"]);
        if let Some(s) = &start_s {
            cmd.arg(s);
        }
        match run(&mut cmd) {
            PickOutcome::Unavailable => {}
            other => return other,
        }
        let mut cmd = Command::new("yad");
        cmd.args(["--file-selection", "--title=Open File"]);
        run(&mut cmd)
    }
}

pub fn pick_save_file(start_dir: Option<&Path>, suggested: &str) -> PickOutcome {
    #[cfg(target_os = "macos")]
    {
        let loc = start_dir
            .map(|p| {
                format!(
                    " default location POSIX file \"{}\"",
                    applescript_escape(&p.to_string_lossy())
                )
            })
            .unwrap_or_default();
        return macos(&format!(
            "POSIX path of (choose file name with prompt \"Save As\" default name \"{}\"{loc})",
            applescript_escape(suggested)
        ));
    }
    #[cfg(target_os = "windows")]
    {
        let init = start_dir
            .map(|p| {
                format!(
                    "$d.InitialDirectory='{}';",
                    powershell_escape(&p.to_string_lossy())
                )
            })
            .unwrap_or_default();
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.SaveFileDialog; $d.FileName='{}'; {init} if($d.ShowDialog() -eq 'OK'){{[Console]::Out.Write($d.FileName)}}",
            powershell_escape(suggested)
        );
        return run(Command::new("powershell").args(["-NoProfile", "-STA", "-Command", &script]));
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let suggested_path = start_dir
            .map(|d| d.join(suggested).to_string_lossy().into_owned())
            .unwrap_or_else(|| suggested.to_string());
        let mut cmd = Command::new("zenity");
        cmd.args([
            "--file-selection",
            "--save",
            "--confirm-overwrite",
            "--title=Save As",
            &format!("--filename={suggested_path}"),
        ]);
        match run(&mut cmd) {
            PickOutcome::Unavailable => {}
            other => return other,
        }
        let mut cmd = Command::new("kdialog");
        cmd.args(["--getsavefilename", suggested]);
        match run(&mut cmd) {
            PickOutcome::Unavailable => {}
            other => return other,
        }
        PickOutcome::Unavailable
    }
}

/// True when a native picker tool is present (Linux only check is cheap).
pub fn native_available() -> bool {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        true
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        ["zenity", "kdialog", "yad"]
            .iter()
            .any(|t| Command::new(t).arg("--version").output().is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_quotes() {
        assert_eq!(applescript_escape("a\"b\\c"), "a\\\"b\\\\c");
        assert_eq!(powershell_escape("a'b"), "a''b");
    }
    #[test]
    fn cancelled_on_failure() {
        // zenity/kdialog exit non-zero on cancel.
        assert_eq!(
            outcome_from(Command::new("false").output()),
            PickOutcome::Cancelled
        );
        assert_eq!(
            outcome_from(Command::new("oxide-no-such-picker-xyz").output()),
            PickOutcome::Unavailable
        );
    }
}
