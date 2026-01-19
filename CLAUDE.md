# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`yun-socket-proxy` is a Rust-based socket proxy application. The project is currently in early development stages.

## Development Commands

### Building
```bash
cargo build              # Debug build
cargo build --release    # Release build with optimizations
```

### Running
```bash
cargo run                # Run in debug mode
cargo run --release      # Run optimized release build
```

### Testing
```bash
cargo test               # Run all tests
cargo test <test_name>   # Run a specific test
cargo test -- --nocapture # Run tests with stdout visible
```

### Code Quality
```bash
cargo check              # Fast compilation check without producing binary
cargo clippy             # Run linter for common mistakes and improvements
cargo fmt                # Format code according to Rust style guidelines
cargo fmt -- --check     # Check formatting without modifying files
```

## Project Structure

- **src/main.rs**: Entry point for the application
- **Cargo.toml**: Project manifest with dependencies and metadata

## Architecture Notes

This is a socket proxy project built with Rust. As the codebase develops, key architectural components will likely include:

- Network socket handling and connection management
- Proxy logic for forwarding traffic between clients and servers
- Async runtime (likely tokio or async-std) for concurrent connection handling
- Configuration management for proxy settings

The project uses Rust edition 2024, which requires a recent Rust toolchain.
