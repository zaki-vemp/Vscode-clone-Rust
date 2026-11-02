# Local hardening and release runbook

This is a desktop application. There are no service endpoints, database
migrations, production credentials, or infrastructure changes in this pass.

## Validate a build

1. Preserve the current workspace and config. Use generated fixture files;
   point `OXIDE_CONFIG_DIR` to an empty temporary directory for app smoke tests.
2. Run format, Clippy, Rust tests, Python advisory-policy tests, the live OSV
   check, and the preview smoke script listed in `HARDENING_REPORT.md`.
3. Build with `cargo build --release --locked`, then package with
   `python3 scripts/package.py --no-build`. Verify the archive SHA-256 file and
   run the packaged binary's `--version` and `--check-preview` commands.
4. No signing/notarization or auto-update channel is configured. Publish only
   after a separate signed-release decision and platform validation.

## Roll back safely

Keep the previous validated binary/archive and its checksum. Stop the editor,
preserve unsaved work, and launch that prior binary with a copied config.
Do not delete workspace/config files or run `git reset --hard` on the main
checkout. Audit commits are isolated in `commits.bundle`; inspect their exact
patches against the matching snapshot before selectively reversing a fix.

## Likely incidents

- **Preview fails or times out:** use the system viewer, preserve the input for
  a local repro only with its owner's permission, check Poppler/FFmpeg versions,
  and run `--check-preview` in an isolated config. Do not remove byte/XML or
  deadline checks to make an untrusted file pass.
- **Output is truncated:** the panel deliberately limits a line to 16 KiB and
  text history to 2 MiB. Send a trusted command's full output to a user-chosen
  log file outside this panel if needed; avoid exposing credentials in logs.
- **Advisory gate fails:** inspect the new advisory and exact locked version,
  trace reachability, add a focused test, and prefer a compatible fix. A review
  must have a reason and deadline. Do not extend reviews silently to turn CI
  green. Current reviews expire on 2026-12-02.
- **Secret scan fails:** report the location/type only, revoke/rotate the
  credential with its owner, then remove it from source and review history.
  Removing the string alone is insufficient; do not print it in issue logs.

## Reproduce measurements

`python3 scripts/hardening_measure.py --output /tmp/hardening.json` compiles and
runs the two ignored release harnesses. Keep machine, profile, inputs, run
counts and order unchanged for comparisons. The RSS samples are test-process
memory; retained text excludes allocator/texture overhead. Rust drops replace
GC in the repeat check. Small CLI preview timings include startup noise.
