use tease::{Choice, Input, Output, Teaser, Value};

fn main() -> Result<(), tease::Error> {
    demo().run()
}

fn demo() -> Teaser {
    Teaser::default()
        .with_title("Threshold classifier")
        .with_description(
            "A small rule-based example of labeled choices and multiple typed results.",
        )
        .with_inputs(vec![
            Input::Number {
                label: Some("Score".into()),
                initial_value: 0.7,
                min: Some(0.0),
                max: Some(1.0),
            },
            Input::Dropdown {
                label: Some("Threshold".into()),
                initial_value: 0,
                options: vec![
                    Choice::text("Standard (0.5)", "standard"),
                    Choice::text("Strict (0.8)", "strict"),
                ],
            },
        ])
        .with_outputs(vec![
            Output::text("Class"),
            Output::number("Margin above threshold", 2),
        ])
        .with_predictor(|values| {
            let score = values[0].as_number()?;
            let threshold = match values[1].as_text()? {
                "standard" => 0.5,
                "strict" => 0.8,
                _ => return Err("Unknown threshold.".into()),
            };
            let class = if score >= threshold {
                "Positive"
            } else {
                "Negative"
            };
            Ok(vec![
                Value::Text(class.into()),
                Value::Number(score - threshold),
            ])
        })
}

#[test]
fn choices_change_class_and_margin() {
    let demo = demo().prepare().unwrap();
    let results = demo
        .predict(&[Value::Number(0.7), Value::Text("standard".into())])
        .unwrap();
    assert_eq!(results[0], Value::Text("Positive".into()));
    assert!((results[1].as_number().unwrap() - 0.2).abs() < 1e-12);
    let results = demo
        .predict(&[Value::Number(0.7), Value::Text("strict".into())])
        .unwrap();
    assert_eq!(results[0], Value::Text("Negative".into()));
    assert!(demo
        .predict(&[Value::Number(1.1), Value::Text("strict".into())])
        .is_err());
}
