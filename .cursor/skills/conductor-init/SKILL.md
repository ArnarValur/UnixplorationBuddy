---
name: conductor-init
description: "Initialize Conductor in a project (TheOracle v2.1). Scaffolds structured development with consolidated identity + operational context, tech stack, workflow mode, code style guides, lazy adr/ and docs/ directories, and optional brownfield targeted-domain scan. Use when starting a new project, running /conductor-init in Cursor, or upgrading to v2.1."
disable-model-invocation: true
---

# Conductor Init (v2.1)

Scaffold a Conductor-managed project through an interactive grill.

## Resolve TheOracle root

Use the first path that exists:

1. **Project bundle:** `.cursor/the-oracle/` (deployed by a prior init in this repo)
2. **`THEORACLE_HOME`** environment variable
3. **Default:** `~/Hermes/TheOracle`

## Instructions

1. Read `{THEORACLE_ROOT}/protocols/file-resolution.md`
2. Read `{THEORACLE_ROOT}/protocols/index-sync.md`
3. Read and execute `{THEORACLE_ROOT}/workflows/conductor-init.md` step by step.

Templates and style guides always load from the canonical install at `~/Hermes/TheOracle/templates/` unless `THEORACLE_HOME` points elsewhere.
