# Tease

[![Crates.io](https://img.shields.io/crates/v/tease.svg)](https://crates.io/crates/tease)
[![Documentation](https://docs.rs/tease/badge.svg)](https://docs.rs/tease)
[![CI](https://github.com/cmccomb/tease/actions/workflows/tests.yml/badge.svg?branch=master)](https://github.com/cmccomb/tease/actions/workflows/tests.yml)
[![License](https://img.shields.io/crates/l/tease.svg)](https://github.com/cmccomb/tease#license)

Turn a Rust function or machine-learning model into a desktop demo. Tease builds
the input form and results view, validates submitted values, and runs predictions
on a worker thread so the window stays responsive.

## Get started

Use Rust 1.88 or newer in a desktop session. Install the [platform prerequisites](#platform-prerequisites),
then add Tease to your application:

```sh
cargo add tease
```

Put this in `src/main.rs` and run `cargo run`:

```rust,no_run
use tease::{Input, Teaser};

fn main() -> Result<(), tease::Error> {
    Teaser::default()
        .with_title("Addition")
        .with_inputs(vec![Input::default(); 2])
        .with_function(|numbers| numbers.iter().sum())
        .run()
}
```

Tease also accepts typed, fallible callbacks for multiple results. Inputs include
numbers, choices, text, images, and CSV tables. Results can include text, plots,
images, tables, and class scores. Live prediction, presets, comparisons, and batch
prediction are optional. See the [feature guide](https://github.com/cmccomb/tease/blob/master/docs/guide.md)
for the APIs, limits, and inference behavior.

## Examples

From a Tease checkout, try:

```sh
cargo run --locked --example addition
cargo run --locked --example exploration
cargo run --locked --example images
cargo run --locked --example batch
cargo run --locked --example live
```

The [example guide](https://github.com/cmccomb/tease/blob/master/docs/examples.md)
lists all demos and checked results, including SmartCore and AutoML models.

## Platform prerequisites

- **macOS:** Xcode Command Line Tools and the system WebKit runtime.
- **Windows:** MSVC Rust toolchain, Visual Studio C++ build tools, and WebView2 Runtime.
- **Linux:** GTK 3 and WebKitGTK 4.1 development and runtime libraries, plus a desktop session.
  On Ubuntu, install `libgtk-3-dev libwebkit2gtk-4.1-dev libxdo-dev libdbus-1-dev`.

Run the GUI on the main thread. Built executables still need the target system's
webview libraries. See [RELEASING.md](https://github.com/cmccomb/tease/blob/master/RELEASING.md)
for distribution and validation checks.

## Documentation

- [API documentation](https://docs.rs/tease)
- [Feature guide](https://github.com/cmccomb/tease/blob/master/docs/guide.md)
- [Examples](https://github.com/cmccomb/tease/blob/master/docs/examples.md)
- [Migration from 0.1.1](https://github.com/cmccomb/tease/blob/master/docs/migration-0.2.md)
- [Changelog](https://github.com/cmccomb/tease/blob/master/CHANGELOG.md)

## License

Licensed under [MIT](https://github.com/cmccomb/tease/blob/master/LICENSE-MIT) or
[Apache-2.0](https://github.com/cmccomb/tease/blob/master/LICENSE-APACHE), at your option.
