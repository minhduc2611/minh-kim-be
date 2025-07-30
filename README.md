# Rust

## Rust Version Management

This project uses specific Rust version management to ensure consistency across development environments:

### 🔧 `rust-toolchain.toml`
- **Purpose**: Specifies the exact Rust version to use (like `.nvmrc` for Node.js)
- **Auto-switching**: `rustup` automatically uses the specified version when you're in this directory
- **Team consistency**: Everyone gets the same Rust version, components, and targets
- **Current version**: `1.88.0` with `rustfmt` and `clippy` components

### 📋 `rust-version` in Cargo.toml
- **Purpose**: Defines the minimum Rust version required for this project
- **Compatibility**: Used by `cargo` for version compatibility checking
- **Current minimum**: `1.75.0`

### 🚀 Getting Started
When you clone this project and run any `rustup` command, it will automatically:
1. Detect the `rust-toolchain.toml` file
2. Install Rust 1.88.0 if you don't have it
3. Switch to using that version for this project

## Update Rust

```bash
rustup update
```

## Run

```bash
cargo run
```