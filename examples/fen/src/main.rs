//! Analyze three positions by sending complete FENs to one persistent engine.
//!
//! Run with `cargo run -p fen`. Requires that you have installed Stockfish.

use shakmaty::fen::Fen;
use std::process::Command;
use suci::{Engine, Go, Position};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spawns the process and performs the UCI handshake once.
    let mut engine = Engine::spawn(Command::new("stockfish"))?;

    // The handshake data is available without another round trip.
    println!("== ID: {:?}", engine.id());
    println!("== Options: {}", engine.options().len());

    // Clear the engine's game state before the first analysis.
    engine.new_game()?;
    engine.wait_until_ready()?;

    // 1. The starting position as a complete FEN.
    println!("== Starting position");
    engine.set_position(&Position::Fen {
        fen: Fen::from_ascii(b"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")?,
        // No moves have been played from this FEN.
        moves: Vec::new(),
    })?;

    // Optional barrier: UCI processes commands in order, but this makes the
    // synchronization explicit before searching.
    engine.wait_until_ready()?;

    // `go` blocks until the engine sends its best move.
    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    // 2. The same engine process is reused; the position after 1. e4 is sent as a FEN.
    println!("== After 1. e4");
    engine.set_position(&Position::Fen {
        fen: Fen::from_ascii(b"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")?,
        moves: Vec::new(),
    })?;
    engine.wait_until_ready()?;

    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    // 3. The position after 1. e4 e5, again as a complete FEN.
    println!("== After 1. e4 e5");
    engine.set_position(&Position::Fen {
        fen: Fen::from_ascii(b"rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")?,
        moves: Vec::new(),
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
