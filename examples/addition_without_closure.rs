use tease::{Input, Teaser};

fn add(x: Vec<f64>) -> f64 {
    x.iter().sum()
}

fn main() {
    Teaser::default()
        .with_title("Addition".to_string())
        .with_description("This is for adding things together.".to_string())
        .with_inputs(vec![Input::default(); 2])
        .with_function(add)
        .run();
}
