use smartcore::{
    dataset::iris, linalg::basic::matrix::DenseMatrix, naive_bayes::gaussian::GaussianNB,
};
use tease::{Input, Output, Teaser, Value};

fn main() -> Result<(), tease::Error> {
    demo().run()
}

fn demo() -> Teaser {
    let labels = [
        "Sepal length (cm)",
        "Sepal width (cm)",
        "Petal length (cm)",
        "Petal width (cm)",
    ];
    let sample = [5.1, 3.5, 1.4, 0.2];
    Teaser::default()
        .with_title("SmartCore iris classification")
        .with_description("A Gaussian naive Bayes model fitted to the Iris dataset. Measurements are in centimeters.")
        .with_inputs(labels.into_iter().zip(sample).map(|(label, value)| Input::Number {
            label: Some(label.into()), initial_value: value, min: Some(0.1), max: Some(10.0),
        }).collect())
        .with_output(Output::text("Predicted species"))
        .with_predictor_factory(|| {
            let dataset = iris::load_dataset();
            let rows: Vec<Vec<f64>> = dataset.data.chunks(dataset.num_features)
                .map(|row| row.iter().map(|&value| f64::from(value)).collect()).collect();
            let x = DenseMatrix::from_2d_vec(&rows).map_err(|error| error.to_string())?;
            let y: Vec<u32> = dataset.target;
            let model = GaussianNB::fit(&x, &y, Default::default()).map_err(|error| error.to_string())?;
            Ok(move |values: &[Value]| {
                let row = values.iter().map(Value::as_number).collect::<Result<Vec<_>, _>>()?;
                let x = DenseMatrix::from_2d_vec(&vec![row]).map_err(|error| error.to_string())?;
                let prediction = model.predict(&x).map_err(|error| error.to_string())?;
                let label = match prediction[0] {
                    0 => "Iris setosa", 1 => "Iris versicolor", 2 => "Iris virginica",
                    _ => return Err("The model returned an unknown species.".into()),
                };
                Ok(vec![Value::Text(label.into())])
            })
        })
}

#[test]
fn documented_samples_produce_all_three_species() {
    let session = demo().prepare().unwrap();
    for (sample, species) in [
        ([5.1, 3.5, 1.4, 0.2], "Iris setosa"),
        ([6.0, 2.9, 4.5, 1.5], "Iris versicolor"),
        ([6.9, 3.1, 5.4, 2.1], "Iris virginica"),
    ] {
        assert_eq!(
            session.predict(&sample.map(Value::Number)).unwrap(),
            [Value::Text(species.into())]
        );
    }
}
