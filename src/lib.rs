//! # A note on "GUI"
//! Whenever you see "GUI" in the documentation, it doesn't mean "Graphical User Interface".
//! It means "whatever controls the engine".
//! It's called that because that's how it's referred to in the UCI protocol and elsewhere.
//!
//! # Example
//! ```rust,no_run
//! use std::process::Command;
//! use suci::{Engine, Go};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut engine = Engine::spawn(Command::new("stockfish"))?;
//! let best_move = engine.go(&Go {
//!     depth: Some(20),
//!     ..Default::default()
//! })?;
//! # Ok(())
//! # }
//! ```
//!
//! # A note on `Display` impls
//! `Display` implementations of messages do **not** include the final newline (`\n`) character.

mod engine;
mod errors;
mod message;
mod parsing;

pub use engine::*;
pub use errors::*;
pub use message::*;
