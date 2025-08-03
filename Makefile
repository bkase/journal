# Journal Project Makefile
#
# This Makefile provides commands for building, testing, and running the journal application.
# It follows the conventions established in the Aethel project with additional journal-specific targets.

.PHONY: all build release test fmt fmt-fix lint clean run run-release help

# Default target: build, format, lint, and test
all: build fmt lint test

# Build targets
build:
	@echo "Building journal project (debug)..."
	cargo build

release:
	@echo "Building journal project (release)..."
	cargo build --release

# Test targets
test:
	@echo "Running all tests..."
	cargo test

# Optional enhanced test runner (if nextest is available via mise)
nextest:
	@echo "Running tests with nextest..."
	mise exec -- cargo nextest run

# Formatting targets
fmt:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

fmt-fix:
	@echo "Auto-formatting code..."
	cargo fmt --all

# Linting targets
lint:
	@echo "Running clippy lints..."
	cargo clippy --all-targets --all-features -- -D warnings

# Clean target
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Run targets
run: build
	@echo "Running journal application..."
	cargo run --bin journal

run-release: release
	@echo "Running journal application (release)..."
	cargo run --release --bin journal

# Optional security audit (if available via mise)
audit:
	@echo "Running security audit..."
	mise exec -- cargo audit

# Help target
help:
	@echo "Available targets:"
	@echo "  all         - Build, format, lint, and test (default)"
	@echo "  build       - Build project in debug mode"
	@echo "  release     - Build project in release mode"
	@echo "  test        - Run all tests"
	@echo "  nextest     - Run tests with nextest (if available)"
	@echo "  fmt         - Check code formatting"
	@echo "  fmt-fix     - Auto-format code"
	@echo "  lint        - Run clippy lints"
	@echo "  clean       - Clean build artifacts"
	@echo "  run         - Build and run journal application"
	@echo "  run-release - Build and run journal application (release)"
	@echo "  audit       - Run security audit (if available)"
	@echo "  help        - Show this help message"