//! # A cookbook full of wonderful things
//! ## The basics
//! Starting out simple, we can run just a few lines of code that will set up a rudimentary GUI.
//! ```rust, no_run
#![doc = include_str!("../examples/minimal.rs")]
//! ```
//! This program draws on the default settings to produce a GUI that looks like this
//! ![](https://raw.githubusercontent.com/cmccomb/tease/master/assets/minimal.png)
//! The default settings are designed to demonstrate some of the functionality of the
//! visual interface. There are inputs, there is an output, and there is a <kdb>Submit</kbd>
//! button that uses the inputs to compute an output.
//!
//! Let's build a simple calculator to add two numbers together. We will need to have two inputs,
//! and we will also need to define a function that sums those inputs together. Our code becomes
//! ```rust, no_run
//! use tease::{Input, Teaser};
//!
//! fn main() {
//!     Teaser::default()
//!         .with_inputs(vec![Input::default(); 2])
//!         .with_function(move |x: Vec<f32> | x.iter().sum())
//!         .run();
//! }
//! ```
//! This will make the GUI look as follows. And it works! We can add numbers together!
//! ![](https://raw.githubusercontent.com/cmccomb/tease/master/assets/addition_basic.png)
//!
//! We can spruce this up a little bit by adding a descriptive header and a short description of
//! what the GUI does. Our code now becomes:
//! ```rust, no_run
#![doc = include_str!("../examples/addition.rs")]
//! ```
//! And when executed it creates this GUI:
//! ![](https://raw.githubusercontent.com/cmccomb/tease/master/assets/addition.png)
//!
//! ## Input Types
//! `tease` provides a variety of different input types for flexible interface creation. These include
//! ### Numbers
//! <iframe style="width:50%; margin-left: 25%; margin-right: 25%" src="https://raw.githubusercontent.com/cmccomb/tease/master/assets/number.html"></iframe>
//!
//! ### Sliders
//! //! <iframe style="width:50%; margin-left: 25%; margin-right: 25%" src="https://raw.githubusercontent.com/cmccomb/tease/master/assets/slider.html"></iframe>
//!
//! ### Dropdowns
//! //! <iframe style="width:50%; margin-left: 25%; margin-right: 25%" src="https://raw.githubusercontent.com/cmccomb/tease/master/assets/dropdown.html"></iframe>
//!
//! ## Fun with Closures
//! By now, you've probably realized something - anything that you can fit in a closure can be used
//! as the backend for a GUI. For instance, you can train a model in [SmartCore](https://smartcorelib.org/)
//! and then provide an interactive demo.
//!```rust, no_run
#![doc = include_str!("../examples/smartcore.rs")]
//! ```
//! ![](https://raw.githubusercontent.com/cmccomb/tease/master/assets/smartcore.png)
//!
