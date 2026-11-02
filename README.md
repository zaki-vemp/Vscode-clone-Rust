# OxideCode

A native Rust desktop code editor with a VS Code inspired workbench, built with
[egui/eframe](https://github.com/emilk/egui). No Electron or embedded webview.
This is an independent editor under development, not a feature-complete or
pixel-identical VS Code replacement.

## Build a runnable package

Install a current stable Rust toolchain and Python 3.11 or newer, then run:

```sh
./build.sh
```

On Windows, use `./build.ps1` in PowerShell. Alternatively, run
`python scripts/package.py`. The build uses `Cargo.lock` and creates a native
release binary, a portable archive, and a SHA-256 checksum in `dist/`.
Linux/macOS archives use `.tar.gz`; Windows uses `.zip`; macOS includes an
unsigned `.app` bundle. Build on each OS to produce its native package.

For only the executable (Python is not required):

```sh
cargo build --release --locked
./target/release/oxide-code . src/main.rs
```

On Windows, the executable is `target/release/oxide-code.exe`. `make build`,
`make run`, `make package`, `make deb`, and `make check` are also available.
The native app needs an active desktop session and graphics driver.

Linux build dependencies on Debian/Ubuntu:

```sh
sudo apt install build-essential pkg-config libx11-dev libxi-dev libgl1-mesa-dev \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev \
  libwayland-dev libssl-dev
```

Windows requires the Rust MSVC toolchain and Visual Studio C++ build tools.
macOS requires Xcode command-line tools. Portable Linux archives include a
`.desktop` file that expects `oxide-code` on PATH and its icon in the user's
icon theme. Packages are not code signed or notarized.

### Debian / Ubuntu installer

To package an existing release build as a `.deb`:

```sh
sudo apt install dpkg-dev
python3 scripts/package.py --format deb --no-build
sudo apt install ./dist/oxide-code_0.1.0-1_amd64.deb
```

To compile and package together, run `./build.sh --format deb` or `make deb`.
The filename uses the Cargo version and native Debian architecture (`amd64`,
`arm64`, or `i386`). The installer places the executable on PATH and adds its
application launcher and icon. A `.sha256` file is generated beside it.
For redistribution, set `--maintainer 'Your Name <you@example.com>'`.

Build on the oldest Debian/Ubuntu release you intend to support: library
requirements are detected from the compiled binary. The current local build
requires glibc 2.39 or newer. Git is recommended; FFmpeg and Poppler are optional
preview helpers (`sudo apt install ffmpeg poppler-utils`). Rust tooling is
installed separately. Check a generated package without installing it with
`python3 scripts/check_deb.py dist/oxide-code_0.1.0-1_amd64.deb`.

## Available features

- Activity bar, explorer, tabs, breadcrumbs, outline, status bar, menus, command
  palette, quick-open, and resizable side/bottom panels.
- Native text editing, undo/redo, per-file cursor state, regex find/replace,
  and syntax highlighting for common source/data formats.
  Local diff against the last saved state and Git diffs.
- Real Git status, stage/unstage, commit, discard confirmation, branch display,
  gutter decorations, and optional inline blame through the Git CLI.
- Workspace search with literal/regex/case/whole-word matching, include/exclude
  globs, gitignore support, cancellation, and validated buffer replacements.
- PTY terminal sessions and split terminal panes, backed by the OS shell.
- Cargo check, clippy, build, test, run, rustfmt, compiler diagnostics, and Rust
  project scaffolding. Rust and Git tooling must be installed separately.
- Six official VS Code themes sourced from the upstream repository: Dark
  Modern, Light Modern, Dark+, Light+, and dark/light high contrast. Three
  additional native palette approximations are included. See
  [theme provenance](src/assets/vscode-themes/SOURCE.md).
- Image/SVG previews with fit/zoom, CSV/TSV and Excel/ODS worksheet tables
  with row filters, inline PDF pages, selectable DOCX text, and video posters.
  Full video/audio playback and original Office layouts open in the system app.
- JSON source/tree views, beautify and minify with validation, preserving property
  order and arbitrary-precision numbers. Invalid JSON is never reformatted.
- Local VSIX/unpacked extension loading for JSON color themes and basic snippets,
  including JSONC and `include` inheritance. Extension files are read directly
  from the original package and persist across app restarts.
- Save/discard/cancel prompts when closing dirty tabs, switching workspaces,
  or quitting; closed buffers release their cached contents.

## Extension compatibility

Open Extensions (`Ctrl+Shift+X`), choose **Load VSIX or Extension Folder…**,
enter a local `.vsix` path or folder containing `package.json`, then choose a
loaded theme. **Import Compatible Packages from VS Code** scans standard local
VS Code extension folders. Use `Ctrl+Space` or **Snippets: Insert Snippet…** to
insert a loaded snippet for the current language. Keep packages at their original
paths. Loaded themes also appear in Settings. The app never executes extension
JavaScript. Basic snippets support nested
defaults, initial mirrored values, first choices, and file/selection variables.
Regex transforms and linked placeholder editing/tab navigation remain unsupported.

**Most VS Code extensions do not run yet.** rust-analyzer, Python, GitLens,
Prettier, debuggers, webviews, remote development, and other code extensions
require a compatible extension host and VS Code API bridge. Native Cargo/Git
features do not imply those extensions are installed. Legacy `.tmTheme` and
file-icon themes are not supported; syntax colors map broad token categories,
not every TextMate scope. The Extensions view reports unsupported packages.

See [compatibility details](docs/extension-compatibility.md) and the
[installed-package audit](docs/installed-extension-audit.md). Thirteen local
packages were validated for their native theme/snippet contributions; this
does not mean their code features work.

## Keyboard shortcuts

| Shortcut | Action |
| --- | --- |
| Ctrl/Cmd+P | Quick Open |
| Ctrl/Cmd+Shift+P | Command Palette |
| Ctrl/Cmd+N | Create a new file |
| Ctrl/Cmd+S / W | Save / close tab |
| Ctrl+Tab / Ctrl+Shift+Tab | Next / previous tab |
| Ctrl/Cmd+F / H | Find / replace in the editor |
| F3 / Shift+F3 | Next / previous match |
| Ctrl/Cmd+G | Go to line |
| Shift+Alt+F | Format current JSON/Rust document |
| Ctrl+Space | Insert a loaded snippet |
| Ctrl/Cmd+, | Settings |
| Ctrl/Cmd+B / J | Toggle sidebar / bottom panel |
| Ctrl/Cmd+Shift+E / F / G / X | Explorer / workspace search / Git / extensions |
| Ctrl/Cmd+` | Toggle terminal |
| Ctrl/Cmd+Shift+` | New terminal |
| Ctrl/Cmd+\ | Toggle split preview |
| F5 | Cargo run (not a debugger) |

The split preview is currently read-only. Language-server completion, real
breakpoint debugging and multiple cursors are not implemented yet. The editor
uses a whole-document egui text widget, so very
large files still need a virtualized editing engine. No measured claim is made
that this build uses less memory or CPU than VS Code on comparable workloads.

## File previews and screenshots

See [screenshots from the running app](docs/screenshots.md).

| Files | Viewer |
| --- | --- |
| PNG, JPEG, GIF, WebP, BMP, ICO, TIFF, SVG | Native image preview, fit/zoom (GIF first frame) |
| CSV, TSV | Read-only table, row filter, source editing toggle |
| XLSX, XLS, XLSB, ODS | Read-only worksheet values and sheet selection |
| PDF | Rasterized page preview, previous/next, fit/zoom |
| DOCX | Selectable document text; system app for original layout |
| MP4, WebM, MKV, MOV, AVI, M4V | Poster preview and Play in System Player |
| DOC, ODT, RTF, PPT/PPTX, ODP, common audio | Open in System App |
| JSON/JSONC | Source and expandable tree; strict JSON beautify/minify |

Install **Poppler** (`pdfinfo`, `pdftoppm`) for inline PDF pages and **FFmpeg**
for video posters. On Ubuntu/Debian: `sudo apt install poppler-utils ffmpeg`.
On macOS: `brew install poppler ffmpeg`. On Windows, put these tools on PATH.
Native image/table/DOCX/JSON previews need no external tools. System viewing
requires a registered desktop app for that format. Viewers do not execute Office
macros or recalculate spreadsheet formulas; Excel values use stored results.

Previews load off the UI thread. The native file limit is 64 MiB (PDF/video
are delegated to their tools), workbook ZIP expansion is capped at 128 MiB,
tables show up to 20,000 rows / 128 columns, and texture sizes are bounded.
The UI offers the system app for larger files. DOCX previews show body text,
not complete page layout. JSONC tree viewing supports comments, but the
strict JSON formatter rejects commented input to avoid deleting comments.

## Settings and resource use

Configuration is stored in `~/.config/oxide-code/config.json` on Linux (or
`$XDG_CONFIG_HOME/oxide-code/config.json`), `~/Library/Application Support/oxide-code/config.json`
on macOS, and `%APPDATA%/oxide-code/config.json` on Windows. Set
`OXIDE_CONFIG_DIR` to isolate an installation or test session.

Syntax layout and outline results are cached until their inputs change.
Dirty-state checks are constant-time, inactive inline blame does not invoke
Git, workspace scans are demand-driven, background searches cancel prior
requests, and command/diagnostic polling does not force unlimited redraws.
Git status is refreshed asynchronously every two seconds while a folder is open.

## Validation

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --locked
python3 scripts/preview_smoke.py --external
```

The preview smoke script creates and checks real PNG, CSV, XLSX and DOCX
fixtures, plus PDF/video when their tools are installed. `--check-preview PATH`
and `--check-extension PATH` provide headless validation from the binary.

Git and PTY integration tests run real local processes. CI validates and
packages the app on Linux, macOS, and Windows. See
[third-party notices](THIRD_PARTY_NOTICES.md) for theme, font, and terminal licenses.

## Hardening status

See [the measured hardening report](HARDENING_REPORT.md) and
[local release/rollback runbook](docs/hardening/runbook.md). Output-panel text is
limited to 2 MiB with 16 KiB lines and bounded streaming queues. Captured Git,
diagnostic, PDF and video helper operations have byte budgets and deadlines.
Workbook XML is preflighted before the legacy spreadsheet parser (256
attributes per element, depth 128, 16 MiB per XML part). Some valid very large
files therefore require their system application.

Known dependency matches remain explicitly reviewed in
`security/advisory-reviews.json`, with review due by 2026-12-02; new or expired
matches fail the CI advisory gate. This is not a claim that the application is
free of vulnerabilities or ready for signed production distribution.
