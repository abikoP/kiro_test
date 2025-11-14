# Project Structure

## Root Directory Layout
```
kiro_test/
├── Cargo.toml          # Project configuration and dependencies
├── .gitignore          # Git ignore rules (excludes /target)
├── src/                # Source code directory
│   └── main.rs         # Application entry point
├── .git/               # Git repository data
├── .kiro/              # Kiro IDE configuration and steering
└── .vscode/            # VS Code configuration
```

## Source Organization
- **`src/main.rs`** - Main application entry point with basic "Hello, world!" implementation
- **`src/`** - All Rust source code should be organized under this directory

## Standard Rust Conventions
- Follow standard Rust project layout
- Use `src/lib.rs` for library crates
- Place modules in `src/` directory or subdirectories
- Use `tests/` directory for integration tests
- Use `examples/` directory for example code
- Place documentation in `docs/` if needed

## Build Artifacts
- **`target/`** - Cargo build output (git-ignored)
- All compiled binaries and intermediate files are placed here

## Configuration Files
- **`Cargo.toml`** - Primary project configuration
- **`.gitignore`** - Currently excludes build artifacts
- IDE-specific configurations in `.vscode/` and `.kiro/`

## Future Structure Considerations
As a web-based YAML editor, consider organizing:
- Static web assets (HTML, CSS, JS) in `static/` or `assets/`
- Templates in `templates/` directory
- Configuration files in `config/`
- API routes in `src/routes/` or similar modular structure