use crate::{Error, Input, Output, Session, Value};
use serde::{Deserialize, Serialize};

// Keep raw strings until Rust validates them: JavaScript Number("") is zero.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    pub(crate) inputs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Response {
    Success {
        values: Vec<serde_json::Value>,
    },
    Error {
        field: Option<usize>,
        message: String,
    },
}

impl Response {
    pub(crate) fn script(&self, request_id: Option<u64>, fatal: bool) -> String {
        // Only strings, indices, and tags are serialized; no raw model JavaScript.
        #[derive(Serialize)]
        struct Reply<'a> {
            request_id: Option<u64>,
            fatal: bool,
            #[serde(flatten)]
            response: &'a Response,
        }
        let json = serde_json::to_string(&Reply {
            request_id,
            fatal,
            response: self,
        })
        .expect("response contains only serializable values");
        format!("window.tease.receive({json});")
    }
}

impl Session {
    fn parse_request(&self, request: Request) -> Result<Vec<Value>, Error> {
        let invalid = || Error::Input {
            field: None,
            message: "Invalid request. Submit all fields again.".into(),
        };
        if request.inputs.len() != self.inputs.len() {
            return Err(invalid());
        }
        self.inputs
            .iter()
            .zip(request.inputs)
            .enumerate()
            .map(|(index, (input, raw))| {
                let text = raw.trim();
                if text.is_empty() && !matches!(input.base(), Input::Text { .. }) {
                    return Err(input.error(index, "A value is required."));
                }
                let value = match input.base() {
                    Input::Number { .. } | Input::Slider { .. } => Value::Number(
                        text.parse::<f64>()
                            .map_err(|_| input.error(index, "Enter a number."))?,
                    ),
                    Input::Dropdown { options, .. } => {
                        let choice = text.parse::<usize>().ok().and_then(|i| options.get(i));
                        choice
                            .ok_or_else(|| {
                                input.error(index, "Choose one of the available options.")
                            })?
                            .value
                            .clone()
                    }
                    Input::Text { .. } => Value::Text(raw),
                    Input::Checkbox { .. } => Value::Bool(
                        text.parse::<bool>()
                            .map_err(|_| input.error(index, "Choose a checkbox value."))?,
                    ),
                    Input::MultiSelect { options, .. } => {
                        let indices: Vec<usize> = serde_json::from_str(text)
                            .map_err(|_| input.error(index, "Choose values from the list."))?;
                        Value::List(
                            indices
                                .into_iter()
                                .map(|i| {
                                    options.get(i).map(|c| c.value.clone()).ok_or_else(|| {
                                        input.error(index, "Choose available options.")
                                    })
                                })
                                .collect::<Result<_, _>>()?,
                        )
                    }
                    Input::Described { .. } => unreachable!(),
                };
                input.validate_value(index, &value)?;
                Ok(value)
            })
            .collect()
    }

    #[cfg(test)]
    fn handle_message(&self, message: &str) -> Response {
        match serde_json::from_str(message) {
            Ok(request) => self.handle_request(request),
            Err(_) => Response::Error {
                field: None,
                message: "Invalid request.".into(),
            },
        }
    }

    pub(crate) fn handle_request(&self, request: Request) -> Response {
        match self
            .parse_request(request)
            .and_then(|values| self.predict(&values))
        {
            Ok(values) => Response::Success {
                values: self
                    .outputs
                    .iter()
                    .zip(values)
                    .map(|(output, value)| match (output, value) {
                        (Output::Number { precision, .. }, Value::Number(n)) => {
                            serde_json::json!(format!("{n:.precision$}"))
                        }
                        (_, Value::Text(text)) => serde_json::json!(text),
                        // predict() already checks output types.
                        (_, Value::Number(n)) => serde_json::json!(n.to_string()),
                        (Output::Plot { .. }, Value::Plot(plot)) => serde_json::json!({"kind":"plot", "plot":plot}),
                        (Output::Scores { probabilities, .. }, Value::Scores(scores)) => serde_json::json!({"kind":"scores", "probabilities":probabilities, "scores":scores}),
                        _ => unreachable!("validated output"),
                    })
                    .collect(),
            },
            Err(error) => {
                let field = match &error {
                    Error::Input { field, .. } => *field,
                    _ => None,
                };
                Response::Error {
                    field,
                    message: error.to_string(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Choice, Teaser};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn bad_requests_never_call_model_and_next_submission_succeeds() {
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = calls.clone();
        let demo = Teaser::default().with_function(move |x| {
            seen.fetch_add(1, Ordering::Relaxed);
            x[0]
        });
        let demo = demo.prepare().unwrap();
        for message in [
            "not json",
            "{}",
            r#"{"inputs":[]}"#,
            r#"{"inputs":[1]}"#,
            r#"{"inputs":[null]}"#,
            r#"{"inputs":["1"],"extra":1}"#,
            r#"{"inputs":[" "]}"#,
            r#"{"inputs":["oops"]}"#,
            r#"{"inputs":["NaN"]}"#,
            r#"{"inputs":["inf"]}"#,
            r#"{"inputs":["1e999"]}"#,
            r#"{"inputs":["1","2"]}"#,
        ] {
            assert!(
                matches!(demo.handle_message(message), Response::Error { .. }),
                "{message}"
            );
        }
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        assert!(
            matches!(demo.handle_message(r#"{"inputs":[" 2.5 "]}"#), Response::Success { values } if values == ["2.50"])
        );
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn selection_is_resolved_by_index_and_errors_target_the_field() {
        let demo = Teaser::default()
            .with_inputs(vec![Input::Dropdown {
                label: Some("Material".into()),
                initial_value: 0,
                options: vec![Choice::text("Steel", "steel"), Choice::text("Wood", "wood")],
            }])
            .with_output(Output::text("Material"))
            .with_predictor(|values| Ok(values.to_vec()));
        let demo = demo.prepare().unwrap();
        for selection in ["-1", "2", "0.5", "steel"] {
            let request = serde_json::json!({"inputs": [selection]}).to_string();
            assert!(matches!(
                demo.handle_message(&request),
                Response::Error { field: Some(0), .. }
            ));
        }
        assert!(
            matches!(demo.handle_message(r#"{"inputs":["1"]}"#), Response::Success { values } if values == ["wood"])
        );
    }

    #[test]
    fn model_errors_and_quoted_results_are_serialized_safely() {
        let demo = Teaser::default()
            .with_output(Output::text("Text"))
            .with_predictor(|values| {
                if values[0].as_number()? < 0.0 {
                    Err("Try a positive value.".into())
                } else {
                    Ok(vec![Value::Text(
                        "\"; window.pwned=true; // </script>\n&".into(),
                    )])
                }
            });
        let demo = demo.prepare().unwrap();
        assert!(
            matches!(demo.handle_message(r#"{"inputs":["-1"]}"#), Response::Error { field: None, message } if message.contains("Prediction failed"))
        );
        let result = demo.handle_message(r#"{"inputs":["1"]}"#);
        let script = result.script(Some(1), false);
        let json = script
            .strip_prefix("window.tease.receive(")
            .unwrap()
            .strip_suffix(");")
            .unwrap();
        let decoded: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(
            decoded["values"][0],
            "\"; window.pwned=true; // </script>\n&"
        );
    }
}
