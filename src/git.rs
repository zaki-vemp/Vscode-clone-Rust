//! Real Git integration: async status snapshot, index/worktree actions,
//! and HEAD/index/working diffs. All invocations use `Command` args —
//! never interpolated shell strings; `--literal-pathspecs` and `--` guard
//! every path operand.

use eframe::egui;
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitEntry {
    /// Repo-relative path.
    pub path: PathBuf,
    /// Rename source when the index reports one.
    pub original_path: Option<PathBuf>,
    /// Index (staged) status letter, ' ' = clean.
    pub index: char,
    /// Worktree (unstaged) status letter, ' ' = clean.
    pub worktree: char,
}

impl GitEntry {
    pub fn is_staged(&self) -> bool {
        self.index != ' ' && self.index != '?' && self.index != '!'
    }
    pub fn is_unstaged(&self) -> bool {
        self.worktree != ' ' && self.worktree != '!'
    }
    pub fn is_untracked(&self) -> bool {
        self.index == '?'
    }
    pub fn is_conflicted(&self) -> bool {
        self.index == 'U'
            || self.worktree == 'U'
            || matches!((self.index, self.worktree), ('D', 'D') | ('A', 'A'))
    }
}

#[derive(Clone, Debug)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct GitSnapshot {
    pub root: PathBuf,
    pub branch: String,
    pub has_head: bool,
    pub entries: Vec<GitEntry>,
    pub branches: Vec<String>,
    pub stashes: Vec<StashEntry>,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

impl GitSnapshot {
    #[allow(dead_code)] // used by explorer/gutter decorations in the same module
    pub fn entry(&self, abs: &Path) -> Option<&GitEntry> {
        self.entries.iter().find(|e| self.root.join(&e.path) == abs)
    }
    pub fn staged(&self) -> impl Iterator<Item = &GitEntry> {
        self.entries.iter().filter(|e| e.is_staged())
    }
    pub fn changed(&self) -> impl Iterator<Item = &GitEntry> {
        self.entries
            .iter()
            .filter(|e| e.is_unstaged() || e.is_untracked())
    }
    pub fn conflicts(&self) -> impl Iterator<Item = &GitEntry> {
        self.entries.iter().filter(|e| e.is_conflicted())
    }
    pub fn untracked(&self) -> impl Iterator<Item = &GitEntry> {
        self.entries.iter().filter(|e| e.is_untracked())
    }
}

#[derive(Clone, Debug)]
pub struct GitDiff {
    #[allow(dead_code)] // consumed when a diff opens (SCM click)
    pub path: PathBuf,
    #[allow(dead_code)]
    pub staged: bool,
    /// None when the baseline side does not exist (unborn HEAD / untracked).
    pub old_text: Option<String>,
    /// None when the file is absent on disk (deleted).
    pub new_text: Option<String>,
    pub error: Option<String>,
}

/// What the user asked the worker to do (needed to react to outcomes).
#[derive(Debug, PartialEq)]
pub enum ActionKind {
    Stage,
    Unstage,
    Commit { message: String },
    Discard { path: PathBuf },
    DiscardAll,
    DiscardStaged { path: PathBuf },
    Clean { path: PathBuf },
    CleanAll,
    StashPush,
    StashApply { pop: bool },
    StashDrop,
    Fetch,
    Pull,
    Push,
    Publish,
    Sync,
    UndoCommit,
    Checkout { branch: String },
    CreateBranch { branch: String },
    DeleteBranch { branch: String },
    MergeBranch { branch: String },
}

pub enum GitResult {
    Snapshot(Box<Result<GitSnapshot, String>>),
    Action {
        kind: ActionKind,
        ok: bool,
        message: String,
    },
    Diff {
        id: u64,
        diff: Result<GitDiff, String>,
    },
    Index {
        path: PathBuf,
        text: Option<String>,
    },
}

enum GitRequest {
    Status {
        dir: PathBuf,
    },
    Index {
        root: PathBuf,
        rel: PathBuf,
    },
    Action(GitAction),
    Diff {
        id: u64,
        root: PathBuf,
        entry: GitEntry,
        staged: bool,
        has_head: bool,
    },
}

#[derive(Debug)]
enum GitAction {
    Stage {
        root: PathBuf,
        paths: Vec<PathBuf>,
    },
    Unstage {
        root: PathBuf,
        paths: Vec<PathBuf>,
        has_head: bool,
    },
    Commit {
        root: PathBuf,
        message: String,
        amend: bool,
        all: bool,
        signoff: bool,
    },
    Discard {
        root: PathBuf,
        path: PathBuf,
    },
    DiscardAll {
        root: PathBuf,
        paths: Vec<PathBuf>,
    },
    DiscardStaged {
        root: PathBuf,
        path: PathBuf,
        has_head: bool,
    },
    Clean {
        root: PathBuf,
        paths: Vec<PathBuf>,
    },
    StashPush {
        root: PathBuf,
        message: Option<String>,
        include_untracked: bool,
        keep_index: bool,
        paths: Vec<PathBuf>,
    },
    StashApply {
        root: PathBuf,
        index: usize,
        pop: bool,
    },
    StashDrop {
        root: PathBuf,
        index: usize,
    },
    Fetch {
        root: PathBuf,
    },
    Pull {
        root: PathBuf,
    },
    Push {
        root: PathBuf,
    },
    Publish {
        root: PathBuf,
        branch: String,
    },
    Sync {
        root: PathBuf,
    },
    UndoCommit {
        root: PathBuf,
    },
    Checkout {
        root: PathBuf,
        branch: String,
    },
    CreateBranch {
        root: PathBuf,
        branch: String,
        checkout: bool,
    },
    DeleteBranch {
        root: PathBuf,
        branch: String,
        force: bool,
    },
    MergeBranch {
        root: PathBuf,
        branch: String,
    },
}

/// Owns the worker thread; the channel serialises operations — requests
/// are always accepted (no dropped clicks) and `pending` tracks them.
pub struct GitState {
    pub snapshot: Option<GitSnapshot>,
    tx: Sender<GitRequest>,
    rx: Receiver<GitResult>,
    pending: usize,
    diff_tag: u64,
    pub last_error: Option<String>,
    ctx: egui::Context,
}

impl GitState {
    pub fn new(ctx: egui::Context) -> Self {
        let (tx, wrx) = channel::<GitRequest>();
        let (wtx, rx) = channel::<GitResult>();
        let wctx = ctx.clone();
        thread::spawn(move || {
            while let Ok(req) = wrx.recv() {
                let res = match req {
                    GitRequest::Status { dir } => GitResult::Snapshot(Box::new(status(&dir))),
                    GitRequest::Action(a) => {
                        let (kind, ok, message) = run_action(a);
                        GitResult::Action { kind, ok, message }
                    }
                    GitRequest::Index { root, rel } => {
                        let t = show_blob_limited(&root, &spec(":", &rel)).ok();
                        GitResult::Index {
                            path: root.join(rel),
                            text: t,
                        }
                    }
                    GitRequest::Diff {
                        id,
                        root,
                        entry,
                        staged,
                        has_head,
                    } => GitResult::Diff {
                        id,
                        diff: diff(&root, &entry, staged, has_head),
                    },
                };
                if wtx.send(res).is_err() {
                    return;
                }
                wctx.request_repaint();
            }
        });
        Self {
            snapshot: None,
            tx,
            rx,
            pending: 0,
            diff_tag: 0,
            last_error: None,
            ctx,
        }
    }

    /// Workspace changed: replace the whole worker so no stale result can
    /// ever arrive (old thread dies when its request channel drops).
    pub fn reset(&mut self, root: PathBuf) {
        *self = GitState::new(self.ctx.clone());
        self.request_status(root);
    }

    fn send(&mut self, r: GitRequest) {
        if self.tx.send(r).is_ok() {
            self.pending += 1;
        }
    }

    /// Auto status refresh only when the queue is idle.
    pub fn request_status(&mut self, dir: PathBuf) {
        if self.pending == 0 {
            self.send(GitRequest::Status { dir });
        }
    }

    pub fn stage(&mut self, root: &Path, paths: Vec<PathBuf>) {
        self.send(GitRequest::Action(GitAction::Stage {
            root: root.to_path_buf(),
            paths,
        }));
    }

    pub fn unstage(&mut self, root: &Path, paths: Vec<PathBuf>, has_head: bool) {
        self.send(GitRequest::Action(GitAction::Unstage {
            root: root.to_path_buf(),
            paths,
            has_head,
        }));
    }

    pub fn commit(&mut self, root: &Path, message: String) {
        self.commit_opts(root, message, false, false, false);
    }
    pub fn commit_opts(
        &mut self,
        root: &Path,
        message: String,
        amend: bool,
        all: bool,
        signoff: bool,
    ) {
        self.send(GitRequest::Action(GitAction::Commit {
            root: root.to_path_buf(),
            message,
            amend,
            all,
            signoff,
        }));
    }

    /// Restore a tracked file from the index — UI must confirm first.
    pub fn discard(&mut self, root: &Path, path: PathBuf) {
        self.send(GitRequest::Action(GitAction::Discard {
            root: root.to_path_buf(),
            path,
        }));
    }
    pub fn discard_all(&mut self, root: &Path, paths: Vec<PathBuf>) {
        self.send(GitRequest::Action(GitAction::DiscardAll {
            root: root.to_path_buf(),
            paths,
        }));
    }
    pub fn discard_staged(&mut self, root: &Path, path: PathBuf, has_head: bool) {
        self.send(GitRequest::Action(GitAction::DiscardStaged {
            root: root.to_path_buf(),
            path,
            has_head,
        }));
    }
    /// Delete untracked file(s) via `git clean` — UI must confirm first.
    pub fn clean(&mut self, root: &Path, paths: Vec<PathBuf>) {
        self.send(GitRequest::Action(GitAction::Clean {
            root: root.to_path_buf(),
            paths,
        }));
    }
    pub fn stash_push(
        &mut self,
        root: &Path,
        message: Option<String>,
        include_untracked: bool,
        keep_index: bool,
        paths: Vec<PathBuf>,
    ) {
        self.send(GitRequest::Action(GitAction::StashPush {
            root: root.to_path_buf(),
            message,
            include_untracked,
            keep_index,
            paths,
        }));
    }
    pub fn stash_apply(&mut self, root: &Path, index: usize, pop: bool) {
        self.send(GitRequest::Action(GitAction::StashApply {
            root: root.to_path_buf(),
            index,
            pop,
        }));
    }
    pub fn stash_drop(&mut self, root: &Path, index: usize) {
        self.send(GitRequest::Action(GitAction::StashDrop {
            root: root.to_path_buf(),
            index,
        }));
    }
    pub fn fetch(&mut self, root: &Path) {
        self.send(GitRequest::Action(GitAction::Fetch {
            root: root.to_path_buf(),
        }));
    }
    pub fn pull(&mut self, root: &Path) {
        self.send(GitRequest::Action(GitAction::Pull {
            root: root.to_path_buf(),
        }));
    }
    pub fn push(&mut self, root: &Path) {
        self.send(GitRequest::Action(GitAction::Push {
            root: root.to_path_buf(),
        }));
    }
    pub fn publish(&mut self, root: &Path, branch: String) {
        self.send(GitRequest::Action(GitAction::Publish {
            root: root.to_path_buf(),
            branch,
        }));
    }
    pub fn sync(&mut self, root: &Path) {
        self.send(GitRequest::Action(GitAction::Sync {
            root: root.to_path_buf(),
        }));
    }
    pub fn undo_commit(&mut self, root: &Path) {
        self.send(GitRequest::Action(GitAction::UndoCommit {
            root: root.to_path_buf(),
        }));
    }
    pub fn checkout(&mut self, root: &Path, branch: String) {
        self.send(GitRequest::Action(GitAction::Checkout {
            root: root.to_path_buf(),
            branch,
        }));
    }
    pub fn create_branch(&mut self, root: &Path, branch: String, checkout: bool) {
        self.send(GitRequest::Action(GitAction::CreateBranch {
            root: root.to_path_buf(),
            branch,
            checkout,
        }));
    }
    pub fn delete_branch(&mut self, root: &Path, branch: String, force: bool) {
        self.send(GitRequest::Action(GitAction::DeleteBranch {
            root: root.to_path_buf(),
            branch,
            force,
        }));
    }
    pub fn merge_branch(&mut self, root: &Path, branch: String) {
        self.send(GitRequest::Action(GitAction::MergeBranch {
            root: root.to_path_buf(),
            branch,
        }));
    }

    /// Invalidate any pending diff result (navigating away).
    pub fn cancel_diff(&mut self) {
        self.diff_tag += 1;
    }

    /// Index baseline text of a path (for editor gutter markers).
    pub fn request_index(&mut self, root: &Path, rel: PathBuf) {
        self.send(GitRequest::Index {
            root: root.to_path_buf(),
            rel,
        });
    }

    /// Newer diff requests supersede older queued ones.
    pub fn request_diff(&mut self, root: &Path, entry: &GitEntry, staged: bool, has_head: bool) {
        self.diff_tag += 1;
        self.send(GitRequest::Diff {
            id: self.diff_tag,
            root: root.to_path_buf(),
            entry: entry.clone(),
            staged,
            has_head,
        });
    }

    /// Drains one result; stale diff responses are dropped here.
    pub fn poll(&mut self) -> Option<GitResult> {
        loop {
            match self.rx.try_recv() {
                Ok(r) => {
                    self.pending = self.pending.saturating_sub(1);
                    if let GitResult::Diff { id, .. } = &r {
                        if *id != self.diff_tag {
                            continue;
                        }
                    }
                    return Some(r);
                }
                Err(_) => return None,
            }
        }
    }

    pub fn is_busy(&self) -> bool {
        self.pending > 0
    }
}

fn git_output(command: &mut Command) -> std::io::Result<std::process::Output> {
    crate::process::capture(
        command,
        crate::process::Limits {
            timeout: std::time::Duration::from_secs(120),
            stdout: 16 * 1024 * 1024,
            stderr: 256 * 1024,
        },
    )
}

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    git_output(Command::new("git").current_dir(root).args(args)).map_err(|e| format!("git: {e}"))
}

/// Walk up from `dir` to the repository working-tree root.
pub fn repo_root(dir: &Path) -> Option<PathBuf> {
    let out = git(dir, &["rev-parse", "--show-toplevel"]).ok()?;
    if !out.status.success() {
        return None;
    }
    let mut b = out.stdout;
    if b.last() == Some(&b'\n') {
        b.pop(); // exactly the trailing newline, nothing else
    }
    Some(bytes_to_path(b))
}

fn branch_of(root: &Path) -> (String, bool) {
    // HEAD may not exist yet (unborn branch): verify separately.
    let has_head = git(root, &["rev-parse", "--verify", "-q", "HEAD^{commit}"])
        .map(|o| o.status.success())
        .unwrap_or(false);
    if let Ok(out) = git(root, &["symbolic-ref", "--short", "-q", "HEAD"]) {
        if out.status.success() {
            return (
                String::from_utf8_lossy(&out.stdout).trim().to_string(),
                has_head,
            );
        }
    }
    // Detached HEAD: show the short hash.
    if has_head {
        if let Ok(out) = git(root, &["rev-parse", "--short", "HEAD"]) {
            if out.status.success() {
                return (
                    String::from_utf8_lossy(&out.stdout).trim().to_string(),
                    true,
                );
            }
        }
    }
    ("master".into(), has_head)
}

#[cfg(unix)]
fn bytes_to_path(b: Vec<u8>) -> PathBuf {
    PathBuf::from(OsString::from_vec(b))
}
#[cfg(not(unix))]
fn bytes_to_path(b: Vec<u8>) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(&b).into_owned())
}

fn status(dir: &Path) -> Result<GitSnapshot, String> {
    let root = repo_root(dir).ok_or_else(|| "not a git repository".to_string())?;
    let (branch, has_head) = branch_of(&root);
    let out = git(
        &root,
        &[
            "-c",
            "core.quotepath=false",
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
        ],
    )?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let entries = parse_porcelain(&out.stdout);
    let branches = git(&root, &["branch", "--format=%(refname:short)"])
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let stashes = git(&root, &["stash", "list", "--format=%gd%x1f%s"])
        .ok()
        .filter(|o| o.status.success())
        .map(|o| parse_stash_list(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    let upstream = git(
        &root,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    )
    .ok()
    .filter(|o| o.status.success())
    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    .filter(|s| !s.is_empty());
    let (ahead, behind) = upstream
        .as_ref()
        .and_then(|_| {
            git(
                &root,
                &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
            )
            .ok()
        })
        .filter(|o| o.status.success())
        .and_then(|o| {
            let t = String::from_utf8_lossy(&o.stdout).to_string();
            let mut it = t.split_whitespace();
            Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
        })
        .unwrap_or((0, 0));
    Ok(GitSnapshot {
        root,
        branch,
        has_head,
        entries,
        branches,
        stashes,
        upstream,
        ahead,
        behind,
    })
}

/// Parse `stash@{N}\x1f<subject>` lines.
pub fn parse_stash_list(text: &str) -> Vec<StashEntry> {
    text.lines()
        .filter_map(|line| {
            let (gd, msg) = line.split_once('\u{1f}')?;
            let index: usize = gd
                .trim_start_matches("stash@{")
                .trim_end_matches('}')
                .parse()
                .ok()?;
            Some(StashEntry {
                index,
                message: msg.trim().to_string(),
            })
        })
        .collect()
}

fn valid_branch(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.starts_with('-')
        || name.contains([' ', '\t', '\n', '~', '^', ':', '?', '*', '[', '\\'])
    {
        return Err(format!("invalid branch name: {name:?}"));
    }
    Ok(())
}

/// Parse `git status --porcelain=v1 -z` output.
/// Records are NUL-separated: "XY <path>"; renames append "orig\0" next.
pub fn parse_porcelain(bytes: &[u8]) -> Vec<GitEntry> {
    let mut it = bytes.split(|b| *b == 0);
    let mut out = Vec::new();
    while let Some(rec) = it.next() {
        if rec.len() < 4 {
            continue;
        }
        let index = rec[0] as char;
        let worktree = rec[1] as char;
        // rec[2] is a space; the path occupies the rest of the record.
        let path = bytes_to_path(rec[3..].to_vec());
        let original_path = if index == 'R' || index == 'C' || worktree == 'R' {
            it.next()
                .filter(|p| !p.is_empty())
                .map(|p| bytes_to_path(p.to_vec()))
        } else {
            None
        };
        out.push(GitEntry {
            path,
            original_path,
            index,
            worktree,
        });
    }
    out
}

fn run_action(a: GitAction) -> (ActionKind, bool, String) {
    match a {
        GitAction::Stage { root, paths } => {
            let mut args: Vec<OsString> = vec![
                "--literal-pathspecs".into(),
                "add".into(),
                "-A".into(),
                "--".into(),
            ];
            args.extend(paths.iter().map(|p| p.clone().into_os_string()));
            let (ok, m) = run_args(&root, &args);
            (ActionKind::Stage, ok, m)
        }
        GitAction::Unstage {
            root,
            paths,
            has_head,
        } => {
            let mut args: Vec<OsString> = vec!["--literal-pathspecs".into()];
            if has_head {
                // `reset -q HEAD -- paths` resets only the index.
                args.extend(["reset".into(), "-q".into(), "HEAD".into(), "--".into()]);
            } else {
                // Unborn HEAD: drop index entries only, never disk content.
                args.extend([
                    "rm".into(),
                    "-q".into(),
                    "--cached".into(),
                    "-r".into(),
                    "-f".into(),
                    "--ignore-unmatch".into(),
                    "--".into(),
                ]);
            }
            args.extend(paths.iter().map(|p| p.clone().into_os_string()));
            let (ok, m) = run_args(&root, &args);
            (ActionKind::Unstage, ok, m)
        }
        GitAction::Commit {
            root,
            message,
            amend,
            all,
            signoff,
        } => {
            let mut args: Vec<OsString> = vec!["commit".into()];
            if all {
                args.push("-a".into());
            }
            if signoff {
                args.push("-s".into());
            }
            if amend {
                args.push("--amend".into());
                if message.trim().is_empty() {
                    args.push("--no-edit".into());
                } else {
                    args.push("-m".into());
                    args.push(message.clone().into());
                }
            } else {
                args.push("-m".into());
                args.push(message.clone().into());
            }
            let (ok, m) = run_args(&root, &args);
            (ActionKind::Commit { message }, ok, m)
        }
        GitAction::Discard { root, path } => {
            // Guard: only blob entries may be restored (no submodule/gitlink).
            let ty = git_output(Command::new("git").current_dir(&root).args([
                OsString::from("cat-file"),
                OsString::from("-t"),
                spec(":", &path),
            ]))
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
            if ty != "blob" {
                return (
                    ActionKind::Discard { path },
                    false,
                    format!("cannot discard: not a tracked blob ({ty})"),
                );
            }
            let (ok, m) = run_args(
                &root,
                &[
                    "--literal-pathspecs".into(),
                    "restore".into(),
                    "--worktree".into(),
                    "--".into(),
                    path.clone().into_os_string(),
                ],
            );
            (ActionKind::Discard { path }, ok, m)
        }
        GitAction::DiscardAll { root, paths } => {
            if paths.is_empty() {
                return (
                    ActionKind::DiscardAll,
                    false,
                    "no tracked changes to discard".into(),
                );
            }
            let mut args: Vec<OsString> = vec![
                "--literal-pathspecs".into(),
                "restore".into(),
                "--worktree".into(),
                "--".into(),
            ];
            args.extend(paths.iter().map(|p| p.clone().into_os_string()));
            let (ok, m) = run_args(&root, &args);
            (ActionKind::DiscardAll, ok, m)
        }
        GitAction::DiscardStaged {
            root,
            path,
            has_head,
        } => {
            if has_head {
                let (ok, m) = run_args(
                    &root,
                    &[
                        "--literal-pathspecs".into(),
                        "restore".into(),
                        "--staged".into(),
                        "--worktree".into(),
                        "--".into(),
                        path.clone().into_os_string(),
                    ],
                );
                (ActionKind::DiscardStaged { path }, ok, m)
            } else {
                // Unborn HEAD: remove from index, then delete the worktree file.
                let _ = run_args(
                    &root,
                    &[
                        "--literal-pathspecs".into(),
                        "rm".into(),
                        "-q".into(),
                        "--cached".into(),
                        "-r".into(),
                        "-f".into(),
                        "--ignore-unmatch".into(),
                        "--".into(),
                        path.clone().into_os_string(),
                    ],
                );
                let abs = root.join(&path);
                let ok = std::fs::remove_file(&abs).is_ok() || !abs.exists();
                (
                    ActionKind::DiscardStaged { path },
                    ok,
                    if ok {
                        String::new()
                    } else {
                        "cannot delete file".into()
                    },
                )
            }
        }
        GitAction::Clean { root, paths } => {
            let single = paths.first().cloned();
            if paths.is_empty() {
                return (
                    ActionKind::CleanAll,
                    false,
                    "no untracked files to clean".into(),
                );
            }
            let mut args: Vec<OsString> = vec![
                "--literal-pathspecs".into(),
                "clean".into(),
                "-f".into(),
                "-d".into(),
                "--".into(),
            ];
            args.extend(paths.iter().map(|p| p.clone().into_os_string()));
            let (ok, m) = run_args(&root, &args);
            if paths.len() == 1 {
                (
                    ActionKind::Clean {
                        path: single.unwrap(),
                    },
                    ok,
                    m,
                )
            } else {
                (ActionKind::CleanAll, ok, m)
            }
        }
        GitAction::StashPush {
            root,
            message,
            include_untracked,
            keep_index,
            paths,
        } => {
            let mut args: Vec<OsString> = vec!["stash".into(), "push".into()];
            if include_untracked {
                args.push("--include-untracked".into());
            }
            if keep_index {
                args.push("--keep-index".into());
            }
            if let Some(msg) = message.as_ref().filter(|s| !s.trim().is_empty()) {
                args.push("-m".into());
                args.push(msg.clone().into());
            }
            if !paths.is_empty() {
                args.push("--".into());
                args.extend(paths.iter().map(|p| p.clone().into_os_string()));
                // pathspec needs literal guard at top level
                let mut full: Vec<OsString> = vec!["--literal-pathspecs".into()];
                full.extend(args);
                let (ok, m) = run_args(&root, &full);
                return (ActionKind::StashPush, ok, m);
            }
            let (ok, m) = run_args(&root, &args);
            (ActionKind::StashPush, ok, m)
        }
        GitAction::StashApply { root, index, pop } => {
            let spec_s = format!("stash@{{{index}}}");
            let (ok, m) = run_args(
                &root,
                &[
                    "stash".into(),
                    if pop { "pop".into() } else { "apply".into() },
                    spec_s.into(),
                ],
            );
            (ActionKind::StashApply { pop }, ok, m)
        }
        GitAction::StashDrop { root, index } => {
            let spec_s = format!("stash@{{{index}}}");
            let (ok, m) = run_args(&root, &["stash".into(), "drop".into(), spec_s.into()]);
            (ActionKind::StashDrop, ok, m)
        }
        GitAction::Fetch { root } => {
            let (ok, m) = run_args(&root, &["fetch".into(), "--all".into(), "--prune".into()]);
            (ActionKind::Fetch, ok, m)
        }
        GitAction::Pull { root } => {
            let (ok, m) = run_args(&root, &["pull".into()]);
            (ActionKind::Pull, ok, m)
        }
        GitAction::Push { root } => {
            let (ok, m) = run_args(&root, &["push".into()]);
            (ActionKind::Push, ok, m)
        }
        GitAction::Publish { root, branch } => {
            if let Err(e) = valid_branch(&branch) {
                return (ActionKind::Publish, false, e);
            }
            let (ok, m) = run_args(
                &root,
                &["push".into(), "-u".into(), "origin".into(), branch.into()],
            );
            (ActionKind::Publish, ok, m)
        }
        GitAction::Sync { root } => {
            let (ok1, m1) = run_args(&root, &["pull".into()]);
            if !ok1 {
                return (ActionKind::Sync, false, m1);
            }
            let (ok2, m2) = run_args(&root, &["push".into()]);
            (
                ActionKind::Sync,
                ok2,
                [m1, m2]
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        }
        GitAction::UndoCommit { root } => {
            let (ok, m) = run_args(&root, &["reset".into(), "--soft".into(), "HEAD~1".into()]);
            (ActionKind::UndoCommit, ok, m)
        }
        GitAction::Checkout { root, branch } => {
            if let Err(e) = valid_branch(&branch) {
                return (ActionKind::Checkout { branch }, false, e);
            }
            let (ok, m) = run_args(&root, &["switch".into(), branch.clone().into()]);
            (ActionKind::Checkout { branch }, ok, m)
        }
        GitAction::CreateBranch {
            root,
            branch,
            checkout,
        } => {
            if let Err(e) = valid_branch(&branch) {
                return (ActionKind::CreateBranch { branch }, false, e);
            }
            let (ok, m) = if checkout {
                run_args(
                    &root,
                    &["switch".into(), "-c".into(), branch.clone().into()],
                )
            } else {
                run_args(&root, &["branch".into(), branch.clone().into()])
            };
            (ActionKind::CreateBranch { branch }, ok, m)
        }
        GitAction::DeleteBranch {
            root,
            branch,
            force,
        } => {
            if let Err(e) = valid_branch(&branch) {
                return (ActionKind::DeleteBranch { branch }, false, e);
            }
            let (ok, m) = run_args(
                &root,
                &[
                    "branch".into(),
                    if force { "-D".into() } else { "-d".into() },
                    branch.clone().into(),
                ],
            );
            (ActionKind::DeleteBranch { branch }, ok, m)
        }
        GitAction::MergeBranch { root, branch } => {
            if let Err(e) = valid_branch(&branch) {
                return (ActionKind::MergeBranch { branch }, false, e);
            }
            let (ok, m) = run_args(&root, &["merge".into(), branch.clone().into()]);
            (ActionKind::MergeBranch { branch }, ok, m)
        }
    }
}

fn run_args(root: &Path, args: &[OsString]) -> (bool, String) {
    match git_output(Command::new("git").current_dir(root).args(args)) {
        Ok(o) if o.status.success() => {
            (true, String::from_utf8_lossy(&o.stdout).trim().to_string())
        }
        Ok(o) => {
            // git reports failures on either stream (e.g. "nothing to commit").
            let mut m = String::from_utf8_lossy(&o.stderr).trim().to_string();
            if m.is_empty() {
                m = String::from_utf8_lossy(&o.stdout).trim().to_string();
            }
            (false, m)
        }
        Err(e) => (false, format!("git: {e}")),
    }
}

const MAX_DIFF_BYTES: u64 = 4 * 1024 * 1024;

fn spec(prefix: &str, rel: &Path) -> OsString {
    let mut s = OsString::from(prefix);
    s.push(rel.as_os_str());
    s
}

/// Read a blob only after confirming its size is within the diff budget.
fn show_blob_limited(root: &Path, spec: &OsString) -> Result<String, String> {
    // Submodules/commit objects are not renderable text.
    let ty = git_output(Command::new("git").current_dir(root).args([
        OsString::from("cat-file"),
        OsString::from("-t"),
        spec.clone(),
    ]))
    .map_err(|e| e.to_string())?;
    if !ty.status.success() {
        return Err(String::from_utf8_lossy(&ty.stderr).trim().to_string());
    }
    if String::from_utf8_lossy(&ty.stdout).trim() != "blob" {
        return Err("not a regular file (submodule?) — diff unsupported".into());
    }
    let size = git_output(Command::new("git").current_dir(root).args([
        OsString::from("cat-file"),
        OsString::from("-s"),
        spec.clone(),
    ]))
    .map_err(|e| e.to_string())?;
    if !size.status.success() {
        return Err(String::from_utf8_lossy(&size.stderr).trim().to_string());
    }
    let n: u64 = String::from_utf8_lossy(&size.stdout)
        .trim()
        .parse()
        .map_err(|_| "bad cat-file size".to_string())?;
    if n > MAX_DIFF_BYTES {
        return Err("file too large to diff".into());
    }
    let out = git_output(Command::new("git").current_dir(root).args([
        OsString::from("cat-file"),
        OsString::from("-p"),
        spec.clone(),
    ]))
    .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    text_or_err(out.stdout)
}

fn text_or_err(b: Vec<u8>) -> Result<String, String> {
    if b.contains(&0) {
        return Err("binary file — diff unsupported".into());
    }
    String::from_utf8(b).map_err(|_| "non-UTF-8 file — diff unsupported".into())
}

/// `staged` => HEAD -> index; else index -> worktree. Side existence is
/// decided from the entry's status letters, never stderr text matching.
fn diff(root: &Path, entry: &GitEntry, staged: bool, has_head: bool) -> Result<GitDiff, String> {
    let mut d = GitDiff {
        path: root.join(&entry.path),
        staged,
        old_text: None,
        new_text: None,
        error: None,
    };
    if entry.is_conflicted() {
        d.error = Some("conflicted file — resolve conflicts first".into());
        return Ok(d);
    }
    if staged {
        // HEAD side: absent when unborn or the index entry is an addition.
        // Renames compare against the ORIGINAL path in HEAD.
        if !has_head || entry.index == 'A' || entry.is_untracked() {
            d.old_text = None;
        } else {
            let head_path = entry.original_path.as_ref().unwrap_or(&entry.path);
            match show_blob_limited(root, &spec("HEAD:", head_path)) {
                Ok(t) => d.old_text = Some(t),
                Err(e) => {
                    d.error = Some(e);
                    return Ok(d);
                }
            }
        }
        // Index side: absent when the index entry is a deletion.
        if entry.index == 'D' {
            d.new_text = None;
        } else {
            match show_blob_limited(root, &spec(":", &entry.path)) {
                Ok(t) => d.new_text = Some(t),
                Err(e) => {
                    d.error = Some(e);
                    return Ok(d);
                }
            }
        }
    } else {
        // Index baseline: absent for untracked entries.
        if entry.is_untracked() {
            d.old_text = None;
        } else {
            match show_blob_limited(root, &spec(":", &entry.path)) {
                Ok(t) => d.old_text = Some(t),
                Err(e) => {
                    d.error = Some(e);
                    return Ok(d);
                }
            }
        }
        // Disk side: absent when the worktree says deleted.
        if entry.worktree == 'D' {
            d.new_text = None;
        } else {
            let abs = root.join(&entry.path);
            match std::fs::metadata(&abs) {
                Ok(md) if md.len() > MAX_DIFF_BYTES => {
                    d.error = Some("file too large to diff".into());
                    return Ok(d);
                }
                Ok(_) => match std::fs::read(&abs) {
                    Ok(b) => match text_or_err(b) {
                        Ok(t) => d.new_text = Some(t),
                        Err(e) => {
                            d.error = Some(e);
                            return Ok(d);
                        }
                    },
                    Err(e) => {
                        d.error = Some(e.to_string());
                        return Ok(d);
                    }
                },
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => d.new_text = None,
                Err(e) => {
                    d.error = Some(e.to_string());
                    return Ok(d);
                }
            }
        }
    }
    Ok(d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{Duration, Instant};

    struct Repo(PathBuf);
    impl Repo {
        fn new() -> Self {
            static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            // Unique per test AND per call; never wipe a preexisting path.
            let d = std::env::temp_dir().join(format!(
                "oxide-git-test-{}-{}-{}",
                std::process::id(),
                N.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&d).unwrap();
            let g = |args: &[&str]| {
                let o = Command::new("git")
                    .current_dir(&d)
                    .args(args)
                    .output()
                    .unwrap();
                assert!(
                    o.status.success(),
                    "{args:?}: {}",
                    String::from_utf8_lossy(&o.stderr)
                );
            };
            g(&["init", "-b", "main"]);
            g(&["config", "user.email", "t@t"]);
            g(&["config", "user.name", "t"]);
            g(&["config", "commit.gpgsign", "false"]);
            Repo(d)
        }
        fn write(&self, rel: &str, text: &str) {
            let p = self.0.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, text).unwrap();
        }
        fn commit_all(&self, msg: &str) {
            let o = Command::new("git")
                .current_dir(&self.0)
                .args(["add", "-A"])
                .output()
                .unwrap();
            assert!(o.status.success());
            let o = Command::new("git")
                .current_dir(&self.0)
                .args(["commit", "-qm", msg])
                .output()
                .unwrap();
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        }
        fn snap(&self) -> GitSnapshot {
            status(&self.0).unwrap()
        }
        fn entry(&self, rel: &str) -> GitEntry {
            self.snap()
                .entries
                .into_iter()
                .find(|e| e.path == Path::new(rel))
                .unwrap()
        }
    }
    impl Drop for Repo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn wait_result(m: &mut GitState) -> GitResult {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(r) = m.poll() {
                return r;
            }
            assert!(Instant::now() < deadline, "worker never replied");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[test]
    fn parses_porcelain_records() {
        let raw = b" M src/a.rs\0M  staged.rs\0R  new name\0old name\0?? untracked.txt\0";
        let e = parse_porcelain(raw);
        assert_eq!(e.len(), 4);
        assert_eq!(e[0].worktree, 'M');
        assert_eq!(e[2].original_path, Some(PathBuf::from("old name")));
        assert!(e[3].is_untracked());
    }

    #[test]
    fn status_staged_unstaged_untracked() {
        let r = Repo::new();
        r.write("a.rs", "fn a() {}\n");
        r.commit_all("init");
        r.write("a.rs", "fn a() { let x = 1; }\n");
        r.write("new file.rs", "x\n");
        let snap = r.snap();
        assert_eq!(snap.branch, "main");
        assert!(snap.has_head);
        let a = snap
            .entries
            .iter()
            .find(|e| e.path == Path::new("a.rs"))
            .unwrap();
        assert!(a.is_unstaged() && !a.is_staged());
        assert!(snap.entries.iter().any(|e| e.is_untracked()));
    }

    #[test]
    fn unborn_has_head_false() {
        let r = Repo::new();
        let snap = r.snap();
        assert!(!snap.has_head);
        assert_eq!(snap.branch, "main");
    }

    #[test]
    fn stage_unstage_preserves_disk() {
        let r = Repo::new();
        r.write("a.rs", "v1\n");
        r.commit_all("init");
        r.write("a.rs", "v2\n");
        let (_, ok, _) = run_action(GitAction::Stage {
            root: r.0.clone(),
            paths: vec![PathBuf::from("a.rs")],
        });
        assert!(ok);
        let (_, ok, _) = run_action(GitAction::Unstage {
            root: r.0.clone(),
            paths: vec![PathBuf::from("a.rs")],
            has_head: true,
        });
        assert!(ok);
        assert_eq!(fs::read_to_string(r.0.join("a.rs")).unwrap(), "v2\n");
    }

    #[test]
    fn unborn_unstage_keeps_disk_and_index_differs() {
        let r = Repo::new();
        r.write("a.rs", "on disk\n");
        run_action(GitAction::Stage {
            root: r.0.clone(),
            paths: vec![PathBuf::from("a.rs")],
        });
        // Change disk after staging so index != worktree.
        r.write("a.rs", "on disk edited\n");
        let has_head = r.snap().has_head;
        assert!(!has_head);
        let (_, ok, _) = run_action(GitAction::Unstage {
            root: r.0.clone(),
            paths: vec![PathBuf::from("a.rs")],
            has_head,
        });
        assert!(ok);
        assert_eq!(
            fs::read_to_string(r.0.join("a.rs")).unwrap(),
            "on disk edited\n"
        );
    }

    #[test]
    fn rename_diff_uses_original_path() {
        let r = Repo::new();
        r.write("old.rs", "v1\n");
        r.commit_all("init");
        fs::rename(r.0.join("old.rs"), r.0.join("new.rs")).unwrap();
        run_action(GitAction::Stage {
            root: r.0.clone(),
            paths: vec![PathBuf::from("new.rs"), PathBuf::from("old.rs")],
        });
        let snap = r.snap();
        let e = snap.entries.iter().find(|e| e.index == 'R').unwrap();
        // staged diff: HEAD side must come from old.rs, not full addition.
        let d = diff(&r.0, e, true, snap.has_head).unwrap();
        assert_eq!(d.old_text.as_deref(), Some("v1\n"));
        assert_eq!(d.new_text.as_deref(), Some("v1\n"));
    }

    #[test]
    fn odd_names_stage_literally() {
        let r = Repo::new();
        r.write("a.rs", "x\n");
        r.commit_all("init");
        for name in ["weird -name*.rs", "with space.rs", "-leading-dash.rs"] {
            r.write(name, "y\n");
            let (_, ok, m) = run_action(GitAction::Stage {
                root: r.0.clone(),
                paths: vec![PathBuf::from(name)],
            });
            assert!(ok, "{name}: {m}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn newline_and_rawbyte_paths() {
        use std::os::unix::ffi::OsStrExt;
        let r = Repo::new();
        r.write("a.rs", "x\n");
        r.commit_all("init");
        let weird = "line1\nline2.rs";
        r.write(weird, "z\n");
        let (kind, ok, m) = run_action(GitAction::Stage {
            root: r.0.clone(),
            paths: vec![PathBuf::from(std::ffi::OsStr::from_bytes(weird.as_bytes()))],
        });
        assert!(ok, "{m}");
        assert_eq!(kind, ActionKind::Stage);
        let snap = r.snap();
        assert!(snap.entries.iter().any(|e| e.path == Path::new(weird)));
    }

    #[test]
    fn commit_metachars_and_failure() {
        let r = Repo::new();
        r.write("a.rs", "x\n");
        run_action(GitAction::Stage {
            root: r.0.clone(),
            paths: vec![PathBuf::from("a.rs")],
        });
        let (kind, ok, _) = run_action(GitAction::Commit {
            root: r.0.clone(),
            message: "msg with $(rm -rf) `back` \"quotes\"".into(),
            amend: false,
            all: false,
            signoff: false,
        });
        assert!(ok);
        assert!(matches!(kind, ActionKind::Commit { .. }));
        let out = Command::new("git")
            .current_dir(&r.0)
            .args(["log", "-1", "--format=%s"])
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&out.stdout).trim(),
            "msg with $(rm -rf) `back` \"quotes\""
        );
        let (_, ok, msg) = run_action(GitAction::Commit {
            root: r.0.clone(),
            message: "again".into(),
            amend: false,
            all: false,
            signoff: false,
        });
        assert!(!ok);
        assert!(!msg.is_empty());
    }

    #[test]
    fn discard_restores_tracked_only() {
        let r = Repo::new();
        r.write("a.rs", "v1\n");
        r.commit_all("init");
        r.write("a.rs", "v2\n");
        let (kind, ok, _) = run_action(GitAction::Discard {
            root: r.0.clone(),
            path: PathBuf::from("a.rs"),
        });
        assert!(ok);
        assert!(matches!(kind, ActionKind::Discard { .. }));
        assert_eq!(fs::read_to_string(r.0.join("a.rs")).unwrap(), "v1\n");
    }

    #[test]
    fn diff_sides_and_deleted() {
        let r = Repo::new();
        r.write("a.rs", "v1\n");
        r.commit_all("init");
        r.write("a.rs", "v2\n");
        let e = r.entry("a.rs");
        let d = diff(&r.0, &e, false, true).unwrap();
        assert_eq!(d.old_text.as_deref(), Some("v1\n"));
        assert_eq!(d.new_text.as_deref(), Some("v2\n"));
        run_action(GitAction::Stage {
            root: r.0.clone(),
            paths: vec![PathBuf::from("a.rs")],
        });
        let e = r.entry("a.rs");
        let d = diff(&r.0, &e, true, true).unwrap();
        assert_eq!(d.old_text.as_deref(), Some("v1\n"));
        assert_eq!(d.new_text.as_deref(), Some("v2\n"));
        fs::remove_file(r.0.join("a.rs")).unwrap();
        let e = r.entry("a.rs");
        let d = diff(&r.0, &e, false, true).unwrap();
        assert!(d.new_text.is_none());
        r.write("u.rs", "u\n");
        let e = r.entry("u.rs");
        let d = diff(&r.0, &e, false, true).unwrap();
        assert!(d.old_text.is_none());
        assert_eq!(d.new_text.as_deref(), Some("u\n"));
    }

    #[test]
    fn binary_and_oversized_rejected() {
        let r = Repo::new();
        r.write("a.rs", "x\n");
        r.commit_all("init");
        fs::write(r.0.join("a.rs"), b"bin\0ary").unwrap();
        let e = r.entry("a.rs");
        let d = diff(&r.0, &e, false, true).unwrap();
        assert!(d.error.is_some());
    }

    #[test]
    fn subfolder_discovers_root() {
        let r = Repo::new();
        r.write("src/a.rs", "x\n");
        r.commit_all("init");
        assert_eq!(repo_root(&r.0.join("src")).unwrap(), r.0);
    }

    #[test]
    fn subfolder_workspace_accepts_snapshot() {
        let r = Repo::new();
        r.write("src/a.rs", "x\n");
        r.commit_all("init");
        // Status run FROM the subfolder still reports the repo root.
        let snap = status(&r.0.join("src")).unwrap();
        assert_eq!(snap.root, r.0);
    }

    #[test]
    fn queued_action_executes_after_status() {
        let r = Repo::new();
        r.write("a.rs", "x\n");
        r.commit_all("init");
        r.write("a.rs", "y\n");
        let mut m = GitState::new(egui::Context::default());
        // Status + stage queued back to back: neither is dropped.
        m.request_status(r.0.clone());
        m.stage(&r.0, vec![PathBuf::from("a.rs")]);
        let mut saw_snap = false;
        let mut saw_action = false;
        for _ in 0..4 {
            match wait_result(&mut m) {
                GitResult::Snapshot(s) => {
                    assert!(s.is_ok());
                    saw_snap = true;
                }
                GitResult::Action { kind, ok, .. } => {
                    assert_eq!(kind, ActionKind::Stage);
                    assert!(ok);
                    saw_action = true;
                }
                _ => {}
            }
            if saw_snap && saw_action {
                break;
            }
        }
        assert!(saw_snap && saw_action);
    }

    #[test]
    fn stash_push_apply_pop_and_drop() {
        let r = Repo::new();
        r.write("a.rs", "v1\n");
        r.commit_all("init");
        r.write("a.rs", "v2\n");
        let (_, ok, m) = run_action(GitAction::StashPush {
            root: r.0.clone(),
            message: Some("wip".into()),
            include_untracked: false,
            keep_index: false,
            paths: vec![],
        });
        assert!(ok, "{m}");
        assert_eq!(fs::read_to_string(r.0.join("a.rs")).unwrap(), "v1\n");
        assert_eq!(r.snap().stashes.len(), 1);
        assert!(r.snap().stashes[0].message.contains("wip"));
        let (_, ok, m) = run_action(GitAction::StashApply {
            root: r.0.clone(),
            index: 0,
            pop: false,
        });
        assert!(ok, "{m}");
        assert_eq!(fs::read_to_string(r.0.join("a.rs")).unwrap(), "v2\n");
        assert_eq!(r.snap().stashes.len(), 1);
        // Restore clean then pop.
        r.write("a.rs", "v1\n");
        let (_, ok, _) = run_action(GitAction::StashApply {
            root: r.0.clone(),
            index: 0,
            pop: true,
        });
        assert!(ok);
        assert!(r.snap().stashes.is_empty());
        // Drop path.
        r.write("a.rs", "v3\n");
        run_action(GitAction::StashPush {
            root: r.0.clone(),
            message: None,
            include_untracked: false,
            keep_index: false,
            paths: vec![],
        });
        let (_, ok, _) = run_action(GitAction::StashDrop {
            root: r.0.clone(),
            index: 0,
        });
        assert!(ok);
        assert!(r.snap().stashes.is_empty());
    }

    #[test]
    fn stash_include_untracked() {
        let r = Repo::new();
        r.write("a.rs", "v1\n");
        r.commit_all("init");
        r.write("new.rs", "n\n");
        let (_, ok, m) = run_action(GitAction::StashPush {
            root: r.0.clone(),
            message: None,
            include_untracked: true,
            keep_index: false,
            paths: vec![],
        });
        assert!(ok, "{m}");
        assert!(!r.0.join("new.rs").exists());
    }

    #[test]
    fn discard_all_and_clean_untracked() {
        let r = Repo::new();
        r.write("a.rs", "v1\n");
        r.write("b.rs", "v1\n");
        r.commit_all("init");
        r.write("a.rs", "v2\n");
        r.write("b.rs", "v2\n");
        r.write("u.rs", "u\n");
        let (_, ok, m) = run_action(GitAction::DiscardAll {
            root: r.0.clone(),
            paths: vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")],
        });
        assert!(ok, "{m}");
        assert_eq!(fs::read_to_string(r.0.join("a.rs")).unwrap(), "v1\n");
        let (_, ok, m) = run_action(GitAction::Clean {
            root: r.0.clone(),
            paths: vec![PathBuf::from("u.rs")],
        });
        assert!(ok, "{m}");
        assert!(!r.0.join("u.rs").exists());
    }

    #[test]
    fn commit_amend_and_undo() {
        let r = Repo::new();
        r.write("a.rs", "v1\n");
        r.commit_all("init");
        r.write("a.rs", "v2\n");
        r.commit_all("second");
        r.write("a.rs", "v3\n");
        run_action(GitAction::Stage {
            root: r.0.clone(),
            paths: vec![PathBuf::from("a.rs")],
        });
        let (_, ok, m) = run_action(GitAction::Commit {
            root: r.0.clone(),
            message: String::new(),
            amend: true,
            all: false,
            signoff: false,
        });
        assert!(ok, "{m}");
        let out = Command::new("git")
            .current_dir(&r.0)
            .args(["log", "-1", "--format=%s"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "second");
        assert_eq!(fs::read_to_string(r.0.join("a.rs")).unwrap(), "v3\n");
        let (_, ok, m) = run_action(GitAction::UndoCommit { root: r.0.clone() });
        assert!(ok, "{m}");
        // Soft reset keeps the change staged.
        let e = r.entry("a.rs");
        assert!(e.is_staged());
    }

    #[test]
    fn branch_create_checkout_delete_merge() {
        let r = Repo::new();
        r.write("a.rs", "v1\n");
        r.commit_all("init");
        let (_, ok, m) = run_action(GitAction::CreateBranch {
            root: r.0.clone(),
            branch: "feature".into(),
            checkout: true,
        });
        assert!(ok, "{m}");
        assert_eq!(r.snap().branch, "feature");
        assert!(r.snap().branches.iter().any(|b| b == "feature"));
        r.write("a.rs", "v2\n");
        r.commit_all("feature work");
        let (_, ok, m) = run_action(GitAction::Checkout {
            root: r.0.clone(),
            branch: "main".into(),
        });
        assert!(ok, "{m}");
        let (_, ok, m) = run_action(GitAction::MergeBranch {
            root: r.0.clone(),
            branch: "feature".into(),
        });
        assert!(ok, "{m}");
        assert_eq!(fs::read_to_string(r.0.join("a.rs")).unwrap(), "v2\n");
        let (_, ok, m) = run_action(GitAction::DeleteBranch {
            root: r.0.clone(),
            branch: "feature".into(),
            force: false,
        });
        assert!(ok, "{m}");
        // Invalid names must not become git options.
        let (_, ok, _) = run_action(GitAction::CreateBranch {
            root: r.0.clone(),
            branch: "--upload-pack=x".into(),
            checkout: false,
        });
        assert!(!ok);
    }

    #[test]
    fn parses_stash_list_format() {
        let v = parse_stash_list("stash@{0}\u{1f}WIP on main: abc msg\nstash@{2}\u{1f}other\n");
        assert_eq!(v.len(), 2);
        assert_eq!(v[1].index, 2);
    }

    #[test]
    fn reset_ignores_old_workspace() {
        let r1 = Repo::new();
        let r2 = Repo::new();
        let mut m = GitState::new(egui::Context::default());
        m.request_status(r1.0.clone());
        m.reset(r2.0.clone()); // queues fresh status for r2
        let mut saw_r2 = false;
        for _ in 0..4 {
            match wait_result(&mut m) {
                GitResult::Snapshot(s) => {
                    let snap = s.unwrap();
                    assert_eq!(snap.root, r2.0);
                    saw_r2 = true;
                    break;
                }
                _ => continue,
            }
        }
        assert!(saw_r2);
    }
}
