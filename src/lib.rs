#![warn(clippy::all)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(clippy::missing_docs_in_private_items)]
#![doc = include_str!("../README.md")]

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    str::FromStr,
};

use wry::{
    application::{
        event::{Event, StartCause, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    webview::{WebView, WebViewAttributes, WebViewBuilder},
};

use num_traits::{zero, Float};

pub mod cookbook;

mod html_chunks;
use html_chunks::{add_dropdown, add_number, add_slider, beginning, end, middle};

/// Types of inputs for the model
#[derive(Clone)]
#[non_exhaustive]
pub enum Input<F: Float + Display = f32> {
    /// A numerical input
    Number {
        /// Label to be shown above input. If value is `None`, a default of the form _Input N_ will be shown.
        label: Option<String>,
        /// Initial value to shown in the textbox
        initial_value: F,
    },
    // /// A textual input
    // TextBox {
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
        min: F,
        /// Maximum value at far right of slider
        max: F,
        /// Step size between minimum and maximum
        step: F,
        /// Initial value to show on the slider
        initial_value: F,
    },
    /// A dropdown input
    Dropdown {
        /// Label to be shown above input. If value is `None`, a default of the form _Input N_ will be shown.
        label: Option<String>,
        /// Set of options to include in the dropdown
        options: Vec<F>,
        /// Initial value to show for the dropdown
        initial_value: usize,
    },
    // Checkbox,
    // CheckboxGroup,
    // Image,
    // Video,
    // Audio,
    // File,
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
    // },
}

impl<F: Float + Display> Input<F> {
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
        }
    }
}

impl<F: Float + Display> Default for Input<F> {
    fn default() -> Self {
        Input::Number {
            label: None,
            initial_value: zero(),
        }
    }
}

/// Types of outputs for the model
#[derive(Clone)]
#[non_exhaustive]
pub enum Output {
    /// A numerical input
    Number {
        /// Label to be shown above output. If value is `None`, a default of the form _Result N_ will be shown.
        label: Option<String>,
        precision: usize,
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
            precision: 2,
        }
    }
}

impl Output {
    fn get_html(&self) -> String {
        let label = match self {
            Output::Number { label, .. } => match label {
                None => {
                    format!("Result")
                }
                Some(string) => string.to_string(),
            },
        };
        format!("<label for=\"output\" class=\"col-form-label mt-3\"><i>{label}</i></label>
         <input type=\"text\" class=\"form-control\" id=\"output\" name=\"output\" aria-describedby=\"output\" readonly>")
    }

    fn get_precision(&self) -> usize {
        match self {
            Output::Number { precision, .. } => *precision,
        }
    }
}

/// Construct a teaser to demonstrate your model
pub struct Teaser<F: Float + Display = f32> {
    title: String,
    description: String,
    inputs: Vec<Input<F>>,
    output: Output,
    function: Box<dyn 'static + Fn(Vec<F>) -> F>,
    use_advanced_function: bool,
    advanced_function: Box<dyn 'static + Fn(Vec<Input>) -> Vec<Output>>,
}

impl<F: Float + Display> Default for Teaser<F> {
    fn default() -> Self {
        Self {
            title: "Demo".to_string(),
            description: "".to_string(),
            inputs: vec![Input::default()],
            output: Output::default(),
            function: Box::new(|x| x[0]),
            use_advanced_function: false,
            advanced_function: Box::new(|x| vec![Output::default()]),
        }
    }
}

impl<F: 'static + Float + Display + FromStr> Teaser<F> {
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
    pub fn with_inputs(mut self, inputs: Vec<Input<F>>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Specify the inputs
    pub fn with_output(mut self, output: Output) -> Self {
        self.output = output;
        self
    }

    /// Specify the function to use.
    pub fn with_function<G>(mut self, predictor: G) -> Self
    where
        G: 'static + Fn(Vec<F>) -> F,
    {
        self.function = Box::new(predictor);
        self
    }

    /// Specify the advanced function to use (note, this will override a function added using `with_function`)
    pub fn with_advanced_function<A>(mut self, predictor: A) -> Self
    where
        A: 'static + Fn(Vec<Input>) -> Vec<Output>,
    {
        self.use_advanced_function = true;
        self.advanced_function = Box::new(predictor);
        self
    }

    /// Run the GUI
    pub fn run(self)
    where
        <F as FromStr>::Err: Debug,
    {
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
                let mut inputs = vec![zero(); 0];
                for number in number_strings {
                    inputs.push(number.parse().unwrap());
                }

                let y = (*self.function)(inputs);
                WEBVIEW
                    .with(|webview| {
                        let webview = webview.borrow();
                        let my_webview = webview.get(&0).unwrap();
                        let precision = self.output.get_precision();
                        my_webview.evaluate_script(&*format!(
                            "document.getElementById('output').value = {:.precision$}",
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
