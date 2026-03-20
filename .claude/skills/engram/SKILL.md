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

## Commands reference

### search — Full-text search across all nodes

```
engram search <query>
```

Example:
```bash
engram search "authentication flow"
engram search "error handling"
```

### traverse — Walk the graph from a node

```
engram traverse <node:id> [--depth N] [--min-weight N] [--budget N]
```

| Flag           | Default | Description                        |
|----------------|---------|------------------------------------|
| `--depth`      | 5       | Max traversal depth                |
| `--min-weight` | 0       | Minimum edge weight to follow      |
| `--budget`     | 4000    | Token budget (stops when exceeded) |

Example:
```bash
engram traverse auth:oauth              # explore auth namespace
engram traverse core:storage --depth 3  # shallow traversal
engram traverse api:endpoints --min-weight 30 --budget 8000  # high-signal deep dive
```

### node get — Read a specific node

```
engram node get <node:id>
```

Example:
```bash
engram node get auth:oauth:google
```

### node create — Save new knowledge

Four input modes:

**1. Flags (best for agents — non-interactive):**
```bash
engram node create <id> --content "description" --weight 60
engram node create <id> -c "description" -w 60
engram node create <id> -c "description" -w 60 --data-lake diagram.png
```

**2. Stdin pipe (for structured YAML):**
```bash
cat <<'EOF' | engram node create
id: auth:oauth:google
content: |
  Google OAuth integration using PKCE flow.
  Tokens stored in encrypted session cookie.
weight: 70
status: active
edges:
  - to: auth:session
    type: uses
    weight: 50
EOF
```

**3. Editor (`--edit` / `-e`):**
```bash
engram node create --edit
engram node create auth:oauth:google --edit  # pre-fills id
```

**4. Interactive (auto-detected when TTY, no flags, no id):**
```bash
engram node create  # prompts for id, content, weight, edges
```

| Flag             | Short | Description                         |
|------------------|-------|-------------------------------------|
| `--content`      | `-c`  | Node content text                   |
| `--weight`       | `-w`  | Importance 0-100 (default: 50)      |
| `--data-lake`    | `-d`  | Link data lake file (repeatable)    |
| `--edit`         | `-e`  | Open $EDITOR                        |

### node update — Update existing node

```
engram node update <id> [--content "new text"] [--weight N]
                        [--add-data-lake file] [--remove-data-lake file]
                        [--edit]
```

Example:
```bash
engram node update auth:oauth:google --content "Updated: now uses PKCE v2" --weight 75
engram node update auth:oauth:google --add-data-lake oauth-flow.png
engram node update auth:oauth:google --edit  # edit in $EDITOR
```

Stdin pipe also works — pipe full node YAML to overwrite.

### node deprecate — Mark a node as deprecated

```
engram node deprecate <id>
```

Example:
```bash
engram node deprecate auth:legacy:basic_auth
```

### backlinks — See what points to a node

```
engram backlinks <id>
```

Example:
```bash
engram backlinks auth:session  # which nodes reference auth:session?
```

### status — Dirty/stale report + weight decay

```
engram status
```

Shows nodes with changed source files (dirty), untouched >30 days (stale), and applies weight decay.

### check — Graph integrity validation

```
engram check
```

Reports broken edges, missing nodes, orphaned files.

### lake — Data lake artifact management

```
engram lake add <file> [--link <node:id>]   # copy file into data lake
engram lake list                             # list all lake files
engram lake remove <filename>                # remove from lake
```

Example:
```bash
engram lake add docs/architecture.png --link core:architecture
engram lake list
engram lake remove old-diagram.png
```

### rebuild-index — Rebuild all derived data

```
engram rebuild-index
```

Rebuilds `_index.yaml`, `_backlinks.yaml`, and SQLite FTS from YAML source of truth.

### tui — Interactive terminal UI

```
engram tui
```

Browse and navigate the node graph interactively.

## Typical agent workflow

```bash
# 1. Starting a task about authentication
engram search "auth"
engram traverse auth:oauth --depth 3

# 2. Read returned nodes for context (faster than reading source)
engram node get auth:oauth:google

# 3. Do the work...

# 4. Save what you learned
engram node create auth:oauth:pkce -c "PKCE flow implementation details..." -w 60

# 5. Update existing node if content changed
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