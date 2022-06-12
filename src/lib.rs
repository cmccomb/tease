use smartcore::linalg::naive::dense_matrix::DenseMatrix;

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
    F: 'static + Fn(Vec<f64>) -> f64,
    // F: 'static + Fn(DenseMatrix<f64>) -> Vec<f64>,
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

            // let x = DenseMatrix::from_2d_array(&[&inputs]);
            // let y = predictor(x);
            let y = predictor(inputs);
            WEBVIEW
                .with(|webview| {
                    let webview = webview.borrow();
                    let my_webview = webview.get(&0).unwrap();
                    my_webview.evaluate_script(&*format!(
                        "document.getElementById('output').value = {}",
                        y
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
