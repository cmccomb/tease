use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tease::{Choice, Error, Input, Output, Teaser, Value};

fn bounded() -> Input {
    Input::Number {
        label: Some("Temperature".into()),
        initial_value: 20.0,
        min: Some(0.0),
        max: Some(100.0),
    }
}
fn slider(min: f64, max: f64, step: f64, initial_value: f64) -> Input {
    Input::Slider {
        label: Some("Gain".into()),
        min,
        max,
        step,
        initial_value,
    }
}
fn dropdown() -> Input {
    Input::Dropdown {
        label: Some("Material".into()),
        initial_value: 0,
        options: vec![
            Choice::text("Steel", "steel"),
            Choice::text("Aluminum", "aluminum"),
        ],
    }
}

#[test]
fn default_and_both_numeric_callback_styles_work() {
    assert_eq!(
        Teaser::default()
            .prepare()
            .unwrap()
            .predict(&[Value::Number(7.0)])
            .unwrap(),
        [Value::Number(7.0)]
    );
    fn sum(values: Vec<f64>) -> f64 {
        values.iter().sum()
    }
    for demo in [
        Teaser::default()
            .with_inputs(vec![Input::default(); 2])
            .with_function(sum),
        Teaser::default()
            .with_inputs(vec![Input::default(); 2])
            .with_function(|values| values.iter().sum()),
    ] {
        let demo = demo.prepare().unwrap();
        assert_eq!(
            demo.predict(&[Value::Number(2.0), Value::Number(3.0)])
                .unwrap(),
            [Value::Number(5.0)]
        );
    }
}

#[test]
fn typed_predictor_accepts_categories_and_multiple_named_results() {
    let demo = Teaser::default()
        .with_inputs(vec![bounded(), dropdown()])
        .with_outputs(vec![
            Output::text("Material"),
            Output::number("Temperature", 2),
        ])
        .with_predictor(|values| {
            Ok(vec![
                Value::Text(values[1].as_text()?.into()),
                Value::Number(values[0].as_number()?),
            ])
        });
    let demo = demo.prepare().unwrap();
    let expected = [Value::Text("steel".into()), Value::Number(20.0)];
    assert_eq!(
        demo.predict(&[Value::Number(20.0), Value::Text("steel".into())])
            .unwrap(),
        expected
    );
}

#[test]
fn every_callback_setter_replaces_the_previous_one() {
    let demo = Teaser::default()
        .with_function(|_| 1.0)
        .with_predictor(|_| Ok(vec![Value::Number(2.0)]))
        .prepare()
        .unwrap();
    assert_eq!(
        demo.predict(&[Value::Number(0.0)]).unwrap(),
        [Value::Number(2.0)]
    );
    let demo = Teaser::default()
        .with_predictor(|_| Ok(vec![Value::Number(2.0)]))
        .with_function(|_| 3.0)
        .prepare()
        .unwrap();
    assert_eq!(
        demo.predict(&[Value::Number(0.0)]).unwrap(),
        [Value::Number(3.0)]
    );
}

#[test]
fn rejected_inputs_do_not_reach_the_model() {
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let demo = Teaser::default()
        .with_inputs(vec![bounded()])
        .with_function(move |values| {
            seen.fetch_add(1, Ordering::Relaxed);
            values[0]
        });
    let demo = demo.prepare().unwrap();
    for values in [
        vec![],
        vec![Value::Text("20".into())],
        vec![Value::Number(-1.0)],
        vec![Value::Number(101.0)],
        vec![Value::Number(f64::NAN)],
        vec![Value::Number(f64::INFINITY)],
        vec![Value::Number(f64::NEG_INFINITY)],
    ] {
        assert!(matches!(demo.predict(&values), Err(Error::Input { .. })));
    }
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    for boundary in [0.0, 100.0] {
        assert!(demo.predict(&[Value::Number(boundary)]).is_ok());
    }
    assert_eq!(calls.load(Ordering::Relaxed), 2);
}

#[test]
fn prediction_error_can_be_corrected_and_resubmitted() {
    let demo = Teaser::default().with_predictor(|values| {
        let divisor = values[0].as_number()?;
        if divisor == 0.0 {
            Err("Divisor must not be zero.".into())
        } else {
            Ok(vec![Value::Number(10.0 / divisor)])
        }
    });
    let demo = demo.prepare().unwrap();
    assert_eq!(
        demo.predict(&[Value::Number(0.0)]),
        Err(Error::Prediction("Divisor must not be zero.".into()))
    );
    assert_eq!(
        demo.predict(&[Value::Number(2.0)]).unwrap(),
        [Value::Number(5.0)]
    );
}

#[test]
fn wrong_count_type_and_nonfinite_results_are_rejected() {
    for result in [
        vec![],
        vec![Value::Number(1.0); 2],
        vec![Value::Text("1".into())],
        vec![Value::Number(f64::NAN)],
        vec![Value::Number(f64::INFINITY)],
    ] {
        let demo = Teaser::default()
            .with_predictor(move |_| Ok(result.clone()))
            .prepare()
            .unwrap();
        assert!(matches!(
            demo.predict(&[Value::Number(0.0)]),
            Err(Error::Output(_))
        ));
    }
    let demo = Teaser::default()
        .with_output(Output::text("Class"))
        .with_predictor(|_| Ok(vec![Value::Number(1.0)]))
        .prepare()
        .unwrap();
    assert!(matches!(
        demo.predict(&[Value::Number(0.0)]),
        Err(Error::Output(_))
    ));
}

#[test]
fn invalid_component_definitions_are_rejected_before_execution() {
    let cases = vec![
        Input::number("NaN", f64::NAN),
        Input::Number {
            label: None,
            initial_value: 0.0,
            min: Some(2.0),
            max: Some(1.0),
        },
        Input::Number {
            label: None,
            initial_value: 0.0,
            min: None,
            max: Some(f64::INFINITY),
        },
        Input::Number {
            label: None,
            initial_value: 3.0,
            min: Some(0.0),
            max: Some(1.0),
        },
        slider(0.0, 0.0, 1.0, 0.0),
        slider(2.0, 1.0, 1.0, 1.0),
        slider(0.0, 1.0, 0.0, 0.0),
        slider(0.0, 1.0, -1.0, 0.0),
        slider(0.0, 1.0, f64::NAN, 0.0),
        slider(0.0, 1.0, 0.1, 0.05),
        slider(0.0, 1.0, 0.1, 1.1),
        slider(-f64::MAX, f64::MAX, 1.0, 0.0),
        Input::Dropdown {
            label: None,
            options: vec![],
            initial_value: 0,
        },
        Input::Dropdown {
            label: None,
            options: vec![Choice::number("One", 1.0)],
            initial_value: 1,
        },
        Input::Dropdown {
            label: None,
            options: vec![Choice::number("Infinite", f64::INFINITY)],
            initial_value: 0,
        },
    ];
    for input in cases {
        let demo = Teaser::default().with_inputs(vec![input]);
        assert!(matches!(demo.validate(), Err(Error::Configuration(_))));
    }
    assert!(Teaser::default().with_inputs(vec![]).validate().is_err());
    assert!(Teaser::default().with_outputs(vec![]).validate().is_err());
    assert!(Teaser::default()
        .with_output(Output::number("Result", 16))
        .validate()
        .is_err());
}

#[test]
fn configuration_errors_are_returned_before_opening_native_window() {
    assert!(matches!(
        Teaser::default().with_inputs(vec![]).run(),
        Err(Error::Configuration(_))
    ));
}

#[test]
fn numeric_adapter_rejects_text_inputs_and_multiple_outputs() {
    assert!(Teaser::default()
        .with_inputs(vec![dropdown()])
        .validate()
        .is_err());
    assert!(Teaser::default()
        .with_output(Output::text("Class"))
        .validate()
        .is_err());
    assert!(Teaser::default()
        .with_outputs(vec![Output::default(); 2])
        .validate()
        .is_err());
    let demo = Teaser::default()
        .with_inputs(vec![Input::Dropdown {
            label: None,
            initial_value: 0,
            options: vec![Choice::number("Double", 2.0)],
        }])
        .with_function(|values| values[0] * 2.0)
        .prepare()
        .unwrap();
    assert_eq!(
        demo.predict(&[Value::Number(2.0)]).unwrap(),
        [Value::Number(4.0)]
    );
}

#[test]
fn decimal_slider_steps_are_checked_without_rejecting_roundoff() {
    let demo = Teaser::default()
        .with_inputs(vec![slider(0.1, 1.0, 0.1, 0.3)])
        .prepare()
        .unwrap();
    for value in [0.1, 0.3, 0.1 + 0.2, 1.0] {
        assert!(demo.predict(&[Value::Number(value)]).is_ok(), "{value}");
    }
    for value in [0.0, 0.25, 1.1] {
        assert!(matches!(
            demo.predict(&[Value::Number(value)]),
            Err(Error::Input { field: Some(0), .. })
        ));
    }
}

#[test]
fn arbitrary_dropdown_values_are_rejected() {
    let demo = Teaser::default()
        .with_inputs(vec![dropdown()])
        .with_output(Output::text("Material"))
        .with_predictor(|values| Ok(values.to_vec()))
        .prepare()
        .unwrap();
    assert!(matches!(
        demo.predict(&[Value::Text("plastic".into())]),
        Err(Error::Input { field: Some(0), .. })
    ));
}
