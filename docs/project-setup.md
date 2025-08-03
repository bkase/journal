Of course. Providing a precise setup guide is crucial for team alignment and efficient onboarding. Drawing inspiration from the `Momentum` project's robust Rust setup, here is a detailed `project-setup.md` document for the `journal` tool.

This guide covers tooling, local development workflows, and continuous integration, ensuring that every engineer works from a consistent and reproducible environment.

---

# **Project Setup & Engineering Guide: `journal`**

- **Version:** 1.0
- **Date:** 2025-08-02
- **Purpose:** To provide a single, authoritative guide for setting up the development environment, running builds, tests, and understanding the CI/CD workflow for the `journal` project.

## 1. Prerequisites

Before you begin, you must have the following tools installed on your system:

1. **Git:** For version control.
2. **[Mise](https://mise.jdx.dev/):** For managing project-specific tool versions. This is the source of truth for our toolchain. Follow the installation instructions on their website.

## 2. Initial Project Setup

To get the project running for the first time, follow these steps:

1. **Install Project Tools:**
   Run the following command in the project root. `mise` will automatically read the `.mise.toml` file and install the exact versions of Rust and other tools required for this project.

   ```bash
   mise install
   ```

   This single command ensures your environment is perfectly configured.

## 3. Tooling Configuration (`.mise.toml`)

This file is the single source of truth for all tool versions used in the project. It prevents "it works on my machine" issues.

**File Location:** `/.mise.toml`

**Content:**

```toml
# .mise.toml
[tools]
# Specifies the exact Rust version for consistent builds.
rust = "1.88.0"

[env]
# Enables detailed backtraces for panics, aiding in debugging.
RUST_BACKTRACE = "1"
```

## 4. Build & Development (`Makefile`)

The `Makefile` is the canonical entry point for all common development tasks. It ensures that both local development and CI use the exact same commands.

**File Location:** `/Makefile`

### Makefile Targets

- `make all`: The default task. Builds, formats, lints, and tests the entire project.
- `make build`: Compiles the project in debug mode.
- `make release`: Compiles the project in release mode with optimizations.
- `make test`: Runs the full test suite.
- `make fmt`: Checks if the code is formatted correctly (used in CI).
- `make fmt-fix`: Automatically formats the code. Run this before committing.
- `make lint`: Runs `clippy` to check for common mistakes and unidiomatic code.
- `make clean`: Removes all build artifacts.
- `make run`: Builds and runs the CLI application.

### Makefile Content

````makefile
# /Makefile

# Ensure mise is activated for all commands
SHELL := /bin/bash
.ONESHELL:
.SHELLFLAGS := -c
.DEFAULT_GOAL := all

# Phony targets don't represent files
.PHONY: all build release check fmt fmt-fix lint test clean run install-tools

# Default target: build, format check, lint, test
all: build fmt lint test

# Install required rust components
install-tools:
 @echo ">> Installing Rust components (rustfmt, clippy)..."
 @eval "$$(mise activate bash)" && rustup component add rustfmt clippy

# Build the workspace
build:
 @echo ">> Building project (debug)..."
 @eval "$$(mise activate bash)" && cargo build --workspace

# Build the workspace in release mode
release:
 @echo ">> Building project (release)..."
 @eval "$$(mise activate bash)" && cargo build --workspace --release

# Check for compilation errors without building binaries
check:
 @echo ">> Checking for compilation errors..."
 @eval "$$(mise activate bash)" && cargo check --workspace

# Check formatting
fmt:
 @echo ">> Checking Rust formatting..."
 @eval "$$(mise activate bash)" && cargo fmt --all -- --check

# Auto-format code
fmt-fix:
 @echo ">> Formatting Rust code..."
 @eval "$$(mise activate bash)" && cargo fmt --all

# Run clippy for linting. `-D warnings` treats warnings as errors.
lint:
 @echo ">> Linting with clippy..."
 @eval "$$(mise activate bash)" && cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all tests in the workspace
test:
 @echo ">> Running tests..."
 @eval "$$(mise activate bash)" && cargo test --workspace

# Clean build artifacts
clean:
 @echo ">> Cleaning build artifacts..."
 @eval "$$(mise activate bash)" && cargo clean

# Build and run the main binary
run: build
 @echo ">> Running journal..."
 @eval "$$(mise activate bash)" && cargo run```

## 5. Continuous Integration (GitHub Actions)

Our CI workflow validates every push and pull request to the `main` branch. It mirrors the local development setup to ensure consistency.

**File Location:** `/.github/workflows/ci.yml`

### CI Workflow Details

*   **Trigger:** Runs on pushes and pull requests to the `main` branch.
*   **Environment:** Uses `ubuntu-latest` runner.
*   **Steps:**
    1.  Checks out the code.
    2.  Installs `mise` using the official `jdx/mise-action`.
    3.  Installs the project's tool versions using `mise install`.
    4.  Installs `rustfmt` and `clippy`.
    5.  Caches `cargo` dependencies to speed up subsequent runs.
    6.  Runs `make fmt` to enforce code style.
    7.  Runs `make lint` to enforce code quality.
    8.  Runs `make test` to execute all tests.
    9.  Runs `make build` to ensure the project compiles.

### Workflow File Content

```yaml
# /.github/workflows/ci.yml

name: Rust CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  build_and_test:
    name: Build & Test
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install mise
        uses: jdx/mise-action@v2

      - name: Install tool versions from .mise.toml
        run: mise install

      - name: Install Rust components
        run: |
          eval "$(mise activate bash)"
          rustup component add rustfmt clippy

      - name: Cache Cargo dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: make fmt

      - name: Lint with Clippy
        run: make lint

      - name: Run tests
        run: make test

      - name: Build project
        run: make build
````

## 6. Onboarding Checklist for Engineers

- [ ] Installed `mise` on your local machine.
- [ ] Ran `mise install` in the project root to get the correct Rust version.
- [ ] Ran `make install-tools` to ensure `rustfmt` and `clippy` are available.
- [ ] Successfully ran `make all` without any errors.
- [ ] Configured your editor (e.g., VS Code with `rust-analyzer`).

