# Tease 0.4.0 local candidate validation

Validated on 2026-09-19 on macOS. Scope: All 0.3.0 features plus validated PNG/JPEG inputs and outputs, CSV import, read-only table previews, row-preserving batch prediction, and native CSV Save dialogs.

At the time of this candidate validation, this was an uncommitted working tree on branch `v0.4.0`, based on
`6b7eaf9`. No commit, push, tag, release, or crates.io publication had been performed at that point.

## Automated evidence

- Rust 1.88.0 and 1.93.1: formatting, strict Clippy on all targets, 37 Rust tests, 9 doctests, and documentation with warnings denied all passed on each toolchain.
- Node: 14 JavaScript tests passed. These execute the shipped scripts in a small DOM harness; they do not replace native WebView testing.
- GitHub Actions workflow passed actionlint. Remote CI has not run for these changes.
- Rust 1.88 Windows MSVC cross-target `cargo check --all-targets` passed. Native Windows linking and runtime were not tested.
- The package check inspected the archive and built/ran a fresh consumer against the unpacked crate outside the source checkout.
- `cargo publish --locked --dry-run --allow-dirty` passed and explicitly aborted upload. The archive contents were compared with current source.
- `git diff --check` passed.

## Native evidence

The images example displayed a generated color gradient and a grayscale result. A native file picker imported a CSV with valid and invalid rows; the result retained row order and the invalid-row error. The native Save dialog wrote a CSV whose cells were checked on disk. The saved mixed-row result was 1→3, 2→5, invalid→error, -3→-5. Browser Blob downloads were replaced with native Save after a WebKit hang was observed.

## Remaining release checks

Native Windows and Linux build/runtime checks and remote CI remain pending. After
approval and commits, repeat packaging from a clean checkout without
`--allow-dirty`. A successful local dry run is not publication. See RELEASING.md.

Raw validation logs were retained locally as `v0.4.0-check-00.log` through `v0.4.0-check-15.log`.
