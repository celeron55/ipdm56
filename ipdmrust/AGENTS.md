# Repository Guidelines

## Project Structure & Module Organization

This is a Rust workspace for iPDM56 v2 firmware, consisting of:
- **common**: Shared utilities and core logic.
- **embedded**: Firmware for the STM32F407 microcontroller.
- **desktop**: Simulation and testing environment.
- **app**: Application-specific code (symlinked to app_example by default).

Source code resides in `src/` directories within each crate. Binaries are built to `target/`.

## Build, Test, and Development Commands

- **Build debug**: `cargo build --exclude embedded`
- **Build release**: `cargo build --release --exclude embedded`
- **Embedded release**: `cd embedded && ./build_release.sh`
- **Test**: `cargo test --exclude embedded`
- **Run desktop**: `cd desktop && cargo run`
- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy --workspace --exclude embedded`

Use `bacon` for continuous compilation during development.

## Coding Style & Naming Conventions

Follow Rust standards: snake_case for functions/variables, CamelCase for types/structs.
- Indent with 4 spaces.
- Use `cargo fmt` for automatic formatting.
- Enable `cargo clippy` for additional checks.

Avoid inline comments; prefer descriptive names.

## Testing Guidelines

- Place unit tests in `src/` alongside code; integration tests in `tests/`.
- Run tests with `cargo test --release --target=x86_64-unknown-linux-gnu -- --test-threads=1` for cross-compilation.
- Focus on functional tests for embedded logic; mock hardware where possible.

No coverage requirements; ensure critical paths are tested.

## Commit & Pull Request Guidelines

- Use descriptive messages, e.g., "ipdmrust: Add A/C compressor control".
- Prefix with "ipdmrust:" for firmware-related changes.
- Keep commits focused and atomic.

For pull requests: Provide clear descriptions, link issues, include screenshots/logs for UI changes.

## Architecture Overview

Embedded Rust firmware using cortex-m for ARM Cortex-M4 (STM32F407). Desktop crate simulates with Piston for benchmarking. Shared common crate for reusable components. Flashing via DFU, UART, or ST-Link.

(Word count: 342)
