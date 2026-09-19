//! Structured data for plots, classification displays, and named scenarios.
use crate::Value;
use serde::{Deserialize, Serialize};

/// How to draw a series of numerical points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotKind {
    /// Connect points in their supplied order.
    Line,
    /// Draw individual points.
    Scatter,
    /// Draw bars from zero to each y value.
    Bar,
}
/// One labeled series in a plot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Series {
    /// Plain-text legend label.
    pub name: String,
    /// Pairs of x and y coordinates, in drawing order.
    pub points: Vec<[f64; 2]>,
}
/// A numerical plot, rendered locally without a chart service.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plot {
    /// Drawing style shared by the series.
    pub kind: PlotKind,
    /// Horizontal axis label, including any units.
    pub x_label: String,
    /// Vertical axis label, including any units.
    pub y_label: String,
    /// Up to 16 series and 10,000 total points.
    pub series: Vec<Series>,
}
impl Plot {
    pub(crate) fn validate(&self) -> bool {
        !self.series.is_empty()
            && self.series.len() <= 16
            && self.series.iter().map(|s| s.points.len()).sum::<usize>() <= 10_000
            && self.series.iter().all(|s| {
                !s.points.is_empty()
                    && s.points
                        .iter()
                        .flatten()
                        .all(|n| n.is_finite() && n.abs() <= 1e100)
            })
    }
}
/// A class label and its model-supplied score. Tease does not infer confidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClassScore {
    /// Plain-text class label, unique within a result.
    pub label: String,
    /// Finite nonnegative score, or a probability when configured as such.
    pub score: f64,
}
/// A named, validated set of model inputs. Loading it does not itself run the model.
#[derive(Clone, Debug)]
pub struct Preset {
    /// Display name for this scenario.
    pub name: String,
    /// Values in model-input order.
    pub inputs: Vec<Value>,
}
impl Preset {
    /// Define a named input scenario.
    pub fn new(name: impl Into<String>, inputs: Vec<Value>) -> Self {
        Self {
            name: name.into(),
            inputs,
        }
    }
}
