[![CI](https://img.shields.io/github/actions/workflow/status/RaoniSilvestre/suci/ci.yml?label=CI)](https://github.com/RaoniSilvestre/suci/actions/workflows/ci.yml)
[![coverage](https://img.shields.io/codecov/c/gh/RaoniSilvestre/suci)](https://app.codecov.io/gh/RaoniSilvestre/suci/)
[![docs.rs](https://img.shields.io/docsrs/suci?logo=docs.rs&label=docs.rs)](https://docs.rs/suci/)
[![crates.io](https://img.shields.io/crates/v/suci?logo=rust&label=crates.io)](https://crates.io/crates/suci)

# SUCI
**S-RUCI** — Simples Rust UCI.

SUCI is a fork of [ruci](https://github.com/tigerros/ruci) by Leonard D.
It keeps the UCI message types and the synchronous engine client, while removing
the upstream macro layer, async support, `no_std` support, and engine-side GUI API.

This crate is a full implementation of the [UCI protocol](https://backscattering.de/chess/uci) using [`shakmaty`](https://crates.io/crates/shakmaty) for relevant types.

Specifically, this crate contains:
- Concrete types and enums for every UCI message: `UciMessage`, `EngineMessage`, and `GuiMessage`.
- The means to convert strings to these types.
- The means to convert these types to a string.
- The `Engine` struct for communicating with an engine (see the [example](examples/go-std), run it with `cargo run -p go-std`).

## Fork differences
- Synchronous `Engine` only.
- No `no_std`, async, or engine-side `Gui` struct.
- No internal macros or sealed message traits.
- No vendored engines, benchmark harness, or compile-fail suite.

## Features
No features are enabled by default.

- `serde`: enables serde support for most types. All implementations are derived with no parameters.

## MSRV
Rust 1.99 or newer.

## License
GPL-3.0-only. See [LICENSE](LICENSE) and [NOTICE](NOTICE).

SUCI is not affiliated with or endorsed by the original `ruci` author.
