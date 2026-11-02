//! Integrated terminal: runs real OS processes (cargo, rustc, git, shells)
//! in a background thread and also supports instant builtins (`bench`, `help`,
//! `clear`) matching the `.aistudio` showcase.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

const MAX_LINES: usize = 5_000;
const MAX_LINE_BYTES: usize = 16 * 1024;
const MAX_HISTORY_BYTES: usize = 2 * 1024 * 1024;
const MAX_QUEUED_LINES: usize = 128;
const TRUNCATED: &str = " … [line truncated]";

/// One printed line in the terminal panel.
#[derive(Clone, Debug)]
pub struct TermLine {
    pub kind: TermKind,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TermKind {
    Input,
    Output,
    Error,
    Success,
    Info,
}

/// Background command runner feeding the terminal panel.
pub struct Terminal {
    lines: VecDeque<TermLine>,
    retained_bytes: usize,
    rx: Option<Receiver<TermLine>>,
    busy: bool,
    cwd: PathBuf,
}

impl Terminal {
    pub fn new(cwd: PathBuf) -> Self {
        let mut t = Self {
            lines: VecDeque::new(),
            retained_bytes: 0,
            rx: None,
            busy: false,
            cwd,
        };
        t.reset_banner();
        t
    }

    pub fn reset_banner(&mut self) {
        self.lines.clear();
        self.retained_bytes = 0;
    }

    pub fn set_cwd(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
    }

    #[allow(dead_code)]
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    pub fn lines(&self) -> &VecDeque<TermLine> {
        &self.lines
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.lines.clear();
        self.retained_bytes = 0;
    }

    pub fn push(&mut self, kind: TermKind, text: impl Into<String>) {
        let text = bounded_text(text.into());
        while self.lines.len() >= MAX_LINES || self.retained_bytes + text.len() > MAX_HISTORY_BYTES
        {
            let Some(line) = self.lines.pop_front() else {
                break;
            };
            self.retained_bytes -= line.text.len();
        }
        self.retained_bytes += text.len();
        self.lines.push_back(TermLine { kind, text });
    }

    /// Run a program directly (no shell): best for `cargo`, `rustc`, `git`.
    pub fn run_program(&mut self, program: &str, args: &[&str], cwd: &Path) {
        if self.busy {
            self.push(TermKind::Error, "a command is already running");
            return;
        }
        let echo = format!("{} $ {} {}", cwd.display(), program, args.join(" "));
        self.push(TermKind::Input, echo);
        self.cwd = cwd.to_path_buf();
        let (tx, rx) = mpsc::sync_channel(MAX_QUEUED_LINES);
        self.rx = Some(rx);
        self.busy = true;
        let program = program.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let cwd = cwd.to_path_buf();
        thread::spawn(move || {
            let mut child = match Command::new(&program)
                .args(&args)
                .current_dir(&cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(error) => {
                    let _ = tx.send(TermLine {
                        kind: TermKind::Error,
                        text: format!("failed to start `{program}`: {error}"),
                    });
                    return;
                }
            };
            let stdout = child.stdout.take().expect("piped stdout");
            let stderr = child.stderr.take().expect("piped stderr");
            let output_tx = tx.clone();
            let output = thread::spawn(move || stream_lines(stdout, TermKind::Output, &output_tx));
            let error_tx = tx.clone();
            let errors = thread::spawn(move || stream_lines(stderr, TermKind::Error, &error_tx));
            let result = child.wait();
            let _ = output.join();
            let _ = errors.join();
            let (kind, text) = match result {
                Ok(status) if status.success() => (
                    TermKind::Success,
                    format!("✓ Finished `{program}` ({status})"),
                ),
                Ok(status) => (TermKind::Error, format!("process exited with {status}")),
                Err(error) => (
                    TermKind::Error,
                    format!("cannot wait for `{program}`: {error}"),
                ),
            };
            let _ = tx.send(TermLine { kind, text });
        });
    }

    /// Run free-form input through builtins (`clear`, `help`, `bench`) or the platform shell.
    #[allow(dead_code)]
    pub fn poll(&mut self) {
        let Some(rx) = self.rx.take() else {
            return;
        };
        // Backpressure applies to both the queue and work done in one UI frame.
        for _ in 0..MAX_QUEUED_LINES {
            match rx.try_recv() {
                Ok(line) => self.push(line.kind, line.text),
                Err(mpsc::TryRecvError::Empty) => {
                    self.rx = Some(rx);
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.busy = false;
                    return;
                }
            }
        }
        self.rx = Some(rx);
    }

    /// Blocking helper used by non-interactive callers (diagnostics refresh).
    pub fn run_blocking(program: &str, args: &[&str], cwd: &Path) -> (bool, String) {
        match crate::process::capture(
            Command::new(program).args(args).current_dir(cwd),
            crate::process::Limits {
                timeout: std::time::Duration::from_secs(180),
                stdout: 16 * 1024 * 1024,
                stderr: 2 * 1024 * 1024,
            },
        ) {
            Err(e) => (false, format!("failed to start `{program}`: {e}")),
            Ok(out) => {
                let mut text = String::new();
                text.push_str(&String::from_utf8_lossy(&out.stdout));
                text.push_str(&String::from_utf8_lossy(&out.stderr));
                (out.status.success(), text)
            }
        }
    }
}

fn bounded_text(text: String) -> String {
    if text.len() <= MAX_LINE_BYTES {
        return text;
    }
    let mut end = MAX_LINE_BYTES - TRUNCATED.len();
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{}", &text[..end], TRUNCATED)
}

fn stream_lines(input: impl Read, kind: TermKind, tx: &SyncSender<TermLine>) {
    let mut reader = BufReader::new(input);
    let mut line = Vec::with_capacity(MAX_LINE_BYTES);
    let mut truncated = false;
    loop {
        let buffer = match reader.fill_buf() {
            Ok(buffer) => buffer,
            Err(error) => {
                let _ = tx.send(TermLine {
                    kind: TermKind::Error,
                    text: format!("cannot read command output: {error}"),
                });
                return;
            }
        };
        if buffer.is_empty() {
            if !line.is_empty() || truncated {
                let _ = tx.send(output_line(&line, truncated, kind));
            }
            return;
        }
        let consumed = buffer.len();
        for &byte in buffer {
            if byte == b'\n' {
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                if tx.send(output_line(&line, truncated, kind)).is_err() {
                    return;
                }
                line.clear();
                truncated = false;
            } else if line.len() < MAX_LINE_BYTES {
                line.push(byte);
            } else {
                truncated = true;
            }
        }
        reader.consume(consumed);
    }
}

fn output_line(bytes: &[u8], truncated: bool, kind: TermKind) -> TermLine {
    let mut text = String::from_utf8_lossy(bytes).into_owned();
    if truncated {
        text.push_str(TRUNCATED);
    }
    TermLine {
        kind,
        text: bounded_text(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocking_cargo_version() {
        let (ok, text) = Terminal::run_blocking("cargo", &["--version"], Path::new("."));
        assert!(ok, "cargo should run: {text}");
        assert!(text.contains("cargo"));
    }

    #[test]
    fn busy_flag_round_trip() {
        let mut t = Terminal::new(PathBuf::from("."));
        t.run_program("cargo", &["--version"], Path::new("."));
        for _ in 0..200 {
            t.poll();
            if !t.is_busy() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(!t.is_busy());
        assert!(t
            .lines()
            .iter()
            .any(|l| matches!(l.kind, TermKind::Success)));
    }
    #[test]
    #[ignore = "manual memory baseline; run with --ignored --nocapture --test-threads=1"]
    fn hardening_measure_output_retention() {
        let mut terminal = Terminal::new(PathBuf::from("."));
        let sample = |phase: &str, terminal: &Terminal| {
            let rss_kib = std::fs::read_to_string("/proc/self/status")
                .ok()
                .and_then(|status| {
                    status
                        .lines()
                        .find(|line| line.starts_with("VmRSS:"))
                        .and_then(|line| line.split_whitespace().nth(1))
                        .and_then(|value| value.parse::<u64>().ok())
                });
            let bytes: usize = terminal.lines.iter().map(|line| line.text.len()).sum();
            println!(
                "HARDENING_METRIC {}",
                serde_json::json!({"case":"output_retention","phase":phase,"rss_kib":rss_kib,"retained_bytes":bytes,"lines":terminal.lines.len()})
            );
        };
        for _ in 0..32 {
            terminal.push(TermKind::Output, "w".repeat(65_536));
        }
        sample("warmup", &terminal);
        for phase in ["N=200", "2N=400", "3N=600"] {
            for _ in 0..200 {
                terminal.push(TermKind::Output, "x".repeat(65_536));
            }
            sample(phase, &terminal);
        }
        terminal.clear();
        sample("cleared", &terminal);
    }
    #[test]
    fn mem_001_output_history_is_bounded_by_bytes() {
        let mut terminal = Terminal::new(PathBuf::from("."));
        for _ in 0..512 {
            terminal.push(TermKind::Output, "猫".repeat(21_846));
        }
        let bytes: usize = terminal.lines().iter().map(|line| line.text.len()).sum();
        assert!(bytes <= 2 * 1024 * 1024, "History retained {bytes} bytes");
        assert!(terminal
            .lines()
            .iter()
            .all(|line| line.text.len() <= 16 * 1024));
        terminal.clear();
        terminal.push(TermKind::Output, "still usable");
        assert_eq!(terminal.lines().len(), 1);
    }
    #[test]
    fn mem_001_streaming_handles_giant_lines_and_unicode() {
        let mut bytes = "猫".repeat(100_000).into_bytes();
        bytes.extend_from_slice(b"\r\nnext\n");
        let (tx, rx) = mpsc::sync_channel(8);
        stream_lines(std::io::Cursor::new(bytes), TermKind::Output, &tx);
        drop(tx);
        let lines: Vec<_> = rx.into_iter().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].text.len() <= MAX_LINE_BYTES);
        assert!(lines[0].text.ends_with(TRUNCATED));
        assert_eq!(lines[1].text, "next");
    }
}
