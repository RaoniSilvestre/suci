//! # SUCI: Simple UCI
//!
//! SUCI is a small, synchronous UCI engine client built on [`shakmaty`].
//!
//! It owns one engine process for a whole session: spawning and the UCI handshake happen once,
//! then each move only sends a position and asks for a best move.
//!
//! # UCI reference
//! The public API maps directly to the [UCI protocol](https://backscattering.de/chess/uci):
//!
//! - [GUI to engine commands](https://backscattering.de/chess/uci/#gui)
//! - [Engine to GUI responses](https://backscattering.de/chess/uci/#engine)
//!
//! # Quick start
//! ```rust,no_run
//! # use std::process::Command;
//! # use suci::{Engine, Go, Position};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Spawns the process and performs the UCI handshake.
//! let mut engine = Engine::spawn(Command::new("stockfish"))?;
//!
//! println!("ID: {:?}", engine.id());
//! println!("{} options", engine.options().len());
//!
//! // Start a new game and synchronize with the engine.
//! engine.new_game()?;
//! engine.wait_until_ready()?;
//!
//! // Send the current position and ask for a move.
//! engine.set_position(&Position::StartPos {
//!     moves: Vec::new(),
//! })?;
//!
//! let best_move = engine.go(&Go {
//!     depth: Some(12),
//!     ..Default::default()
//! })?;
//!
//! println!("best move: {best_move:?}");
//!
//! // Shut down the process. `quit` consumes the engine and waits for it to exit.
//! engine.quit()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Session lifecycle
//! A normal session is:
//!
//! 1. [`Engine::spawn`] once.
//! 2. [`Engine::new_game`] when a game starts.
//! 3. [`Engine::set_position`] and [`Engine::go`] for every move.
//! 4. Call [`Engine::quit`] for graceful shutdown, or simply drop the engine.
//!
//! Do not spawn a new process for every move. The UCI handshake and process startup are the
//! expensive parts. [`Engine`] owns the child process for its entire lifetime. Dropping it
//! without calling [`Engine::quit`] still kills and reaps the process.
//!
//! # Positions
//! UCI has no "apply this move" command. Each turn sends the complete position:
//!
//! - [`Position::StartPos`] with the moves played from the initial position.
//! - [`Position::Fen`] with a FEN and any moves played from that FEN.
//!
//! [`Fen`] and [`UciMove`] are re-exported at the crate root, so you do not need a
//! direct `shakmaty` dependency to construct positions.
//!
//! Use [`Engine::set_position`] followed by [`Engine::go`].
//!
//! ```rust
//! # use suci::{Fen, Position, UciMove};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // The starting position plus the moves played so far.
//! let from_start = Position::StartPos {
//!     moves: vec![UciMove::from_ascii(b"e2e4")?],
//! };
//!
//! // The same idea expressed as a FEN plus moves played from that FEN.
//! let fen = Fen::from_ascii(b"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")?;
//! let from_fen = Position::Fen {
//!     fen,
//!     moves: Vec::new(),
//! };
//! # let _ = (from_start, from_fen);
//! # Ok(())
//! # }
//! ```
//!
//! # Options and readiness
//! Options are usually sent once after spawning:
//!
//! ```rust
//! # use suci::SetOption;
//! let option = SetOption {
//!     name: "Threads".into(),
//!     value: Some("1".into()),
//! };
//! # let _ = option;
//! ```
//!
//! Call [`Engine::wait_until_ready`] after changing options or starting a new game. It is not
//! required between [`Engine::set_position`] and [`Engine::go`]; UCI commands are processed in
//! order.
//!
//! # Errors
//! Public operations return concrete errors:
//!
//! - [`EngineStartError`] for spawning and the initial handshake.
//! - [`ReadWriteError`] for sending commands and reading engine responses.
//! - [`ReadError`] when reading or parsing an engine response fails.
//!
//! # Logging
//! SUCI can emit log events through [`tracing`](https://crates.io/crates/tracing) when the
//! `tracing` feature is enabled. The feature is disabled by default.
//!
//! # Examples
//! The repository contains two runnable examples:
//!
//! - `examples/fen`: sends [`Position::Fen`] for each analysis.
//! - `examples/startpos`: sends [`Position::StartPos`] with move histories.
//!
//! # A note on "GUI"
//! Whenever you see "GUI" in the documentation, it doesn't mean "Graphical User Interface".
//! It means "whatever controls the engine", as the UCI protocol calls it.
//!
//! # A note on `Display` impls
//! `Display` implementations of messages do **not** include the final newline (`\n`) character.
//!
//! [`shakmaty`]: https://crates.io/crates/shakmaty

#[macro_use]
mod logging;
mod engine;
mod errors;
mod message;
mod parsing;

pub use engine::*;
pub use errors::*;
pub use message::*;
pub use shakmaty::fen::Fen;
pub use shakmaty::uci::UciMove;
