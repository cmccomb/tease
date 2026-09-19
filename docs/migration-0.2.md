# Migrating to Tease 0.2

0.2 is a breaking development release. The published 0.1.1 API and unpublished
`master` differ; migrate from the code you actually use.

## From published 0.1.1

- `Teaser` and numerical runtime values use `f64`, as in published 0.1.1.
- The short `.with_function(|values| values.iter().sum())` path remains supported.
- `run()` returns `Result<(), tease::Error>` for setup failures. Return it from
  `main`, or propagate it with `?`. Run the native event loop on the main thread.
- Prefer `Input::number("Label", initial)` to constructing a numerical variant.
  `Input::Number` now also includes `min: Option<f64>` and `max: Option<f64>`.
- Dropdowns use `Vec<Choice>` rather than `Vec<f64>`. For the same numerical
  behavior, map each option with `Choice::number(value.to_string(), value)`.
  `initial_value` is still a zero-based selection index.
- Use `Output::number("Result", 2)` or include `precision` when constructing
  `Output::Number`. Use `Output::text("Class")` for a text result.
- Labels and descriptions are plain text. Replace HTML descriptions with text.
- Invalid component defaults are rejected before a window is created. Empty
  input or output lists, invalid ranges, off-step sliders, non-finite values,
  and invalid initial dropdown indices are configuration errors.

## From unpublished master

The generic `Teaser<F>` / `Input<F>` API was never published. Remove generic
parameters and use `f64` at the interface boundary. Models using `f32` may convert
inside their callback, handling conversion failures as model errors.

`with_advanced_function` has been removed. It stored a callback that the GUI never
called. Use `with_predictor` with runtime `Value`s rather than component definitions.

## Fallible and multiple-result callbacks

```rust
use tease::{Input, Output, Teaser, Value};

let demo = Teaser::default()
    .with_inputs(vec![Input::number("Numerator", 4.0), Input::number("Divisor", 2.0)])
    .with_outputs(vec![Output::number("Quotient", 3), Output::text("Status")])
    .with_predictor(|values| {
        let numerator = values[0].as_number()?;
        let divisor = values[1].as_number()?;
        if divisor == 0.0 {
            return Err("Divisor must not be zero.".into());
        }
        Ok(vec![Value::Number(numerator / divisor), Value::Text("Calculated".into())])
    });
let session = demo.prepare()?;
assert_eq!(session.predict(&[Value::Number(4.0), Value::Number(2.0)])?,
    vec![Value::Number(2.0), Value::Text("Calculated".into())]);
# Ok::<(), tease::Error>(())
```

Input values and output values follow their component order. Result count, type,
and finiteness are validated. The latest callback setter replaces any previous
callback; both setters use the same execution path. Numeric callbacks require
numerical inputs and exactly one numerical output. Typed callbacks return
`Result<Vec<Value>, String>`; adapt library errors using `.map_err(|e| e.to_string())`.

## Thread ownership and headless sessions

`run()` prepares the model and predicts on a dedicated worker thread. The supplied
`with_function` or `with_predictor` closure must now be `Send` (not `Sync`). If your
model is not `Send`, construct it inside `with_predictor_factory`; the factory
moves to the worker, but its returned model callback stays there.

```rust
use std::{cell::Cell, rc::Rc};
use tease::{Teaser, Value};

let demo = Teaser::default().with_predictor_factory(|| {
    let count = Rc::new(Cell::new(0)); // Deliberately not Send.
    Ok(move |values: &[Value]| {
        count.set(count.get() + 1);
        Ok(vec![Value::Number(values[0].as_number()? + f64::from(count.get()))])
    })
});
let session = demo.prepare()?;
assert_eq!(session.predict(&[Value::Number(1.0)])?, vec![Value::Number(2.0)]);
assert_eq!(session.predict(&[Value::Number(1.0)])?, vec![Value::Number(3.0)]);
# Ok::<(), tease::Error>(())
```

For headless use, call `prepare()` once, then reuse `Session::predict`. Preparation
and predictions run synchronously on the caller's thread in this mode. Earlier
unpublished 0.2 drafts had `Teaser::predict`; replace that with
`let session = demo.prepare()?` and use the session for subsequent predictions.

The GUI disables form controls until model preparation or inference completes.
Recoverable validation/model errors permit resubmission. A callback panic with
Rust's default unwinding strategy is displayed as a fatal model error and retires
that worker; with `panic = "abort"` it terminates the process. Panics propagate to
headless callers. Closing the window exits promptly without waiting for inference;
there is no cooperative cancellation API or guarantee that callback cleanup runs.
Use callbacks for predictions, not work whose completion is required on exit.
