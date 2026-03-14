# dex

[![CI](https://github.com/dexs-oss/dex/actions/workflows/ci.yml/badge.svg)](https://github.com/dexs-oss/dex/actions/workflows/ci.yml)

**Codebase context protocol — generate `.dex/` for any project.**

`dex` scans your codebase and generates a `.dex/` directory containing a structured, machine-readable map of your project. Both humans and AI agents can use it to quickly understand architecture, conventions, and critical paths.

## Install

```bash
cargo install dex
```

## Usage

```bash
# Generate .dex/ for the current directory
dex init

# Generate .dex/ for a specific path
dex init /path/to/project
```

## What it generates

```
.dex/
├── README          # Explains the .dex/ directory
├── context.toml    # Project metadata: languages, frameworks, structure
└── paths.toml      # Entry points, critical paths, public API surface
```

### context.toml

```toml
[project]
name = "my-project"
type = "web-service"
languages = ["rust", "typescript"]
frameworks = ["axum", "react"]
build_systems = ["cargo", "npm"]

[structure]
style = "modular"
source_roots = ["src/", "frontend/src/"]
test_roots = ["tests/"]

[status]
schema_version = 1
dex_version = "0.1.0"
```

### paths.toml

```toml
[[entry_points]]
name = "main"
file = "src/main.rs"
description = "Application entry point"

[[public_api]]
name = "API endpoints"
definition = "src/api/router.rs"
```

## Supported languages

- Rust (Cargo.toml)
- TypeScript / JavaScript (package.json)
- Python (pyproject.toml)
- Go (go.mod)

## The `.dex/` protocol

The `.dex/` format is an open standard. Any tool can read and write it. Commit `.dex/` to your repo so that every contributor (human or AI) can orient themselves instantly.

**Schema version:** 1

## License

MIT
