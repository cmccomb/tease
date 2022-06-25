#![warn(clippy::all)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(clippy::missing_docs_in_private_items)]
#![doc = include_str!("../README.md")]

use std::fmt::Formatter;
use std::{cell::RefCell, collections::HashMap, fmt};

use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    webview::{WebView, WebViewAttributes, WebViewBuilder},
};

pub mod cookbook;

mod html_chunks;
use html_chunks::{add_dropdown, add_number, add_slider, beginning, end, middle};

/// Types of inputs for the model
#[derive(Clone)]
pub enum Input {
    /// A numerical input
    Number {
        /// Label to be shown above input. If value is `None`, a default of the form _Input N_ will be shown.
        label: Option<String>,
        /// Initial value to shown in the textbox
        initial_value: f64,
    },
    // /// A textual input
    // Text {
    //     /// Label to be shown above input. If value is `None`, a default of the form _Input N_ will be shown.
    //     label: Option<String>,
    //     /// Initial value to shown in the textbox
    //     initial_value: String,
    // },
    /// A slider input
    Slider {
        /// Label to be shown above input. If value is `None`, a default of the form _Input N_ will be shown.
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
        /// Label to be shown above input. If value is `None`, a default of the form _Input N_ will be shown.
        label: Option<String>,
        /// Set of options to include in the dropdown
        options: Vec<f64>,
        /// Initial value to show for the dropdown
        initial_value: usize,
    },
    // /// A radio button selector
    // Radio {
    //     /// Label to be shown above input. If value is `None`, a default of the form _Input N_ will be shown.
    //     label: Option<String>,
    //     /// Set of options to include in the selection
    //     options: Vec<f64>,
    // },
    // /// An area to upload a file. Specifying a file type will add a preview of the file after loading.
    // File {
    //     /// Label to be shown above input. If value is `None`, a default of the form _Input N_ will be shown.
    //     label: Option<String>,
    //     /// Type of file. If value is `None`, not preview of file will be given.
    //     filetype: Option<TypeOfFile>,
    // },
}

// /// File types with custom file previews
// #[derive(Clone)]
// pub enum TypeOfFile {
//     /// Reads and previews image files
//     Image,
//     /// Reads and previews audio files
//     Audio,
//     /// Reads and previews 3D models
//     Model,
//     /// Reads and previews CSVs
//     CSV,
// }

impl Input {
    fn get_html(&self, idx: usize) -> String {
        match self {
            Input::Number {
                initial_value,
                label,
            } => add_number(idx, initial_value, label),
            Input::Slider {
                min,
                max,
                step,
                initial_value,
                label,
            } => add_slider(idx, initial_value, max, min, step, label),
            Input::Dropdown {
                initial_value,
                options,
                label,
            } => add_dropdown(idx, initial_value, options, label),
            // Input::File { label, filetype } => match filetype {
            //     Some(ft) => match ft {
            //         TypeOfFile::Image => "".to_string(),
            //         TypeOfFile::Audio => "".to_string(),
            //         TypeOfFile::Model => "".to_string(),
            //         TypeOfFile::CSV => "".to_string(),
            //     },
            //     None => "".to_string(),
            // },
            _ => "".to_string(),
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Input::Number {
            label: None,
            initial_value: 0.0,
        }
    }
}

/// Types of outputs for the model
pub enum Output {
    /// A numerical input
    Number {
        /// Label to be shown above output. If value is `None`, a default of the form _Result N_ will be shown.
        label: Option<String>,
    },
    // Vector {
    //     label: Option<String>,
    // },
    // Text {
    //     label: Option<String>,
    // },
    // File {
    //     label: Option<String>,
    //     filetype: TypeOfFile,
    // },
    // HTML {
    //     label: Option<String>,
    // },
}

impl Default for Output {
    fn default() -> Self {
        Self::Number {
            label: Some("Result".to_string()),
        }
    }
}

impl Output {
    fn get_html(&self) -> String {
        let label = match self {
            Output::Number { label } => match label {
                None => {
                    format!("Result")
                }
                Some(string) => string.to_string(),
            },
        };
        format!("<label for=\"output\" class=\"col-form-label mt-3\"><i>{label}</i></label>
         <input type=\"text\" class=\"form-control\" id=\"output\" name=\"output\" aria-describedby=\"output\" readonly>")
    }
}

/// Construct a teaser to demonstrate your model
pub struct Teaser {
    title: String,
    description: String,
    inputs: Vec<Input>,
    output: Output,
    function: Box<dyn 'static + Fn(Vec<f64>) -> f64>,
}

impl Default for Teaser {
    fn default() -> Self {
        Self {
            title: "Demo".to_string(),
            description: "".to_string(),
            inputs: vec![Input::default()],
            output: Output::default(),
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

    /// Specify the inputs
    pub fn with_output(mut self, output: Output) -> Self {
        self.output = output;
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
            html = format!("{} {}", html, input.get_html(idx));
        }
        html = format!("{} {} {} {}", html, middle(), self.output.get_html(), end());

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
