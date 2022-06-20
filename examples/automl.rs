use automl;
use smartcore;
use tease;
use tease::Input;

fn main() {
    let mut model = automl::SupervisedModel::new(
        smartcore::dataset::diabetes::load_dataset(),
        automl::Settings::default_regression(),
    );

    model.train();
    tease::Teaser::default()
        .with_function(move |x| {
            let y: Vec<f32> = x.iter().map(|el| *el as f32).collect();
            model.predict(vec![y; 1])[0] as f64
        })
        .with_inputs(vec![Input::default(); 10])
        .run();
}
