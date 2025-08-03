# Journal

An interactive, empathetic journaling application built on the [Aethel](https://github.com/aethel/aethel) document management system.

[![Rust](https://img.shields.io/badge/rust-1.88.0-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Overview

Journal is an interactive CLI tool that provides empathetic coaching conversations for morning and evening reflection sessions. It combines the power of AI-driven conversational coaching with the robust document management capabilities of Aethel, creating a comprehensive journaling experience that preserves your thoughts and insights in a structured, searchable format.

### Key Features

- **Interactive Coaching**: AI-powered coaching conversations tailored for morning intention-setting and evening reflection
- **Session Management**: Seamless session resumption and state tracking
- **AI Analysis**: Automatic generation of insights, themes, and actionable items from your journal sessions
- **Structured Storage**: All entries stored as structured documents using Aethel's vault system
- **Git-Friendly**: Version control ready with plain Markdown files
- **Privacy-First**: All data stored locally in your vault

## Architecture

The project consists of two main components:

### 1. Aethel Document Management System (`external/aethel/`)
A radically minimal document management system providing:
- **Doc**: Markdown files with YAML front-matter, identified by UUID
- **Pack**: Schema definitions and templates for document types
- **Patch**: JSON-based document mutation operations

### 2. Journal Application (`src/journal/`)
An interactive CLI application built using Elm-like architecture:
- **State**: Application state management (Initializing, InSession, etc.)
- **Action**: Event-driven state transitions
- **Effects**: Side effect handling (AI calls, file operations, etc.)
- **Update**: Pure state transition logic

## Quick Start

### Prerequisites

- Rust 1.88.0 (managed via [mise](https://mise.jdx.dev/))
- [Claude CLI](https://github.com/anthropics/claude-cli) for AI coaching

### Installation

```bash
# Clone the repository
git clone https://github.com/bkase/journal.git
cd journal

# Install Rust toolchain
mise install

# Build the project
make build
```

### Usage

```bash
# Start a new journaling session
journal

# The application will:
# 1. Check for existing sessions and offer to resume
# 2. Prompt you to choose morning or evening mode
# 3. Guide you through an interactive coaching conversation
# 4. Generate AI analysis of your session
# 5. Save everything as structured documents in your vault
```

### Session Types

- **Morning Sessions**: Focus on intention-setting, goal planning, and positive momentum
- **Evening Sessions**: Emphasize reflection, learning extraction, and gratitude practice

## Development

### Build Commands

```bash
make build          # Build workspace in debug mode
make release        # Build workspace in release mode
make test           # Run all tests
make fmt            # Check code formatting
make fmt-fix        # Auto-format code
make lint           # Run clippy linting
make clean          # Clean build artifacts
```

### Project Structure

```
journal/
├── external/aethel/        # Aethel document management system
│   ├── crates/
│   │   ├── aethel-core/    # L0: Core library
│   │   └── aethel-cli/     # L1: CLI interface
│   └── tests/cases/        # Golden test fixtures
├── src/journal/            # Journal application
│   └── src/
│       ├── action.rs       # Action definitions and input parsing
│       ├── effects.rs      # Side effect handling
│       ├── main.rs         # Application entry point and main loop
│       ├── state.rs        # State management and session types
│       └── update.rs       # Pure state transition logic
└── docs/                   # Project documentation
    ├── project-setup.md    # Detailed development guide
    └── spec.md            # Software design document
```

### Testing

The project includes comprehensive test coverage:

- **Unit Tests**: Component-level testing for all modules
- **Integration Tests**: End-to-end CLI command testing
- **Golden Tests**: Deterministic snapshot testing for Aethel operations

```bash
# Run all tests
make test

# Run tests with coverage
cargo test

# Update golden test snapshots (for Aethel)
cd external/aethel && UPDATE_GOLDEN=1 cargo test
```

## How It Works

### Session Flow

1. **Initialization**: Check for active sessions in the Aethel vault
2. **Mode Selection**: Choose between morning or evening journaling
3. **Interactive Conversation**: Engage in AI-coached reflection
4. **Analysis Generation**: AI analyzes your session for insights and patterns
5. **Document Creation**: Final entry saved with transcript and analysis

### Data Storage

All journal data is stored in your local Aethel vault as structured documents:

- **Session Documents** (`journal.session`): Temporary session state and transcripts
- **Entry Documents** (`journal.entry`): Final journal entries with AI analysis
- **Index Files**: Track active sessions for seamless resumption

### AI Integration

The application integrates with Claude CLI for:
- **Coaching Conversations**: Context-aware questions and responses
- **Session Analysis**: Deep analysis of emotional journeys and insights
- **Action Item Generation**: Specific, actionable recommendations

## Philosophy

Journal embraces the same principles as Aethel:

- **No Overbuilding**: Implement only what's needed for effective journaling
- **Local-First**: All data stays on your machine
- **Git-Friendly**: Plain text files that work perfectly with version control
- **Privacy-Focused**: No data sent to servers except AI coaching calls
- **Structured but Flexible**: Schema validation with room for personal expression

## Configuration

### Environment Variables

- `JOURNAL_VAULT_PATH`: Path to your Aethel vault (default: `~/journal-vault`)

### Vault Structure

```
your-vault/
├── .aethel/
│   ├── indexes/
│   │   └── journal.index.json    # Active session tracking
│   └── config.json               # Vault configuration
├── docs/                         # Your journal entries
├── packs/
│   └── journal@0.1.0/           # Journal pack with schemas
└── sessions/                     # Temporary session documents
```

## Contributing

We welcome contributions! Please ensure:

1. All tests pass (`make test`)
2. Code is formatted (`make fmt-fix`)
3. No clippy warnings (`make lint`)
4. New features include appropriate tests

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- Built on the [Aethel](https://github.com/aethel/aethel) document management system
- Powered by [Claude](https://claude.ai) for AI coaching capabilities
- Inspired by reflective journaling practices and emotional intelligence research