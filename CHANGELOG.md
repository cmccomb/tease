# Changelog

## 0.5.4 — 2026-10-24

- Put a compact interactive response curve near the top of the rustdoc home page.
- Added bar plots to the response example, reused its plot preview beside
  `Output::plot`, and kept the full demo collapsed.
- Kept the crates.io README concise; no API or runtime behavior changes.

## 0.5.3 — 2026-10-24

- Added focused live previews beside the slider, dropdown, and probability APIs.
- Collapsed the full rustdoc demo so it opens when requested.
- No API or runtime behavior changes.

## 0.5.2 — 2026-10-22

- Added a live example that previews number, slider, dropdown, plot, and
  class-score widgets with a small analytical model.
- Added screenshots from the runnable examples to the README and example guide.
- Added a self-contained interactive browser preview on GitHub Pages, linked
  from the README and embedded in rustdoc. Its example model runs locally in
  the browser. The crate homepage now points to the live preview.
- No public API changes.

## 0.5.1 — 2026-10-22

- Shortened the crate README for easier reading on crates.io and added release,
  documentation, CI, and license badges.
- Moved detailed feature behavior and limits into a packaged feature guide.
- Refreshed one yanked transitive lockfile entry.
- No API or runtime behavior changes.

## 0.5.0 — 2026-09-19

- Opt-in debounced live prediction with editable inputs during inference.
- One running request and one replaceable pending snapshot.
- Revision checks discard results for superseded inputs; pause clears queued work.
- IME composition/file-read handling, a delayed live example, and lifecycle tests.


## 0.4.0 — 2026-09-19

- Validated local PNG/JPEG inputs, previews, and image outputs.
- CSV import previews, read-only result tables, and complete CSV export.
- Sequential batch prediction with row-level results and errors.
- Image-processing and CSV batch examples with file/format regression tests.


## 0.3.0 — 2026-09-19

- Local line/scatter/bar plots and validated model-score/probability bars.
- Text areas, checkboxes, multi-selects, units, and help text.
- Validated named presets and session-local scenario comparisons.
- Analytical response example and rendering/control regression tests.


## 0.2.0 — 2026-09-19

### Added

- Dedicated inference worker, model initialization status, bounded single-request
  execution, matching request IDs, and prompt window closure during inference.
- `with_predictor_factory` for constructing models that are not Send on their
  owning worker, plus `prepare()` and reusable headless `Session`s.
- Typed numerical/text values, labeled choices, multiple named outputs, and
  fallible callbacks through one validated prediction path.
- Configuration, field, and output validation with recoverable errors and
  plain-text rendering throughout the interface.
- Real SmartCore regression/classification and AutoML regression examples,
  checked sample predictions, and slow-inference/error-recovery demonstrations.
- Stable/MSRV CI across macOS, Windows, and Linux, Rust and JavaScript lifecycle
  regression tests, and a clean-consumer package verification script.
- MIT and Apache-2.0 license texts, migration guidance, and a release checklist.

### Changed

- Rust 1.88 is the development baseline. Wry is updated to 0.57 and window/event
  handling now uses Tao 0.37. Repository dependency resolution uses Cargo.lock.
- Model examples use published AutoML 0.3.1 and matching SmartCore 0.4.2.
- The interface uses a small local stylesheet and script; bundled Bootstrap,
  jQuery, and Popper are removed from the runtime.
- `run()` returns setup errors; model failures appear in the UI. The window uses
  a readable default size and adapts to narrow layouts.

### Breaking changes

- Direct callback captures must be Send. Non-Send models use a worker-side factory.
- Input number definitions gain optional bounds; dropdowns use labeled `Choice`s.
- Output number definitions include precision, and text results are supported.
- Descriptions are plain text rather than HTML.
- The unpublished generic floating-point and unused advanced-callback APIs are
  replaced. Numeric interface values use f64, consistent with published 0.1.1.
- Earlier unpublished 0.2 drafts used `Teaser::predict`; prepare a `Session` first.

See `docs/migration-0.2.md` for code examples and the panic/closure semantics.
