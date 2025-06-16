# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Building
- `cargo build` - Build debug version of wasmtime CLI at `target/debug/wasmtime`
- `cargo build --release` - Build optimized version at `target/release/wasmtime`
- `cargo build -p <crate-name>` - Build a specific crate (e.g., `cargo build -p wasmtime-wasi`)

### Testing
- `cargo test` - Run tests in `tests/` folder and spec tests (good starting point)
- `ci/run-tests.py` - Run full test suite matching CI (excludes problematic crates)
- `cargo test -p <crate-name>` - Test specific crate (e.g., `cargo test -p wasmtime-wasi`)
- `cargo test -p cranelift-tools` - Test Cranelift filetests
- Requires wasm32 targets: `rustup target add wasm32-wasip1 wasm32-unknown-unknown`

### Formatting and Linting
- `cargo fmt --all` - Format all code using rustfmt
- `scripts/format-all.sh` - Format all sources (wrapper around cargo fmt)
- `cargo fmt --all -- --check` - Check formatting without modifying files

### Prerequisites
- Initialize git submodules: `git submodule update --init`
- Install Rust toolchain via rustup
- Optional: libclang for fuzzing infrastructure

## Architecture Overview

Wasmtime is a WebAssembly runtime built on the Cranelift code generator. The codebase is organized into several key components:

### Core Architecture
- **wasmtime crate**: Main safe API for WebAssembly execution
- **wasmtime-environ**: Compilation environment and module artifacts
- **wasmtime-cranelift**: Cranelift-based compiler backend
- **wasmtime-runtime**: Low-level runtime implementation with VM context

### Key Concepts
- **Engine**: Global compilation context, thread-safe, stores configuration
- **Store**: WebAssembly "store" containing instances, memories, tables (single-threaded)
- **InstanceHandle**: Low-level representation of WebAssembly instances
- **VMContext**: Raw pointer to JIT-accessible module state

### Component Model
- **wasmtime-component-***: WebAssembly Component Model implementation
- **wit-bindgen**: WIT (WebAssembly Interface Types) bindings generation

### WASI Implementation
- **wasmtime-wasi**: WASI Preview 2 implementation
- **wasi-common**: WASI Preview 1 compatibility layer
- **wasmtime-wasi-***: Specialized WASI modules (http, nn, threads, etc.)

### Compilation Pipeline
1. **wasmtime-environ**: Parse and validate WebAssembly modules
2. **wasmtime-cranelift**: Translate to Cranelift IR and compile to machine code
3. **wasmtime-runtime**: Execute compiled code with runtime support

### Code Organization
- `crates/` - All Wasmtime crates organized by functionality
- `cranelift/` - Cranelift code generator (separate from main wasmtime)
- `tests/` - Integration tests and test suites
- `examples/` - Usage examples in multiple languages
- `docs/` - Comprehensive documentation

### Alternative Backends
- **winch**: Fast baseline compiler for quick compilation
- **pulley**: Portable bytecode interpreter

## Development Workflow

### Single Test Execution
Find specific test files in `tests/` directory and run with:
```bash
cargo test <test-name>
```

### Debugging
- Use `target/debug/wasmtime` for debug builds with more information
- Profiling available via `perf`, `vtune`, or built-in profiling features

### Crate Structure
The repository follows Rust's workspace model with internal crates marked as `wasmtime-internal-*` on crates.io to indicate they're not public APIs.