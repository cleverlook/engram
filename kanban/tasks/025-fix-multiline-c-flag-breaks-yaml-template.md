---
id: 25
title: 'Fix: multiline -c flag breaks YAML template'
status: done
priority: critical
created: 2026-03-20T18:58:33.091695+01:00
updated: 2026-03-20T19:21:35.5808+01:00
started: 2026-03-20T19:15:35.625018+01:00
completed: 2026-03-20T19:21:35.580799+01:00
tags:
    - bug
    - cli
    - agent-ux
claimed_by: kolkhoz-infand
claimed_at: 2026-03-20T19:21:35.5808+01:00
class: standard
---

## Problem
When `-c` flag contains literal newlines, the YAML template in `src/commands/node.rs:12-25` breaks because newlines in content aren't indented under the `content: |` block.

Example: `engram node create x:y -c "line1\nline2"` produces invalid YAML.

## Root cause
Template substitution (`format!()`) inserts raw content without YAML-escaping newlines.

## Fix
In the template generation (node.rs lines ~57-65), indent each line of content with 2 spaces when inserting into the YAML template. Or use serde_yaml to serialize instead of string templating.

## Files
- `src/commands/node.rs` (template generation, ~lines 12-25 and 57-65)
