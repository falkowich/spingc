.PHONY: all check fmt lint build release windows clean

# Default target
all: check build

# Check that it compiles without building
check:
	cargo check

# Format code with rustfmt
fmt:
	cargo fmt

# Lint with clippy
lint:
	cargo clippy -- -D warnings

# Debug build
build:
	cargo build

# Release build (optimized)
release:
	cargo build --release

# Cross-compile for Windows
windows:
	cross build --target x86_64-pc-windows-gnu --release

# Format + lint + release build
ship: fmt lint release windows

# Remove build artifacts
clean:
	cargo clean
