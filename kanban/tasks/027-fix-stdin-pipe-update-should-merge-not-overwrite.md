---
id: 27
title: 'Fix: stdin pipe update should merge, not overwrite'
status: archived
priority: high
created: 2026-03-20T18:58:43.453746+01:00
updated: 2026-03-20T19:06:57.006528+01:00
started: 2026-03-20T19:06:57.006528+01:00
completed: 2026-03-20T19:06:57.006528+01:00
tags:
    - bug
    - cli
    - agent-ux
class: standard
---

## Problem
Stdin update replaces the entire node. If piped YAML omits `edges`, they get deleted. Same for `data_lake`, `source_files`. This is destructive and surprising.

Flag-based update already merges correctly — only changes specified fields.

## Root cause
`src/commands/node.rs` (~line 145): stdin YAML is deserialized into a full Node and replaces existing. No merge logic.

## Fix
Stdin update should merge like flag mode: load existing node, overlay only fields present in piped YAML. Use `serde_yaml::Value` to detect which fields were explicitly provided, then merge into existing Node.

## Files
- `src/commands/node.rs` (~lines 140-145, stdin update logic)
