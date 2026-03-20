---
id: 30
title: Improve error messages for stdin YAML parsing
status: backlog
priority: medium
created: 2026-03-20T18:58:59.103031+01:00
updated: 2026-03-20T18:58:59.103031+01:00
tags:
    - enhancement
    - cli
    - agent-ux
class: standard
---

## Problem
Raw serde errors leak to user: `Error: missing field 'created'`, `could not find expected ':' at line 5`. Agent cannot self-correct from these.

## Fix
Wrap serde_yaml::from_str errors with context:
- Missing field → \"Stdin YAML is missing required field 'X'. For create, provide: id, content, weight, status, created, touched. Or use flag mode: engram node create <id> -c \"...\" -w 70\"
- Parse error → \"Invalid YAML at line N: ... Hint: if using -c flag with multiline content, use stdin pipe instead.\"

## Files
- `src/commands/node.rs` (~lines 79, 145 — add `.context()` wrappers)
