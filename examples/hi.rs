fn main() {
    let model = tease::train_linear_regression();
    tease::run(
        move |x| model.predict(&x).unwrap(),
        &[tease::Input::Number(0.3)],
    )
    .expect("Could not run");
}
