# Changelog

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
