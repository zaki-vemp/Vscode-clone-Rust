# Hardening work log — 2026-11-02

## System map (before audit changes)

| Component | Boundary / data | Critical journey |
| --- | --- | --- |
| Rust/egui native process | CLI paths, input events, local source text, settings | Open, edit, save, close |
| Workspace/search workers | Filesystem traversal, UTF-8 files, regex queries, unsaved buffers | Find/replace across workspace |
| Image/document previews | User-selected image, SVG, CSV, Office ZIP/XML, PDF/video files | Open/reload/switch/close previews |
| Child processes | Git, Cargo/rustfmt, interactive PTYs, Poppler/FFmpeg | Build/check, inspect Git, use terminal |
| Extension data loader | ZIP packages and local manifests/themes/snippets | Import compatible data contributions |
| Package/CI | Rust lockfile, Python packaging, Linux/macOS/Windows workflows | Validate, build, package, rollback |

There is no application web API, database, session/authentication service,
cloud deployment, or Electron renderer. Security tests use generated local
fixtures and loopback listeners only. Project contents may contain sensitive
source data; no production/user documents are used in probes. Dependency
queries send only public registry crate names and versions.

The main checkout was already extensively dirty. An isolated Git snapshot at
`/tmp/oxide-hardening-audit` preserves the starting files and receives one
commit per logical audit fix; the user's checkout history/index is untouched.
Baseline snapshot: `d32a80888fa15d2205ac946d99b5f624482eb8c4`.

## Baseline

- Existing full suite: 63 passed, 0 failed (local Rust 1.96.0, Linux x86_64).
- Native scanner and output-retention measurements are captured before fixes
  by ignored tests named `hardening_measure_*`, using release compilation.
- Findings remain candidates until a runnable test/probe confirms them.

## Candidates under investigation

- Preview tools may follow embedded network references in a local media file.
- Build output may retain unbounded bytes despite a line-count cap.
- Scanner work may grow quadratically with minified string-heavy input.
- External preview tools have no deadline or captured-output budget.
- Save/config error handling and dependency advisories need triage.

## Final disposition

- SEC-001: workbook attribute-iteration path confirmed and mitigated; the crate
  version remains flagged. Oversized fixtures reject before legacy parsing.
- SEC-002: dangling-link New File repro confirmed; atomic create fixes it.
- MEM-001: retained output growth confirmed; payload plateaus at 2 MiB.
- PERF-001: repeated suffix scanning confirmed; same release benchmark becomes
  linear in the measured input sizes.
- OPS-001: capture contract failures reproduced; deadlines/output budgets and
  Unix process-group cleanup implemented for captured helper paths.
- OPS-002: missing CI security gate confirmed; pinned redacted Gitleaks and
  expiring version-specific OSV reviews added and locally exercised.
- Media-network candidate: not confirmed. The owned-loopback HLS fixture was
  rejected with zero requests before and after; no SSRF finding is claimed.

Full evidence, scope and untested areas are in `HARDENING_REPORT.md`.
