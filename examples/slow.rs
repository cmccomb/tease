use std::{thread, time::Duration};
use tease::{Input, Output, Teaser, Value};

fn main() -> Result<(), tease::Error> {
    demo().run()
}

fn demo() -> Teaser {
    Teaser::default()
        .with_title("Slow model and responsive UI")
        .with_description("Preparation takes half a second. Prediction waits for the selected delay. Move or close the window while it runs. Negative values demonstrate a recoverable model error.")
        .with_inputs(vec![
            Input::number("Value to square", 3.0),
            Input::Slider { label: Some("Delay (seconds)".into()), min: 0.0, max: 5.0, step: 1.0, initial_value: 3.0 },
        ])
        .with_output(Output::number("Squared value", 2))
        .with_predictor_factory(|| {
            thread::sleep(Duration::from_millis(500));
            Ok(move |values: &[Value]| {
                let value = values[0].as_number()?;
                thread::sleep(Duration::from_secs_f64(values[1].as_number()?));
                if value < 0.0 { return Err("Use a non-negative value in this demonstration.".into()); }
                Ok(vec![Value::Number(value * value)])
            })
        })
}

#[test]
fn slow_example_can_recover_from_model_errors() {
    let session = demo().prepare().unwrap();
    assert!(matches!(
        session.predict(&[Value::Number(-1.0), Value::Number(0.0)]),
        Err(tease::Error::Prediction(_))
    ));
    assert_eq!(
        session
            .predict(&[Value::Number(3.0), Value::Number(0.0)])
            .unwrap(),
        [Value::Number(9.0)]
    );
}
