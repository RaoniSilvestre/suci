[![CI](https://img.shields.io/github/actions/workflow/status/tigerros/ruci/ci.yml?label=CI)](https://github.com/tigerros/ruci/actions/workflows/ci.yml)
[![coverage](https://img.shields.io/codecov/c/gh/tigerros/ruci)](https://app.codecov.io/gh/tigerros/ruci/)
[![docs.rs](https://img.shields.io/docsrs/ruci?logo=docs.rs&label=docs.rs)](https://docs.rs/ruci/)
[![crates.io](https://img.shields.io/crates/v/ruci?logo=rust&label=crates.io)](https://crates.io/crates/ruci)

# ruci
**R**ust **U**niversal **C**hess **I**nterface.

This crate is a full implementation of the [UCI protocol](https://backscattering.de/chess/uci) using [`shakmaty`](https://crates.io/crates/shakmaty) for relevant types.
The UCI protocol is the most widely used way to communicate with chess engines and vice versa.

Specifically, this crate contains:
- Types to represent every UCI message.
- The means to convert strings to these types.
- The means to convert these types to a string.
- The `Engine` struct for communicating with an engine (see the [example](examples/go-std), run it with `cargo run -p go-std`).

## Features
No features are enabled by default.

- `serde`: enables serde support for most types. All implementations are derived with no parameters.

## Safety
`ruci` forbids unsafe code.
