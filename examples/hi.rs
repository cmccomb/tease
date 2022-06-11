fn main() {
    let model = tease::train_linear_regression();
    tease::run(move |x| model.predict(&x).unwrap()).expect("Could not run");
}
