use crate::Error;

/// A runtime model value, distinct from a component's display configuration.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Value {
    /// A finite numerical value.
    Number(f64),
    /// A text value, such as a category or class label.
    Text(String),
}
impl Value {
    /// Read a numerical value or return a callback-friendly error message.
    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            Self::Number(value) => Ok(*value),
            Self::Text(_) => Err("Expected a numerical value.".into()),
        }
    }
    /// Read a text value or return a callback-friendly error message.
    pub fn as_text(&self) -> Result<&str, String> {
        match self {
            Self::Text(value) => Ok(value),
            Self::Number(_) => Err("Expected a text value.".into()),
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
    pub(crate) fn label(&self, index: usize) -> String {
        let (Self::Number { label, .. }
        | Self::Slider { label, .. }
        | Self::Dropdown { label, .. }) = self;
        label
            .clone()
            .unwrap_or_else(|| format!("Input {}", index + 1))
    }
    pub(crate) fn is_numeric(&self) -> bool {
        match self {
            Self::Dropdown { options, .. } => options
                .iter()
                .all(|choice| matches!(choice.value, Value::Number(_))),
            _ => true,
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
        let initial = match self {
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
                options[*initial_value].value.clone()
            }
        };
        self.validate_value(index, &initial)
            .map_err(|error| invalid(&error.to_string()))
    }
    pub(crate) fn validate_value(&self, index: usize, value: &Value) -> Result<(), Error> {
        let invalid = |message: &str| self.error(index, message);
        if matches!(value, Value::Number(n) if !n.is_finite()) {
            return Err(invalid("Enter a finite number."));
        }
        match self {
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
        }
        Ok(())
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
    /// Define a plain-text result.
    pub fn text(label: impl Into<String>) -> Self {
        Self::Text {
            label: Some(label.into()),
        }
    }
    pub(crate) fn label(&self, index: usize) -> String {
        let (Self::Number { label, .. } | Self::Text { label }) = self;
        label
            .clone()
            .unwrap_or_else(|| format!("Result {}", index + 1))
    }
}
