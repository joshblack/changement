# changement

Changement is a Rust-based CLI tool for managing versioning and publishing of packages in monorepo projects. The project is currently in early development stage with core CLI functionality planned but not yet implemented.

**Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.**

## Working Effectively

### Bootstrap and Build
- Ensure Rust is installed: `rustc --version` (requires Rust 1.88.0 or later)
- Ensure Cargo is installed: `cargo --version` (requires Cargo 1.88.0 or later)
- Build the project: `cargo build` -- takes under 0.2 seconds. NEVER CANCEL.
- Build optimized release: `cargo build --release` -- takes under 0.1 seconds. NEVER CANCEL.
- Run the application: `cargo run` or directly execute `./target/debug/changement`

### Testing and Quality Assurance
- Run tests: `cargo test` -- takes under 0.1 seconds. NEVER CANCEL. Currently no tests exist.
- Run linter: `cargo clippy` -- takes under 0.2 seconds. NEVER CANCEL.
- Check code formatting: `cargo fmt --check` -- takes under 0.2 seconds. NEVER CANCEL.
- Auto-format code: `cargo fmt`

### Development Workflow
- **CRITICAL**: The current implementation only prints "Hello, world!" despite README describing full CLI functionality
- **IMPORTANT**: The CLI features described in README.md (init, new, version, publish, tag commands) are not yet implemented
- Always build and test after making changes: `cargo build && cargo test && cargo clippy`
- Always format code before committing: `cargo fmt`

## Validation

### Manual Testing Scenarios
Since the CLI features are not yet implemented, validation currently consists of:
1. **Basic Build Validation**: Run `cargo build` and verify successful compilation
2. **Execution Test**: Run `./target/debug/changement` and verify "Hello, world!" output
3. **Quality Checks**: Run `cargo clippy` and `cargo fmt --check` with no errors

### Future Validation (when CLI is implemented)
When the CLI functionality described in README is implemented, always test:
1. **Init Command**: `changement init` in a new directory
2. **New Command**: `changement new -p test-package -m "Test change" -b minor`
3. **Configuration**: Verify `.changes/config.json` is created and valid
4. **Change Files**: Verify `.changes/` directory contains expected markdown files

## Project Structure

### Repository Root
```
.
├── .git/
├── .gitignore          # Excludes /target and .DS_Store
├── Cargo.toml          # Rust project configuration
├── Cargo.lock          # Dependency lock file
├── README.md           # Project documentation
├── src/
│   └── main.rs         # Main application entry point (currently Hello World)
└── target/             # Build artifacts (gitignored)
```

### Key Files
- **Cargo.toml**: Rust project configuration, currently minimal with no dependencies
- **src/main.rs**: Main application code, currently only contains `println!("Hello, world!");`
- **README.md**: Comprehensive documentation of planned CLI features and configuration

## Common Commands Reference

### Quick Reference Outputs
The following are outputs from frequently run commands to save time:

#### Repository Listing
```bash
ls -la
# Output:
# .git/
# .gitignore          # Excludes /target and .DS_Store  
# Cargo.toml          # Rust project configuration
# Cargo.lock          # Dependency lock file
# README.md           # Project documentation
# src/                # Source code directory
# target/             # Build artifacts (after building)
```

#### Cargo.toml Contents
```toml
[package]
name = "changement"
version = "0.1.0"
edition = "2024"

[dependencies]
# No dependencies currently
```

#### Current Application Output
```bash
cargo run
# Output: Hello, world!
```

### Build Commands
```bash
# Debug build (fast, unoptimized)
cargo build                    # ~0.05 seconds (subsequent builds), ~0.16 seconds (from clean)

# Release build (optimized)  
cargo build --release          # ~0.05 seconds (subsequent builds)

# Clean build artifacts
cargo clean                    # Removes ~13.8MB build artifacts
```

### Test Commands
```bash
# Run all tests
cargo test                     # ~0.05 seconds (currently 0 tests)

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Quality Commands
```bash
# Run clippy linter
cargo clippy                   # ~0.14 seconds

# Check formatting
cargo fmt --check              # ~0.15 seconds

# Auto-format code
cargo fmt                      # ~0.1 seconds
```

### Run Commands
```bash
# Run in debug mode
cargo run                      # Prints "Hello, world!"

# Run with arguments (when CLI is implemented)
cargo run -- --help
cargo run -- init
cargo run -- new -p package -m "message" -b minor
```

## Development Guidelines

### Code Quality
- Always run `cargo fmt` before committing
- Always run `cargo clippy` and fix any warnings
- Write tests for new functionality when implementing CLI features
- Follow Rust naming conventions and best practices

### Important Notes
- **BUILD TIMES**: All builds are extremely fast (<0.2 seconds). No timeouts needed.
- **NO CI/CD**: The repository currently has no GitHub Actions or CI/CD configured
- **IMPLEMENTATION GAP**: The README describes full CLI functionality that is not yet implemented
- **PACKAGE MANAGEMENT**: Despite README mentioning `npm i -g changement`, this is a Rust project, not a Node.js package

### When Implementing CLI Features
The README describes the following planned commands that need implementation:
- `init`: Initialize changement in a new project  
- `new`: Create a new change for a package (`-p package`, `-m message`, `-b bump-type`)
- `version`: Apply all changes and update package versions (`--filter` option)
- `publish`: Publish packages to registry
- `tag`: Create git tags for current versions (`--filter` option)

Configuration should be stored in `.changes/config.json` with schema for version and ignore fields.

## Troubleshooting

### Common Issues
- **Cargo not found**: Install Rust via rustup.rs or package manager
- **Build failures**: Run `cargo clean` then `cargo build`
- **Permission errors**: Ensure write access to project directory

### Performance Notes
- All operations are extremely fast (<0.2 seconds)
- No network dependencies in current implementation
- Build artifacts in `target/` directory can be safely deleted with `cargo clean`