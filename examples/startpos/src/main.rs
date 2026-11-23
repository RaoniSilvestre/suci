//! Analyze three positions by sending `startpos` with move histories to one persistent engine.
//!
//! Run with `cargo run -p startpos`. Requires that you have installed Stockfish.

use std::process::Command;
use suci::{Engine, Go, Position, UciMove};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spawns the process and performs the UCI handshake once.
    let mut engine = Engine::spawn(Command::new("stockfish"))?;

    // The handshake data is available without another round trip.
    println!("== ID: {:?}", engine.id());
    println!("== Options: {}", engine.options().len());

    // Clear the engine's game state before the first analysis.
    engine.new_game()?;
    engine.wait_until_ready()?;

    // 1. The starting position with an empty move history.
    println!("== Starting position");
    engine.set_position(&Position::StartPos { moves: Vec::new() })?;

    // Optional barrier: UCI processes commands in order, but this makes the
    // synchronization explicit before searching.
    engine.wait_until_ready()?;

    // `go` blocks until the engine sends its best move.
    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    // 2. The same engine process is reused; the position after 1. e4 is sent as a move history.
    println!("== After 1. e4");
    engine.set_position(&Position::StartPos {
        moves: vec![UciMove::from_ascii(b"e2e4")?],
    })?;
    engine.wait_until_ready()?;

    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    // 3. The position after 1. e4 e5 uses the complete history from the initial position.
    println!("== After 1. e4 e5");
    engine.set_position(&Position::StartPos {
        moves: vec![UciMove::from_ascii(b"e2e4")?, UciMove::from_ascii(b"e7e5")?],
    })?;
    engine.wait_until_ready()?;

    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    // Ask the engine to exit and wait for it. Dropping would also kill and reap the process.
    engine.quit()?;

    Ok(())
}
