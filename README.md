# About _tease_
This crate provides the ability to rapidly generate a GUI for a wide array of possible rust functions. 

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
