use tease::{Input, Teaser};

fn main() -> Result<(), tease::Error> {
    Teaser::default()
        .with_title("Addition")
        .with_description("Add two numbers together.")
        .with_inputs(vec![
            Input::number("First number", 0.0),
            Input::number("Second number", 0.0),
        ])
        .with_function(|x| x.iter().sum())
        .run()
}
