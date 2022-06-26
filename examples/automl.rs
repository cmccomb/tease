use automl::{Settings, SupervisedModel};
use smartcore::dataset::diabetes;
use tease::{Input, Teaser};

fn main() {
    let mut model = SupervisedModel::new(diabetes::load_dataset(), Settings::default_regression());
    model.train();

    Teaser::default()
        .with_function(move |x| {
            let y: Vec<f32> = x.iter().map(|el| *el as f32).collect();
            model.predict(vec![y; 1])[0] as f64
        })
        .with_inputs(vec![Input::default(); 10])
        .run();
}
