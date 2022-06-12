use smartcore::{
    linalg::naive::dense_matrix::DenseMatrix,
    linear::{
        lasso::{Lasso, LassoParameters},
        linear_regression::{
            LinearRegression, LinearRegressionParameters, LinearRegressionSolverName,
        },
    },
};

use std::{cell::RefCell, collections::HashMap};

use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    webview::{WebView, WebViewAttributes, WebViewBuilder},
};

mod html_chunks;
use html_chunks::{add_dropdown, add_number, add_slider, add_text, beginning, end};

pub enum Input {
    Number(f64),
    Text(String),
    Slider {
        min: f64,
        max: f64,
        step: f64,
        initial_value: f64,
    },
    Dropdown {
        options: Vec<f64>,
        initial_value: usize,
    },
}

pub fn run<F>(predictor: F, inputs: &[Input])
//-> wry::Result<()>
where
    F: 'static + Fn(DenseMatrix<f64>) -> Vec<f64>,
{
    thread_local! {
        static WEBVIEW: RefCell<HashMap<usize, WebView>> = RefCell::new(HashMap::new());
    }

    let mut html = beginning();
    for (idx, input) in inputs.iter().enumerate() {
        html = format!(
            "{} {}",
            html,
            match input {
                Input::Number(iv) => add_number(idx, iv),
                Input::Text(iv) => add_text(idx, iv),
                Input::Slider {
                    min,
                    max,
                    step,
                    initial_value,
                } => add_slider(idx, initial_value, max, min, step),
                Input::Dropdown {
                    initial_value,
                    options,
                } => add_dropdown(idx, initial_value, options),
            }
        );
    }
    html = format!("{} {}", html, end());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Tease")
        .build(&event_loop)
        .unwrap();

    let mut webview_settings = WebViewAttributes::default();
    webview_settings.devtools = true;
    let mut webview_builder = WebViewBuilder::new(window).unwrap();
    webview_builder.webview = webview_settings;
    let _webview = webview_builder
        .with_html(html)
        .unwrap()
        .with_ipc_handler(move |_window: &Window, req: String| {
            let number_strings = req.split(",");
            let mut inputs = vec![0.0; 0];
            for number in number_strings {
                inputs.push(number.parse().unwrap());
            }

            let x = DenseMatrix::from_2d_array(&[&inputs]);
            let y = predictor(x);
            // webview
            println!("{}", y[0]);
            WEBVIEW
                .with(|webview| {
                    let webview = webview.borrow();
                    let my_webview = webview.get(&0).unwrap();
                    my_webview.evaluate_script(&*format!(
                        "document.getElementById('output').value = {}",
                        y[0]
                    ))
                })
                .expect("TODO: panic message");
        })
        .build()
        .unwrap();

    WEBVIEW.with(|wv| {
        let mut hash = HashMap::new();
        hash.insert(0_usize, _webview);
        wv.replace(hash);
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Wry application started!"),
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::ExitWithCode(0),
            _ => {}
        }
    });
}

pub fn train_linear_regression() -> LinearRegression<f64, DenseMatrix<f64>> {
    // Longley dataset (https://www.statsmodels.org/stable/datasets/generated/longley.html)
    let x = DenseMatrix::from_2d_array(&[
        &[234.289, 235.6, 159.0, 107.608, 1947., 60.323],
        &[259.426, 232.5, 145.6, 108.632, 1948., 61.122],
        &[258.054, 368.2, 161.6, 109.773, 1949., 60.171],
        &[284.599, 335.1, 165.0, 110.929, 1950., 61.187],
        &[328.975, 209.9, 309.9, 112.075, 1951., 63.221],
        &[346.999, 193.2, 359.4, 113.270, 1952., 63.639],
        &[365.385, 187.0, 354.7, 115.094, 1953., 64.989],
        &[363.112, 357.8, 335.0, 116.219, 1954., 63.761],
        &[397.469, 290.4, 304.8, 117.388, 1955., 66.019],
        &[419.180, 282.2, 285.7, 118.734, 1956., 67.857],
        &[442.769, 293.6, 279.8, 120.445, 1957., 68.169],
        &[444.546, 468.1, 263.7, 121.950, 1958., 66.513],
        &[482.704, 381.3, 255.2, 123.366, 1959., 68.655],
        &[502.601, 393.1, 251.4, 125.368, 1960., 69.564],
        &[518.173, 480.6, 257.2, 127.852, 1961., 69.331],
        &[554.894, 400.7, 282.7, 130.081, 1962., 70.551],
    ]);

    let y: Vec<f64> = vec![
        83.0, 88.5, 88.2, 89.5, 96.2, 98.1, 99.0, 100.0, 101.2, 104.6, 108.4, 110.8, 112.6, 114.2,
        115.7, 116.9,
    ];

    let lr = LinearRegression::fit(
        &x,
        &y,
        LinearRegressionParameters::default().with_solver(LinearRegressionSolverName::QR),
    )
    .unwrap();

    lr
}

pub fn train_lasso_regression() -> Lasso<f64, DenseMatrix<f64>> {
    // Longley dataset (https://www.statsmodels.org/stable/datasets/generated/longley.html)
    let x = DenseMatrix::from_2d_array(&[
        &[234.289, 235.6, 159.0, 107.608, 1947., 60.323],
        &[259.426, 232.5, 145.6, 108.632, 1948., 61.122],
        &[258.054, 368.2, 161.6, 109.773, 1949., 60.171],
        &[284.599, 335.1, 165.0, 110.929, 1950., 61.187],
        &[328.975, 209.9, 309.9, 112.075, 1951., 63.221],
        &[346.999, 193.2, 359.4, 113.270, 1952., 63.639],
        &[365.385, 187.0, 354.7, 115.094, 1953., 64.989],
        &[363.112, 357.8, 335.0, 116.219, 1954., 63.761],
        &[397.469, 290.4, 304.8, 117.388, 1955., 66.019],
        &[419.180, 282.2, 285.7, 118.734, 1956., 67.857],
        &[442.769, 293.6, 279.8, 120.445, 1957., 68.169],
        &[444.546, 468.1, 263.7, 121.950, 1958., 66.513],
        &[482.704, 381.3, 255.2, 123.366, 1959., 68.655],
        &[502.601, 393.1, 251.4, 125.368, 1960., 69.564],
        &[518.173, 480.6, 257.2, 127.852, 1961., 69.331],
        &[554.894, 400.7, 282.7, 130.081, 1962., 70.551],
    ]);

    let y: Vec<f64> = vec![
        83.0, 88.5, 88.2, 89.5, 96.2, 98.1, 99.0, 100.0, 101.2, 104.6, 108.4, 110.8, 112.6, 114.2,
        115.7, 116.9,
    ];

    let lr = Lasso::fit(&x, &y, LassoParameters::default()).unwrap();

    lr
}
