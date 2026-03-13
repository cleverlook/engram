# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Engram is a CLI tool written in Rust that provides persistent, structured memory for AI agents. It stores knowledge as a weighted node graph in YAML files inside `.engram/` directories — human-readable, git-friendly, no external services. The full specification is in `ENGRAM_SPEC.md`.

## Build & Development Commands

```bash
cargo build              # Build the project
cargo run                # Run the CLI
cargo test               # Run all tests
cargo test <test_name>   # Run a single test
cargo test --test init_test  # Run a specific test file
cargo clippy             # Lint
cargo fmt                # Format code
```

## Testing

Integration tests live in `tests/` and run the compiled binary against a `tempfile::TempDir`. Each test creates a fresh `.engram/` via `helpers::setup_engram()`. Test files are organized by feature: `init_test.rs`, `node_test.rs`, `search_test.rs`, `traverse_test.rs`, `status_check_test.rs`, `rebuild_test.rs`. Shared helpers are in `tests/helpers.rs`.

## Tech Stack

- **Rust** (edition 2024)
- **clap** (derive) — CLI argument parsing
- **serde + serde_yaml** — YAML serialization/deserialization
- **rusqlite** (bundled, vtab) — SQLite for FTS5 full-text search
- **md5** — Source file hash for dirty detection
- **chrono** — Date handling
- **anyhow** — Error handling

## Architecture

### Module Structure

```
src/
├── main.rs           — Entry point, routes CLI commands
├── cli.rs            — clap Parser/Subcommand definitions
├── storage.rs        — Node id ↔ filesystem path mapping, load/save, find .engram/
├── db.rs             — SQLite: open, create tables, upsert/delete/search/rebuild
├── indexing.rs       — _index.yaml and _backlinks.yaml management
├── models/
│   ├── node.rs       — Node, Edge, NodeStatus, EdgeType
│   ├── index.rs      — NamespaceIndex, IndexEntry, NamespaceSummary
│   └── backlinks.rs  — NamespaceBacklinks, NodeBacklinks, IncomingEdge
└── commands/
    ├── init.rs       — Create .engram/ structure, SQLite, SKILL.md
    ├── node.rs       — CRUD: get, create, update, deprecate
    ├── search.rs     — FTS5 full-text search
    ├── traverse.rs   — Weighted BFS graph traversal
    ├── backlinks.rs  — Display incoming edges
    ├── status.rs     — Dirty detection, stale reporting, weight decay
    ├── check.rs      — Graph integrity validation
    └── rebuild_index.rs — Full rebuild of indexes, backlinks, SQLite
```

### Data Flow

Every node mutation (create/update/deprecate) triggers three side effects in order:
1. **YAML write** (`storage::save_node`) — source of truth
2. **Index update** (`indexing::update_index_for_node` + `update_backlinks_for_node`) — `_index.yaml` and `_backlinks.yaml`
3. **SQLite sync** (`db::upsert_node`) — derived FTS5 cache

### Key Design Decisions

- **YAML is always source of truth.** SQLite is a derived cache, rebuilt from YAML via `rebuild-index`.
- **Namespace hierarchy maps to filesystem.** Node id `auth:oauth:google` → `nodes/auth/oauth/google.yaml`. The `storage::node_path()` function handles this mapping.
- **`.engram/` is discovered by walking upward** from cwd, similar to how git finds `.git/`. See `storage::find_engram_dir()`.
- **Templates live in `templates/`** and are embedded via `include_str!()` at compile time.

### CLI Commands

`init` | `node get/create/update/deprecate` | `search` | `traverse` | `backlinks` | `status` | `check` | `rebuild-index`

### Core Algorithm

Graph traversal uses weighted BFS (`BinaryHeap`) with a token budget (default 4000), max depth (default 5), and min-weight filter. Visited nodes get their `touched` date updated and weight incremented by 1. See `ENGRAM_SPEC.md` § Traversal Algorithm for pseudocode.

### Node Lifecycle

Nodes have status: `active` → `dirty` (source files changed) → `stale` (30+ days untouched) → `deprecated`. Weight decay runs during `engram status` (-1 after 30 days, -5 after 90 days untouched).
