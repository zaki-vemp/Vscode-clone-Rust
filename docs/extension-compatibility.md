# VS Code extension compatibility

This app currently loads **JSON color-theme and basic snippet contributions**
from local VSIX files and unpacked VS Code extension folders. It does **not** run most VS Code
extensions. An extension appearing in the list means its supported theme/snippet
data loaded successfully; it does not imply any extension code has executed.

## Implemented

- Read real `package.json` manifests and `contributes.themes` entries.
- Read ZIP/Deflate VSIX archives directly, without extraction.
- Load JSON/JSONC themes, comments, trailing commas, `include` inheritance,
  dark/light base styles, and RGB/RGBA colors.
- Map workbench colors to existing native UI surfaces; map broad TextMate
  token scopes to the native scanner's keyword/type/string/comment/number/
  function token classes.
- Persist package paths and selected themes; remove loaded packages.
- Bound theme file sizes, include depth, and contribution counts; reject
  include traversal, folder symlinks escaping a package, and include cycles.

Legacy XML `.tmTheme` files, TextMate grammars, semantic token engines,
file/product icon contributions, and JavaScript activation are not implemented.
Language-specific scope rules cannot yet reproduce all upstream syntax colors.
A mixed theme/code extension loads its themes and labels its code unavailable.
Packages with no supported theme/snippet data return an explicit compatibility error.

## Basic snippets

The app reads `contributes.snippets` JSON files, filters them by language, and
provides a native insertion picker (Ctrl+Space). Nested defaults, initial
mirrored values, first choices, and common file/selection variables expand.
Regex transforms are rejected explicitly. Linked editing of repeated
placeholders and Tab navigation are not implemented. Import Compatible
Packages from VS Code discovers installed data contributions without running
their activation code. See [the local audit](installed-extension-audit.md).

## Why most extensions need more work

VS Code's desktop extension host runs Node.js and exposes the `vscode` API.
Extensions commonly depend on document lifecycle events, commands, editors,
configuration, diagnostics, file system providers, language clients, terminals,
webviews, task providers, and debugger adapters. Loading a VSIX or starting a
Node process alone does not implement those contracts.

A viable route is an **optional, separately managed Node.js host** beside the
Rust UI. Keep the default native editor independent of that process. The
remaining implementation requires:

1. A versioned RPC transport, extension activation, module resolution, workspace
   trust and lifecycle, API capability negotiation, and crash/restart handling.
2. Correct document versions, UTF-16 positions, text edits, selections, undo
   transactions, and configuration/events across the process boundary.
3. Commands, language providers, diagnostics and LSP integration, followed by
   file systems, tasks, terminals, source control, and DAP debugging.
4. Webview/custom-editor support, remote extension placement, native modules,
   and platform-specific resources for extensions that use those features.
5. A declared API version and reproducible conformance tests using representative
   extensions, recording activation errors and exercising real user operations.

No percentage or claim of "most extensions" is justified before that suite
passes. An unofficial small `vscode` shim would not meet that requirement.
Some extensions also have dependencies or distribution terms that must be
checked individually. This app does not query the Microsoft Marketplace.

## References

- [Extension host](https://code.visualstudio.com/api/advanced-topics/extension-host)
- [VS Code API](https://code.visualstudio.com/api/references/vscode-api)
- [Extension manifest](https://code.visualstudio.com/api/references/extension-manifest)
- [Color themes](https://code.visualstudio.com/api/extension-guides/color-theme)
- [Code - OSS source](https://github.com/microsoft/vscode)
