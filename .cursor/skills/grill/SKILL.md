---
name: grill
description: "Repeatable domain-refinement for a Conductor project. Sharpens context.md, batches ADR proposals, optionally writes prd.md. Use when the user runs /grill in Cursor or wants domain glossary and architecture decisions after /conductor-init. Not the native Antigravity /grill-me."
disable-model-invocation: true
---

# Grill (Conductor)

Domain refinement, ADR batching, and lazy PRD writing for a project initialized with `/conductor-init`.

## Resolve TheOracle root

Use the first path that exists:

1. **Project bundle:** `.cursor/the-oracle/`
2. **`THEORACLE_HOME`**
3. **Default:** `~/Hermes/TheOracle`

## Instructions

1. Read `{THEORACLE_ROOT}/protocols/file-resolution.md`
2. Read `{THEORACLE_ROOT}/protocols/index-sync.md`
3. Read and execute `{THEORACLE_ROOT}/workflows/grill.md` step by step.

## When NOT to use

- No `conductor/` directory → run `/conductor-init` first
- Native Antigravity grill-me experience → use `/grill-me` instead
- New feature track → use `/new-track`
