use tease::{Input, Teaser};

fn add(x: Vec<f64>) -> f64 {
    x.iter().sum()
}

fn main() -> Result<(), tease::Error> {
    Teaser::default()
        .with_title("Addition")
        .with_inputs(vec![Input::default(); 2])
        .with_function(add)
        .run()
}
