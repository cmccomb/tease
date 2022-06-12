use tease::{Input, Teaser};

fn main() {
    Teaser::default()
        .with_title("Addition".to_string())
        .with_description("This is for adding things together.".to_string())
        .with_inputs(vec![Input::Number(0.0); 2])
        .run(move |x| x.iter().sum());
}
