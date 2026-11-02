//! Workspace model: real files on a real filesystem.
//!
//! The workspace lazily caches directory listings for the explorer tree,
//! owns the open [`TextBuffer`]s, and scaffolds new Rust projects (binary
//! or library crates generated entirely by this app, no `cargo new` needed).

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::buffer::TextBuffer;

/// Names excluded from the explorer tree (VS Code `files.exclude` defaults).
const EXCLUDED_NAMES: &[&str] = &[".git", ".svn", ".hg", "CVS", ".DS_Store", "Thumbs.db"];
/// One line of `git blame` output.
#[derive(Clone, Debug)]
pub struct BlameLine {
    pub author: String,
    pub time: i64,
    pub summary: String,
}

/// One node of the folder tree shown in the explorer.
#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

/// An open folder with its lazily-read directory listings and open buffers.
pub struct Workspace {
    pub root: PathBuf,
    dir_cache: HashMap<PathBuf, Vec<FileEntry>>,
    buffers: HashMap<PathBuf, TextBuffer>,
}

impl Workspace {
    pub fn open_folder(path: &Path) -> io::Result<Self> {
        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Choose an existing folder",
            ));
        }
        let root = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        Ok(Self {
            root,
            dir_cache: HashMap::new(),
            buffers: HashMap::new(),
        })
    }

    pub fn refresh(&mut self) {
        self.dir_cache.clear();
    }

    /// Cached listing of one directory (sorted: folders first, then
    /// case-insensitive name), minus `files.exclude` names.
    pub fn children(&mut self, dir: &Path) -> Vec<FileEntry> {
        let dir = self.key(dir);
        self.dir_cache
            .entry(dir.clone())
            .or_insert_with(|| read_dir(&dir))
            .clone()
    }

    fn key(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        }
    }

    /// Get (loading from disk on first use) the buffer for a file.
    pub fn buffer(&mut self, path: &Path) -> io::Result<&mut TextBuffer> {
        let key = self.key(path);
        if !self.buffers.contains_key(&key) {
            let text = fs::read_to_string(&key)
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", key.display())))?;
            self.buffers
                .insert(key.clone(), TextBuffer::from_text(&text));
        }
        Ok(self.buffers.get_mut(&key).expect("just inserted"))
    }

    pub fn get(&self, path: &Path) -> Option<&TextBuffer> {
        self.buffers.get(&self.key(path))
    }

    pub fn get_mut(&mut self, path: &Path) -> Option<&mut TextBuffer> {
        let key = self.key(path);
        self.buffers.get_mut(&key)
    }

    pub fn dirty_paths(&self) -> Vec<PathBuf> {
        self.buffers
            .iter()
            .filter(|(_, b)| b.is_dirty())
            .map(|(p, _)| p.clone())
            .collect()
    }

    pub fn close_buffer(&mut self, path: &Path) {
        self.buffers.remove(&self.key(path));
    }

    /// Write a buffer back to disk and mark it saved.
    pub fn save(&mut self, path: &Path) -> io::Result<()> {
        let key = self.key(path);
        let text = self
            .buffers
            .get(&key)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "No editable buffer for this file")
            })?
            .text();
        if let Some(parent) = key.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&key, text)?;
        if let Some(buf) = self.buffers.get_mut(&key) {
            buf.mark_saved();
        }
        Ok(())
    }

    /// Save every open dirty buffer.
    pub fn save_all(&mut self) -> io::Result<usize> {
        let dirty: Vec<PathBuf> = self
            .buffers
            .iter()
            .filter(|(_, b)| b.is_dirty())
            .map(|(p, _)| p.clone())
            .collect();
        let mut n = 0;
        for p in dirty {
            self.save(&p)?;
            n += 1;
        }
        Ok(n)
    }

    /// Create an empty file (VS Code "New File" behavior).
    pub fn new_file(&mut self, dir: &Path, name: &str) -> io::Result<PathBuf> {
        let dir = self.key(dir);
        validate_name(name)?;
        let path = dir.join(name);
        if path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "a file or folder with that name already exists",
            ));
        }
        fs::create_dir_all(&dir)?;
        // Atomic create refuses every existing entry, including dangling
        // symlinks, and closes the exists()/write() race.
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        self.buffers.insert(path.clone(), TextBuffer::from_text(""));
        self.refresh();
        Ok(path)
    }

    /// Create an empty folder.
    pub fn new_folder(&mut self, dir: &Path, name: &str) -> io::Result<PathBuf> {
        let dir = self.key(dir);
        validate_name(name)?;
        let path = dir.join(name);
        if path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "a file or folder with that name already exists",
            ));
        }
        fs::create_dir_all(&path)?;
        self.refresh();
        Ok(path)
    }

    /// Rename a file or folder, re-keying any open buffers (including
    /// buffers of files inside a renamed folder).
    pub fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        let from = self.key(from);
        let to = self.key(to);
        if let Some(name) = to.file_name().and_then(|n| n.to_str()) {
            validate_name(name)?;
        }
        if !from.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "no such file"));
        }
        if to.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "a file or folder with that name already exists",
            ));
        }
        fs::rename(&from, &to)?;
        let moved: Vec<(PathBuf, PathBuf)> = self
            .buffers
            .keys()
            .filter(|k| k.starts_with(&from))
            .map(|k| {
                let suffix = k.strip_prefix(&from).unwrap_or(k.as_path());
                (k.clone(), to.join(suffix))
            })
            .collect();
        for (old, new) in moved {
            if let Some(buf) = self.buffers.remove(&old) {
                self.buffers.insert(new, buf);
            }
        }
        self.refresh();
        Ok(())
    }

    pub fn delete(&mut self, path: &Path) -> io::Result<()> {
        let key = self.key(path);
        if key.is_dir() {
            fs::remove_dir_all(&key)?;
            let doomed: Vec<PathBuf> = self
                .buffers
                .keys()
                .filter(|k| k.starts_with(&key))
                .cloned()
                .collect();
            for k in doomed {
                self.buffers.remove(&k);
            }
        } else {
            fs::remove_file(&key)?;
            self.buffers.remove(&key);
        }
        self.refresh();
        Ok(())
    }

    pub fn git_blame(&self, path: &Path) -> Vec<BlameLine> {
        let rel = self.rel_path(path);
        let out = std::process::Command::new("git")
            .args(["blame", "--porcelain", "--", &rel])
            .current_dir(&self.root)
            .output();
        let Ok(out) = out else { return Vec::new() };
        if !out.status.success() {
            return Vec::new();
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let mut lines: Vec<BlameLine> = Vec::new();
        let (mut author, mut time, mut summary) = (String::new(), 0i64, String::new());
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("author ") {
                author = rest.to_string();
            } else if let Some(rest) = line.strip_prefix("author-time ") {
                time = rest.trim().parse().unwrap_or(0);
            } else if let Some(rest) = line.strip_prefix("summary ") {
                summary = rest.to_string();
            } else if line.starts_with('\t') {
                lines.push(BlameLine {
                    author: author.clone(),
                    time,
                    summary: summary.clone(),
                });
            }
        }
        lines
    }

    /// Display path relative to the workspace root when possible.
    pub fn rel_path(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string_lossy().into_owned())
    }
}

fn validate_name(name: &str) -> io::Result<()> {
    if name.is_empty() || name.contains(['/', '\\']) {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "a file name cannot be empty or contain '/' or '\\'",
        ))
    } else {
        Ok(())
    }
}

fn read_dir(dir: &Path) -> Vec<FileEntry> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<FileEntry> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            !EXCLUDED_NAMES.contains(&name.as_ref())
        })
        .map(|path| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            FileEntry {
                is_dir: path.is_dir(),
                path,
                name,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase()),
        )
    });
    out
}

const MAIN_TEMPLATE: &str = r#"//! Application entry point.

fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
"#;

const LIB_TEMPLATE: &str = r#"//! Library crate root.

/// Adds one to the input.
pub fn add_one(x: i32) -> i32 {
    x + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_one() {
        assert_eq!(add_one(1), 2);
    }
}
"#;

impl Workspace {
    /// Scaffold a new Rust project (binary or library) under `parent/name`.
    /// Everything is generated by this app in pure Rust — the result is a
    /// plain `cargo` project you can build immediately.
    pub fn new_rust_project(parent: &Path, name: &str, lib: bool) -> io::Result<PathBuf> {
        if name.is_empty() || name.contains(['/', '\\', ' ', ':']) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid project name",
            ));
        }
        let root = parent.join(name);
        if root.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "project already exists",
            ));
        }
        fs::create_dir_all(root.join("src"))?;
        let kind = if lib { "lib" } else { "bin" };
        let cargo_toml = format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
             # See https://doc.rust-lang.org/cargo/ for manifest documentation.\n\
             [lib]\npath = \"src/lib.rs\"\n"
        );
        let cargo_toml = if lib {
            cargo_toml
        } else {
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
                 [[bin]]\nname = \"{name}\"\npath = \"src/main.rs\"\n"
            )
        };
        fs::write(root.join("Cargo.toml"), cargo_toml)?;
        if lib {
            fs::write(root.join("src").join("lib.rs"), LIB_TEMPLATE)?;
        } else {
            fs::write(root.join("src").join("main.rs"), MAIN_TEMPLATE)?;
        }
        fs::write(
            root.join("README.md"),
            format!("# {name}\n\nA Rust {kind} crate created with OxideCode.\n"),
        )?;
        fs::write(root.join(".gitignore"), "/target/\n")?;
        Ok(root)
    }
}

impl Workspace {
    /// Snapshot of every open buffer (search replaces disk content).
    pub fn buffer_snapshots(&self) -> Vec<(PathBuf, String)> {
        self.buffers
            .iter()
            .map(|(p, b)| (p.clone(), b.text()))
            .collect()
    }

    /// Replace matches across the frozen snapshots — truly all-or-none:
    /// every target is prevalidated and every output computed BEFORE a
    /// single buffer changes. Buffers only; disk untouched.
    pub fn apply_replacements(
        &mut self,
        results: &crate::search::SearchResults,
        matcher: &crate::search::Matcher,
        replacement: &str,
        regex_mode: bool,
    ) -> Result<(usize, Vec<PathBuf>), String> {
        // Phase 1: validate every file against its frozen text.
        for f in &results.files {
            let abs = self.root.join(&f.path);
            let cur = match self.get(&abs) {
                Some(b) => {
                    let t = b.text();
                    if !b.is_dirty() {
                        // Clean buffer: disk may have changed under us.
                        let disk = std::fs::read_to_string(&abs)
                            .map_err(|e| format!("{}: {e}", abs.display()))?;
                        if disk != t {
                            return Err(format!("{} changed on disk", f.path.display()));
                        }
                    }
                    t
                }
                None => {
                    std::fs::read_to_string(&abs).map_err(|e| format!("{}: {e}", abs.display()))?
                }
            };
            if cur != f.text {
                return Err(format!("{} changed since search", f.path.display()));
            }
        }
        // Phase 2: compute ALL new texts (sorted, non-overlapping ranges).
        let mut outs: Vec<(PathBuf, String)> = Vec::new();
        let mut total = 0usize;
        for f in &results.files {
            let mut ranges: Vec<crate::search::SearchMatch> = f.matches.clone();
            ranges.sort_by_key(|m| m.range.start);
            for w in ranges.windows(2) {
                if w[1].range.start < w[0].range.end {
                    return Err("overlapping match ranges".into());
                }
            }
            let mut text = String::with_capacity(f.text.len());
            let mut prev = 0usize;
            for m in &ranges {
                let rep = matcher.replacement_text(&f.text, &m.range, replacement, regex_mode)?;
                text.push_str(&f.text[prev..m.range.start]);
                text.push_str(&rep);
                prev = m.range.end;
                total += 1;
            }
            text.push_str(&f.text[prev..]);
            outs.push((self.root.join(&f.path), text));
        }
        // Phase 3: preload every buffer (any failure still aborts all),
        // THEN mutate.
        for (abs, _) in &outs {
            self.buffer(abs)
                .map_err(|e| format!("{}: {e}", abs.display()))?;
        }
        let mut paths = Vec::new();
        for (abs, text) in outs {
            self.get_mut(&abs).unwrap().set_text(&text);
            paths.push(abs);
        }
        Ok((total, paths))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_root(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("oxide-test-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn open_save_round_trip() {
        let root = tmp_root("ws");
        fs::write(root.join("a.rs"), "fn a() {}\n").unwrap();
        let mut ws = Workspace::open_folder(&root).unwrap();
        assert_eq!(ws.children(&root).len(), 1);
        let buf = ws.buffer(&PathBuf::from("a.rs")).unwrap();
        assert!(!buf.is_dirty());
        buf.set_text("fn a() {}\nfn b() {}\n");
        ws.save(&PathBuf::from("a.rs")).unwrap();
        assert_eq!(
            fs::read_to_string(root.join("a.rs")).unwrap(),
            "fn a() {}\nfn b() {}\n"
        );
        assert!(!ws.get(&PathBuf::from("a.rs")).unwrap().is_dirty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn new_rust_projects() {
        let root = tmp_root("newproj");
        let bin = Workspace::new_rust_project(&root, "demo_bin", false).unwrap();
        assert!(bin.join("src").join("main.rs").exists());
        assert!(bin.join("Cargo.toml").exists());
        let lib = Workspace::new_rust_project(&root, "demo_lib", true).unwrap();
        assert!(lib.join("src").join("lib.rs").exists());
        assert!(Workspace::new_rust_project(&root, "demo_bin", false).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn new_folder_creates_and_caches() {
        let root = tmp_root("newfolder");
        let mut ws = Workspace::open_folder(&root).unwrap();
        let dir = ws.new_folder(Path::new("."), "sub").unwrap();
        assert!(dir.is_dir());
        assert!(ws
            .children(&root)
            .iter()
            .any(|e| e.name == "sub" && e.is_dir));
        assert!(ws.new_folder(Path::new("."), "sub").is_err());
        assert!(ws.new_folder(Path::new("."), "a/b").is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rename_rekeys_open_buffer() {
        let root = tmp_root("rename");
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("sub/a.rs"), "fn a() {}\n").unwrap();
        let mut ws = Workspace::open_folder(&root).unwrap();
        ws.buffer(&root.join("sub/a.rs")).unwrap();
        ws.rename(&root.join("sub/a.rs"), &root.join("sub/b.rs"))
            .unwrap();
        assert!(ws.get(&root.join("sub/b.rs")).is_some());
        assert!(ws.get(&root.join("sub/a.rs")).is_none());
        // Renaming a folder re-keys buffers inside it.
        ws.rename(&root.join("sub"), &root.join("renamed")).unwrap();
        assert!(ws.get(&root.join("renamed/b.rs")).is_some());
        let _ = fs::remove_dir_all(&root);
    }
    #[cfg(unix)]
    #[test]
    fn sec_002_new_file_rejects_dangling_symlink() {
        let root = tmp_root("dangling-create");
        let project = root.join("project");
        fs::create_dir(&project).unwrap();
        let outside = root.join("outside.rs");
        std::os::unix::fs::symlink(&outside, project.join("new.rs")).unwrap();
        let mut workspace = Workspace::open_folder(&project).unwrap();
        let result = workspace.new_file(&project, "new.rs");
        assert!(
            result.is_err(),
            "New File must reject an existing directory entry, including dangling links"
        );
        assert!(
            !outside.exists(),
            "New File must not follow an existing symlink"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
