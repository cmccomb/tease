# Tease 0.3.0 local candidate validation

Validated on 2026-09-19 on macOS. Scope: Line/scatter/bar plots, class-score and probability bars, presets, pinned scenario comparisons, text/checkbox/multiselect controls, units, and help text.

At the time of this candidate validation, this was an uncommitted working tree on branch `v0.3.0`, based on
`6b7eaf9`. No commit, push, tag, release, or crates.io publication had been performed at that point.

## Automated evidence

- Rust 1.88.0 and 1.93.1: formatting, strict Clippy on all targets, 31 Rust tests, 9 doctests, and documentation with warnings denied all passed on each toolchain.
- Node: 8 JavaScript tests passed. These execute the shipped scripts in a small DOM harness; they do not replace native WebView testing.
- GitHub Actions workflow passed actionlint. Remote CI has not run for these changes.
- Rust 1.88 Windows MSVC cross-target `cargo check --all-targets` passed. Native Windows linking and runtime were not tested.
- The package check inspected the archive and built/ran a fresh consumer against the unpacked crate outside the source checkout.
- `cargo publish --locked --dry-run --allow-dirty` passed and explicitly aborted upload. The archive contents were compared with current source.
- `git diff --check` passed.

## Native evidence

The exploration example rendered plots and probability bars in a native macOS WebKit window. Gentle and Steep presets produced 0.6225 and 0.9526. A pinned comparison preserved its inputs and result and displayed the +0.3301 difference. Chart labels and the default window size were adjusted after visual inspection.

## Remaining release checks

Native Windows and Linux build/runtime checks and remote CI remain pending. After
approval and commits, repeat packaging from a clean checkout without
`--allow-dirty`. A successful local dry run is not publication. See RELEASING.md.

Raw validation logs were retained locally as `v0.3.0-check-00.log` through `v0.3.0-check-15.log`.
