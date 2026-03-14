# dex CLI — Design Specification

**Date:** 2026-03-14
**Author:** Claude Opus (dexs-oss)
**Status:** Approved

## Problem

Every time an AI agent or new human contributor encounters a codebase, they waste significant effort rediscovering project structure, conventions, architecture decisions, and critical paths. This understanding is trapped in tribal knowledge, scattered docs, or nowhere at all.

There is no standard, machine-readable format for describing a project's intent and structure.

## Solution

**`dex`** — a Rust CLI that scans any codebase and generates a `.dex/` context directory containing a structured, human+AI-readable map of the project.

## Core Commands

### `dex init`
Analyze a codebase and generate the `.dex/` context directory.

**Behavior:**
1. Walk the file tree, identify languages, frameworks, and build systems
2. Detect project structure patterns (monorepo, library, service, CLI, etc.)
3. Identify entry points, public APIs, and critical paths
4. Generate `.dex/context.toml` and `.dex/paths.toml`

### `dex sync` (v0.2.0)
Update the `.dex/` context as code evolves.

### `dex show [--module <name>]` (v0.2.0)
Structured lookup of context data. Print project summary or specific module details. No NLP — just structured queries against the TOML files.

### `dex lint` (v0.3.0)
Check if code matches the project's own conventions (as captured in `.dex/conventions.toml`). Depends on convention inference landing first in v0.2.0.

## The `.dex/` Format

**Schema version:** The format is versioned independently from the CLI via `schema_version` in `context.toml`. This allows tooling to detect and handle format changes.

**Version control:** `.dex/` should be committed to the repository. It is a shared team artifact. `dex sync` (v0.2.0) will keep it current. A `README` is generated inside `.dex/` explaining this.

### Directory Structure
```
.dex/
├── README                # Explains the .dex/ directory
├── context.toml          # Main context file (always generated)
├── paths.toml            # Entry points, critical paths, public APIs (always generated)
├── architecture.toml     # Module descriptions and relationships (v0.2.0)
├── conventions.toml      # Coding conventions and patterns (v0.2.0)
└── decisions.toml        # Key architectural decisions and rationale (v0.2.0)
```

### `context.toml` — Main Context File
```toml
[project]
name = "my-project"
type = "web-service"           # cli, library, web-service, monorepo, etc.
languages = ["rust", "typescript"]
frameworks = ["axum", "react"]
build_systems = ["cargo", "npm"]
description = "A brief description of what this project does"

[project.repository]
url = "https://github.com/org/repo"
default_branch = "main"

[structure]
style = "modular"              # modular, layered, monorepo, flat
source_roots = ["src/", "frontend/src/"]
test_roots = ["tests/", "frontend/tests/"]
config_root = "config/"

# For monorepos: list workspace members
# [[structure.workspaces]]
# name = "core"
# path = "crates/core/"
# type = "library"

[status]
schema_version = 1
dex_version = "0.1.0"
last_sync = "2026-03-14T20:00:00Z"
```

### `paths.toml` — Entry Points & Critical Paths
```toml
[[entry_points]]
name = "main"
file = "src/main.rs"
description = "Application entry point"

[[critical_paths]]
name = "request-handling"
description = "HTTP request → router → handler → response"
files = ["src/api/router.rs", "src/api/handlers/", "src/middleware/"]

[[public_api]]
name = "REST API"
definition = "src/api/router.rs"
```

### `architecture.toml` (v0.2.0)
```toml
[[modules]]
name = "auth"
path = "src/auth/"
purpose = "authentication and session management"  # heuristic-based, approximate
depends_on = ["database", "config"]                # high-confidence for Rust/Go, best-effort for TS/Python
confidence = "high"                                # high | medium | low
public_api = ["src/auth/mod.rs"]
```

**Note on `purpose` field:** Generated via heuristic matching (directory names, framework patterns, export analysis). Not NLP. Will be approximate — users should refine. Labeled as `# auto-generated` in output.

**Note on `depends_on`:** Inferred from import/use statements. Confidence varies by language:
- **Rust, Go:** High confidence (explicit module/package imports)
- **TypeScript/JavaScript:** Medium confidence (path aliases, barrel re-exports may cause gaps)
- **Python:** Medium confidence (dynamic imports, `importlib` not tracked)

### `conventions.toml` (v0.2.0)
```toml
[naming]
files = "snake_case"
types = "PascalCase"
functions = "snake_case"

[dependencies]
error_handling = ["thiserror", "anyhow"]    # detected from manifest
async_runtime = "tokio"                      # detected from manifest
test_framework = "built-in"                  # detected from manifest + test files

[style]
imports = "grouped"  # inferred from source analysis
```

### `decisions.toml` (v0.2.0 — manual only)
```toml
[[decisions]]
title = "Use axum over actix-web"
date = "2026-01-15"
rationale = "Tokio-native, simpler API, better tower integration"
```

This file is scaffolded empty by `dex init`. Populated manually by developers. `dex sync` preserves it.

## Tech Stack

- **Language:** Rust (edition 2024)
- **CLI framework:** `clap` (derive API)
- **File walking:** `ignore` crate (respects .gitignore)
- **TOML:** `toml` + `serde`
- **Syntax analysis:** `tree-sitter` with language grammars
- **Output formatting:** `colored` for terminal output

## MVP Scope (v0.1.0)

The first release focuses on `dex init` only:

1. Language and framework detection (from file extensions + manifests like Cargo.toml, package.json, go.mod, pyproject.toml)
2. Project type detection (cli, library, web-service, monorepo)
3. Project structure analysis (source roots, test roots, config)
4. Entry point and critical path identification
5. Generate `.dex/context.toml` and `.dex/paths.toml`
6. Support for: Rust, TypeScript/JavaScript, Python, Go
7. Monorepo/workspace detection (Cargo workspaces, npm workspaces, Go modules)

**Deferred:**
- v0.2.0: `dex sync`, `dex show`, `architecture.toml`, `conventions.toml`, `decisions.toml` scaffold
- v0.3.0: `dex lint`

## Non-Goals

- Not a replacement for documentation
- Not a linter (convention drift checking is a future feature, not core)
- Not an AI/LLM tool — uses static analysis and heuristics only
- Not a build tool

## Success Criteria

- A user can run `dex init` in any Rust/TS/Python/Go project and get a useful `.dex/` directory in under 5 seconds
- The generated context is accurate enough that an AI agent reading only `.dex/` can orient itself in the project
- Single binary, no runtime dependencies, cross-platform
