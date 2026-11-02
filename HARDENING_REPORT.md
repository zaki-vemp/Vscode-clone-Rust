# Hardening Report — OxideCode — 2026-11-02

## Summary

- Six verified findings: **0 Critical, 0 High, 5 Medium, 1 Low**.
- Five fixed; one vulnerable dependency path mitigated with validated input
  limits. Its dependency advisory remains visible. No major dependency upgrade,
  auth/session change, infrastructure change, or operation on real user data
  was performed.
- Biggest wins: minified-string scanning is linear rather than quadratic;
  build-output history now has a byte budget and bounded streaming queues;
  untrusted workbook XML and captured child processes have explicit budgets.
- Main remaining risks: legacy dependency maintenance, other native decoder
  failure modes, non-atomic document saves, unconstrained preview concurrency,
  Windows descendant cleanup, and unsigned desktop distribution. This is a
  measured hardening pass, **not a production-readiness certification**.

## System and environment

A native Rust/egui desktop process reads local source/media/Office files,
imports data from VS Code packages, and starts Git/Cargo/rustfmt/PTY/Poppler/
FFmpeg children. There is no API server, database, login/session service,
Electron renderer, WebRTC service, AWS deployment, or production environment.
The web/database/cloud sections of the supplied brief are not applicable.

[Recon and work log](docs/hardening/findings.md) records the system map.
All tests use generated temporary files and test subprocesses. The only network
probe used an HTTP server bound to an owned `127.0.0.1` ephemeral port.
Advisory queries sent public registry crate names/versions only.

Machine: Linux 7.0.0-38-generic, x86_64, glibc 2.39; Rust 1.96.0.
Scanner measurements use release compilation, five samples per size, median,
and identical generated inputs/method on this machine. The saved before/after
JSON includes all samples, not only the best run.

## Before / after

| Metric | Before | After | Change |
| --- | --- | --- | --- |
| Scanner, 1,000 JSON key/value pairs | 28.722 ms | 0.099 ms | 290.1× faster |
| Scanner, 2,000 JSON key/value pairs | 113.352 ms | 0.209 ms | 542.4× faster |
| Scanner, 4,000 JSON key/value pairs | 448.968 ms | 0.391 ms | 1148.3× faster |
| Output text retained after 600 long lines | 39.50 MiB | 2.00 MiB | Bounded history payload |
| 16,000-attribute workbook fixture | Accepted; 293.26 ms | Rejected; 6.11 ms | Rejected before legacy attribute iteration |
| New File through a dangling symlink | Created the external target | Fails; external target absent | Atomic creation |
| Captured helper: 300 ms sleep with a 50 ms budget | Completed successfully after the sleep | Deadline error; test returns before 250 ms | Deadline enforced |
| Captured helper: 128 KiB output with a 1 KiB budget | Accepted output | Output-budget error | Bytes enforced while reading |

Scanner numbers measure `tokenize_line`, not whole-app startup, typing latency,
or all languages. They do not establish a comparison with VS Code.
The workbook numbers are one bounded local repro per input size; normal files
still pass. The helper measurements are contract tests of generated subprocesses.

### Repeated-work memory check

Warm-up pushes 32 lines, then three rounds of 200 lines of 65,536 bytes each.
The harness clears history at the end. Rust has no forced GC: owned values are
dropped deterministically; the allocator may retain freed pages.

| Point | Before retained text | After retained text | Before process RSS | After process RSS |
| --- | --- | --- | --- | --- |
| warmup | 2.00 MiB | 0.50 MiB | 7.30 MiB | 5.90 MiB |
| N=200 | 14.50 MiB | 2.00 MiB | 19.80 MiB | 8.44 MiB |
| 2N=400 | 27.00 MiB | 2.00 MiB | 32.32 MiB | 8.55 MiB |
| 3N=600 | 39.50 MiB | 2.00 MiB | 44.85 MiB | 8.66 MiB |
| cleared | 0.00 MiB | 0.00 MiB | 37.54 MiB | 8.12 MiB |

The after history payload remains exactly 2 MiB across N/2N/3N. RSS changes by
about 0.22 MiB across those samples, rather than growing with the retained text.
RSS is the **test process**, not the GUI application; the same scanner tests
run earlier in both processes, so allocator state also contributes. Clearing
history yields zero retained text in both builds. This is unbounded history
retention, not proof of unreachable heap objects. String capacities, queue data,
texture/allocator overhead are additional to the text-payload budget.

### Normal preview checks

Generated PNG, CSV, XLSX, DOCX, PDF and video-poster fixtures pass against the
release binary. Five fresh CLI processes after a warm-up per format were
recorded in [before](docs/hardening/preview-baseline.json) and
[after](docs/hardening/preview-after.json), with binary SHA-256 values.
Small-fixture timings have millisecond-level startup noise; no general preview
speed-up is claimed. Workbook preflight intentionally adds validation work.

## Findings

### SEC-001 — Legacy spreadsheet XML permits expensive attribute iteration [Medium]

- **Where:** [`src/preview.rs:517`](src/preview.rs#L517),
  [`src/preview.rs:572`](src/preview.rs#L572); calamine 0.26.1 uses
  quick-xml 0.31.0 `attributes()` in its workbook readers.
- **What:** XML from user-selected Office ZIP files reached the older attribute
  iterator without an attribute-count boundary. A ZIP expansion-size check
  alone did not prevent quadratic CPU work within one XML tag.
- **Evidence:** The new 1,000-attribute workbook regression failed before the
  fix and passes now. The saved [local XML repro](docs/hardening/xml-probe.json)
  shows the baseline accepting 2,000/8,000/16,000 attributes, taking
  8.82/114.36/293.26 ms; all three are rejected after the fix. Source inspection
  confirms the path described by [RUSTSEC-2026-0194](https://rustsec.org/advisories/RUSTSEC-2026-0194.html).
- **Impact:** A crafted local workbook can consume substantial parser CPU;
  repeated preview requests can tie up workers. This is not remote API access
  or proven remote code execution.
- **Fix:** Read a <=64 MiB immutable snapshot, validate XML with current
  quick-xml before passing the **same bytes** to calamine, enforce <=256
  attributes per element, <=128 nesting, <=16 MiB per XML part, aggregate XML
  and ZIP expansion limits, and reject document type declarations. Snapshot
  validation avoids reopening a replaceable path after checking it.
- **Tests:** `sec_001_rejects_excessive_workbook_attributes`,
  `sec_001_accepts_normal_workbook`,
  `sec_001_workbook_xml_depth_and_doctype_are_bounded`.
- **Status:** **Mitigated**, `4a1bed56a4eadfb9e5cdbc323a634f02b69d7bdf`
  plus test coverage `c058cc2b1097c9a4cd96ebc4da99573938aaa981`.
  The underlying 0.31 advisory is not removed; its review expires 2026-12-02.

### SEC-002 — New File follows an existing dangling symlink [Medium]

- **Where:** [`src/project.rs:147`](src/project.rs#L147).
- **What:** `Path::exists()` follows links and returns false for a dangling
  symlink. The subsequent `fs::write` followed that entry and created its target.
- **Evidence:** In a generated workspace, `new.rs` pointed to an absent file
  outside the workspace. The regression initially failed because New File
  succeeded and created that outside target. It now returns an error and leaves
  the target absent.
- **Impact:** A malicious/shared workspace can cause unexpected file creation
  outside the selected directory. Arbitrary overwrite of existing user files
  was **not** demonstrated; the report does not claim it.
- **Fix:** Use `OpenOptions::create_new(true)` at creation, atomically refusing
  any existing entry and removing the check/write race.
- **Test:** `sec_002_new_file_rejects_dangling_symlink` (Unix); ordinary project
  creation and save tests still pass.
- **Status:** **Fixed**, `6101eef3528037337cb6f9083d0ab767700f6ff4`.

### MEM-001 — Output history/capture has no effective byte bound [Medium]

- **Where:** [`src/terminal.rs:12`](src/terminal.rs#L12),
  `Terminal::push`, `run_program`, `poll`, and `stream_lines`.
- **What:** A 5,000-line cap did not cap line bytes; `Command::output()` captured
  complete stdout/stderr before an unbounded channel copied it into the UI.
- **Evidence:** The Unicode regression retained 33,555,456 bytes before failing
  its 2 MiB assertion. The repeat harness reached 41,418,752 retained text bytes
  at 600 lines. Before/after tables above provide the identical-method results.
- **Impact:** Noisy builds or long log lines can grow the editor's memory and
  create large bursts of UI work, even before the 5,000-line cap is reached.
- **Fix:** 2 MiB text history, 16 KiB UTF-8-safe lines with a visible truncation
  marker, a bounded 128-line channel, chunked streaming reads, and a bounded
  number of messages processed per UI poll. Blocking diagnostic captures use
  the separate limits added under OPS-001.
- **Tests:** `mem_001_output_history_is_bounded_by_bytes`,
  `mem_001_streaming_handles_giant_lines_and_unicode`, existing busy/finish tests,
  and the repeated-work benchmark.
- **Status:** **Fixed**, `77e7438e33342fd4ad0bcfde803110611d1eb6f2`
  plus the blocking-capture integration in OPS-001.

### PERF-001 — String tokenization repeatedly collects the remainder of a line [Medium]

- **Where:** [`src/highlight.rs:131`](src/highlight.rs#L131).
- **What:** Every string allocated a vector of all subsequent character
  positions, then used only the prefix up to its closing quote. Many short
  strings on one line caused quadratic repeated work.
- **Evidence:** The unchanged optimized benchmark rises from 28.72 ms at 14 KB
  to 448.97 ms at 56 KB before the fix. After the fix the same sizes take
  0.099/0.209/0.391 ms. Raw samples are saved, and escaped Unicode and unclosed
  string behavior have regression coverage.
- **Impact:** A small edit to a minified JSON/string-heavy line invalidates the
  cached layout and can stall the UI's scanner path for hundreds of milliseconds.
- **Fix:** Iterate character positions lazily and stop at the closing quote,
  consuming the escaped character directly. No token categories were removed.
- **Tests:** `perf_001_strings_preserve_escaped_unicode_and_unclosed_tail`,
  existing scanner tests, `hardening_measure_minified_strings`.
- **Status:** **Fixed**, `d65d48d8368f9ccf6d582adbf6c80207d7a6b379`.

### OPS-001 — Captured helper processes lack deadlines/output budgets [Medium]

- **Where:** [`src/process.rs:39`](src/process.rs#L39), integrated in
  `src/preview.rs`, `src/git.rs`, and `Terminal::run_blocking`.
- **What:** Synchronous captured output waited for helpers and collected their
  complete output without an application deadline or byte limit.
- **Evidence:** Tests using the same original `Command::output` behavior
  accepted a 300 ms child despite a requested 50 ms budget and accepted 128 KiB
  despite a 1 KiB output budget. Both failed before enforcement and pass now.
- **Impact:** Stalled Git/diagnostic/media tools can retain workers indefinitely;
  excessive output can exhaust the capture process's memory.
- **Fix:** Shared bounded capture with a backpressured chunk queue, stdout/stderr
  limits, explicit error propagation, and timeout termination/reaping. Unix
  helpers use a separate process group, killed on error before reaping. Limits:
  PDF info 10 s, PDF render 20 s, video poster 15 s, Git 120 s, blocking
  diagnostics 180 s. Long-running interactive/output commands retain their
  separate execution model rather than acquiring a blanket short deadline.
- **Tests:** `ops_001_hanging_capture_has_deadline`,
  `ops_001_capture_rejects_excess_output`, Git integration tests and real
  Poppler/FFmpeg preview smoke checks.
- **Status:** **Fixed for these capture paths**,
  `d69c08751caffd78f7d8d1694650d41023d7f63c`.
  Windows descendant/job-object cleanup is not implemented or verified; see limits.

### OPS-002 — CI has no dependency/secret security review gate [Low]

- **Where:** [`.github/workflows/build.yml:34`](.github/workflows/build.yml#L34),
  `scripts/dependency_audit.py`, `security/advisory-reviews.json`.
- **What:** The existing workflow gated format/lint/tests/build only, so new
  advisories or committed secrets would not fail an explicit security check.
- **Evidence:** Initial workflow inspection confirmed the missing steps.
  OSV found 14 advisory identifiers across eight of 466 public registry crates.
  Gate tests prove that an unknown identifier, changed version, or expired
  review fails, while a matching current review passes.
- **Impact:** Security/maintenance changes in a large native dependency graph
  can go unnoticed between manual audits.
- **Fix:** Pinned Gitleaks 8.30.1 download verified against a fixed SHA-256,
  redacted source/history checks, full Git history checkout, a live OSV gate
  with network timeouts, and explicit version-specific reviews expiring
  2026-12-02. Unknown/expired matches fail; lookup failures do not report success.
- **Status:** **Implemented and locally tested**,
  `fd71b17d1a877557061c30f4ce0e313bd4e98f73`.
  The GitHub-hosted workflow itself has not run in this session.

## Automated scans and dependency triage

- **OSV:** 466 registry packages queried before and after; eight package matches
  remain. Saved [baseline](docs/hardening/dependency-baseline.json),
  [after](docs/hardening/dependency-after.json), and [advisories](docs/hardening/advisories.json).
- **Gitleaks 8.30.1:** official release checksum verified; 100% redaction;
  original Git history and isolated working-source scan returned exit 0, no
  findings. This is not a guarantee that no secrets exist anywhere on the host.
  Binary/session metadata/build outputs were not part of the source snapshot;
  the local directory scan capped files at 2 MiB.
- **Targeted regex/history scan:** 65 working text files and 46 distinct history
  text blobs across three original commits, no matches for selected high-confidence
  patterns. This was supplemental, not a replacement for Gitleaks.
- **Clippy/format:** clean, warnings denied.
- Semgrep, Trivy, cargo-audit, heaptrack and Valgrind were not installed.
  RustSec's OSV export was used for dependency checks. Node/Python-service/
  Electron/SQL/container/cloud-specific scanners do not match this app's stack.
  Python packaging/audit helpers were reviewed and byte-compiled; no Python
  service or separate Python requirements file exists.

### Remaining advisory matches

| Package(s) | Classification / reachability | Disposition |
| --- | --- | --- |
| quick-xml 0.31.0 | Quadratic attributes reachable through calamine; namespace-resolver advisory not used by its plain Reader | SEC-001 boundary mitigates attribute work; version remains flagged |
| zip 2.4.2 | Matches concern extraction, symlink/permission restoration, dual-reader differential or AES authentication | App/calamine use random-access reads, no extraction; AES feature not enabled. Reviewed as not reachable through those APIs |
| paste 1.0.15 | Unmaintained macro crate | Maintenance risk; upstream replacement |
| ttf-parser 0.25.1, rustybuzz 0.20.1 | Unmaintained font/text-shaping dependencies | Maintenance risk; upstream GUI/SVG migration |
| zbus_names 4.3.5, zvariant / zvariant_derive 5.15.1 | Maintenance/merge into newer zbus | Maintenance risk; upstream accessibility/D-Bus migration |

Review records are narrow, version-specific and expiring, not a blanket audit
ignore. No rule was disabled for a speed result, and no TLS/input validation
was weakened. Major/breaking replacements were not silently applied.

## Verification and reproducibility

```sh
cargo test --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
python3 -m unittest scripts.test_dependency_audit
python3 scripts/dependency_audit.py --output /tmp/dependency-audit.json
python3 scripts/hardening_measure.py --output /tmp/hardening-after.json
cargo build --release --locked
python3 scripts/preview_smoke.py --binary target/release/oxide-code --external
python3 scripts/hardening_media_probe.py --expect-blocked
```

Final local Rust suite: **72 passed, 3 intentionally ignored** (two manual
measurement harnesses and a subprocess fixture). The two measurement harnesses
were separately executed successfully. Python policy-gate tests pass.
The release builds, passes actual file preview checks, and starts/renders in
an isolated desktop test session. No production data was used.

The loopback HLS-in-MP4 probe made **zero HTTP requests** before and after: this
FFmpeg build rejected the misleading format/extension. It is **not** reported
as a confirmed SSRF vulnerability. Other protocols/FFmpeg builds were not
exhaustively tested.

## Commits and rollback

The original checkout contained extensive uncommitted editor work. Its Git
index/history was left untouched. One logical fix per commit was recorded in
an isolated repo at `/tmp/oxide-hardening-audit`, starting from a copy of the
actual working files, not the older main-branch HEAD. See
[the audit history bundle](docs/hardening/commits.bundle). It contains the
snapshot and audit commits for review, without session metadata or built binaries.

Review independently with `git clone docs/hardening/commits.bundle /tmp/oxide-audit-review`.
Do not blindly reset the user's dirty checkout or cherry-pick these changes
onto an unrelated baseline. The bundle allows inspecting/reversing each fix;
back up current work first. See [the local runbook](docs/hardening/runbook.md).

## Needs your decision

1. **Dependency migration:** plan a separately tested calamine/ZIP migration and
   GUI/font/accessibility dependency updates. Several 0.x/API changes and the
   ZIP 2→9 transition are breaking changes; they require the brief's approval
   before implementation. Current reviews must be revisited by 2026-12-02.
2. **Signed distribution/update design:** publisher identities, signing/notarization
   credentials and an update channel are required. No keys, cloud accounts,
   IAM or release service was created or modified.
3. **Document persistence policy:** atomic-save and disk-full fault-injection,
   symlink/hard-link behavior, crash recovery and workspace-backup policy need
   a dedicated data-safety pass before production use.

## Not covered / limitations

- No Windows/macOS execution, hosted-CI result, signing/notarization, update
  rollout, hours-long soak, full fuzz corpus or production-scale workspace.
- No heap snapshot/allocator profiler was available. Repeat-harness RSS plus
  exact retained text are the memory evidence; GUI/GPU memory and PTY session
  histories were not given a general plateau certification.
- Preview requests still spawn workers independently; there is no global
  concurrency cap or parser-process memory sandbox. ZIP budgets/XML checks do
  not prove all XLS/XLSB/ODS cell-range allocation paths safe.
- Async long-running output commands, interactive PTYs and the separate
  stdin-based Format Document rustfmt path do not all use the new capture helper.
  Windows kills the direct captured child; descendant cleanup can still need
  job objects. Those lifecycle paths need further tests.
- Existing document/config saves still use non-atomic writes; no disk-full or
  power-loss recovery guarantee is made. Real user files were not fault-tested.
- Broad VS Code extension compatibility and unfinished editor features are
  outside this audit; importing theme/snippet data does not execute extensions.

## Recommended next steps

1. Review the remaining dependency migrations before the advisory-review expiry.
2. Add parser isolation/concurrency and large-sheet allocation tests, Windows
   job-object cleanup, and targeted PTY/formatting lifecycle tests.
3. Complete atomic persistence/crash recovery, then signed staged distribution
   and a longer multi-format/workspace soak on all supported platforms.
