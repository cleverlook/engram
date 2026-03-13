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
cargo clippy             # Lint
cargo fmt                # Format code
```

## Current State

The project is in early development — only a stub `main.rs` exists. Implementation should follow `ENGRAM_SPEC.md` as the source of truth.

## Tech Stack

- **Rust** (edition 2024)
- **clap** — CLI argument parsing
- **serde_yaml** — YAML serialization/deserialization
- **rusqlite** — SQLite for FTS5 full-text search (optional cache, never source of truth)
- **md5** — Source file hash for dirty detection

## Architecture

The codebase implements a CLI (`clap`) around a node-graph memory system:

- **Nodes**: Self-contained knowledge stored as YAML files, organized in hierarchical namespaces (`:` separator maps to filesystem directories, e.g., `auth:oauth:google` → `nodes/auth/oauth/google.yaml`)
- **Graph**: Directed weighted edges stored inside source nodes (outgoing). Backlinks (`_backlinks.yaml`) are derived and stored per namespace.
- **Indexes**: `_index.yaml` at each namespace level summarizes contained nodes. Rebuilt via `engram rebuild-index`.
- **SQLite**: Derived cache for FTS5 search only. YAML files are always the source of truth. Database lives at `.engram/engram.db` and is gitignored.
- **Data Lake**: `.engram/data_lake/` stores artifacts (images, PDFs, notes) referenced by nodes.

### Key CLI Commands

`init` | `node get/create/update/deprecate` | `search` | `traverse` | `backlinks` | `status` | `check` | `rebuild-index`

### Core Algorithm

Graph traversal uses weighted BFS with a token budget (default 4000), max depth (default 5), and min-weight filter. Visited nodes get their `touched` date updated and weight incremented by 1. See `ENGRAM_SPEC.md` § Traversal Algorithm for pseudocode.

### Node Lifecycle

Nodes have status: `active` → `dirty` (source files changed) → `stale` (30+ days untouched) → `deprecated`. Weight decay runs during `engram status` (-1 after 30 days, -5 after 90 days untouched).
