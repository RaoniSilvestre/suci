//! Analyze the same game using `startpos` and move histories.
//!
//! Requires that you have installed Stockfish.

use shakmaty::uci::UciMove;
use std::borrow::Cow;
use std::process::Command;
use suci::{Engine, Go, Position};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::spawn(Command::new("stockfish"))?;

    println!("== ID: {:?}", engine.id());
    println!("== Options: {}", engine.options().len());

    engine.new_game()?;
    engine.wait_until_ready()?;

    println!("== Starting position");
    engine.set_position(&Position::StartPos {
        moves: Cow::Borrowed(&[]),
    })?;
    engine.wait_until_ready()?;

    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    println!("== After 1. e4");
    engine.set_position(&Position::StartPos {
        moves: Cow::Borrowed(&[UciMove::from_ascii(b"e2e4")?]),
    })?;
    engine.wait_until_ready()?;

    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    println!("== After 1. e4 e5");
    engine.set_position(&Position::StartPos {
        moves: Cow::Borrowed(&[UciMove::from_ascii(b"e2e4")?, UciMove::from_ascii(b"e7e5")?]),
    })?;
    engine.wait_until_ready()?;

    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    engine.quit()?;
    engine.wait()?;

    Ok(())
}
