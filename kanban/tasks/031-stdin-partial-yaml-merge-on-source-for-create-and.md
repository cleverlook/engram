---
id: 31
title: 'Stdin partial YAML: merge on source for create and update'
status: backlog
priority: critical
created: 2026-03-20T19:07:09.127332+01:00
updated: 2026-03-20T19:07:09.127332+01:00
tags:
    - bug
    - cli
    - agent-ux
class: standard
---

## Problem
Stdin mode deserializes directly into Node struct, requiring ALL fields (including created/touched). This makes it unusable for agents.

## Design
Stdin should accept **partial YAML** and merge onto source:

**Create:** partial YAML + defaults
- Required: `id` (or positional arg)
- Everything else optional with sensible defaults:
  - `content` → "" 
  - `weight` → 50
  - `status` → active
  - `created`/`touched` → Utc::now()
  - `edges` → []
  - `source_files` → []
  - `data_lake` → []

**Update:** partial YAML + existing node
- Required: `id` (positional arg already has it)
- Load existing node from disk
- Overlay only fields present in stdin YAML
- Preserve everything not mentioned (edges, source_files, data_lake, timestamps)
- Auto-update `touched` to Utc::now()

## Implementation
1. Deserialize stdin into `serde_yaml::Value` (not Node)
2. For create: build default Node, overlay stdin fields
3. For update: load existing Node as Value, overlay stdin fields
4. Deserialize merged Value into Node
5. Save

## Files
- `src/commands/node.rs` (~lines 71-79 create stdin, ~lines 140-145 update stdin)
- `src/models/node.rs` (may need Default impl for Node)
