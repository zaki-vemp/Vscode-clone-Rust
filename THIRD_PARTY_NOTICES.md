# Third-party notices

OxideCode is an independent project and is not affiliated with Microsoft.

- The built-in VS Code color-theme JSON files are copied unmodified from
  microsoft/vscode, revision `dbe2dc08afb1ba895994cb839d7f97cf5fcbdc2b`.
  Copyright Microsoft Corporation. MIT license:
  `src/assets/vscode-themes/LICENSE.txt`. Source and provenance:
  `src/assets/vscode-themes/SOURCE.md`.
- Codicons font: Microsoft/vscode-codicons. The included font is covered by
  `src/assets/codicon-LICENSE.txt`.
- The vendored egui_term terminal widget is MIT licensed. See
  `vendor/egui_term/LICENSE` and `vendor/egui_term/UPSTREAM.md`.
- Rust dependency names, versions, and licenses can be inspected with
  `cargo metadata --locked --format-version=1`.

The VS Code name refers to the upstream editor and extension formats. This
project uses its own name and icon. External extension packages retain their
own licenses; the app reads user-selected packages in place.
