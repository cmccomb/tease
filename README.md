[![Github CI](https://github.com/cmccomb/rust-automl/actions/workflows/tests.yml/badge.svg)](https://github.com/cmccomb/automl/actions)
[![Crates.io](https://img.shields.io/crates/v/automl.svg)](https://crates.io/crates/automl)
[![docs.rs](https://img.shields.io/docsrs/automl/latest?logo=rust)](https://docs.rs/automl)

# About _tease_
This crate provides the ability to rapidly generate a GUI (a _teaser_) for a wide array of possible rust functions. It is intended to be used for prototyping machine learningn models. 

# Usage
Running this code will produce a GUI for adding two numbers together
```rust
use teaser::{Teaser, inputs}
Teaser::run(
    move |x| x.iter().sum(),
    vec![Input::Number(0.0), Input::Number(0.0)],
);
```
And it will look something like this:
![](https://raw.githubusercontent.com/cmccomb/tease/master/assets/addition.png)