[![CI](https://img.shields.io/github/actions/workflow/status/RaoniSilvestre/suci/ci.yml?label=CI)](https://github.com/RaoniSilvestre/suci/actions/workflows/ci.yml)
[![coverage](https://img.shields.io/codecov/c/gh/RaoniSilvestre/suci)](https://app.codecov.io/gh/RaoniSilvestre/suci/)
[![docs.rs](https://img.shields.io/docsrs/suci?logo=docs.rs&label=docs.rs)](https://docs.rs/suci/)
[![crates.io](https://img.shields.io/crates/v/suci?logo=rust&label=crates.io)](https://crates.io/crates/suci)

# SUCI
**S-RUCI** — Simples Rust UCI.

SUCI is a fork of [ruci](https://github.com/tigerros/ruci) by Leonard D.
It keeps the UCI message types and the synchronous engine client, while removing
the upstream macro layer, async support, `no_std` support, and engine-side GUI API.

SUCI is a [UCI protocol](https://backscattering.de/chess/uci) engine client using [`shakmaty`](https://crates.io/crates/shakmaty) for relevant types.

Specifically, this crate contains:
- Concrete types for engine commands and responses.
- Parsing for engine responses.
- Formatting for commands and responses.
- The `Engine` struct for communicating with an engine (see the [examples](examples), run them with `cargo run -p fen` and `cargo run -p startpos`).

## Fork differences
- Synchronous `Engine` only.
- No `no_std`, async, or engine-side `Gui` struct.
- No internal macros or sealed message traits.
- No vendored engines, benchmark harness, or compile-fail suite.

## Logging
SUCI uses [`tracing`](https://crates.io/crates/tracing). The library does not install a subscriber.

Applications that install a subscriber can use:

- `RUST_LOG=suci=debug` for engine lifecycle events.
- `RUST_LOG=suci=trace` for the full UCI traffic.
- `RUST_LOG=trace` for trace-level output from every crate.

## MSRV
Rust 1.99 or newer.

## License
GPL-3.0-only. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

SUCI is not affiliated with or endorsed by the original `ruci` author.
