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

pub enum Input {
    Number(f64),
    Text(String),
    Slider {
        min: f64,
        max: f64,
        step: f64,
        initial_value: f64,
    },
}

pub fn run<F>(predictor: F, inputs: &[Input]) -> wry::Result<()>
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
                Input::Text(_) => "".to_string(),
                Input::Slider {
                    min,
                    max,
                    step,
                    initial_value,
                } => add_slider(idx, initial_value, max, min, step),
            }
        );
    }
    html = format!("{} {}", html, end());

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Tease")
        .build(&event_loop)?;

    let mut webview_settings = WebViewAttributes::default();
    webview_settings.devtools = true;
    let mut webview_builder = WebViewBuilder::new(window)?;
    webview_builder.webview = webview_settings;
    let _webview = webview_builder
        .with_html(html)?
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
        .build()?;

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

fn add_number(index: usize, initial_value: &f64) -> String {
    format!("\
    <label for=\"exampleInput{index}\" class=\"col-sm-2 col-form-label mt-3\"><i>x<sub>{index}</sub> = </i></label>\
    <div class=\"col-sm-10 mt-3\">\
        <input type=\"text\" class=\"form-control input\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\">\
    </div>")
}

fn add_text(index: usize, initial_value: &str) -> String {
    format!("\
    <label for=\"exampleInput{index}\" class=\"col-sm-2 col-form-label mt-3\"><i>x<sub>{index}</sub> = </i></label>\
    <div class=\"col-sm-10 mt-3\">\
        <input type=\"text\" class=\"form-control input\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\">\
    </div>")
}

fn add_slider(index: usize, initial_value: &f64, max: &f64, min: &f64, step: &f64) -> String {
    format!("\
    <label for=\"exampleInput{index}\" class=\"col-sm-2 col-form-label mt-3\"><i>x<sub>{index}</sub> = </i></label>\
    <div class=\"col-sm-10 mt-3 form-group\" style=\"display: flex\">\
            <input type=\"text\" class=\"form-control col-sm-2\" value=\"{initial_value}\" readonly>
            <input type=\"range\" class=\"form-control input col-sm-10\" min=\"{min}\" max=\"{max}\" step=\"{step}\" id=\"exampleInput{index}\" name=\"x{index}\" aria-describedby=\"input6\" placeholder=\"x{index}\" value=\"{initial_value}\" oninput=\"this.previousElementSibling.value = this.value\">\
    </div>")
}

fn beginning() -> String {
    "<html lang=\"en\">
        <head>
            <meta charset=\"utf-8\">
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1, shrink-to-fit=no\">
            <link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/bootstrap@4.3.1/dist/css/bootstrap.min.css\" integrity=\"sha384-ggOyR0iXCbMQv3Xipma34MD+dH/1fQ784/j6cY/iJTQUOhcWr7x9JvoRxT2MZw1T\" crossorigin=\"anonymous\">
            <script type=\"text/javascript\">
                function run_calculation() {
                    var numbers = [];
                    var classes = document.getElementsByClassName('input');
                    Array.from(classes).forEach((x, i) => numbers.push(Number(x.value)));
                    ipc.postMessage(numbers.toString());
                }
            </script>
        </head>
        <body>
            <div class=\"container\">
                <div class=\"row mt-3\">
                    <div class=\"col text-center\">
                        <form action=\"#\" method=\"POST\" onsubmit=\"run_calculation()\">
                            <div class=\"form-group row\" id=\"input-group\">".to_string()
}

fn end() -> String {
    "                        </div>         
        
                            <div class=\"form-group\" id=\"submit\">
                                <button type=\"submit\" class=\"btn btn-primary\">Submit</button>
                            </div>
                        
                            <div class=\"form-group row\" id=\"output-group\">
                                <label for=\"output\" class=\"col-sm-2 col-form-label\"><i>y = </i></label>
                                <div class=\"col-sm-10\">
                                    <input type=\"text\" class=\"form-control\" id=\"output\" name=\"output\" aria-describedby=\"output\" readonly>
                                </div>
                            </div>
                        </form>
                    </div>
                </div>  
            </div>
            <script src=\"https://code.jquery.com/jquery-3.3.1.slim.min.js\" integrity=\"sha384-q8i/X+965DzO0rT7abK41JStQIAqVgRVzpbzo5smXKp4YfRvH+8abtTE1Pi6jizo\" crossorigin=\"anonymous\"></script>
            <script src=\"https://cdn.jsdelivr.net/npm/popper.js@1.14.7/dist/umd/popper.min.js\" integrity=\"sha384-UO2eT0CpHqdSJQ6hJty5KVphtPhzWj9WO1clHTMGa3JDZwrnQq4sF86dIHNDz0W1\" crossorigin=\"anonymous\"></script>
            <script src=\"https://cdn.jsdelivr.net/npm/bootstrap@4.3.1/dist/js/bootstrap.min.js\" integrity=\"sha384-JjSmVgyd0p3pXB1rRibZUAYoIIy6OrQ6VrjIEaFf/nJGzIxFDsf4x0xIM+B07jRM\" crossorigin=\"anonymous\"></script>
        </body>
    </html>".to_string()
}
