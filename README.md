[![CI](https://img.shields.io/github/actions/workflow/status/RaoniSilvestre/suci/ci.yml?label=CI)](https://github.com/RaoniSilvestre/suci/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/suci?logo=docs.rs&label=docs.rs)](https://docs.rs/suci/)
[![crates.io](https://img.shields.io/crates/v/suci?logo=rust&label=crates.io)](https://crates.io/crates/suci)

# SUCI: Simple UCI

SUCI is a fork of [ruci](https://github.com/tigerros/ruci) by Leonard D.
It keeps the UCI engine client and message formatting, while removing the upstream macro
layer, async support, `no_std` support, and engine-side GUI API.

SUCI is a [UCI protocol](https://backscattering.de/chess/uci) engine client using
[`shakmaty`](https://crates.io/crates/shakmaty) for relevant types.

This fork exists to tailor the library to its use in
[chess-tui](https://github.com/thomas-mauran/chess-tui), removing unused and overly
complicated parts of the original library.

## Quick start

Using this library assumes that you can launch an engine through
`std::process::Command`; the engine runs as a child process alongside your code. The
library makes it easier to communicate with that process from Rust.

```rust
use std::borrow::Cow;
use std::process::Command;
use suci::{Engine, Go, Position};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spawns the process and performs the UCI handshake.
    let mut engine = Engine::spawn(Command::new("stockfish"))?;

    println!("ID: {:?}", engine.id());
    println!("{} options", engine.options().len());

    // Start a fresh game and synchronize.
    engine.new_game()?;
    engine.wait_until_ready()?;

    // Send the position and ask for a move.
    engine.set_position(&Position::StartPos {
        moves: Cow::Borrowed(&[]),
    })?;

    let best_move = engine.go(&Go {
        depth: Some(12),
        ..Default::default()
    })?;
    println!("best move: {best_move:?}");

    // Shut down the process. `quit` consumes the engine and waits for it to exit.
    engine.quit()?;

    Ok(())
}
```

## Session lifecycle

Use one engine for the whole game:

1. `Engine::spawn` once.
2. `Engine::new_game` when the game starts.
3. `Engine::set_position` and `Engine::go` for every move.
4. Call `Engine::quit` for graceful shutdown, or simply drop the engine.

Do not spawn a new process for each move. The handshake and process startup are the expensive
parts. `Engine` owns the child process for its entire lifetime. Dropping it without calling
`Engine::quit` still kills and reaps the process.

## Positions

UCI has no command for applying a single move, so each turn sends the complete position:

- `Position::StartPos` with the moves played from the initial position.
- `Position::Fen` with a FEN and any moves played from it.

Both examples in [`examples/`](examples) analyze the same positions using the two forms:

```bash
cargo run -p fen
cargo run -p startpos
```

## Options and readiness

Options are usually sent once after spawning:

```rust
use suci::{Engine, SetOption};

fn configure(engine: &mut Engine) -> Result<(), Box<dyn std::error::Error>> {
    engine.set_option(&SetOption {
        name: "Threads".into(),
        value: Some("1".into()),
    })?;
    engine.wait_until_ready()?;
    Ok(())
}
```

Call `wait_until_ready` after changing options or starting a new game. It is not required between
`set_position` and `go`; UCI commands are processed in order.

## Logging

SUCI uses [`tracing`](https://crates.io/crates/tracing) to emit log events.

## Fork differences

- Synchronous `Engine` only.
- No `no_std`, async, or engine-side `Gui` struct.
- No public raw message enums, `send`, or `read`.
- No internal macros or sealed message traits.
- No vendored engines, benchmark harness, or compile-fail suite.

## MSRV

Rust 1.99 or newer.

## License

GPL-3.0-only. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

SUCI is not affiliated with or endorsed by the original `ruci` author.
