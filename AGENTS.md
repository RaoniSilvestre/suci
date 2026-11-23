# AGENTS.md

## Project

SUCI (`suci`) is a small, synchronous UCI engine client built on `shakmaty`.
It is a fork of `ruci` by Leonard D. and is licensed under GPL-3.0-only.

The project intentionally favors a small public API over protocol completeness in
the public surface. The engine and its messages are the core; internal parsing and
rare protocol messages should not leak into the public API without a concrete need.

## Commands

```bash
cargo fmt --all --check
cargo test --workspace
cargo +nightly clippy --workspace --all-targets -- --deny warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo publish --dry-run --allow-dirty
cargo run -p fen
cargo run -p startpos
```

The example requires `stockfish` on `PATH`.

## Architecture

- `src/engine.rs`
  - public `Engine`
  - public `UciInfo`
  - process ownership, spawning, handshake, lifecycle (`quit`, `Drop`)
  - private `EngineStream<E, G>` used to keep protocol I/O testable
- `src/message/`
  - public command and response types
  - private `Response` parser
  - private message/parameter keyword enums
- `src/errors.rs`
  - concrete public error types
- `src/parsing.rs`
  - shared parsing helpers
- `src/uci_moves.rs`
  - UCI move list formatting/parsing

Keep the public API limited to what is actually useful. Do not reintroduce:

- public `UciMessage` / `EngineMessage` / `GuiMessage` enums
- public `send`/`read` methods
- `from_process(&mut Child)` or borrowed process APIs
- public pointer enums
- per-message `From` conversions into public enums
- macros for message/type generation

## Public API rules

- `Engine` owns the child process and kills/reaps it on drop.
- `Engine::spawn(Command)` performs the UCI handshake and returns a ready engine.
- Keep `id()` and `options()` available from the handshake result.
- Commands use methods: `set_position`, `set_option`, `go`, `new_game`, `stop`,
  `ponder_hit`, `quit`, `wait_until_ready`.
- `quit` consumes the engine, sends `quit`, and waits for the process to exit.
  Dropping the engine is the fallback shutdown path and kills/reaps the process.
- `go` returns `BestMove`; `Info` messages are currently logged at `trace`, not
  exposed through the public API.
- If a future feature needs live `Info` access, add a clearly named API and discuss
  it first rather than exposing the raw message enum.

## Dependencies and features

Keep dependencies minimal.

- Runtime: `shakmaty`, `tracing`
- Example: `shakmaty`
- No `serde`, `no_std`, async runtime, vendored engines, benchmarks, or
  compile-fail suites.
- Do not add a dependency without a concrete use case.

MSRV is Rust 1.99, matching `rust-version` in `Cargo.toml`.

## Style

- Match the surrounding code and rustfmt output.
- Do not use raw identifiers such as `r#move` or `r#type`.
  - move field: `movement`
  - engine option type: `option_type`
- Prefer standard `Option` combinators over custom helper traits.
  - Use `new_value.or(old_value)` for “replace only if present”.
- File names should match their primary type. Put the most relevant type first.
- Prefer concrete types and explicit code over generic machinery.
- Public items need documentation; private implementation details do not.
- Public types and methods that directly represent a UCI command or response
  should link to the relevant `backscattering.de/chess/uci` anchor.
- Errors must implement `Display` and `std::error::Error`.

## Logging

The library uses `tracing` but does not install a subscriber.

- `debug`: engine lifecycle
- `trace`: UCI traffic and internal details

Applications may install their own subscriber and use:

```text
RUST_LOG=suci=debug
RUST_LOG=suci=trace
```

Do not add `tracing-subscriber` to the crate or examples. Do not add logging setup
to examples.

## Tests

- Keep tests in `#[cfg(test)]` modules next to the code.
- Use standard `assert_eq!`, `assert!`, and `matches!`.
- Do not add test-only helper modules, assertion macros, or `pretty_assertions`.
- Prefer testing the private `EngineStream` with `&[u8]` and `Vec<u8>`.
- Keep the example focused on the public API; no helper functions that hide the
  calls being demonstrated.

## Examples

There are two examples, both using one persistent engine process for the same
three positions:

1. starting position
2. after `1. e4`
3. after `1. e4 e5`

- `examples/fen` sends `Position::Fen` each time.
- `examples/startpos` sends `Position::StartPos` with the move history.

Show `set_position`, `wait_until_ready`, and `go` explicitly each time. Do not
abstract them behind helpers. Do not initialize tracing in the examples.

## Git and publishing

- Do not commit unless explicitly asked.
- Do not push, force-push, tag, or publish to crates.io unless explicitly asked.
- Keep GPL-3.0-only licensing and the `NOTICE` attribution.
