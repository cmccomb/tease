[![Github CI](https://github.com/cmccomb/tease/actions/workflows/tests.yml/badge.svg)](https://github.com/cmccomb/tease/actions)
[![Crates.io](https://img.shields.io/crates/v/tease.svg)](https://crates.io/crates/tease)
[![docs.rs](https://img.shields.io/docsrs/tease/latest?logo=rust)](https://docs.rs/tease)

# About _tease_
This crate provides the ability to rapidly generate a GUI (a _teaser_) for a wide array of possible rust functions. It is intended to be used for prototyping interfaces for machine learning models. Inspired by [gradio](https://gradio.app/).

# Usage
Running this code will produce a GUI for adding two numbers together
```rust, no_run
use tease::{Teaser, Input};
Teaser::default()
    .with_title("Addition".to_string())
    .with_description("This is for adding things together.".to_string())
    .with_inputs(vec![Input::default(); 2])
    .with_function(|x: Vec<f32>| x.iter().sum())
    .run();
```
And it will look something like this:
![](https://raw.githubusercontent.com/cmccomb/tease/master/assets/addition.png)
But, you can also run more complex demos! Take a look at [this example](https://github.com/cmccomb/tease/blob/master/examples/smartcore.rs), for instance. It trains a smartcore machine learning model and then outputs a GUI for it!
![](https://raw.githubusercontent.com/cmccomb/tease/master/assets/smartcore.png)
_The best part is that the compiled binary can be distributed as a standalone teaser of your model!_