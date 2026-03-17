---
id: 24
title: 'TUI: In-app node editing (create/edit/deprecate/add edges)'
status: backlog
priority: medium
created: 2026-03-17T17:43:40.234663+01:00
updated: 2026-03-17T17:43:40.234663+01:00
tags:
    - tui
    - editing
depends_on:
    - 19
class: standard
---

Add mutation capabilities to the TUI:
- c: create node (mini-form: id, content, weight)
- e: edit current node (open $EDITOR or inline editing)
- d: deprecate current node (with confirmation dialog)
- a: add edge from current node to another (pick from list)
- Must sync app.nodes with disk after each mutation (YAML -> index -> SQLite)
