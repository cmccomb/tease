//! Runnable examples for numerical and typed prediction callbacks.
//!
//! # A minimal demo
//! ```no_run
#![doc = include_str!("../examples/minimal.rs")]
//! ```
//!
//! # Addition with a numerical closure
//! ```no_run
#![doc = include_str!("../examples/addition.rs")]
//! ```
//!
//! # Labels and multiple results
//! ```no_run
#![doc = include_str!("../examples/classification.rs")]
//! ```
//!
//! # Worker-local model initialization
//! ```no_run
#![doc = include_str!("../examples/slow.rs")]
//! ```
//!
//! The repository also includes `smartcore` and `automl` examples with
//! fallible model callbacks. Run them with `cargo run --locked --example smartcore`
//! or `cargo run --locked --example automl` after installing desktop prerequisites.

#![doc = include_str!("../docs/migration-0.2.md")]
#![doc = include_str!("../docs/examples.md")]
