# Tease

Turn a Rust function or machine-learning model into a desktop demo. Tease builds
the input form and results view, validates submitted values, and runs predictions
on a worker thread so the window stays responsive.

## Try it live

Drag **Offset** to reshape the curve, then switch between line, scatter, and bar plots.
The number and plot update together.

<iframe title="Interactive Tease response-curve preview" src="https://cmccomb.com/tease/live/?preview=hero" width="100%" height="550" style="max-width:820px;border:1px solid #d1e0e9;border-radius:10px" loading="eager"><a href="https://cmccomb.com/tease/live/">Open the interactive preview</a></iframe>

These controls and results use Tease's HTML renderer. The browser preview runs
the example's analytical model in JavaScript; the [desktop example](https://github.com/cmccomb/tease/blob/master/examples/preview.rs)
runs its Rust callback on a worker thread.

## Get started

Use Rust 1.88 or newer in a desktop session. Install the
[platform prerequisites](#platform-prerequisites), then add Tease to your application:

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

## Explore the widgets

Try a [slider](enum.Input.html#variant.Slider), a
[dropdown](enum.Input.html#variant.Dropdown), a
[plot](enum.Output.html#method.plot), or
[probability bars](enum.Output.html#method.probabilities) on their API pages.
The [feature guide](https://github.com/cmccomb/tease/blob/master/docs/guide.md)
covers typed callbacks, text and file inputs, tables, presets, comparisons, and batch prediction.

## More examples

Run `cargo run --locked --example preview` from a Tease checkout to open the full
response demo. The [example guide](https://github.com/cmccomb/tease/blob/master/docs/examples.md)
lists the other demos and checked results, including SmartCore and AutoML models.

## Platform prerequisites

- **macOS:** Xcode Command Line Tools and the system WebKit runtime.
- **Windows:** MSVC Rust toolchain, Visual Studio C++ build tools, and WebView2 Runtime.
- **Linux:** GTK 3 and WebKitGTK 4.1 development and runtime libraries, plus a desktop session.
  On Ubuntu, install `libgtk-3-dev libwebkit2gtk-4.1-dev libxdo-dev libdbus-1-dev`.

Run the GUI on the main thread. Built executables still need the target system's
webview libraries. See the [release guide](https://github.com/cmccomb/tease/blob/master/RELEASING.md)
for distribution and validation checks.

## License

Licensed under [MIT](https://github.com/cmccomb/tease/blob/master/LICENSE-MIT) or
[Apache-2.0](https://github.com/cmccomb/tease/blob/master/LICENSE-APACHE), at your option.
