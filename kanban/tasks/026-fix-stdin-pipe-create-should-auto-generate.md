---
id: 26
title: 'Fix: stdin pipe create should auto-generate timestamps'
status: archived
priority: high
created: 2026-03-20T18:58:38.620065+01:00
updated: 2026-03-20T19:06:56.990926+01:00
started: 2026-03-20T19:06:56.990925+01:00
completed: 2026-03-20T19:06:56.990925+01:00
tags:
    - bug
    - cli
    - agent-ux
class: standard
---

## Problem
`cat <<EOF | engram node create` requires `created` and `touched` fields in YAML. If missing: `Error: missing field 'created'`. The SKILL.md example omits these fields and doesn't work.

## Root cause
Stdin YAML is deserialized directly into Node struct which has required DateTime fields. No defaults are applied.

## Fix
For stdin create: auto-fill `created` and `touched` with `Utc::now()` if not provided. Use `serde(default)` or post-parse fill. Same behavior as flag mode which already auto-generates these.

## Files
- `src/commands/node.rs` (~line 79, stdin parsing)
- `src/models/node.rs` (Node struct, add Default for DateTime fields)
