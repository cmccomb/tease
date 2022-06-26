use smartcore::linalg::naive::dense_matrix::DenseMatrix;
use smartcore::linear::linear_regression::{LinearRegression, LinearRegressionParameters};
use tease::{Input, Output, Teaser};

fn main() {
    let model = train_linear_regression();
    Teaser::default()
        .with_title("Linear Regression".to_string())
        .with_description(
            "This demonstration runs a <a href=\"https://smartcorelib.org/\"> Smartcore</a> model and shows the results in real time."
                .to_string(),
        )
        .with_output(



            Output::Number{
            label: Some("Total Employment".to_string()), precision: 1 })
        .with_inputs(vec![
            Input::Number {
                label: Some("GNP Deflator".to_string()),
                initial_value: 83.0,
            },
            Input::Number {
                label: Some("GNP".to_string()),
                initial_value: 234.289,
            },
            Input::Number {
                label: Some("Unemployment".to_string()),
                initial_value: 235.6,
            },
            Input::Slider {
                label: Some("Size of Armed Forces".to_string()),
                min: 140.0,
                max: 300.0,
                step: 0.1,
                initial_value: 159.0,
            },
            Input::Number {
                label: Some("Population".to_string()),
                initial_value: 107.608,
            },
            Input::Dropdown {
                label: Some("Year".to_string()),
                options: (1940..2022).step_by(1).map(f64::from).collect(),
                initial_value: 0,
            },
        ])
        .with_function(move |x| {
            model
                .predict(&DenseMatrix::from_2d_vec(&vec![x; 1]))
                .unwrap()[0]
        })
        .run();
}

pub fn train_linear_regression() -> LinearRegression<f64, DenseMatrix<f64>> {
    // Longley dataset (https://www.statsmodels.org/stable/datasets/generated/longley.html)
    let x = DenseMatrix::from_2d_array(&[
        &[83.0, 234.289, 235.6, 159.0, 107.608, 1947.],
        &[88.5, 259.426, 232.5, 145.6, 108.632, 1948.],
        &[88.2, 258.054, 368.2, 161.6, 109.773, 1949.],
        &[89.5, 284.599, 335.1, 165.0, 110.929, 1950.],
        &[96.2, 328.975, 209.9, 309.9, 112.075, 1951.],
        &[98.1, 346.999, 193.2, 359.4, 113.270, 1952.],
        &[99.0, 365.385, 187.0, 354.7, 115.094, 1953.],
        &[100.0, 363.112, 357.8, 335.0, 116.219, 1954.],
        &[101.2, 397.469, 290.4, 304.8, 117.388, 1955.],
        &[104.6, 419.180, 282.2, 285.7, 118.734, 1956.],
        &[108.4, 442.769, 293.6, 279.8, 120.445, 1957.],
        &[110.8, 444.546, 468.1, 263.7, 121.950, 1958.],
        &[112.6, 482.704, 381.3, 255.2, 123.366, 1959.],
        &[114.2, 502.601, 393.1, 251.4, 125.368, 1960.],
        &[115.7, 518.173, 480.6, 257.2, 127.852, 1961.],
        &[116.9, 554.894, 400.7, 282.7, 130.081, 1962.],
    ]);

    let y: Vec<f64> = vec![
        60.323, 61.122, 60.171, 61.187, 63.221, 63.639, 64.989, 63.761, 66.019, 67.857, 68.169,
        66.513, 68.655, 69.564, 69.331, 70.551,
    ];

    let lr = LinearRegression::fit(&x, &y, LinearRegressionParameters::default()).unwrap();

    lr
}
