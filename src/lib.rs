#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod bridge;
mod components;
mod desktop;
mod html;
mod media;
mod rich;
pub use media::{ImageData, Table};
mod worker;
pub use rich::{ClassScore, Plot, PlotKind, Preset, Series};

pub mod cookbook;
pub use components::{Choice, Input, Output, Value};

use std::fmt;

/// A configuration, input, prediction, output, or desktop error.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A component definition is invalid.
    Configuration(String),
    /// A submitted field is invalid. Indices are zero-based.
    Input {
        /// The index of the affected input, if known.
        field: Option<usize>,
        /// A message suitable for displaying to the user.
        message: String,
    },
    /// The model returned an error.
    Prediction(String),
    /// Results do not match the output definitions.
    Output(String),
    /// The native window or webview could not be created.
    Desktop(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) => write!(f, "Invalid configuration: {message}"),
            Self::Input { message, .. } => f.write_str(message),
            Self::Prediction(message) => write!(f, "Prediction failed: {message}"),
            Self::Output(message) => write!(f, "Invalid prediction output: {message}"),
            Self::Desktop(message) => write!(f, "Desktop error: {message}"),
        }
    }
}
impl std::error::Error for Error {}

type Predictor = dyn Fn(&[Value]) -> Result<Vec<Value>, String>;
type Factory = dyn FnOnce() -> Result<Box<Predictor>, String> + Send;

/// Configure a standalone model demo.
///
/// Model initialization and GUI predictions run on a dedicated worker thread.
/// Use [`Self::prepare`] to create a headless prediction session.
pub struct Teaser {
    title: String,
    description: String,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    factory: Box<Factory>,
    numeric_callback: bool,
    presets: Vec<Preset>,
    comparison: bool,
    live_delay: Option<u64>,
}

impl Default for Teaser {
    fn default() -> Self {
        Self {
            title: "Demo".into(),
            description: String::new(),
            inputs: vec![Input::default()],
            outputs: vec![Output::default()],
            factory: Box::new(|| Ok(Box::new(|values| Ok(vec![values[0].clone()])))),
            numeric_callback: true,
            presets: Vec::new(),
            comparison: false,
            live_delay: None,
        }
    }
}

impl Teaser {
    /// Set the native window title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    /// Set a plain-text description. HTML is displayed literally.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
    /// Define the inputs in callback argument order.
    pub fn with_inputs(mut self, inputs: Vec<Input>) -> Self {
        self.inputs = inputs;
        self
    }
    /// Define a single output.
    pub fn with_output(self, output: Output) -> Self {
        self.with_outputs(vec![output])
    }
    /// Define outputs in the same order as the callback's returned values.
    pub fn with_outputs(mut self, outputs: Vec<Output>) -> Self {
        self.outputs = outputs;
        self
    }
    /// Adapt a simple numerical function to the common prediction pipeline.
    ///
    /// All inputs must have numerical values and there must be one numerical
    /// output. For mixed inputs, multiple results, or errors, use [`Self::with_predictor`].
    /// The most recent callback setter replaces any earlier callback.
    pub fn with_function(mut self, function: impl Fn(Vec<f64>) -> f64 + Send + 'static) -> Self {
        self.factory = Box::new(move || {
            Ok(Box::new(move |values: &[Value]| {
                let numbers = values
                    .iter()
                    .map(Value::as_number)
                    .collect::<Result<_, _>>()?;
                Ok(vec![Value::Number(function(numbers))])
            }))
        });
        self.numeric_callback = true;
        self
    }
    /// Set a typed, fallible prediction callback.
    ///
    /// Values arrive in input order after validation. Return values in output
    /// order, or an error message to show in the interface. The callback owns
    /// any captured model and must implement `Send`, but need not be `Sync`.
    /// For models that are not `Send`, use [`Self::with_predictor_factory`].
    /// The most recent callback setter replaces any earlier callback.
    pub fn with_predictor(
        mut self,
        predictor: impl Fn(&[Value]) -> Result<Vec<Value>, String> + Send + 'static,
    ) -> Self {
        self.factory = Box::new(move || Ok(Box::new(predictor)));
        self.numeric_callback = false;
        self
    }
    /// Initialize a model on the inference thread, once per session.
    ///
    /// The factory must be `Send`; the returned callback and its captured model
    /// need not be `Send` or `Sync`. Initialization errors are displayed in the
    /// GUI. [`Self::prepare`] invokes the same factory on the calling thread.
    pub fn with_predictor_factory<F, P>(mut self, factory: F) -> Self
    where
        F: FnOnce() -> Result<P, String> + Send + 'static,
        P: Fn(&[Value]) -> Result<Vec<Value>, String> + 'static,
    {
        self.factory =
            Box::new(move || factory().map(|predictor| Box::new(predictor) as Box<Predictor>));
        self.numeric_callback = false;
        self
    }

    /// Add named input scenarios. Values are validated before the GUI opens.
    pub fn with_presets(mut self, presets: Vec<Preset>) -> Self {
        self.presets = presets;
        self
    }
    /// Allow pinning up to three results and comparing them with the current result.
    pub fn with_comparison(mut self, enabled: bool) -> Self {
        self.comparison = enabled;
        self
    }
    /// Enable live prediction after edits settle for 50–5,000 milliseconds.
    /// One request runs at a time; only the latest pending input snapshot is retained.
    /// The user can pause live updates in the window.
    pub fn with_live_prediction(mut self, debounce_ms: u64) -> Self {
        self.live_delay = Some(debounce_ms);
        self
    }
    /// Check component definitions and numerical adapter compatibility.
    pub fn validate(&self) -> Result<(), Error> {
        if self.live_delay.is_some_and(|ms| !(50..=5000).contains(&ms)) {
            return Err(Error::Configuration(
                "Live prediction delay must be 50–5,000 ms.".into(),
            ));
        }
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return Err(Error::Configuration(
                "At least one input and output are required.".into(),
            ));
        }
        for (index, input) in self.inputs.iter().enumerate() {
            input.validate(index)?;
        }
        for preset in &self.presets {
            if preset.inputs.len() != self.inputs.len() {
                return Err(Error::Configuration(format!(
                    "Preset {} has the wrong input count.",
                    preset.name
                )));
            }
            for (index, (input, value)) in self.inputs.iter().zip(&preset.inputs).enumerate() {
                input
                    .validate_value(index, value)
                    .map_err(|e| Error::Configuration(format!("Preset {}: {e}", preset.name)))?;
            }
        }
        for output in &self.outputs {
            if let Output::Number { precision, .. } = output {
                if *precision > 15 {
                    return Err(Error::Configuration(
                        "Output precision must be at most 15.".into(),
                    ));
                }
            }
        }
        if self.numeric_callback
            && (self.inputs.iter().any(|input| !input.is_numeric())
                || !matches!(self.outputs.as_slice(), [Output::Number { .. }]))
        {
            return Err(Error::Configuration(
                "with_function requires numerical inputs and one numerical output; use with_predictor for typed results.".into(),
            ));
        }
        Ok(())
    }
    /// Initialize a reusable prediction session on the current thread.
    ///
    /// Use this for headless tests or applications that manage their own threads.
    /// Initialization runs once; [`Session::predict`] can be called repeatedly.
    pub fn prepare(self) -> Result<Session, Error> {
        self.validate()?;
        let predictor = (self.factory)().map_err(Error::Prediction)?;
        Ok(Session {
            inputs: self.inputs,
            outputs: self.outputs,
            predictor,
        })
    }

    /// Open the native GUI on the main thread after validating configuration.
    ///
    /// Returns setup errors. Model initialization and predictions run on a
    /// dedicated worker. Closing the window does not wait for running inference;
    /// callbacks must not rely on graceful completion when the application exits.
    pub fn run(self) -> Result<(), Error> {
        self.validate()?;
        desktop::run(self)
    }
}

/// A prepared model session for repeated, synchronous, headless predictions.
///
/// Created by [`Teaser::prepare`]. This session stays on the thread where the
/// model was initialized and does not require the callback to be `Send` or `Sync`.
pub struct Session {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    predictor: Box<Predictor>,
}

impl Session {
    /// Predict rows independently, preserving row order and individual validation/model errors.
    /// The model is reused and rows execute sequentially on the calling thread.
    pub fn predict_batch(&self, rows: &[Vec<Value>]) -> Vec<Result<Vec<Value>, Error>> {
        rows.iter().map(|row| self.predict(row)).collect()
    }
    /// Validate inputs, invoke the model, and validate its results.
    ///
    /// Expected failures should be returned as `Err`. Panics propagate for
    /// headless callers; the desktop worker reports a panic and retires the model.
    pub fn predict(&self, values: &[Value]) -> Result<Vec<Value>, Error> {
        if values.len() != self.inputs.len() {
            return Err(Error::Input {
                field: None,
                message: format!(
                    "Expected {} inputs, received {}.",
                    self.inputs.len(),
                    values.len()
                ),
            });
        }
        for (index, (input, value)) in self.inputs.iter().zip(values).enumerate() {
            input.validate_value(index, value)?;
        }
        let results = (self.predictor)(values).map_err(Error::Prediction)?;
        if results.len() != self.outputs.len() {
            return Err(Error::Output(format!(
                "Expected {} results, received {}.",
                self.outputs.len(),
                results.len()
            )));
        }
        for (index, (output, value)) in self.outputs.iter().zip(&results).enumerate() {
            match (output, value) {
                (Output::Number { .. }, Value::Number(number)) if number.is_finite() => {}
                (Output::Text { .. }, Value::Text(_)) => {}
                (Output::Image { .. }, Value::Image(image)) if image.bytes().is_ok() => {}
                (Output::Table { .. }, Value::Table(table)) if table.validate().is_ok() => {}
                (Output::Plot { .. }, Value::Plot(plot)) if plot.validate() => {}
                (Output::Scores { probabilities, .. }, Value::Scores(scores))
                    if !scores.is_empty()
                        && scores.len() <= 100
                        && scores.iter().enumerate().all(|(i, c)| {
                            c.score.is_finite()
                                && c.score >= 0.0
                                && (!probabilities || c.score <= 1.0)
                                && !scores[..i].iter().any(|v| v.label == c.label)
                        })
                        && (!probabilities
                            || (scores.iter().map(|s| s.score).sum::<f64>() - 1.0).abs()
                                <= 1e-6) => {}
                _ => {
                    return Err(Error::Output(format!(
                        "Result {} has the wrong type or is not finite.",
                        index + 1
                    )))
                }
            }
        }
        Ok(results)
    }
}
