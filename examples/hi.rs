use tease::{run, train_linear_regression, Input};

fn main() {
    let model = train_linear_regression();
    run(
        move |x| model.predict(&x).unwrap(),
        &[
            Input::Number(234.289),
            Input::Number(235.6),
            Input::Number(159.0),
            Input::Number(107.608),
            Input::Slider {
                min: 1940.0,
                max: 1970.0,
                step: 0.33,
                initial_value: 1947.0,
            },
            Input::Number(60.323),
        ],
    )
    .expect("Could not run");
}
