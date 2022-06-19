use tease::{Input, Teaser};

fn main() {
    Teaser::default()
        .with_title("Addition".to_string())
        .with_description("This is for adding things together.".to_string())
        .with_inputs(vec![
            Input::Number {
                label: None,
                initial_value: 0.0
            };
            2
        ])
        .with_function(move |x| x.iter().sum())
        .run();
}
