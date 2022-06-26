use tease::{Input, Teaser};

fn main() {
    Teaser::default()
        .with_title("Addition".to_string())
        .with_description("This is for adding things together.".to_string())
        .with_inputs(vec![Input::default(); 2])
        .with_function(|x: Vec<f32>| x.iter().sum())
        .run();
}
