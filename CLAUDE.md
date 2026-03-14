# CLAUDE.md

## What is dex?

A Rust CLI that scans any codebase and generates a `.dex/` context directory — structured TOML files describing the project's languages, frameworks, structure, entry points, and public APIs.

## Architecture

Single-pass scanner: walk file tree → detect languages → parse manifests → analyze structure → find entry points → serialize to TOML.

```
src/
├── main.rs              # CLI (clap) — delegates to scanner + output
├── scanner/
│   ├── mod.rs           # Orchestrator: walk tree, collect all data, build result
│   ├── languages.rs     # File extension → language mapping
│   ├── manifests.rs     # Parse Cargo.toml, package.json, go.mod, pyproject.toml
│   ├── structure.rs     # Detect source/test/config roots and project style
│   └── entry_points.rs  # Find main files, bin targets, public API surface
├── models/
│   ├── context.rs       # Serde structs for context.toml
│   └── paths.rs         # Serde structs for paths.toml
└── output.rs            # Write .dex/ directory
```

## Commands

```bash
cargo test              # Run all 27 tests (unit + integration)
cargo clippy -- -D warnings  # Lint (must pass with zero warnings)
cargo fmt --check       # Format check
cargo run -- init .     # Run dex on itself
```

## Key decisions

- No tree-sitter in v0.1.0 — only file extensions, directory patterns, and manifest parsing
- TOML for output (human-readable, git-friendly)
- `ignore` crate for file walking (respects .gitignore)
- Edition 2024 Rust

## Testing

- Unit tests are colocated in each module (`#[cfg(test)]`)
- Integration tests in `tests/integration_test.rs` use `assert_cmd` against fixture projects in `tests/fixtures/`
- Fixtures cover: Rust CLI, TypeScript web app, Python library, Go service, Rust workspace/monorepo
