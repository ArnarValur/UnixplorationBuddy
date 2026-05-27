---
description: Resume session context, reconcile the conductor index, and present project status with actionable options
---

# Conductor — Resume Protocol

When the user invokes `/conductor`, execute the following sequence to restore session context, self-heal the index, and present actionable status.

---

## Step 1: Load Context

Read the following files from the project's `conductor/` directory. If any file is missing, note it but do not halt — many files are created lazily.

### 1a. Required (halt if missing)

| File | Purpose |
|------|---------|
| `conductor/project-context.md` | Identity + operational — product, tech stack, guidelines, caution levels, constraints |
| `conductor/workflow.md` | Strict vs Light workflow rules |
| `conductor/pulse.md` | Current project state, recent decisions, next steps |
| `conductor/tracks.md` | All tracks organized by domain |

> **IMPORTANT:** If `conductor/` does not exist at all, halt immediately and tell the user:
>
> > "Conductor is not initialized. Run `/conductor-init` to set up the project."
>
> If `conductor/project-context.md` is missing but `conductor/` exists, halt and tell the user:
>
> > "`conductor/project-context.md` is missing — the conductor is broken. Run `/conductor-init` to repair (it will detect the existing conductor and offer a re-init / migration path)."

### 1b. Optional (absence is a valid state — lazy files)

| File / dir | Purpose | When present, treat as |
|------------|---------|------------------------|
| `conductor/context.md` | Domain glossary | Authoritative domain language |
| `conductor/context-map.md` | Bounded-context registry | Multi-context project |
| `conductor/prd.md` | Living product requirements | Current scope |
| `conductor/adr/*.md` | Architectural decision records | Settled decisions — do NOT re-litigate without explicit user request |
| `conductor/docs/` | Long-form human-authored documentation | Awareness only — list the directory but do not read all contents; load specific files on user request |
| `conductor/agent-rules/*.md` | Book-sourced coding rules (from agent-rules plugin) | Read all `.md` files in this directory |
| `conductor/relay.md` | Cross-session handoff messages | Last session's notes — surface in the status report |

For lazy files that are absent, note the absence internally. The user may want to create them via `/grill` or `/new-track`.

---

## Step 2: Defensive Index Reconcile

Reconcile `conductor/index.md` against actual files on disk:

1. Scan `conductor/` for these files and check if `index.md` has a matching link:
   - `context.md` → should appear as `- [Domain Glossary](./context.md)` under `## Context`
   - `context-map.md` → `- [Context Map](./context-map.md)` under `## Context`
   - `prd.md` → `- [Product Requirements](./prd.md)` under `## Context`
   - `agent-rules/` → `- [Agent Rules](./agent-rules/)` under `## Context`
   - First real file in `adr/` (not `.gitkeep`) → `- [ADR Directory](./adr/)` under `## Decisions` (create section if missing)
   - Real files in `docs/` → `- [Project Docs](./docs/)` under `## Documentation` (create section if missing)
2. For each disk file with no matching link → queue an append.
3. For each link in `index.md` whose target does not exist on disk → queue a **dead-link warning** (do NOT auto-remove).
4. Apply queued appends idempotently (no-op when the link already exists).
5. **Report inline** in the status output:
   - `N links added` (silently OK if N=0)
   - `M dead links` (with paths — surface to the user as bugs to fix)
   - `K orphans` (informational only — files on disk with no matching rule)

This step is idempotent and safe to run on every `/conductor` invocation.

---

## Step 3: Status Report

Generate a structured status overview of all tracks and domain knowledge.

The status report should present:

```text
🎵 Conductor Online

📍 Last Session: {date from pulse.md} — {focus}
🔄 Active Tracks: {count}
⚠️ Blockers: {count or "None"}

📚 Domain knowledge:
  • Glossary: {N terms in context.md, or "not yet — run /grill"}
  • PRD: {present | not yet}
  • ADRs: {M recorded, most recent: "{title}" on {date}}
  • Docs: {K files in conductor/docs/, or "none"}

🛠️ Index:
  • {N links added by reconcile, or omitted if 0}
  • {M dead links found, with paths if any}

Quick Status:
{list of active tracks with one-line status each}

Ready for orders, Captain. What's our heading?
```

Parse `conductor/tracks.md` to identify all registered tracks and their paths. For each active track, read its `metadata.json` and `plan.md` to determine:

- Current phase and task in progress
- Overall progress (tasks completed / total)
- Any blockers

---

## Step 4: Await Orders

Present the following options to the user:

| Action | Description |
|--------|-------------|
| **Grill** | Refine domain language, batch ADRs, optionally write PRD → invoke `/grill` |
| **New Track** | Create a new track (domain-aware) → invoke `/new-track` |
| **Implement** | Pick a track and start working on tasks from its `plan.md` |
| **Review** | Review completed work on a track |
| **Checkpoint** | Save session state and classify decisions → invoke `/checkpoint` |
| **Revert** | Roll back recent changes on a track |

Wait for the user's selection and proceed accordingly:

- **Grill** → Tell the user to invoke `/grill` or begin the grill protocol inline.
- **New Track** → Tell the user to invoke `/new-track` or begin the new-track protocol inline. Mention that the spec interview will be domain-aware on top of `context.md` / `adr/` / `prd.md`.
- **Implement** → Load the selected track's `plan.md`, find the next pending task, and begin implementation. Follow the workflow rules in `conductor/workflow.md` and the ADRs in `conductor/adr/` (do not re-litigate settled decisions).
- **Review** → Read the track's `spec.md` and `plan.md`, verify completed tasks against acceptance criteria, and present a review summary.
- **Checkpoint** → Tell the user to invoke `/checkpoint` or begin the checkpoint protocol inline.
- **Revert** → Identify the target track and changes to revert. Confirm with the user before executing any destructive operations.

---

## Session Behavior

Once initialized, maintain awareness of:

- Current track context throughout the session
- Workflow rules from `conductor/workflow.md`
- Domain language from `conductor/context.md` — use these terms verbatim when discussing the project
- Settled architectural decisions from `conductor/adr/*.md` — defer to them unless the user explicitly asks to reopen a decision (in which case the result is a new ADR that supersedes the old one, not an edit)
- Caution levels from `conductor/project-context.md`
- Any blockers or urgent items from `conductor/pulse.md`
- **`conductor/project-context.md` is user-owned.** Do not propose edits to it during a session. Surface them at `/checkpoint` time as Pulse-bucket items if they truly need recording.
- **`conductor/docs/` has no command writers.** Do not auto-generate documentation there. Reading is fine on user request.
