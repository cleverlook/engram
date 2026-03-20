---
id: 29
title: 'Fix SKILL.md: stdin examples and document working patterns'
status: done
priority: high
created: 2026-03-20T18:58:54.643928+01:00
updated: 2026-03-20T19:27:51.655185+01:00
started: 2026-03-20T19:25:54.975352+01:00
completed: 2026-03-20T19:27:51.655185+01:00
tags:
    - docs
    - agent-ux
depends_on:
    - 25
    - 31
claimed_by: kolkhoz-infand
claimed_at: 2026-03-20T19:27:51.655185+01:00
class: standard
---

## Problem
SKILL.md contains non-working examples and doesn't accurately describe all three agent interfaces.

## Three agent interfaces to document

### 1. Flags (quick path)
- Simple content, weight, edges, source_files
- Single command, no YAML knowledge needed
- Example: `engram node create x:y -c "content" -w 70 --add-edge "a:b:related:50"`

### 2. Stdin partial YAML (structured path)
- Multiline content, complex edges, full control over fields
- Only provide fields you want to set/change, rest auto-filled or preserved
- Example: `cat <<EOF | engram node create ...`

### 3. Direct YAML edit + rebuild-index (full control)
- Edit .engram/nodes/**/*.yaml directly
- Run `engram rebuild-index` to sync indexes and SQLite
- Best when bulk editing, reorganizing, or when CLI is insufficient

## Rules
- Every example in SKILL.md MUST work as-is — no lying to the agent
- Document trade-offs of each interface (when to use which)
- Include error recovery guidance

## Blocked by
- #25 (multiline -c fix)
- #31 (partial YAML merge)
