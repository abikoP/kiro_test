# Technology Stack

## Language & Runtime
- **Rust** (Edition 2024)
- Modern Rust features and syntax

## Build System
- **Cargo** - Rust's package manager and build tool

## Project Configuration
- Package name: `kiro_test`
- Version: `0.1.0`
- Currently minimal dependencies (ready for web framework integration)

## Common Commands

### Development
```bash
# Build the project
cargo build

# Run the application
cargo run

# Build for release
cargo build --release

# Check code without building
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Dependencies
```bash
# Add a new dependency
cargo add <package_name>

# Update dependencies
cargo update
```

## Expected Tech Stack Additions
Given the web-based YAML editing purpose, likely future additions:
- Web framework (e.g., Axum, Warp, or Actix-web)
- YAML parsing library (e.g., serde_yaml)
- Serialization framework (serde)
- Web templating or API framework