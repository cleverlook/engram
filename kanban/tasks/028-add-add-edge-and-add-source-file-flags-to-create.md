---
id: 28
title: Add --add-edge and --add-source-file flags to create and update
status: backlog
priority: medium
created: 2026-03-20T18:58:49.791556+01:00
updated: 2026-03-20T18:58:49.791556+01:00
tags:
    - enhancement
    - cli
    - agent-ux
class: standard
---

## Problem
Edges and source_files can only be set via stdin YAML or editor mode. No CLI flags exist. This forces agents into a two-step flow (create basic node, then edit YAML directly) or stdin mode (which has its own bugs).

## Proposed flags
For both `create` and `update`:
- `--add-edge <TO:TYPE:WEIGHT>` (repeatable) — e.g. `--add-edge "arch:data_flow:related:60"`
- `--add-source-file <PATH>` (repeatable) — e.g. `--add-source-file src/db.rs`

For `update` only:
- `--remove-edge <TO>` — remove edge by target id
- `--remove-source-file <PATH>` — remove source file

## Files
- `src/cli.rs` (~lines 89-133, add flags to CreateNode and UpdateNode)
- `src/commands/node.rs` (apply new flags in create/update logic)
