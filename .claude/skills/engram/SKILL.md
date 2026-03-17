---
name: engram
description: Persistent structured memory for AI agents. Use when you need to store, retrieve, search, or traverse project knowledge. Use before starting work to load context, during work to pull related knowledge, and after work to save what you learned.
allowed-tools: Bash(engram:*)
---

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
