use crate::{ClassScore, Error, Plot};
use serde::{Deserialize, Serialize};

/// A runtime model value, distinct from a component's display configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Value {
    /// A finite numerical value.
    Number(f64),
    /// A text value, such as a category or class label.
    Text(String),
    /// A checkbox value.
    Bool(bool),
    /// Selected values from a multi-select input, in selection order.
    List(Vec<Value>),
    /// A numerical plot.
    Plot(Plot),
    /// Class scores or probabilities.
    Scores(Vec<ClassScore>),
}
impl Value {
    /// Read a numerical value or return a callback-friendly error message.
    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            Self::Number(value) => Ok(*value),
            _ => Err("Expected a numerical value.".into()),
        }
    }
    /// Read a checkbox value.
    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Self::Bool(value) => Ok(*value),
            _ => Err("Expected a boolean.".into()),
        }
    }
    /// Read a multi-selection.
    pub fn as_list(&self) -> Result<&[Value], String> {
        match self {
            Self::List(values) => Ok(values),
            _ => Err("Expected a selection list.".into()),
        }
    }
    /// Read a text value or return a callback-friendly error message.
    pub fn as_text(&self) -> Result<&str, String> {
        match self {
            Self::Text(value) => Ok(value),
            _ => Err("Expected a text value.".into()),
        }
    }
}

/// A dropdown label and the value passed to the model when selected.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    /// Plain-text label displayed in the dropdown.
    pub label: String,
    /// Model value; dropdown messages carry indices, never arbitrary values.
    pub value: Value,
}
impl Choice {
    /// Define a labeled numerical choice.
    pub fn number(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value: Value::Number(value),
        }
    }
    /// Define a labeled text choice.
    pub fn text(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: Value::Text(value.into()),
        }
    }
}

/// An input component definition. All fields are required in submitted forms.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Input {
    /// A numerical textbox with optional inclusive bounds.
    Number {
        /// Plain-text label, or a generated one-based label.
        label: Option<String>,
        /// Initial finite value.
        initial_value: f64,
        /// Inclusive minimum, when present.
        min: Option<f64>,
        /// Inclusive maximum, when present.
        max: Option<f64>,
    },
    /// A bounded numerical slider.
    Slider {
        /// Plain-text label, or a generated one-based label.
        label: Option<String>,
        /// Inclusive minimum and step origin.
        min: f64,
        /// Inclusive maximum.
        max: f64,
        /// Positive finite increment from the minimum.
        step: f64,
        /// Initial finite value, on a step within the range.
        initial_value: f64,
    },
    /// A dropdown whose options may carry numerical or text values.
    Dropdown {
        /// Plain-text label, or a generated one-based label.
        label: Option<String>,
        /// Non-empty list of labeled choices.
        options: Vec<Choice>,
        /// Zero-based index of the initially selected option.
        initial_value: usize,
    },
    /// A single- or multiple-line text input.
    Text {
        /// Plain-text label.
        label: Option<String>,
        /// Initial text, which may be empty.
        initial_value: String,
        /// Display multiple lines when true.
        multiline: bool,
        /// Maximum Unicode scalar values; must be 1–100,000.
        max_length: usize,
    },
    /// A boolean checkbox.
    Checkbox {
        /// Plain-text label.
        label: Option<String>,
        /// Initial checked state.
        initial_value: bool,
    },
    /// A list of independently selectable labeled values.
    MultiSelect {
        /// Plain-text label.
        label: Option<String>,
        /// Nonempty, unique numerical or text choices.
        options: Vec<Choice>,
        /// Initially selected option indices, without duplicates.
        initial_values: Vec<usize>,
    },
    /// Presentation metadata around an input; use `with_unit` and `with_help`.
    Described {
        /// The underlying component.
        input: Box<Input>,
        /// Displayed measurement unit.
        unit: Option<String>,
        /// Plain-text explanation shown below the control.
        help: Option<String>,
    },
}
impl Default for Input {
    fn default() -> Self {
        Self::Number {
            label: None,
            initial_value: 0.0,
            min: None,
            max: None,
        }
    }
}
impl Input {
    /// Create an unbounded numerical textbox with a label.
    pub fn number(label: impl Into<String>, initial_value: f64) -> Self {
        Self::Number {
            label: Some(label.into()),
            initial_value,
            min: None,
            max: None,
        }
    }
    /// Create a text input with a 4,096-character limit.
    pub fn text(
        label: impl Into<String>,
        initial_value: impl Into<String>,
        multiline: bool,
    ) -> Self {
        Self::Text {
            label: Some(label.into()),
            initial_value: initial_value.into(),
            multiline,
            max_length: 4096,
        }
    }
    /// Create a checkbox.
    pub fn checkbox(label: impl Into<String>, initial_value: bool) -> Self {
        Self::Checkbox {
            label: Some(label.into()),
            initial_value,
        }
    }
    /// Show a unit alongside the label, without changing the model value.
    pub fn with_unit(self, value: impl Into<String>) -> Self {
        match self {
            Self::Described { input, help, .. } => Self::Described {
                input,
                help,
                unit: Some(value.into()),
            },
            input => Self::Described {
                input: Box::new(input),
                unit: Some(value.into()),
                help: None,
            },
        }
    }
    /// Show explanatory text below the control.
    pub fn with_help(self, value: impl Into<String>) -> Self {
        match self {
            Self::Described { input, unit, .. } => Self::Described {
                input,
                unit,
                help: Some(value.into()),
            },
            input => Self::Described {
                input: Box::new(input),
                help: Some(value.into()),
                unit: None,
            },
        }
    }
    pub(crate) fn base(&self) -> &Self {
        match self {
            Self::Described { input, .. } => input.base(),
            input => input,
        }
    }
    pub(crate) fn label(&self, index: usize) -> String {
        let (Self::Number { label, .. }
        | Self::Slider { label, .. }
        | Self::Dropdown { label, .. }
        | Self::Text { label, .. }
        | Self::Checkbox { label, .. }
        | Self::MultiSelect { label, .. }) = self.base()
        else {
            unreachable!()
        };
        label
            .clone()
            .unwrap_or_else(|| format!("Input {}", index + 1))
    }
    pub(crate) fn is_numeric(&self) -> bool {
        match self.base() {
            Self::Dropdown { options, .. } => options
                .iter()
                .all(|choice| matches!(choice.value, Value::Number(_))),
            Self::Number { .. } | Self::Slider { .. } => true,
            _ => false,
        }
    }
    pub(crate) fn error(&self, index: usize, message: &str) -> Error {
        Error::Input {
            field: Some(index),
            message: format!("{}: {message}", self.label(index)),
        }
    }
    pub(crate) fn validate(&self, index: usize) -> Result<(), Error> {
        let invalid =
            |message: &str| Error::Configuration(format!("{}: {message}", self.label(index)));
        let initial = match self.base() {
            Self::Number {
                initial_value,
                min,
                max,
                ..
            } => {
                if min.iter().chain(max).any(|bound| !bound.is_finite())
                    || matches!((min, max), (Some(a), Some(b)) if a > b)
                {
                    return Err(invalid("Bounds must be finite and ordered."));
                }
                Value::Number(*initial_value)
            }
            Self::Slider {
                min,
                max,
                step,
                initial_value,
                ..
            } => {
                if !min.is_finite()
                    || !max.is_finite()
                    || min >= max
                    || !step.is_finite()
                    || *step <= 0.0
                    || !((max - min) / step).is_finite()
                {
                    return Err(invalid(
                        "Slider bounds must be finite and increasing, with a positive finite step.",
                    ));
                }
                Value::Number(*initial_value)
            }
            Self::Dropdown {
                options,
                initial_value,
                ..
            } => {
                if options.is_empty() || *initial_value >= options.len() {
                    return Err(invalid(
                        "Choose an initial index within a non-empty list of options.",
                    ));
                }
                if options
                    .iter()
                    .any(|choice| matches!(choice.value, Value::Number(n) if !n.is_finite()))
                {
                    return Err(invalid("Option values must be finite."));
                }
                if options
                    .iter()
                    .any(|c| !matches!(c.value, Value::Number(_) | Value::Text(_)))
                {
                    return Err(invalid("Choices must contain numbers or text."));
                }
                options[*initial_value].value.clone()
            }
            Self::Text {
                initial_value,
                max_length,
                ..
            } => {
                if *max_length == 0 || *max_length > 100_000 {
                    return Err(invalid("Text length limit must be 1–100,000."));
                }
                Value::Text(initial_value.clone())
            }
            Self::Checkbox { initial_value, .. } => Value::Bool(*initial_value),
            Self::MultiSelect {
                options,
                initial_values,
                ..
            } => {
                if options.is_empty()
                    || options.len() > 1000
                    || options.iter().enumerate().any(|(i, c)| {
                        !matches!(&c.value, Value::Number(n) if n.is_finite())
                            && !matches!(c.value, Value::Text(_))
                            || options[..i].iter().any(|other| other.value == c.value)
                    })
                {
                    return Err(invalid(
                        "Multi-select choices must be unique finite numbers or text.",
                    ));
                }
                Value::List(
                    initial_values
                        .iter()
                        .map(|i| {
                            options.get(*i).map(|c| c.value.clone()).ok_or_else(|| {
                                invalid("Initial choice index is outside the available options.")
                            })
                        })
                        .collect::<Result<_, _>>()?,
                )
            }
            Self::Described { .. } => unreachable!(),
        };
        self.validate_value(index, &initial)
            .map_err(|error| invalid(&error.to_string()))
    }
    pub(crate) fn validate_value(&self, index: usize, value: &Value) -> Result<(), Error> {
        let invalid = |message: &str| self.error(index, message);
        if matches!(value, Value::Number(n) if !n.is_finite()) {
            return Err(invalid("Enter a finite number."));
        }
        match self.base() {
            Self::Number { min, max, .. } => {
                let n = value.as_number().map_err(|_| invalid("Enter a number."))?;
                if min.is_some_and(|min| n < min) || max.is_some_and(|max| n > max) {
                    return Err(invalid("Value is outside the allowed range."));
                }
            }
            Self::Slider { min, max, step, .. } => {
                let n = value.as_number().map_err(|_| invalid("Enter a number."))?;
                if n < *min || n > *max {
                    return Err(invalid("Value is outside the slider range."));
                }
                let steps = (n - min) / step;
                let tolerance = (8.0 * f64::EPSILON * steps.abs().max(1.0)).min(1e-6);
                if !steps.is_finite() || (steps - steps.round()).abs() > tolerance {
                    return Err(invalid("Value must follow the slider step."));
                }
            }
            Self::Dropdown { options, .. } => {
                if !options.iter().any(|choice| choice.value == *value) {
                    return Err(invalid("Choose one of the available options."));
                }
            }
            Self::Text { max_length, .. } => {
                let text = value.as_text().map_err(|_| invalid("Enter text."))?;
                if text.chars().count() > *max_length {
                    return Err(invalid("Text exceeds the character limit."));
                }
            }
            Self::Checkbox { .. } => {
                value
                    .as_bool()
                    .map_err(|_| invalid("Choose a checkbox value."))?;
            }
            Self::MultiSelect { options, .. } => {
                let values = value
                    .as_list()
                    .map_err(|_| invalid("Choose values from the list."))?;
                if values
                    .iter()
                    .enumerate()
                    .any(|(i, v)| values[..i].contains(v) || !options.iter().any(|c| c.value == *v))
                {
                    return Err(invalid("Selections must be unique available choices."));
                }
            }
            Self::Described { .. } => unreachable!(),
        }
        Ok(())
    }
    pub(crate) fn encode(&self, value: &Value) -> String {
        match (self.base(), value) {
            (Self::Dropdown { options, .. }, v) => options
                .iter()
                .position(|c| c.value == *v)
                .unwrap()
                .to_string(),
            (Self::MultiSelect { options, .. }, Value::List(values)) => serde_json::to_string(
                &values
                    .iter()
                    .map(|v| options.iter().position(|c| c.value == *v).unwrap())
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            (_, Value::Text(v)) => v.clone(),
            (_, Value::Bool(v)) => v.to_string(),
            (_, Value::Number(v)) => v.to_string(),
            _ => unreachable!("validated input value"),
        }
    }
}

/// A result display definition; actual results are returned as [`Value`]s.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Output {
    /// A finite numerical result.
    Number {
        /// Plain-text label, or a generated one-based label.
        label: Option<String>,
        /// Decimal places, from zero through fifteen.
        precision: usize,
    },
    /// A plain-text result such as a class label.
    Text {
        /// Plain-text label, or a generated one-based label.
        label: Option<String>,
    },
    /// A numerical line, scatter, or bar plot.
    Plot {
        /// Plain-text label.
        label: Option<String>,
    },
    /// Ranked class-score bars (up to 100 classes).
    Scores {
        /// Plain-text label.
        label: Option<String>,
        /// Require scores in `[0,1]` summing to one, and display percentages.
        probabilities: bool,
    },
}
impl Default for Output {
    fn default() -> Self {
        Self::Number {
            label: Some("Result".into()),
            precision: 2,
        }
    }
}
impl Output {
    /// Define a numerical result and its decimal precision.
    pub fn number(label: impl Into<String>, precision: usize) -> Self {
        Self::Number {
            label: Some(label.into()),
            precision,
        }
    }
    /// Define a plot result.
    pub fn plot(label: impl Into<String>) -> Self {
        Self::Plot {
            label: Some(label.into()),
        }
    }
    /// Define ranked, nonnegative model scores without interpreting them as probabilities.
    pub fn scores(label: impl Into<String>) -> Self {
        Self::Scores {
            label: Some(label.into()),
            probabilities: false,
        }
    }
    /// Define normalized class probabilities supplied by the model.
    pub fn probabilities(label: impl Into<String>) -> Self {
        Self::Scores {
            label: Some(label.into()),
            probabilities: true,
        }
    }
    /// Define a plain-text result.
    pub fn text(label: impl Into<String>) -> Self {
        Self::Text {
            label: Some(label.into()),
        }
    }
    pub(crate) fn label(&self, index: usize) -> String {
        let (Self::Number { label, .. }
        | Self::Text { label }
        | Self::Plot { label }
        | Self::Scores { label, .. }) = self;
        label
            .clone()
            .unwrap_or_else(|| format!("Result {}", index + 1))
    }
}
