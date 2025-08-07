# changement

Changement is a Rust CLI application for managing versioning and publishing packages in monorepo projects. It provides commands for initializing change tracking, creating changes, versioning, publishing, and tagging packages.

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Prerequisites
- Rust toolchain (rustc 1.88.0+ and cargo 1.88.0+) - should already be available in most environments
- If Rust is not installed, use: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` and restart terminal

### Build and Test Commands
- **Build debug version**: `cargo build --verbose` -- takes 25 seconds on fresh build. NEVER CANCEL. Set timeout to 60+ seconds.
- **Build release version**: `cargo build --release` -- takes 7 seconds. NEVER CANCEL. Set timeout to 30+ seconds.
- **Run tests**: `cargo test --verbose` -- takes <1 second. Set timeout to 30+ seconds.
- **Check formatting**: `cargo fmt --all -- --check` -- takes <2 seconds. Set timeout to 30+ seconds.
- **Run linting**: `cargo clippy --all-targets --all-features -- -D warnings` -- takes 4 seconds. NEVER CANCEL. Set timeout to 30+ seconds.

### Installation and Usage
- **Install locally**: `cargo install --path . --force` -- takes 9 seconds. NEVER CANCEL. Set timeout to 60+ seconds.
- **Run the CLI**: `./target/debug/changement --help` or `changement --help` (if installed)
- **Test functionality**: `changement init` in a test directory to verify the tool works

## Validation

### Always Run These Steps Before Committing
1. **Format check**: `cargo fmt --all -- --check` - REQUIRED by CI
2. **Lint check**: `cargo clippy --all-targets --all-features -- -D warnings` - REQUIRED by CI  
3. **Run tests**: `cargo test --verbose` - REQUIRED by CI
4. **Build verification**: `cargo build --verbose` - REQUIRED by CI

### Manual Testing Scenarios
ALWAYS manually test the CLI functionality after making changes:
1. Build the project: `cargo build`
2. Test help command: `./target/debug/changement --help`
3. Test init command in a temporary directory:
   ```bash
   mkdir /tmp/test-changement && cd /tmp/test-changement
   /path/to/target/debug/changement init
   ls -la .changes/
   cat .changes/config.json
   ```
4. Verify the init command creates:
   - `.changes/` directory
   - `.changes/config.json` with version 1 and empty ignore array

### CI Pipeline Validation
The GitHub Actions CI (.github/workflows/ci.yml) runs 4 jobs that MUST pass:
- **test**: Runs `cargo test --verbose`
- **fmt**: Runs `cargo fmt --all -- --check`  
- **clippy**: Runs `cargo clippy --all-targets --all-features -- -D warnings`
- **build**: Runs `cargo build --verbose`

## Repository Structure

### Key Files and Directories
- `src/main.rs` - Main CLI application code with commands and logic
- `Cargo.toml` - Rust project configuration and dependencies
- `Cargo.lock` - Locked dependency versions (do not modify manually)
- `.github/workflows/ci.yml` - CI pipeline configuration
- `.github/dependabot.yml` - Dependency update automation
- `README.md` - Project documentation and usage examples

### Source Code Organization
```
src/
├── main.rs          # CLI parser, main function, and command implementations
```

### Current CLI Commands
- `init` - Initialize changement in a new project (creates .changes/ directory and config.json)
- Only the init command is currently implemented

### Dependencies
- `clap` v4.0+ with derive features for CLI parsing - this is the only runtime dependency

## Development Workflow

### Making Changes
1. Always build and test first to ensure baseline works: `cargo build && cargo test`
2. Make your changes to `src/main.rs` or add new files as needed
3. Build incrementally: `cargo build` (faster on incremental builds)
4. Test your changes: `cargo test`
5. Run validation commands: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`
6. Test CLI functionality manually as described in validation section

### Adding New Commands
- Add new command variants to the `Commands` enum in `src/main.rs`
- Implement command logic as new functions following the `init_command()` pattern
- Add match arms in `main()` function to handle new commands
- Add tests in the `#[cfg(test)]` module

### Common File Outputs

#### Repository Root Contents
```
.
├── .git/
├── .github/
│   ├── dependabot.yml
│   └── workflows/
│       └── ci.yml
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── README.md
├── src/
│   └── main.rs
└── target/          # Build outputs (git ignored)
```

#### Cargo.toml
```toml
[package]
name = "changement"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
```

## Troubleshooting

### Build Issues
- If build fails with dependency errors, try `cargo clean && cargo build`
- If Rust edition 2024 is not supported, it may indicate an older Rust version

### Test Issues  
- Current test suite only has one placeholder test (`test_example`)
- Tests run very quickly (<1 second) so timeouts should not be an issue

### CLI Issues
- If binary doesn't work after install, check that `~/.cargo/bin` is in PATH
- Use `./target/debug/changement` for local testing without installing

## Performance Notes
- Initial build downloads dependencies and takes ~25 seconds
- Incremental builds are much faster (~1-3 seconds)
- Release builds take ~7 seconds and produce optimized binaries
- All operations are lightweight - no heavy computation or I/O