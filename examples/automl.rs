use automl::{DenseMatrix, RegressionAlgorithm, RegressionModel, RegressionSettings};
use smartcore::dataset::diabetes;
use tease::{Input, Output, Teaser, Value};

fn main() -> Result<(), tease::Error> {
    demo().run()
}

fn demo() -> Teaser {
    let dataset = diabetes::load_dataset();
    let rows: Vec<Vec<f64>> = dataset
        .data
        .chunks(dataset.num_features)
        .map(|row| row.iter().map(|&value| f64::from(value)).collect())
        .collect();
    let defaults = rows[0].clone();
    Teaser::default()
        .with_title("AutoML diabetes regression")
        .with_description(
            "A demonstration using standardized research data; not a clinical prediction tool.",
        )
        .with_inputs(
            dataset
                .feature_names
                .iter()
                .zip(defaults)
                .map(|(label, value)| Input::number(label, value))
                .collect(),
        )
        .with_output(Output::number("Predicted progression", 2))
        .with_predictor_factory(move || {
            let x = DenseMatrix::from_2d_vec(&rows).map_err(|error| error.to_string())?;
            let y: Vec<f64> = dataset.target.into_iter().map(f64::from).collect();
            // Keep this demo's startup bounded and deterministic.
            let settings = RegressionSettings::default()
                .only(&RegressionAlgorithm::default_linear())
                .with_number_of_folds(3)
                .shuffle_data(false);
            let mut model = RegressionModel::new(x, y, settings);
            model.train().map_err(|error| error.to_string())?;
            Ok(move |values: &[Value]| {
                let row = values
                    .iter()
                    .map(Value::as_number)
                    .collect::<Result<Vec<_>, _>>()?;
                let x = DenseMatrix::from_2d_vec(&vec![row]).map_err(|error| error.to_string())?;
                let prediction = model.predict(x).map_err(|error| error.to_string())?;
                Ok(vec![Value::Number(prediction[0])])
            })
        })
}

#[test]
fn published_automl_model_predicts_through_tease() {
    let dataset = diabetes::load_dataset();
    let input: Vec<_> = dataset.data[..dataset.num_features]
        .iter()
        .map(|&value| Value::Number(f64::from(value)))
        .collect();
    let prediction = demo().prepare().unwrap().predict(&input).unwrap()[0]
        .as_number()
        .unwrap();
    assert!(prediction.is_finite());
    assert!((prediction - 206.117_070).abs() < 0.001, "{prediction}");
}
