# engram

Persistent structured memory for AI agents.

Stores knowledge as a weighted node graph in YAML files inside `.engram/` — human-readable, git-friendly, no external services.

## Install

```bash
# Homebrew (macOS/Linux)
brew install cleverlook/tap/engram

# Shell installer
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/cleverlook/engram/releases/latest/download/engram-installer.sh | sh

# From source
cargo install --git https://github.com/cleverlook/engram
```

## Quick start

```bash
# Initialize in your project
engram init

# Create a node
engram node create auth:oauth:google -c "Google OAuth uses PKCE" -w 65

# Or interactively
engram node create

# Search
engram search "OAuth"

# Traverse the graph from a node
engram traverse auth:oauth:google

# Check graph integrity
engram check

# See dirty/stale nodes
engram status
```

## Commands

```
engram init                            Initialize .engram/ in current directory
engram node get <id>                   Print node
engram node create [id] [options]      Create node (flags, interactive, $EDITOR, or stdin)
engram node update <id> [options]      Update node
engram node deprecate <id>             Mark as deprecated
engram search <query>                  Full-text search (SQLite FTS5)
engram traverse <id> [options]         Traverse graph from node
engram backlinks <id>                  Show incoming edges
engram status                          Dirty/stale nodes, weight decay
engram check                           Graph integrity report
engram rebuild-index                   Rebuild indexes, backlinks, SQLite
engram lake add <file> [-l <node>]     Add file to data lake, optionally link to node
engram lake list                       List data lake files
engram lake remove <file>              Remove file from data lake
engram completion <shell> [--install]  Generate/install shell completions (zsh/bash/fish)
```

## How it works

Nodes are YAML files organized in namespaces mapped to the filesystem:

```
.engram/
├── nodes/
│   ├── _index.yaml
│   ├── auth/
│   │   ├── _index.yaml
│   │   ├── _backlinks.yaml
│   │   └── oauth/
│   │       └── google.yaml
│   └── redis/
│       └── session_store.yaml
├── data_lake/
└── engram.db          # FTS5 search cache (gitignored)
```

Node id `auth:oauth:google` maps to `nodes/auth/oauth/google.yaml`. YAML is always the source of truth. SQLite is a derived cache for full-text search, rebuilt via `engram rebuild-index`.

## For AI agents

engram is designed to be used by AI agents through the CLI. `engram init` installs a skill to `.claude/skills/engram/` that teaches agents how to use the memory graph.

Agents pipe YAML to stdin for non-interactive use:

```bash
echo 'id: decisions:why_redis
content: |
  Chose Redis over Memcached for session storage because
  we need TTL per key and pub/sub for invalidation.
weight: 70
status: active
created: 2024-03-10T00:00:00Z
touched: 2024-03-10T00:00:00Z
edges:
  - to: redis:session_store
    type: rationale
    weight: 80' | engram node create
```

## License

MIT
