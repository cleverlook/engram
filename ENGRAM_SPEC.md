# engram — Specification

> Portable long-term memory for AI agents, built on principles of human associative memory.

---

## Overview

engram is a CLI tool written in Rust that gives AI agents persistent, structured memory about a project. It stores knowledge as a weighted node graph in YAML files inside `.engram/` — human-readable, git-friendly, no external services required.

The agent uses engram the same way a human uses it. There is no separate "agent API" — just the CLI and direct file access. The CLI provides mechanics. Thinking is the agent's job.

---

## Core Concepts

### Node

The fundamental unit of memory. A node stores a piece of knowledge about the project. It should be **self-contained and focused** — understandable without reading source files. Size and format are up to the agent.

### Graph

Nodes are connected by directed weighted edges. Edges are stored inside the node that owns them (outgoing only). Incoming edges (backlinks) are stored in `_backlinks.yaml` per namespace — derived from YAML, committed to git.

### Namespaces

Nodes are organized in hierarchical namespaces using `:` as separator.

```
auth:oauth:google
redis:session_store
decisions:why_redis
payments:stripe:webhooks
```

The namespace hierarchy maps directly to the filesystem:

```
nodes/auth/oauth/google.yaml
nodes/redis/session_store.yaml
nodes/decisions/why_redis.yaml
```

### Weights

Integer 0–100. Higher = more important / stronger connection.

- **Node weight** — centrality. How fundamental is this concept to the project.
- **Edge weight** — connection strength. How tightly coupled are two concepts.

Weights are set by the agent at creation time based on semantics. They increase when a node is frequently visited during traversal, decrease with age/disuse.

### Status

- `active` — current, trustworthy
- `dirty` — source file changed since node was created, needs review
- `stale` — not touched in a long time, may be outdated
- `deprecated` — concept no longer exists in the codebase

---

## File Structure

```
project/
├── src/
└── .engram/
    ├── .gitignore
    ├── data_lake/
    │   ├── architecture_diagram.png
    │   ├── api_contract.pdf
    │   └── meeting_notes_2024.md
    └── nodes/
        ├── _index.yaml
        ├── auth/
        │   ├── _index.yaml
        │   ├── _backlinks.yaml
        │   └── oauth/
        │       ├── _index.yaml
        │       ├── _backlinks.yaml
        │       └── google.yaml
        ├── decisions/
        │   ├── _index.yaml
        │   ├── _backlinks.yaml
        │   └── why_redis.yaml
        └── redis/
            ├── _index.yaml
            ├── _backlinks.yaml
            └── session_store.yaml
```

`.engram/.gitignore`:
```
engram.db
```

---

## Schema

### Node (`nodes/{namespace}/{id}.yaml`)

```yaml
id: auth:oauth:google
content: |
  Google OAuth использует Authorization Code Flow с PKCE.
  Callback URL настраивается через GOOGLE_REDIRECT_URI env var.
weight: 65
status: active
source_files:              # optional
  - src/auth/oauth/google.ts
source_hash: a3f9c2        # optional, md5 of source_files at creation time
created: 2024-03-10T00:00:00Z
touched: 2024-03-10T00:00:00Z
data_lake:                 # optional, paths relative to .engram/data_lake/
  - meeting_notes_2024.md
  - diagrams/oauth_flow.png
edges:
  - to: redis:session_store
    type: uses
    weight: 80
  - to: auth:oauth:flow
    type: implements
    weight: 90
```

**Fields:**

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | yes | Unique identifier, namespace:name format |
| `content` | string | yes | The knowledge. Self-contained, focused. |
| `weight` | int 0-100 | yes | Node importance |
| `status` | enum | yes | active / dirty / stale / deprecated |
| `source_files` | []string | no | Files this node was derived from |
| `source_hash` | string | no | MD5 of source_files at creation time |
| `created` | datetime | yes | ISO 8601 / RFC 3339 (UTC) |
| `touched` | datetime | yes | ISO 8601 / RFC 3339 (UTC), updated on every traversal visit |
| `data_lake` | []string | no | Paths relative to `.engram/data_lake/` |
| `edges` | []Edge | no | Outgoing edges |

**Edge fields:**

| Field | Type | Required | Description |
|---|---|---|---|
| `to` | string | yes | Target node id |
| `type` | enum | yes | uses / depends_on / implements / rationale / related |
| `weight` | int 0-100 | yes | Connection strength |

### Namespace Index (`_index.yaml`)

```yaml
namespace: auth:oauth
source_paths:              # filesystem paths that map to this namespace
  - src/auth/oauth/
nodes:
  - id: auth:oauth:google
    weight: 65
    status: active
    tags: [oauth, security, google]
  - id: auth:oauth:flow
    weight: 80
    status: active
    tags: [oauth, security]
namespaces:
  - auth:oauth:providers
```

Top-level `_index.yaml`:

```yaml
namespaces:
  - name: auth
    node_count: 12
    tags: [security, jwt, oauth]
  - name: redis
    node_count: 5
    tags: [cache, session]
  - name: decisions
    node_count: 6
    tags: [architecture, rationale]
  - name: payments
    node_count: 8
    tags: [stripe, webhooks]
```

### Backlinks (`_backlinks.yaml`)

Derived from outgoing edges across all nodes. Rebuilt by `engram rebuild-index`. Committed to git — source of truth for incoming edges does not depend on SQLite.

```yaml
namespace: auth:oauth
backlinks:
  - node: auth:oauth:google
    incoming:
      - from: auth:sessions
        type: depends_on
        weight: 80
      - from: payments:checkout
        type: uses
        weight: 40
  - node: auth:oauth:flow
    incoming:
      - from: auth:oauth:google
        type: implements
        weight: 90
```

---

## SQLite

SQLite is an optional cache for **full-text search only** (FTS5). Without it, `engram search` scans all YAML files. With it, search is a single query.

SQLite is always derived from YAML. Never treat it as source of truth. Rebuilt by `engram rebuild-index`.

```sql
CREATE VIRTUAL TABLE nodes_fts USING fts5(id, content, tags);

CREATE TABLE nodes (
  id TEXT PRIMARY KEY,
  namespace TEXT,
  weight INTEGER,
  status TEXT,
  source_hash TEXT,
  touched TEXT
);
```

---

## CLI Commands

```
engram init                            Initialize .engram/ in current directory

engram node get <id>                   Print node
engram node create                     Create node (stdin or $EDITOR)
engram node update <id>                Update node
engram node deprecate <id>             Mark as deprecated

engram search <query>                  Full-text search (SQLite FTS5)
engram traverse <id>                   Traverse graph from node
              [--depth N]
              [--min-weight N]
              [--budget N]
engram backlinks <id>                  Show all nodes pointing to this node

engram status                          Show dirty/stale nodes, apply weight decay
engram check                           Check graph integrity
engram rebuild-index                   Rebuild _index.yaml, _backlinks.yaml, SQLite
```

### `engram init`

- Creates `.engram/nodes/` and `.engram/data_lake/`
- Creates `.engram/.gitignore` with `engram.db`
- Creates top-level `_index.yaml`
- Creates `SKILL.md` in project root (or appends to existing `CLAUDE.md`)
- Builds SQLite index

### `engram node create`

Accepts YAML on stdin or opens `$EDITOR`. Validates schema, writes file, updates `_index.yaml`, updates `_backlinks.yaml` for any edges, updates SQLite.

### `engram status`

- Compares `source_hash` against current file hashes → marks dirty nodes
- Reports stale nodes (not touched in 30+ days)
- Applies weight decay

### `engram traverse <id>`

Weighted BFS from entry node.

**Hard stops:**
- Cycle detected
- Token budget exhausted (default: 4000, configurable via `--budget`)
- Max depth reached (default: 5, configurable via `--depth`)

**Soft signals (influence priority queue):**
- Edge weight below `--min-weight` — skip
- Node status `deprecated` — skip
- Node status `dirty` — include, annotate as `[POSSIBLY STALE]`
- All neighbors already visited — natural stop

Output: ordered list of nodes by traversal priority, with path annotated.

### `engram check`

Validates graph integrity. Reports:

- **Broken edges** — `to` node does not exist
- **Broken data_lake refs** — file does not exist in `.engram/data_lake/`
- **Orphan nodes** — no incoming or outgoing edges, weight < 10, not touched in 180+ days

Agent decides what to do with findings — engram does not auto-fix.

### `engram rebuild-index`

Walks all YAML files, rebuilds:
- `_index.yaml` at each namespace level
- `_backlinks.yaml` at each namespace level
- SQLite FTS index

---

## Traversal Algorithm

```
function traverse(entry_id, budget, max_depth, min_weight):
  visited = {}
  context = []
  queue = PriorityQueue()  // ordered by weight descending

  queue.push(entry_id, priority=100)

  while queue not empty and budget > 0:
    node_id = queue.pop()

    if node_id in visited: continue
    if depth(node_id) > max_depth: continue

    node = load_node(node_id)

    if node.status == deprecated: continue

    visited.add(node_id)
    context.append(node)
    budget -= len(node.content)

    node.touched = today()
    node.weight = min(100, node.weight + 1)
    save_node(node)

    for edge in node.edges:
      if edge.weight >= min_weight and edge.to not in visited:
        queue.push(edge.to, priority=edge.weight)

  return context
```

---

## Weight Decay

Run as part of `engram status`:

```
for each node:
  days = today - node.touched
  if days > 90:
    node.weight = max(0, node.weight - 5)
  elif days > 30:
    node.weight = max(0, node.weight - 1)
```

---

## Dirty Node Detection

Run during `engram status` and lazily during traversal:

```
for each node where source_files is set:
  current_hash = md5(concat(read(f) for f in source_files))
  if current_hash != node.source_hash:
    node.status = dirty
```

---

## SKILL.md

Generated by `engram init`.

```markdown
# engram — Memory

This project has a persistent memory graph in `.engram/`.
Use it. Don't ignore it.

## Tools

  engram search <keywords>         full-text search across all nodes
  engram traverse <node:id>        explore graph from a known node
              [--depth N]
              [--min-weight N]
              [--budget N]
  engram node get <node:id>        read a specific node
  engram node create               save a new piece of knowledge
  engram node update <id>          update existing node
  engram backlinks <id>            see what points to this node
  engram status                    dirty/stale nodes, weight decay
  engram check                     graph integrity report
  engram rebuild-index             rebuild indexes and backlinks

## How you might use this

**Before starting work — load context first, then read code:**
  Check _index.yaml to see what namespaces exist.
  If the task touches a known namespace — traverse it before reading source files.
  It's cheaper than reading cold.

**While working — pull context as scope expands:**
  When you encounter something unfamiliar — check if memory has it.
  When scope expands to a new module — check its namespace index.
  When you need to understand why something was built a certain way — search decisions namespace.

**After completing work — save what's worth keeping:**
  If you discovered something that would save future-you time — create or update a node.
  If you changed code covered by existing nodes — update them, reset source_hash.

## Memory is a tool, not a constraint

Use the CLI, read YAML files directly, query engram.db directly — whatever works.
The point is to not work blind when knowledge already exists.

## Node guidelines
- Node should be self-contained — understandable without reading source files
- Stay focused — one concern per node
- source_files and source_hash are optional — not all knowledge comes from code
- data_lake files can be referenced for richer context (diagrams, docs, notes)
- Weight reflects importance: core architecture = 80+, implementation details = 20-40
- Prefer updating existing nodes over creating near-duplicates
```

---

## Tech Stack

- **Language:** Rust
- **YAML:** `serde_yaml`
- **SQLite:** `rusqlite`
- **CLI:** `clap`
- **Hashing:** `md5` crate

---

## Out of Scope (v1)

- Vector/semantic search (FTS5 sufficient for v1)
- Multi-agent conflict resolution
- Remote sync
- Web UI
- LLM calls inside the CLI (the agent is the LLM)
