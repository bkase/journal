# Project: Journal

An interactive, empathetic journaling application built on the Aethel document management system, providing AI-powered coaching conversations for personal reflection and growth.

## Features

- Interactive AI coaching with Claude for morning intention-setting and evening reflection
- Session management with persistence and seamless resume functionality
- AI-powered analysis of journal entries for insights and themes
- Structured data storage using Aethel's vault system
- Privacy-first design with local storage
- Git-friendly plain text Markdown files

## Tech Stack

- **Language**: Rust 1.88.0 (managed via mise)
- **Architecture**: Elm-like (State/Action/update/effects pattern)
- **Document Management**: Aethel (L0: core library, L1: CLI wrapper)
- **AI Integration**: Claude CLI subprocess
- **CLI Framework**: clap 4.5
- **Terminal UI**: crossterm, dialoguer, console
- **Testing**: cargo test, cargo-nextest, golden tests

## Structure

- `src/journal/` - Main journal application (main.rs, state.rs, action.rs, update.rs, effects.rs)
- `external/aethel/` - Aethel document management system (consider this submodule read-only)
- `docs/` - Project documentation (spec.md, project-setup.md)
- `todos/` - Todo management directory

## Architecture

The project uses a layered architecture:

- **L0 (aethel-core)**: Pure Rust library providing Doc/Pack/Patch primitives
- **L1 (aethel-cli)**: CLI wrapper with JSON-first I/O
- **L2 (journal)**: Interactive journaling application using Elm-like pattern

Components interact through:

- State transitions trigger effects
- Effects interact with Aethel vault and Claude CLI
- User input converted to actions
- Pure update functions transform state

## Commands

- Build: `make all` or `cargo build`
- Test: `make test` or `cargo test`
- Lint: `make lint` or `cargo clippy --all-targets --all-features -- -D warnings`
- Dev/Run: `make run` or `cargo run --bin journal`
- Format: `make fmt-fix` or `cargo fmt --all`
- Clean: `make clean` or `cargo clean`

## Testing

Create tests using:

- **Unit tests**: Add `#[test]` functions in source files
- **Integration tests**: Use `assert_cmd` for CLI testing

## Editor

- The user can open folders with vim out-of-band

