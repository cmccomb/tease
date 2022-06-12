[![Github CI](https://github.com/cmccomb/tease/actions/workflows/tests.yml/badge.svg)](https://github.com/cmccomb/tease/actions)
[![Crates.io](https://img.shields.io/crates/v/tease.svg)](https://crates.io/crates/tease)
[![docs.rs](https://img.shields.io/docsrs/tease/latest?logo=rust)](https://docs.rs/tease)

# About _tease_
This crate provides the ability to rapidly generate a GUI (a _teaser_) for a wide array of possible rust functions. It is intended to be used for prototyping machine learningn models. 

# Usage
Running this code will produce a GUI for adding two numbers together
```rust, no_run
use tease::{Teaser, Input};
Teaser::default()
    .with_title("Addition".to_string())
    .with_description("This is for adding things together.".to_string())
    .with_inputs(vec![Input::Number(0.0); 2])
    .run(move |x| x.iter().sum());
```
And it will look something like this:
![](https://raw.githubusercontent.com/cmccomb/tease/master/assets/addition.png)