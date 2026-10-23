# Releasing Tease

Use this checklist when preparing a release. Tests on one platform do not
establish another platform's runtime status.

## Local candidate checks

Use Rust 1.88 and current stable. Install the desktop prerequisites in README.md.

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
node --test tests/ui.test.cjs tests/components.test.cjs tests/files.test.cjs tests/live.test.cjs tests/preview.test.cjs
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
python3 scripts/check-package.py
cargo publish --locked --dry-run --allow-dirty
```

The package script inspects the archive and builds/runs a fresh consumer using the
unpacked crate, with no path back to the working source tree. `--allow-dirty` is for
local candidate verification only. For final release verification, use a clean
approved commit and repeat `cargo publish --locked --dry-run` without that flag.

## Platform validation

Record the exact commit/toolchain/OS and results in the release review. CI covers
stable and Rust 1.88 on all three platforms. The package-consumer check runs on
stable. Desktop smoke checks additionally require a real graphical session.

- [ ] macOS native build, tests, and runtime behavior.
- [ ] Windows native build/tests and WebView2 runtime behavior.
- [ ] Linux native build/tests and GTK/WebKitGTK runtime behavior.
- [ ] Blank/range/model errors recover without losing inputs.
- [ ] Iris samples, Longley default, and AutoML default match the example guide.
- [ ] Slow preparation/inference leaves the window responsive; duplicates are
      blocked, stale results ignored, and closure during work exits promptly.
- [ ] Both license texts, migration guide, and changelog appear in the crate archive.

A Windows cross-compilation check establishes type checking only, not native
linking or runtime behavior. Keep any missing evidence explicitly pending.

## Executable distribution

Build an example separately on each operating system with
`cargo build --release --locked --example NAME`. A binary is native to its target
OS/architecture; it is not an installer or a universal executable.

- macOS needs system WebKit; distributing an app bundle may require signing and
  notarization under the distributor's own identity. The crate does not do that.
- Windows needs the WebView2 Runtime and the appropriate MSVC runtime.
- Linux needs compatible GTK 3/WebKitGTK 4.1 runtime libraries and a desktop session.

Do not describe a bare executable as self-contained. Test on a clean target
machine before advertising a supported distribution route.

## Release actions

After review and explicit approval: commit and push the branch, wait for CI,
resolve remaining platform checks, and merge the approved change. Then separately
approve publication, create the version tag, publish the crate, and create release
notes from CHANGELOG.md. Verify the actual crates.io version and docs.rs build.
The public browser demo is served from the `gh-pages` branch. When its source in
`docs/live/` changes, copy `docs/.nojekyll`, `docs/index.html`, and `docs/live/`
to that branch and verify the deployed page and its predictions before publishing
a README that links to or embeds it. The iframe appears in rustdoc; crates.io
sanitizes it and retains the image and link.
Never infer publication from a successful dry run. Do not reuse or move an existing
release tag to a different commit.
