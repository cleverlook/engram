---
name: engram
description: Persistent structured memory for AI agents. Use when you need to store, retrieve, search, or traverse project knowledge. Use before starting work to load context, during work to pull related knowledge, and after work to save what you learned.
allowed-tools: Bash(engram:*)
---

# engram — Memory

This project has a persistent memory graph in `.engram/`.
Memory is faster than re-reading code. Use it before, during, and after work.

## When to use engram

**ALWAYS check memory before starting work.** A single `engram search` or `engram traverse` call can save dozens of file reads. Specifically:

- **Starting a task?** → `engram search <topic>` to see what's already known
- **Touching a module?** → `engram traverse <namespace:module>` to load its context graph
- **Need to understand why something exists?** → `engram search "decision <topic>"` or `engram backlinks <id>`
- **Finished work?** → `engram node create` or `engram node update` to save what you learned
- **Something feels wrong?** → `engram status` + `engram check` to find stale/broken nodes

## Three interfaces for writing nodes

Choose the right interface for the job:

### 1. Flags — quick, single-line content

Best for simple nodes or quick updates. Supports multiline content (newlines in `-c` are handled correctly).

```bash
# Create with content, weight, edges, and source files
engram node create <id> -c "description" -w 60
engram node create <id> -c "description" -w 60 \
  --add-edge "other:node:related:50" \
  --add-source-file src/main.rs

# Update specific fields (merges into existing — preserves everything not mentioned)
engram node update <id> -c "new content" -w 75
engram node update <id> --add-edge "new:target:uses:60" --remove-edge "old:target"
engram node update <id> --add-source-file src/new.rs --remove-source-file src/old.rs
```

| Flag                  | Short | Create | Update | Description                         |
|-----------------------|-------|--------|--------|-------------------------------------|
| `--content`           | `-c`  | yes    | yes    | Node content text                   |
| `--weight`            | `-w`  | yes    | yes    | Importance 0-100 (default: 50)      |
| `--data-lake`         | `-d`  | yes    | —      | Link data lake file (repeatable)    |
| `--add-data-lake`     |       | —      | yes    | Add data lake file (repeatable)     |
| `--remove-data-lake`  |       | —      | yes    | Remove data lake file (repeatable)  |
| `--add-edge`          |       | yes    | yes    | Add edge: `"target:type:weight"`    |
| `--remove-edge`       |       | —      | yes    | Remove edge by target id            |
| `--add-source-file`   |       | yes    | yes    | Add source file path (repeatable)   |
| `--remove-source-file`|       | —      | yes    | Remove source file (repeatable)     |
| `--edit`              | `-e`  | yes    | yes    | Open $EDITOR                        |

Edge format: `"target_id:edge_type:weight"` — e.g. `"auth:session:uses:50"`.
Valid edge types: `uses`, `depends_on`, `implements`, `rationale`, `related`.

### 2. Stdin pipe — partial YAML for complex nodes

Best for multiline content, multiple edges, or batch operations. **Only provide the fields you want to set** — everything else gets sensible defaults (create) or is preserved from the existing node (update).

**Create — only `id` is required:**
```bash
cat <<'EOF' | engram node create
id: auth:oauth:google
content: |
  Google OAuth integration using PKCE flow.
  Tokens stored in encrypted session cookie.
weight: 70
edges:
  - to: auth:session
    type: uses
    weight: 50
source_files:
  - src/auth/oauth.rs
EOF
```

Defaults when omitted: `content: ""`, `weight: 50`, `status: active`, timestamps auto-generated.

**Update — pipe only fields to change:**
```bash
cat <<'EOF' | engram node update auth:oauth:google
content: |
  Updated: now uses PKCE v2 flow.
  Tokens stored in encrypted httpOnly cookie.
weight: 80
EOF
```

Existing edges, source_files, and other fields are preserved.

### 3. Direct YAML edit + rebuild-index — full control

Best for bulk editing, reorganizing nodes, or when the CLI is insufficient.

```bash
# Edit YAML files directly
$EDITOR .engram/nodes/auth/oauth/google.yaml

# Rebuild all indexes and search cache from YAML source of truth
engram rebuild-index
```

Node id maps to filesystem: `auth:oauth:google` → `.engram/nodes/auth/oauth/google.yaml`

## Read commands

### search — Full-text search across all nodes

```bash
engram search "authentication flow"
engram search "error handling"
```

### traverse — Walk the graph from a node

```bash
engram traverse auth:oauth              # explore auth namespace
engram traverse core:storage --depth 3  # shallow traversal
engram traverse api:endpoints --min-weight 30 --budget 8000  # high-signal deep dive
```

| Flag           | Default | Description                        |
|----------------|---------|------------------------------------|
| `--depth`      | 5       | Max traversal depth                |
| `--min-weight` | 0       | Minimum edge weight to follow      |
| `--budget`     | 4000    | Token budget (stops when exceeded) |

### node get — Read a specific node

```bash
engram node get auth:oauth:google
```

### backlinks — See what points to a node

```bash
engram backlinks auth:session  # which nodes reference auth:session?
```

## Lifecycle commands

### node deprecate — Mark a node as deprecated

```bash
engram node deprecate auth:legacy:basic_auth
```

### status — Dirty/stale report + weight decay

```bash
engram status
```

Shows nodes with changed source files (dirty), untouched >30 days (stale), and applies weight decay.

### check — Graph integrity validation

```bash
engram check
```

Reports broken edges, missing nodes, orphaned files.

### rebuild-index — Rebuild all derived data

```bash
engram rebuild-index
```

Rebuilds `_index.yaml`, `_backlinks.yaml`, and SQLite FTS from YAML source of truth.

### lake — Data lake artifact management

```bash
engram lake add docs/architecture.png --link core:architecture
engram lake list
engram lake remove old-diagram.png
```

### tui — Interactive terminal UI

```bash
engram tui
```

## Typical agent workflow

```bash
# 1. Starting a task — load context
engram search "auth"
engram traverse auth:oauth --depth 3

# 2. Read specific nodes
engram node get auth:oauth:google

# 3. Do the work...

# 4. Save what you learned (flags for simple, stdin for complex)
engram node create auth:oauth:pkce -c "PKCE flow implementation details" -w 60 \
  --add-edge "auth:oauth:google:implements:70" \
  --add-source-file src/auth/pkce.rs

# 5. Update existing node
engram node update auth:oauth:google -c "Updated after PKCE migration" -w 75
```

## Node guidelines

- Nodes should be self-contained — understandable without reading source files
- One concern per node
- `source_files` and `source_hash` are optional — not all knowledge comes from code
- Weight reflects importance: core architecture = 80+, implementation details = 20-40
- Prefer updating existing nodes over creating near-duplicates
- Node id maps to filesystem: `auth:oauth:google` → `nodes/auth/oauth/google.yaml`

## Direct access

You can also read YAML files directly from `.engram/nodes/` or query `.engram/engram.db` (SQLite FTS5) — whatever is fastest. The `_index.yaml` in each namespace directory lists all nodes in that namespace.
