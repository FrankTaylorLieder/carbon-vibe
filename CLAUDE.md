# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust project called "carbon-vibe" using the 2024 edition. It's currently a minimal setup with a basic "Hello, world!" program.

## Common Commands

### Building and Running
- `cargo build` - Build the project
- `cargo run` - Build and run the project
- `cargo check` - Check the project for errors without building
- `cargo clippy` - Run the Rust linter
- `cargo fmt` - Format the code

### Testing
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a specific test

## Project Structure

- `src/main.rs` - The main entry point of the application
- `Cargo.toml` - Project configuration and dependencies

## Architecture

This is currently a minimal Rust binary project with no external dependencies. The project structure follows standard Rust conventions with the main entry point in `src/main.rs`.

## References
- Carbon Intensity API v2.0.0 Documentation: https://carbon-intensity.github.io/api-definitions/#carbon-intensity-api-v2-0-0

## Development Workflow

- Remember to update the DEVELOPMENT LOG for all changes as we work on this project.