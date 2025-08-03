# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a journal project that builds an interactive, empathetic journaling tool using the Aethel document management system. The project consists of two main components:

1. **Aethel**: A radically minimal document management system (in `external/aethel/`) 
2. **Journal**: An interactive journaling application planned to be built on top of Aethel

## Architecture

The project follows a layered architecture with the Aethel vault as the single source of truth:

- **L0 (aethel-core)**: Pure Rust crate with minimal I/O, providing Doc/Pack/Patch primitives
- **L1 (aethel-cli)**: CLI wrapper with JSON-first I/O for vault operations 
- **Journal**: Interactive journaling application using Elm-like architecture (State/Action/update/effects pattern)

## Key Development Commands

### Aethel Development (in external/aethel/)
```bash
# Build and test Aethel
make build          # Build workspace in debug mode
make release        # Build workspace in release mode
make test           # Run all tests
make fmt            # Check code formatting
make fmt-fix        # Auto-format code
make lint           # Run clippy linting
make clean          # Clean build artifacts

# Optional enhanced commands (if available via mise)
make nextest        # Faster test runner
make audit          # Security vulnerability check
```

### Environment Setup
```bash
# Install project tools (uses .mise.toml for exact versions)
mise install

# For journal (when implemented)
make all            # Build, format, lint, and test
make run            # Build and run the CLI application
```

## Core Concepts

### Aethel Primitives
- **Doc**: Markdown file with YAML front-matter, identified by UUID
- **Pack**: Directory declaring types (schemas), templates, and migrations
- **Patch**: JSON object describing mutations to a Doc

### Journal Application Design
- Uses Elm-like architecture with State enum (Initializing, PromptingForNew, InSession, etc.)
- Session state persisted as `journal.session` documents in Aethel vault
- Active session tracking via `.aethel/indexes/journal.index.json`
- Interactive coaching conversations with transcript preservation

## Important Constraints

- **No overbuilding**: Implement only what's needed
- **JSON-first I/O**: All CLI operations support JSON input/output
- **Atomic operations**: All Doc writes use "write to temp, rename" strategy
- **No hidden state**: Everything stored in plain files within vault
- **Git-friendly**: Designed to work seamlessly with version control

## Error Handling

All errors follow protocol-defined codes:
- **400xx**: Bad Request / Malformed input
- **404xx**: Not Found  
- **409xx**: Conflict
- **422xx**: Validation errors
- **500xx**: System errors

## Testing Strategy

- **Unit tests**: Individual component testing
- **Integration tests**: CLI command end-to-end testing
- **Golden tests**: Deterministic snapshot testing with fixture directories
- Golden test updates: `UPDATE_GOLDEN=1 cargo test`

## File Structure

```
journal/
├── docs/                    # Project documentation
│   ├── project-setup.md     # Detailed setup and engineering guide
│   └── spec.md             # Software design document for journal
└── external/aethel/        # Aethel document management system
    ├── crates/
    │   ├── aethel-core/     # L0: Core library
    │   └── aethel-cli/      # L1: CLI interface
    ├── tests/cases/         # Golden test fixtures
    └── docs/               # Protocol and implementation docs
```

## Development Notes

- Use `mise` for tool version management (Rust 1.88.0)
- All development commands defined in Makefiles
- Schema validation using JSON Schema Draft 2020-12
- Comprehensive golden test suite for CLI behavior validation
- Protocol-first design with machine-readable specifications