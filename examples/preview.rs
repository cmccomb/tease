//! A compact, runnable preview of Tease's controls, plots, and class probabilities.
use tease::{Choice, ClassScore, Input, Output, Plot, PlotKind, Series, Teaser, Value};

fn demo() -> Teaser {
    Teaser::default()
        .with_title("Tune a response curve")
        .with_description("Change the inputs to update the model output. Pause live prediction to use Predict manually.")
        .with_inputs(vec![
            Input::Number {
                label: Some("Gain".into()),
                initial_value: 1.0,
                min: Some(0.1),
                max: Some(5.0),
            }
            .with_unit("×")
            .with_help("Higher gain makes the transition steeper."),
            Input::Slider {
                label: Some("Offset".into()),
                min: -3.0,
                max: 3.0,
                step: 0.1,
                initial_value: 0.0,
            },
            Input::Dropdown {
                label: Some("Plot style".into()),
                options: vec![
                    Choice::text("Line", "line"),
                    Choice::text("Scatter", "scatter"),
                    Choice::text("Bar", "bar"),
                ],
                initial_value: 0,
            },
        ])
        .with_outputs(vec![
            Output::number("Response at x = 1", 4),
            Output::plot("Response curve"),
            Output::probabilities("Outcomes at x = 1"),
        ])
        .with_live_prediction(200)
        .with_predictor(|values| {
            let gain = values[0].as_number()?;
            let offset = values[1].as_number()?;
            let kind = match values[2].as_text()? {
                "scatter" => PlotKind::Scatter,
                "bar" => PlotKind::Bar,
                _ => PlotKind::Line,
            };
            let response = |x: f64| 1.0 / (1.0 + (-gain * x - offset).exp());
            let probability = response(1.0);
            let plot = Plot {
                kind,
                x_label: "Input x".into(),
                y_label: "Response probability".into(),
                series: {
                    let mut series = vec![Series {
                        name: "response".into(),
                        points: (0..=40)
                            .map(|i| {
                                let x = -3.0 + f64::from(i) * 0.15;
                                [x, response(x)]
                            })
                            .collect(),
                    }];
                    if !matches!(kind, PlotKind::Bar) {
                        series.push(Series {
                            name: "reference".into(),
                            points: vec![[-3.0, 0.5], [3.0, 0.5]],
                        });
                    }
                    series
                },
            };
            Ok(vec![
                Value::Number(probability),
                Value::Plot(plot),
                Value::Scores(vec![
                    ClassScore {
                        label: "Positive".into(),
                        score: probability,
                    },
                    ClassScore {
                        label: "Negative".into(),
                        score: 1.0 - probability,
                    },
                ]),
            ])
        })
}

fn main() -> Result<(), tease::Error> {
    demo().run()
}
