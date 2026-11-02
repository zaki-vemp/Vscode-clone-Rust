# Vendored egui_term

- Upstream: https://github.com/Harzu/egui_term
- Commit: `c3d2dc18f0689f4cb3cbe5a1fedb59af9d636739` (post-0.1 master; last
  version compatible with egui 0.32 / alacritty_terminal 0.25.0)
- Vendored because published 0.1 is too old and newer master needs egui 0.35.
- `Cargo.toml` trimmed: workspace/example metadata removed, `egui` pinned
  to 0.32, `publish = false` added. `src/` and `LICENSE` are verbatim
  upstream except the patches below.

## Local patches (minimal, keep upstream otherwise untouched)

1. `src/view.rs` `process_input`: upstream dropped ALL input when either the
   view lacked focus OR the pointer was outside the widget, which killed
   keyboard input whenever the mouse left the pane. Now keyboard events
   require focus only; pointer events require pointer-inside-or-active-drag
   so drag-selection still works beyond the rect.
2. `src/bindings.rs`: `Ctrl+U` (and `Ctrl+Shift+U`) emitted `'\x51'` ('Q');
   corrected to `'\x15'` (NAK — line kill), matching xterm/VS Code.
3. `src/lib.rs`: `pub use types::Size;` — upstream left the `Resize` payload
   type unexportable, so tests/consumers could not resize the PTY without
   going through `TerminalView`.

4. `src/backend/mod.rs`: PTY children receive `TERM=xterm-256color`,
   `COLORTERM=truecolor`, and `TERM_PROGRAM=OxideCode` instead of inheriting
   a build shell with `TERM=dumb`. This enables normal interactive prompts/colors.
