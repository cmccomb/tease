use smartcore::linalg::BaseMatrix;
use tease::{run, Input};

fn main() {
    run(
        move |x| x.iter().sum(),
        // move |x| {
        //     let mut y = vec![0.0_f64; 0];
        //     for i in 0..x.shape().0 {
        //         y.push(x.get_row_as_vec(i).iter().sum())
        //     }
        //     y
        // },
        &[Input::Number(0.0), Input::Number(0.0)],
    );
}
