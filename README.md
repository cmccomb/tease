# Tease

Turn a Rust function or machine-learning model into a standalone desktop demo.
Tease generates labeled inputs and results in a native webview, validates input
before calling your model, and displays prediction errors without restarting.

Tease 0.2.0 uses the typed callback API introduced in 0.2. See
[migration guidance](docs/migration-0.2.md) when upgrading from 0.1.1.

Add it to a Rust application with `cargo add tease@0.2.0`, then put the example
below in `src/main.rs` and run `cargo run`. Install the desktop prerequisites below
first; the GUI must run on the main thread in a desktop session.

## A numerical demo

```no_run
use tease::{Input, Teaser};

fn main() -> Result<(), tease::Error> {
    Teaser::default()
        .with_title("Addition")
        .with_inputs(vec![Input::default(); 2])
        .with_function(|numbers| numbers.iter().sum())
        .run()
}
```

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

Inputs support numbers with optional bounds, sliders, and labeled dropdowns.
Dropdown choices carry numbers or text. Outputs support numbers and plain text.
All labels, descriptions, errors, and text results are rendered as text; HTML is
not supported. Numerical values must be finite. Callbacks should return errors
rather than panic. With unwinding enabled, the desktop worker reports a callback
panic and retires the model; headless sessions propagate panics to the caller.

Use `Teaser::validate()` to check configuration. For headless predictions, call
`let session = demo.prepare()?` once and reuse `session.predict(&values)`.

## Background inference

`run()` creates the model and executes predictions on a dedicated worker thread.
The UI shows preparation/progress states and disables form controls until each
request finishes. Duplicate requests and stale results are ignored. The window
can still be moved, resized, or closed during computation.

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

## Examples and checks

```sh
cargo run --locked --example addition
cargo run --locked --example classification
cargo run --locked --example division
cargo run --locked --example smartcore
cargo run --locked --example smartcore_classification
cargo run --locked --example slow
cargo run --locked --example automl

cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
node --test tests/ui.test.cjs
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

CI is configured for Windows, macOS, and Linux on both stable and Rust 1.88.
The model and worker tests do not need a display server. See the
[example guide](docs/examples.md) for input meanings and checked predictions:
SmartCore regression and Iris classification, AutoML regression, typed results,
error recovery, and a deliberately slow model.

## Distribution and release status

See [RELEASING.md](RELEASING.md) for archive,
clean-consumer, platform, and publication checks. Run
`python3 scripts/check-package.py` to verify the package locally. A successful
local build or dry run does not mean the crate has been published or every target
platform has been tested.

Closing the window does not wait for model work, and callbacks must not depend on
finishing or running cleanup on exit. Cooperative cancellation, plots, parameter
sweeps, and installer/signing automation are outside this release.

## License

Licensed under either MIT or Apache-2.0, at your option. See LICENSE-MIT and
LICENSE-APACHE. Dependency licenses remain their respective owners' terms.
