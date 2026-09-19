use tease::{Input, Output, Teaser, Value};

fn main() -> Result<(), tease::Error> {
    demo().run()
}

fn demo() -> Teaser {
    Teaser::default()
        .with_title("Division and error recovery")
        .with_description("Try a blank field, a negative divisor, or zero, then correct the input and predict again.")
        .with_inputs(vec![
            Input::number("Numerator", 10.0),
            Input::Number { label: Some("Divisor".into()), initial_value: 2.0, min: Some(0.0), max: None },
        ])
        .with_outputs(vec![Output::number("Quotient", 3), Output::text("Status")])
        .with_predictor(|values| {
            let divisor = values[1].as_number()?;
            if divisor == 0.0 { return Err("Divisor must not be zero.".into()); }
            Ok(vec![Value::Number(values[0].as_number()? / divisor), Value::Text("Calculated".into())])
        })
}

#[test]
fn divide_by_zero_is_recoverable() {
    let demo = demo().prepare().unwrap();
    assert!(matches!(
        demo.predict(&[Value::Number(10.0), Value::Number(0.0)]),
        Err(tease::Error::Prediction(_))
    ));
    assert_eq!(
        demo.predict(&[Value::Number(10.0), Value::Number(2.0)])
            .unwrap(),
        [Value::Number(5.0), Value::Text("Calculated".into())]
    );
}
