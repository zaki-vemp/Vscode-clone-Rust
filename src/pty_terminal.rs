//! Interactive PTY sessions via vendored `egui_term` (alacritty backend).
//! The noninteractive command runner in `terminal.rs` stays for cargo
//! tasks; this module owns only real shell sessions.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

use eframe::egui;
use egui_term::{BackendSettings, PtyEvent, TerminalBackend};

/// Which session is primary/split/focused. `active` is the primary visible
/// pane, `split` an optional distinct secondary, `focused` the pane that
/// owns the keyboard (or None).
#[derive(Default, Debug)]
pub struct PaneState {
    pub active: Option<u64>,
    pub split: Option<u64>,
    pub focused: Option<u64>,
    /// Last pane that held focus — toolbar actions still target it after
    /// keyboard focus is surrendered (e.g. click on the header row).
    pub last_focused: Option<u64>,
}

impl PaneState {
    /// A new session becomes the primary pane; a distinct split survives.
    fn add_primary(&mut self, id: u64) {
        self.active = Some(id);
        self.focused = Some(id);
        self.last_focused = Some(id);
    }

    /// A new session occupies the split pane and takes focus.
    fn add_split(&mut self, id: u64) {
        self.split = Some(id);
        self.focused = Some(id);
        self.last_focused = Some(id);
    }

    /// Session-list select: the split session only gains focus; any other
    /// session replaces the primary pane (split kept when distinct).
    fn select(&mut self, id: u64) {
        if self.split == Some(id) {
            self.focused = Some(id);
        } else {
            self.active = Some(id);
            self.focused = Some(id);
        }
        self.last_focused = Some(id);
    }

    /// Click on a visible pane focuses it without changing active/split.
    fn focus_pane(&mut self, id: u64) {
        if self.active == Some(id) || self.split == Some(id) {
            self.focused = Some(id);
            self.last_focused = Some(id);
        }
    }

    fn kill(&mut self, id: u64, fallback: Option<u64>) {
        if self.active == Some(id) {
            // Primary goes away: promote the split, else last remaining.
            self.active = self.split.take().or(fallback);
        }
        if self.split == Some(id) {
            self.split = None;
        }
        if self.split == self.active {
            self.split = None;
        }
        if self.focused == Some(id) {
            self.focused = self.split.or(self.active);
        }
        if self.last_focused == Some(id) {
            self.last_focused = self.split.or(self.active);
        }
    }

    /// Session that toolbar Kill/Clear actions apply to.
    pub fn target(&self) -> Option<u64> {
        self.focused.or(self.last_focused).or(self.active)
    }
}

/// Focus resolution for a click: `Err(())` = clicked a pane (focus that
/// session), `Ok` = outside all panes (unfocus) — caller applies it.
pub fn click_focus(pane_rects: &[(u64, egui::Rect)], pos: egui::Pos2) -> Result<u64, ()> {
    pane_rects
        .iter()
        .find(|(_, r)| r.contains(pos))
        .map(|(id, _)| *id)
        .ok_or(())
}

pub struct Session {
    pub id: u64,
    pub backend: TerminalBackend,
    pub title: String,
    pub profile: String,
    pub exited: bool,
}

#[derive(Clone, Debug)]
pub struct Profile {
    pub name: String,
    pub shell: String,
    pub args: Vec<String>,
}

pub struct TerminalManager {
    sessions: Vec<Session>,
    next_id: u64,
    pub panes: PaneState,
    pub profiles: Vec<Profile>,
    pub profile_idx: usize,
    attempted_initial_session: bool,
    tx: Sender<(u64, PtyEvent)>,
    rx: Receiver<(u64, PtyEvent)>,
    pub last_error: Option<String>,
    ctx: egui::Context,
    cwd: PathBuf,
}

impl TerminalManager {
    pub fn new(ctx: egui::Context, cwd: PathBuf) -> Self {
        let (tx, rx) = channel();
        Self {
            sessions: Vec::new(),
            next_id: 1,
            panes: PaneState::default(),
            profiles: profiles(),
            profile_idx: 0,
            attempted_initial_session: false,
            tx,
            rx,
            last_error: None,
            ctx,
            cwd,
        }
    }

    /// New sessions spawn in this directory (called on folder change).
    pub fn set_cwd(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
    }

    /// Lazy first session: attempted once, never auto-respawned.
    pub fn ensure_session(&mut self) {
        if !self.attempted_initial_session {
            self.attempted_initial_session = true;
            if self.sessions.is_empty() {
                self.spawn_primary();
            }
        }
    }

    fn spawn(&mut self) -> Option<u64> {
        let profile = self.profiles.get(self.profile_idx)?.clone();
        let id = self.next_id;
        self.next_id += 1;
        match TerminalBackend::new(
            id,
            self.ctx.clone(),
            self.tx.clone(),
            BackendSettings {
                shell: profile.shell.clone(),
                args: profile.args.clone(),
                working_directory: Some(self.cwd.clone()),
            },
        ) {
            Ok(backend) => {
                self.sessions.push(Session {
                    id,
                    backend,
                    title: profile.name.clone(),
                    profile: profile.name,
                    exited: false,
                });
                self.last_error = None;
                Some(id)
            }
            Err(e) => {
                self.last_error = Some(format!("failed to spawn {}: {e}", profile.shell));
                None
            }
        }
    }

    /// Explicit "New Terminal": session becomes the primary pane.
    pub fn new_session(&mut self) -> Option<u64> {
        let id = self.spawn()?;
        self.panes.add_primary(id);
        Some(id)
    }

    /// Internal spawn for the lazy path (same as New when empty).
    fn spawn_primary(&mut self) -> Option<u64> {
        self.new_session()
    }

    /// Explicit New using a chosen profile from the dropdown.
    pub fn new_session_with(&mut self, profile_idx: usize) -> Option<u64> {
        if profile_idx < self.profiles.len() {
            self.profile_idx = profile_idx;
        }
        self.new_session()
    }

    /// Split view: a brand-new session occupies the split pane (primary
    /// preserved). With zero sessions, spawn primary only. Max two panes.
    pub fn split_session(&mut self) {
        if self.panes.split.is_some() {
            return;
        }
        if self.sessions.is_empty() || self.panes.active.is_none() {
            self.spawn_primary();
            return;
        }
        if let Some(id) = self.spawn() {
            self.panes.add_split(id);
        }
    }

    pub fn kill(&mut self, id: u64) {
        self.sessions.retain(|s| s.id != id); // drop shuts the PTY down
        let fallback = self.sessions.last().map(|s| s.id);
        self.panes.kill(id, fallback);
    }

    /// Session-list selection.
    pub fn select(&mut self, id: u64) {
        self.panes.select(id);
    }

    /// Click on a visible pane focuses it only.
    pub fn focus_pane(&mut self, id: u64) {
        self.panes.focus_pane(id);
    }

    pub fn unfocus(&mut self) {
        self.panes.focused = None;
    }

    /// `Exit` keeps the session for scrollback; kill() removes it.
    pub fn poll_events(&mut self) {
        while let Ok((id, ev)) = self.rx.try_recv() {
            let Some(s) = self.sessions.iter_mut().find(|s| s.id == id) else {
                continue;
            };
            match ev {
                PtyEvent::Title(t) => s.title = t,
                PtyEvent::ResetTitle => s.title = s.profile.clone(),
                PtyEvent::Exit | PtyEvent::ChildExit(_) => s.exited = true,
                // Unsafe OSC events (clipboard/open) are ignored by design.
                _ => {}
            }
        }
    }

    pub fn get(&mut self, id: u64) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| s.id == id)
    }

    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    /// Write text to a session (e.g. "clear" sends form-feed).
    pub fn send(&mut self, id: u64, bytes: &[u8]) {
        if let Some(s) = self.get(id) {
            s.backend
                .process_command(egui_term::BackendCommand::Write(bytes.to_vec()));
        }
    }
}

/// Default shell first (SHELL / COMSPEC), then common shells found on PATH.
fn profiles() -> Vec<Profile> {
    #[cfg(windows)]
    {
        let mut v = Vec::new();
        let comspec = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".into());
        v.push(Profile {
            name: "cmd".into(),
            shell: comspec,
            args: vec![],
        });
        for (name, exe) in [("PowerShell", "powershell.exe"), ("pwsh", "pwsh.exe")] {
            if which(exe) {
                v.push(Profile {
                    name: name.into(),
                    shell: exe.into(),
                    args: vec![],
                });
            }
        }
        v
    }
    #[cfg(not(windows))]
    {
        let mut v = Vec::new();
        let default = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        v.push(Profile {
            name: Path::new(&default)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| default.clone()),
            shell: default,
            args: vec![],
        });
        for (name, path) in [
            ("bash", "/bin/bash"),
            ("zsh", "/bin/zsh"),
            ("fish", "/usr/bin/fish"),
            ("sh", "/bin/sh"),
        ] {
            if path_exists(path) && !v.iter().any(|p| p.shell == path) {
                v.push(Profile {
                    name: name.into(),
                    shell: path.into(),
                    args: vec![],
                });
            }
        }
        v
    }
}

#[cfg(not(windows))]
fn path_exists(p: &str) -> bool {
    Path::new(p).is_file()
}

#[cfg(windows)]
fn which(exe: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(exe).is_file()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mgr() -> TerminalManager {
        TerminalManager::new(egui::Context::default(), std::env::temp_dir())
    }

    #[test]
    fn profiles_resolve_a_default() {
        let p = profiles();
        assert!(!p.is_empty());
        assert!(p.iter().all(|p| !p.shell.is_empty()));
    }

    #[test]
    fn pane_state_invariants() {
        // Pure state machine — no PTY needed.
        let mut p = PaneState::default();
        p.add_primary(1);
        p.add_split(2);
        assert_eq!(p.active, Some(1));
        assert_eq!(p.split, Some(2));
        assert_ne!(p.active, p.split);
        // Clicking the split pane focuses only.
        p.focus_pane(1);
        p.select(2);
        assert_eq!(p.active, Some(1));
        assert_eq!(p.focused, Some(2));
        // New session becomes primary, keeps the distinct split.
        p.add_primary(3);
        assert_eq!(p.active, Some(3));
        assert_eq!(p.split, Some(2));
        assert_eq!(p.focused, Some(3));
        // Kill primary -> split promoted.
        p.kill(3, Some(4));
        assert_eq!(p.active, Some(2));
        assert_eq!(p.split, None);
        assert_eq!(p.focused, Some(2));
        // Kill split just clears it.
        p.add_split(5);
        p.kill(5, Some(9));
        assert_eq!(p.split, None);
        assert_eq!(p.active, Some(2));
        // Kill everything -> empty.
        p.kill(2, None);
        assert_eq!(p.active, None);
        assert_eq!(p.focused, None);
    }

    #[test]
    fn click_focus_resolves_panes() {
        let rects = vec![
            (
                1u64,
                egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0)),
            ),
            (
                2u64,
                egui::Rect::from_min_size(egui::pos2(100.0, 0.0), egui::vec2(100.0, 100.0)),
            ),
        ];
        assert_eq!(click_focus(&rects, egui::pos2(50.0, 50.0)), Ok(1));
        assert_eq!(click_focus(&rects, egui::pos2(150.0, 50.0)), Ok(2));
        assert_eq!(click_focus(&rects, egui::pos2(500.0, 50.0)), Err(()));
    }

    #[cfg(unix)]
    #[test]
    fn ensure_session_attempts_once_and_error_sticks() {
        let mut m = mgr();
        // Point the default profile at a shell that cannot spawn.
        m.profiles[0] = Profile {
            name: "bogus".into(),
            shell: "/nonexistent/oxide-sh".into(),
            args: vec![],
        };
        m.ensure_session();
        assert!(m.is_empty());
        assert!(m.last_error.is_some());
        let next = m.next_id;
        m.ensure_session(); // must NOT retry
        assert_eq!(m.next_id, next);
        assert!(m.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn kill_last_does_not_respawn() {
        // Real app flow: lazy first spawn via ensure, user kills the last
        // session, subsequent ensure calls must not auto-respawn.
        let mut m = mgr();
        m.ensure_session();
        let id = m.sessions()[0].id;
        m.kill(id);
        assert!(m.is_empty());
        m.ensure_session();
        assert!(m.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn pty_smoke() {
        use std::time::{Duration, Instant};

        fn screen(s: &mut Session) -> String {
            let mut out = String::new();
            for cell in s.backend.sync().grid.display_iter() {
                out.push(cell.c);
            }
            out
        }
        fn wait_for(m: &mut TerminalManager, id: u64, needle: &str, secs: u64) {
            let deadline = Instant::now() + Duration::from_secs(secs);
            while !screen(m.get(id).unwrap()).contains(needle) {
                assert!(Instant::now() < deadline, "missing `{needle}`");
                std::thread::sleep(Duration::from_millis(50));
                m.poll_events();
            }
        }

        let mut m = mgr();
        // Deterministic profile: plain /bin/sh.
        let sh_idx = m
            .profiles
            .iter()
            .position(|p| p.shell == "/bin/sh")
            .unwrap_or_else(|| {
                m.profiles.push(Profile {
                    name: "sh".into(),
                    shell: "/bin/sh".into(),
                    args: vec![],
                });
                m.profiles.len() - 1
            });
        let id = m.new_session_with(sh_idx).expect("spawn sh");

        // Markers are assembled from pieces so the command ECHO can never
        // satisfy the assertion — only real output can.
        m.send(id, b"printf '%s%s\\n' OXIDE_ READY\n");
        wait_for(&mut m, id, "OXIDE_READY", 10);

        // cwd persists across commands.
        m.send(id, b"cd /\nprintf '%s%s%s\\n' P W $(pwd)\n");
        wait_for(&mut m, id, "PW/", 5);

        // Ctrl+C interrupts sleep; the shell answers again within seconds.
        m.send(id, b"sleep 30\n");
        std::thread::sleep(Duration::from_millis(300));
        m.send(id, b"\x03");
        m.send(id, b"printf '%s%s\\n' DO NE\n");
        wait_for(&mut m, id, "DONE", 5);

        // Real resize: cols/rows reported by `stty size` must track it.
        let cell = egui_term::Size::new(7.0, 14.0);
        m.get(id)
            .unwrap()
            .backend
            .process_command(egui_term::BackendCommand::Resize(
                egui_term::Size::new(400.0, 300.0),
                cell,
            ));
        m.send(id, b"stty size\n");
        wait_for(&mut m, id, "21", 5);
        m.get(id)
            .unwrap()
            .backend
            .process_command(egui_term::BackendCommand::Resize(
                egui_term::Size::new(800.0, 600.0),
                cell,
            ));
        m.send(id, b"stty size\n");
        wait_for(&mut m, id, "42", 5);

        // exit marks this session; a second one is unaffected.
        let id2 = m.new_session().expect("spawn sh 2");
        m.send(id, b"exit\n");
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            m.poll_events();
            if m.sessions().iter().find(|s| s.id == id).unwrap().exited {
                break;
            }
            assert!(Instant::now() < deadline, "no exit event");
            std::thread::sleep(Duration::from_millis(50));
        }
        assert!(!m.sessions().iter().find(|s| s.id == id2).unwrap().exited);
    }
}
