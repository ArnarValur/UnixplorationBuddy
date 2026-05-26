# Source: TheOracle v2.1 @ 2026-05-25

---
name: grill
description: "Repeatable domain refinement — sharpen context.md, batch ADR proposals, offer to write/update prd.md when product scope crystallizes."
reads:
  - conductor/project-context.md
  - conductor/context.md
  - conductor/context-map.md
  - conductor/prd.md
  - conductor/adr/
writes:
  - conductor/context.md
  - conductor/prd.md         # conditional — only when product scope emerges
  - conductor/adr/*.md       # conditional — batched at command end, user-approved
  - conductor/index.md       # link appends only, via protocols/index-sync.md
---

# 🎵 Grill — Domain Refinement Session

When the user invokes `/grill`, execute this repeatable interview loop to sharpen the project's domain language, batch architectural decisions, and lazily produce a living PRD as scope crystallizes.

> **What this is NOT:** `/grill` is separate from the native Antigravity `/grill-me`. The native command is left untouched. `/grill` is the Conductor-aware variant that produces written output (`context.md` updates, ADRs, optionally `prd.md`).

> **Repeatable by design.** Three back-to-back `/grill` runs produce three separate ADR batches, not one giant batch. Each run is scoped to its own conversation context.

---

## Step 1: Pre-flight

1. Resolve `conductor/` via [`protocols/file-resolution.md`](../protocols/file-resolution.md). If `conductor/` does not exist, halt with:
   > "Conductor is not initialized in this project. Run `/conductor-init` first."
2. Read `conductor/project-context.md`. This file is the identity + operational source of truth — never write to it from `/grill`.

---

## Step 2: Read Existing Context

Load all v2.1 context documents that exist (lazy files may not — that's fine):

| File | If present, treat as |
|------|----------------------|
| `conductor/context.md` | Authoritative current domain glossary |
| `conductor/context-map.md` | Bounded-context registry (multi-context projects only) |
| `conductor/prd.md` | Current product scope — features, capabilities, in/out of scope |
| `conductor/adr/*.md` | Settled architectural decisions — do NOT re-litigate without explicit user request |

For files that do not exist, note their absence internally. Do not pre-create them.

---

## Step 3: Orientation Summary (Re-entry Contract)

**MANDATORY before the first question.** Produce a 1-paragraph orientation summary of what's already established. This prevents re-litigating settled questions.

Template:

> Orientation: This project is **{name from project-context}**, a **{tagline}** built on **{tech stack one-liner}**. The domain glossary currently defines **{N entities}** with the central one being **{primary entity}**. There are **{M ADRs}** on record — most recently *{title of most recent ADR}* on {date}. {If PRD exists: The PRD covers **{1-line scope summary}**.} {Else: No PRD has been written yet.}

If no `context.md` exists yet, adapt: *"The domain glossary is empty — this is the first refinement pass."*

Present the orientation, then ask the user:

> "Does this match your understanding? Anything to correct before we begin?"

Apply any corrections inline, then proceed to Step 4.

---

## Step 4: Refinement Topic Selection

Ask the user what they want to refine in this session:

| Focus | Outcome |
|-------|---------|
| **Domain language** | Sharpen `context.md` — definitions, entity relationships, terminology boundaries |
| **Product scope** | Crystallize `prd.md` — features, capabilities, what's in / out |
| **Architectural decision** | Talk through a specific decision and propose ADRs |
| **General exploration** | Open-ended interview — surface candidate domain terms, decisions, scope edges |

The user may pick more than one. Mix freely during the interview.

---

## Step 5: Interview Loop

**One question at a time.** Wait for the response before asking the next. For each question:

1. **Read the codebase first** (when the question is grounded in code). Cite specific files and lines in your question.
2. **Recommend 2–3 answers** based on what you've read, plus a write-in option. Default recommendations are based on the project's established conventions in `project-context.md`, `context.md`, and `adr/`.
3. **Be specific.** Vague questions produce vague answers. Anchor every question to a concrete entity, file, or decision.

### What to accumulate as you go

As the conversation unfolds, maintain three internal lists. **Do NOT write to disk during the interview** — only at command end (Step 7).

| List | Triggers | Example entry |
|------|----------|---------------|
| **Domain term proposals** | A new entity / relationship / terminology boundary surfaces | "Entity: `Booking` — a confirmed reservation for one or more travellers, distinct from `Inquiry`" |
| **PRD updates** | A capability, feature, in/out-of-scope statement, or NFR crystallizes | "Feature: Multi-day booking flow (in scope)" |
| **ADR candidates** | A decision satisfies all three criteria: hard to reverse, surprising without context, real trade-off | "Chose SurrealDB over PostgreSQL for embedded graph queries" |

If the user is uncertain, **propose** based on the codebase or established context, then let them confirm/refine.

### Stopping conditions

- The user says "that's enough", "we're done", or similar
- A natural pause point: three consecutive answers reached without new candidate entries
- Token / time budget signals the session should wrap

---

## Step 6: Domain Glossary Update

For each **Domain term proposal** accumulated:

1. If `context.md` does NOT exist, create it from the greenfield template (see [`conductor-v2.1-design-brief.md`](../conductor-v2.1-design-brief.md) → `context.md` Templates → Greenfield).
2. Present each proposal to the user as a single batch:
   > "These terms came up — should I add them to the glossary?"
   > {list each term with proposed definition}
3. For approved terms, append rows to the `## Entities` table and (when relevant) `## Relationships` and `## Terminology Boundaries` sections of `context.md`.
4. Update the `> Last refined: {datetime}` line in the file header.

If `context.md` was created this run, queue an index-sync append (handled in Step 8).

---

## Step 7: PRD Offer

If any **PRD updates** were accumulated in Step 5:

1. If `prd.md` does NOT yet exist, ask:
   > "Several scope items surfaced today. Want me to start a PRD at `conductor/prd.md`?"
   - If yes, create `prd.md` with these sections:
     - `## Overview` — 1-paragraph product description (pull from `project-context.md`)
     - `## In Scope` — capabilities the product will deliver
     - `## Out of Scope` — explicit non-goals
     - `## Open Questions` — items requiring future grills
   - Queue an index-sync append for `prd.md`.
   - If no, drop the PRD updates list silently. They can resurface in a later grill.
2. If `prd.md` already exists, ask:
   > "Want me to update the PRD with what we discussed?"
   - If yes, apply updates surgically (no rewrite). Append a `## Update — {datetime}` block summarizing the changes.

---

## Step 8: ADR Batch (Command-End)

This is the D3 + D10 batching point — runs once, at command end.

1. **Filter candidates.** Re-check each ADR candidate against the three criteria (hard to reverse, surprising without context, real trade-off). Drop anything that doesn't meet all three.
2. **Present the batch:**
   > "These decisions crystallized this session. Which should be recorded as ADRs?"
   > {numbered list of candidates with proposed titles + 1-sentence summary}
3. For each candidate, the user can: **approve**, **reject**, or **defer** (re-surface in a future grill — keep in pulse Session Memory).
4. For each approved candidate, write `conductor/adr/{NNNN}-{short-title-kebab}.md` using the format in [`conductor-v2.1-design-brief.md`](../conductor-v2.1-design-brief.md) → D3:

   ```markdown
   # {Short title of the decision}

   > **Recorded:** {YYYY-MM-DD HH:MM}
   > **Status:** accepted

   {1–3 sentences: what's the context, what did we decide, and why.}
   ```

   Optional sections (only when they add genuine value): `## Considered Options`, `## Consequences`, `## Superseded by`.

5. **Number sequentially** — read `conductor/adr/` to find the highest existing `NNNN` and increment.
6. **No double-processing.** Settled candidates (approved or explicitly rejected here) MUST NOT resurface in `/checkpoint`'s decision classifier (see [S1] in the design brief, applied in `workflows/checkpoint.md`).

If this is the **first ADR** ever written for the project, queue an index-sync append for the `adr/` directory.

---

## Step 9: Index Sync

For every lazy file or directory created this session, apply the rules in [`protocols/index-sync.md`](../protocols/index-sync.md). Cases:

- `context.md` created → append `- [Domain Glossary](./context.md)` under `## Context`
- `prd.md` created → append `- [Product Requirements](./prd.md)` under `## Context`
- First ADR written → create `## Decisions` section + append `- [ADR Directory](./adr/)`

Re-running an append must be a no-op (idempotent check before writing).

---

## Step 10: Git Commit

Stage and commit all `conductor/` changes:

```bash
git add conductor/
git commit -m "grill: {1-line summary of focus} ({N glossary updates}, {M ADRs}, {prd: yes|no})"
```

If nothing changed (no glossary edits, no ADRs approved, no PRD touch), skip the commit and tell the user:

> "Nothing new to record from this session — orientation was useful but no candidates landed."

---

## Step 11: Confirm

Tell the user what landed:

> "✅ Grill complete.
> - Domain glossary: **{N terms added}**
> - ADRs recorded: **{M}** ({list titles})
> - PRD: **{created | updated | unchanged}**
> - Index: **{K links appended}**
>
> Next: `/new-track` is now domain-aware on this glossary, or run `/grill` again to refine further."

---

## Session Behavior

While `/grill` is active:

- Treat `conductor/project-context.md` as read-only (no command writes here — see brief D7 + [S3]).
- Treat `conductor/docs/` as read-only (no command writes here — see brief D12 + [P1]).
- Treat ADRs in `conductor/adr/` as settled. Surface them in answers but never edit them; superseding ADRs use the `## Superseded by` link.
- Never auto-create empty stubs for lazy files. Lazy = "exists only when there's something to write."
- Never write a dead link to `index.md`.
