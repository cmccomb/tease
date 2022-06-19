#![warn(clippy::all)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(clippy::missing_docs_in_private_items)]
#![doc = include_str!("../README.md")]

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

#[derive(Clone)]
/// Types of inputs for the model
pub enum Input {
    /// A numerical input
    Number {
        /// Label to be shown at left. If value is `None`, a default of the form $x_N$ will be shown.
        label: Option<String>,
        /// Initial value to shown in the textbox
        initial_value: f64,
    },
    /// A text input
    Text {
        /// Label to be shown at left. If value is `None`, a default of the form $x_N$ will be shown.
        label: Option<String>,
        /// Initial value to shown in the textbox
        initial_value: String,
    },
    /// A slider input
    Slider {
        /// Label to be shown at left. If value is `None`, a default of the form $x_N$ will be shown.
        label: Option<String>,
        /// Minimum value at far left of slider
        min: f64,
        /// Maximum value at far right of slider
        max: f64,
        /// Step size between minimum and maximum
        step: f64,
        /// Initial value to show on the slider
        initial_value: f64,
    },
    /// A dropdown input
    Dropdown {
        /// Label to be shown at left. If value is `None`, a default of the form $x_N$ will be shown.
        label: Option<String>,
        /// Set of options to include in the dropdown
        options: Vec<f64>,
        /// Initial value to show for the dropdown
        initial_value: usize,
    },
    /// A dropdown input
    Tab {
        /// Label to be shown on tab
        label: String,
        /// List of inputs to be shown in tab
        inputs: Vec<Input>,
    },
}

/// Construct a teaser to demonstrate your model
pub struct Teaser {
    title: String,
    description: String,
    inputs: Vec<Input>,
    function: Box<dyn 'static + Fn(Vec<f64>) -> f64>,
}

impl Default for Teaser {
    fn default() -> Self {
        Self {
            title: "Demo".to_string(),
            description: "".to_string(),
            inputs: vec![Input::Number {
                label: None,
                initial_value: 0.0,
            }],
            function: Box::new(|x| x[0]),
        }
    }
}

impl Teaser {
    /// Add a title to the GUI
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    /// Add a description to the GUI
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Specify the inputs
    pub fn with_inputs(mut self, inputs: Vec<Input>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Specify the function to use
    pub fn with_function<F>(mut self, predictor: F) -> Self
    where
        F: 'static + Fn(Vec<f64>) -> f64,
    {
        self.function = Box::new(predictor);
        self
    }

    /// Run the GUI
    pub fn run(self) {
        thread_local! {
            static WEBVIEW: RefCell<HashMap<usize, WebView>> = RefCell::new(HashMap::new());
        }

        let mut html = beginning(self.description);
        for (idx, input) in self.inputs.iter().enumerate() {
            html = format!(
                "{} {}",
                html,
                match input {
                    Input::Number { initial_value, .. } => add_number(idx, initial_value),
                    Input::Text { initial_value, .. } => add_text(idx, initial_value),
                    Input::Slider {
                        min,
                        max,
                        step,
                        initial_value,
                        ..
                    } => add_slider(idx, initial_value, max, min, step),
                    Input::Dropdown {
                        initial_value,
                        options,
                        ..
                    } => add_dropdown(idx, initial_value, options),
                    _ => {
                        "".to_string()
                    }
                }
            );
        }
        html = format!("{} {}", html, end());

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(self.title)
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

                let y = (*self.function)(inputs);
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
}
