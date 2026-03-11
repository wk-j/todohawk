# TODO Tracker CLI

## Project Overview
A Rust CLI tool that scans codebases to find and track TODO, FIXME, HACK, XXX, and other annotation comments. It helps developers manage technical debt by providing visibility into scattered code annotations.

## Architecture

### Directory Structure
```
src/
  main.rs          - Entry point, CLI argument parsing (clap derive)
  scanner.rs       - File walking and TODO detection logic
  parser.rs        - Comment parsing, pattern matching, tag extraction
  reporter.rs      - Output formatting (table, json, markdown, summary)
  config.rs        - Configuration loading (.todorc, CLI args)
  types.rs         - Shared types (TodoItem, Priority, Tag, etc.)
tests/
  integration/     - End-to-end CLI tests
  fixtures/        - Sample files with various TODO patterns
```

### Key Features (target)
1. **Scan**: Recursively scan directories for TODO-style comments
2. **Filter**: Filter by tag (TODO, FIXME, HACK), author, file pattern, priority
3. **Report**: Output as table (default), JSON, markdown, or summary count
4. **Config**: Support `.todorc` config file for project-specific settings
5. **Ignore**: Respect `.gitignore` and custom ignore patterns

### Supported Comment Patterns
- `// TODO: message` / `// TODO(author): message`
- `# TODO: message` / `# FIXME: message`
- `/* TODO: message */`
- `<!-- TODO: message -->`
- Tags: TODO, FIXME, HACK, XXX, NOTE, OPTIMIZE, BUG, WARN

## Technology
- **Language**: Rust (edition 2024)
- **CLI framework**: clap v4 with derive macros
- **File walking**: walkdir
- **Pattern matching**: regex
- **Output coloring**: colored
- **Serialization**: serde + serde_json
- **Error handling**: anyhow
- **Testing**: assert_cmd + predicates + tempfile

## Conventions
- Use `anyhow::Result` for error handling in application code
- Use thiserror for library-level error types if needed
- Keep modules focused: one responsibility per file
- Write unit tests in each module, integration tests in `tests/`
- Use `cargo fmt` and `cargo clippy` before committing
- Follow Rust 2024 edition idioms

## Build & Test Commands
```bash
cargo build                  # Build
cargo run -- --help          # Run with help
cargo test                   # Run all tests
cargo clippy                 # Lint
cargo fmt --check            # Check formatting
```
