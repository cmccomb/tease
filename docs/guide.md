# Tease feature guide

Detailed behavior and limits for Tease 0.5. Start with the [README](../README.md)
for installation and a small runnable demo. See the [example guide](examples.md)
for complete models and checked predictions.

## Typed results and prediction errors

Component definitions (`Input`, `Output`) describe the interface. `Value` carries
actual model inputs and outputs. `with_predictor` accepts a fallible callback;
`with_function` adapts a numerical closure to the same validation and execution path.

```no_run
use tease::{Input, Output, Teaser, Value};

fn main() -> Result<(), tease::Error> {
    Teaser::default()
        .with_inputs(vec![Input::number("Score", 0.7)])
        .with_outputs(vec![Output::text("Class"), Output::number("Score", 2)])
        .with_predictor(|values| {
            let score = values[0].as_number()?;
            if !(0.0..=1.0).contains(&score) {
                return Err("Score must be between zero and one.".into());
            }
            let class = if score >= 0.5 { "Positive" } else { "Negative" };
            Ok(vec![Value::Text(class.into()), Value::Number(score)])
        })
        .run()
}
```

Inputs support bounded numbers, sliders, dropdowns, text areas, checkboxes, and
multi-select choices. Add units and help with `with_unit` and `with_help`. Outputs
support numbers, text, line/scatter/bar plots, and ranked class scores.
All labels, descriptions, errors, and text results are rendered as text; HTML is
not supported. Numerical values must be finite. Callbacks should return errors
rather than panic. With unwinding enabled, the desktop worker reports a callback
panic and retires the model; headless sessions propagate panics to the caller.

Use `Teaser::validate()` to check configuration. For headless predictions, call
`let session = demo.prepare()?` once and reuse `session.predict(&values)`.

## Visual exploration

Use `Output::plot` with `Value::Plot(Plot { kind, x_label, y_label, series })`.
`PlotKind` supports line, scatter, and bar charts. Each `Series` has a name and
`Vec<[f64; 2]>` points. Charts include hover labels, keyboard-focusable marks, and
an expandable data table. Up to 16 series and 10,000 total finite points are allowed;
coordinate magnitudes are limited to 1e100.

Use `Output::scores` for nonnegative model scores, or `Output::probabilities` for
values in `[0,1]` summing to one (tolerance 1e-6). Return `Value::Scores` containing
unique `ClassScore { label, score }` entries. The model supplies all scores; the UI
neither invents confidence nor calibrates it.

Add `with_presets(vec![Preset::new(name, values)])` to load named scenarios. In
manual mode, loading a preset waits for Predict; in live mode it schedules a
debounced prediction. `with_comparison(true)` lets users pin up to three successful
results alongside their submitted inputs, then compare them with the current result.
Numerical differences use the displayed precision. Snapshots last for this window
session. `cargo run --example exploration` demonstrates these features together.

## Images and batch data

`Input::image` accepts local PNG/JPEG files; `Output::image` displays
`Value::Image(ImageData)`. Use `ImageData::from_bytes` to validate/encode bytes and
`bytes()` to decode/validate them for a model. Encoded files are limited to 8 MiB,
dimensions to 4096×4096, and decoder allocation to 128 MiB. File data stays local.
Unsupported formats and malformed images are rejected. The image example uses a
built-in gradient, grayscale conversion, and contrast adjustment.

`Input::table` imports UTF-8 CSV with a header; `Output::table` displays
`Value::Table(Table { columns, rows })` and exports CSV. Tables support 1–64 unique,
nonempty column names and up to 1,000 rows, with 8,192 bytes per cell and 2 MiB
total text. CSV inputs and presets must also fit the 2 MiB encoded CSV limit.
The interface previews the first 100 rows; export contains all rows. Tables are
read-only in this release. CSV quoting preserves commas, quotes, and newlines.

`Session::predict_batch` reuses the model sequentially and returns one `Result`
per submitted row, preserving order. The batch example maps CSV rows into model
inputs, retains row errors, and exports both successful predictions and errors.

The Export CSV button opens a native Save dialog and reports save errors or
cancellation in the interface. Linux needs a working XDG desktop portal.
Formula-like text cells receive a leading apostrophe on UI export to keep
spreadsheet applications from evaluating them; finite numerical strings are
preserved. `Table::to_csv()` instead preserves exact cell contents for programmatic
round trips. Pending file reads block submission; replacing a selection discards
an older read rather than overwriting the newer selection.

## Live prediction

Opt in with `with_live_prediction(200)` (milliseconds, allowed range 50–5,000).
The first prediction runs after model preparation and subsequent changes wait for
the debounce interval. Inputs remain editable during computation. There is one
running request and at most one pending snapshot; further edits replace pending
work. A result from older inputs is discarded even when its request ID matches.
Pause the Live prediction checkbox to stop pending automatic work and use Predict.
An already-running callback continues; pausing does not cancel user code.

Live validation/model errors recover on the next edit. Fatal failures disable the
controls and clear pending work. Text composition waits for committed text; image
and CSV predictions wait for file reads to finish. A live model should be safe to
call repeatedly. `cargo run --example live` intentionally waits one second per call,
so rapid edits and the model-call counter make the queue behavior observable.

## Background inference

`run()` creates the model and executes predictions on a dedicated worker thread.
The UI shows preparation/progress states. In manual mode, it disables form controls
until each request finishes; live mode leaves inputs editable. Duplicate requests
and stale results are ignored. The window can still be moved, resized, or closed
during computation.

Direct callbacks must be `Send`, but need not be `Sync`. Use a factory to create
models that cannot be moved between threads:

```no_run
use tease::{Teaser, Value};

fn main() -> Result<(), tease::Error> {
    Teaser::default()
        .with_predictor_factory(|| {
            // Load or train a model here, on the inference worker.
            let multiplier = 2.0;
            Ok(move |values: &[Value]| Ok(vec![Value::Number(values[0].as_number()? * multiplier)]))
        })
        .run()
}
```

The factory runs once per session; its returned callback need not be `Send`.
For headless use, `prepare()` runs the same initialization on the calling thread.
Closing the window does not wait for model work, so callbacks must not depend on
finishing or running cleanup on exit. Cooperative cancellation is not supported.

## Build and runtime prerequisites

Rust **1.88 or newer** is required for the supported development baseline,
including the published AutoML example's dependency stack. Keep `Cargo.lock`
for reproducible repository builds and use `--locked`. Downstream applications
should keep their own lockfile. The crate uses Wry 0.57 and Tao 0.37.

- **macOS:** Xcode Command Line Tools (`xcode-select --install`); WebKit is supplied
  by macOS. Run the GUI on the main thread in a desktop session.
- **Windows:** the MSVC Rust toolchain and Visual Studio C++ build tools;
  Microsoft Edge WebView2 Runtime must be installed for the demo to run.
- **Linux:** GTK 3 and WebKitGTK 4.1 development libraries to build, corresponding
  runtime libraries plus an X11 or Wayland desktop session to run. On Ubuntu:
  `sudo apt-get install libgtk-3-dev libwebkit2gtk-4.1-dev libxdo-dev libdbus-1-dev`.

A compiled executable still depends on these platform webview/runtime libraries;
it is not a universal binary that runs on every operating system.
