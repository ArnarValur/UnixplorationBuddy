# Index Sync Protocol

<!-- Source: TheOracle v2.1 @ 2026-05-25 -->

**PROTOCOL: How to keep `conductor/index.md` accurate as files come into existence lazily.**

> **Used by:** `/grill`, `/new-track`, `/checkpoint`, `/conductor`
>
> **Why this exists:** v2.1 creates several files lazily — they only appear once something has been written to them (`context.md`, `prd.md`, ADRs). Without a shared protocol, each writer would invent its own append logic and `index.md` would drift. This is also a hard requirement for Obsidian users: clicking a dead `index.md` link auto-creates an empty file, which would defeat lazy creation and pollute the repo.
>
> **Invariant:** `index.md` lists only files that actually exist on disk. No dead links, ever.

---

## Append Rules

When a writer creates a file (or directory) for the first time, it MUST append the corresponding link to `conductor/index.md` per this table. Subsequent writes to the same file are no-ops for index sync.

| File / dir created            | Section in `index.md` | Link text                                | Section behavior |
|-------------------------------|-----------------------|------------------------------------------|------------------|
| `context.md`                  | `## Context`          | `- [Domain Glossary](./context.md)`      | Append under existing section |
| `context-map.md`              | `## Context`          | `- [Context Map](./context-map.md)`      | Append under existing section |
| `prd.md`                      | `## Context`          | `- [Product Requirements](./prd.md)`     | Append under existing section |
| `agent-rules/`                | `## Context`          | `- [Agent Rules](./agent-rules/)`        | Append under existing section |
| First file in `adr/`          | `## Decisions` (new)  | `- [ADR Directory](./adr/)`              | Create section if missing |
| `.docs/` migrated to `docs/`  | `## Documentation` (new) | `- [Project Docs](./docs/)`           | Create section if missing |

**Notes:**

- Individual ADR files are NOT linked from the root `index.md` — only the directory link. ADRs are browsed via the directory listing or a future `adr/index.md`.
- Individual style guides, individual track files, and individual pulse-archive files follow the same rule: link the container, not each file.

---

## Append Algorithm

For each file-creation event in a workflow:

1. **Determine the link** from the Append Rules table.
2. **Read** `conductor/index.md`.
3. **Idempotency check:** if the exact link text already exists anywhere in the file, return (no-op). Re-running an append must never duplicate links.
4. **Target section lookup:**
   - If "Section behavior" is *Append under existing section* and the section heading exists, insert the link as the next bullet under that heading.
   - If "Section behavior" is *Create section if missing*, create the heading **before** the `## Quick Start` section (if present) or at the end of the file. Then insert the link as the first bullet under the new heading.
5. **Write** the updated `index.md`.
6. **Verify** the link target exists on disk (defensive — refuse to commit a dead link).

The workflow's reader/writer contract MUST declare `conductor/index.md` under `writes` if it can trigger any of these appends.

---

## Reconcile Path (`/conductor` resume)

`/conductor` may run a defensive reconcile at the start of a session to catch any drift (files created on disk without their corresponding index append — e.g., from external edits, prior buggy runs, or pre-v2.1 conductors).

### Algorithm

1. **Scan** `conductor/` for files matching the Append Rules table.
2. **Diff** against `conductor/index.md`:
   - For each disk file that has a rule but no matching link, queue an append.
   - For each link in `index.md` whose target does not exist on disk, queue a **dead-link warning** (do NOT auto-remove — surfaces a real bug; let the user decide).
3. **Apply** queued appends per the algorithm above.
4. **Report** to the user:
   - `N links added`
   - `M dead links found` (with paths)
   - `K orphans` (files on disk that exist but have no Append Rule — informational only)

The reconcile is **idempotent** and **safe to re-run**. It never deletes links automatically.

---

## What This Protocol Does NOT Do

- It does not write content into the linked files themselves — that is the calling workflow's responsibility.
- It does not validate the content of the linked files.
- It does not garbage-collect dead links automatically — that is a user decision (the dead-link warning surfaces them).
- It does not maintain track-level `index.md` files. Tracks have their own simpler indexing (see `protocols/new-track.md`).

---

## Future Extension: `conductor-doctor`

A future `conductor-doctor` command will use this protocol's reconcile algorithm as one of its checks. The protocol is designed to be the building block, not the user-facing diagnostic.
