use smartcore::linalg::BaseMatrix;
use tease::{Input, Teaser};

fn main() {
    Teaser::run(
        move |x| x.iter().sum(),
        vec![Input::Number(0.0), Input::Number(0.0)],
    );
}
