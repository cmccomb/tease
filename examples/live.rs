//! Bounded live inference: edit while the model is working, or pause automatic updates.
use std::{cell::Cell, rc::Rc, thread, time::Duration};
use tease::{Input, Output, Teaser, Value};
fn demo() -> Teaser {
    Teaser::default().with_title("Live predictions")
        .with_description("Change Value while a one-second prediction runs. Only the newest pending inputs are retained. Pause Live prediction to use the Predict button.")
        .with_inputs(vec![Input::Number{label:Some("Value".into()),initial_value:2.0,min:Some(-10.0),max:Some(10.0)}])
        .with_outputs(vec![Output::number("Squared value",0),Output::number("Model calls",0)])
        .with_live_prediction(200).with_comparison(true)
        .with_predictor_factory(|| {
            let count=Rc::new(Cell::new(0));
            Ok(move |values:&[Value]| {
                count.set(count.get()+1);thread::sleep(Duration::from_secs(1));
                let x=values[0].as_number()?;
                Ok(vec![Value::Number(x*x),Value::Number(f64::from(count.get()))])
            })
        })
}
fn main() -> Result<(), tease::Error> {
    demo().run()
}
#[test]
fn live_configuration_rejects_extreme_intervals() {
    for delay in [0, 49, 5001] {
        assert!(Teaser::default()
            .with_live_prediction(delay)
            .validate()
            .is_err());
    }
    for delay in [50, 200, 5000] {
        assert!(Teaser::default()
            .with_live_prediction(delay)
            .validate()
            .is_ok());
    }
}
