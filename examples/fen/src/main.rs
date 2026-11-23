//! Analyze the same game using explicit FEN positions.
//!
//! Requires that you have installed Stockfish.

use shakmaty::fen::Fen;
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
    engine.set_position(&Position::Fen {
        fen: Cow::Owned(Fen::from_ascii(
            b"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        )?),
        moves: Cow::Borrowed(&[]),
    })?;
    engine.wait_until_ready()?;

    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    println!("== After 1. e4");
    engine.set_position(&Position::Fen {
        fen: Cow::Owned(Fen::from_ascii(
            b"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        )?),
        moves: Cow::Borrowed(&[]),
    })?;
    engine.wait_until_ready()?;

    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    println!("== After 1. e4 e5");
    engine.set_position(&Position::Fen {
        fen: Cow::Owned(Fen::from_ascii(
            b"rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        )?),
        moves: Cow::Borrowed(&[]),
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
