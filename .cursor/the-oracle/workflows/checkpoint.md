# Source: TheOracle v2.1 @ 2026-05-25

---
name: checkpoint
description: "Save session state and classify accumulated decisions. Updates pulse.md, archives if needed, repurposes Step 3 as the ADR gate (architectural → conductor/adr/, operational → pulse Session Memory, ephemeral → drop), and only processes decisions NOT already handled by a /grill or /new-track command-end batch."
reads:
  - conductor/pulse.md
  - conductor/relay.md
  - conductor/tracks.md
  - conductor/tracks/*/plan.md
  - conductor/tracks/*/metadata.json
  - conductor/adr/              # for sequential ADR numbering
writes:
  - conductor/pulse.md
  - conductor/pulse-archive/{YYYY-MM-DD}.md   # conditional — 200-line guardrail
  - conductor/relay.md
  - conductor/tracks.md
  - conductor/tracks/*/plan.md
  - conductor/tracks/*/metadata.json
  - conductor/adr/*.md          # conditional — classifier-approved decisions
  - conductor/index.md          # link appends only, via protocols/index-sync.md
---

# 🎵 Checkpoint — Save Session State (v2.1 ADR gate)

When the user invokes `/checkpoint`, execute this sequence to persist the current session state. **In v2.1, Step 3 is the ADR gate** — it classifies accumulated decisions into architectural (→ ADR), operational (→ pulse Session Memory), and ephemeral (→ drop). This replaces v2.0's "Decision Log Update", which appended everything into `pulse.md` and created competing decision logs.

Supports `--quick` flag: `/checkpoint --quick` skips Step 3 (the decision classifier).

---

## Step 1: Session Summary

Determine the session focus. Either:

- Infer from conversation context (preferred), or
- Ask the user: "What was the main focus of this session?"

Gather:

- What was worked on
- What was completed
- What is still in progress
- Any decisions made (free-form — these feed Step 3)
- Suggested next steps

---

## Step 2: Update `conductor/pulse.md`

Update the following sections in `conductor/pulse.md`:

```markdown
**Last Updated:** {current timestamp}
**Session Focus:** {summary}

## 🚀 Active Tracks
## ✅ Recently Completed
## ⚠️ Blockers
## 🧠 Session Memory
## 📋 Next Session Suggestions
```

These section names are parsed by Conductor. **Do not rename them.**

### 200-Line Archiving Guardrail

After updating, check if `pulse.md` exceeds 200 lines. If it does:

1. **Session Memory:** Keep only the **last 2 sessions**. Move older entries to `conductor/pulse-archive/{YYYY-MM-DD}.md`.
2. **Recently Completed:** Keep only the **last 5 entries**. Move older rows to the same archive file.
3. Add a reference below each trimmed section:
   > 📦 Full history: `conductor/pulse-archive/{YYYY-MM-DD}.md`
4. Archive files are **append-only** — add new archived content at the bottom of existing files.

---

## Step 3: Decision Classifier (the ADR Gate — D11 + S1)

> **Skipped with `--quick` flag.**
>
> **v2.1 redefines this step.** v2.0 unconditionally appended every decision into `pulse.md`'s Session Memory, creating two competing decision logs (ephemeral in pulse, durable in `adr/`). v2.1 classifies decisions at checkpoint time so each one ends up in exactly one place.

### Step 3a: Scope (S1 — no double-processing)

`/checkpoint` only classifies decisions that were **not already handled by a command-end batch**. Source these from:

- The Session Memory entries that were appended during free-form work (no `/grill` or `/new-track` active at the time).
- Decisions inferred from conversation context that surfaced outside any batching command's session.

Decisions **already approved** by a prior `/grill` or `/new-track` ADR batch are already in `conductor/adr/` — do NOT re-record them.

Decisions **explicitly rejected** by a prior batch are settled-as-dropped — do NOT re-surface them. Re-surfacing a deliberate rejection erodes trust in the batching UX (see brief D11 + S1).

If unsure whether a decision was already handled, ask the user — do not silently re-prompt.

### Step 3b: Classify

Present the unhandled decisions to the user as a single batch:

> "I noticed **{N}** decisions this session that weren't already batched into ADRs. Classify each:
>
> | # | Decision | A → ADR (architectural) | B → Pulse (operational) | C → Drop (ephemeral) |
> |---|----------|-------------------------|-------------------------|----------------------|
> | 1 | {decision summary} | | | |
> | … | … | | | |"

Per-bucket guidance (apply the same three criteria as `/grill` for the ADR bucket):

| Bucket | Test | Examples |
|--------|------|----------|
| **ADR** (architectural) | Hard to reverse, surprising without context, real trade-off | "Chose token-bucket over leaky-bucket", "Adopted event sourcing for orders", "Pinned Vue 3.5+ for `<script setup>` features" |
| **Pulse** (operational) | Workflow/process notes, session-scoped reminders, future-self breadcrumbs | "Sprinted on the navbar — finished header, hamburger pending", "Considered Sentry but parked it for later" |
| **Drop** (ephemeral) | Thinking-out-loud, exploratory tangents that didn't pan out, transient debugging notes | "Tried `npx clear-cache` first, didn't help", "Wondered if SSR was the issue — it wasn't" |

### Step 3c: Write ADR-bucket decisions

For each decision the user classified as **ADR**:

1. Number sequentially from the highest existing `conductor/adr/NNNN-*.md`.
2. Write `conductor/adr/{NNNN}-{short-title-kebab}.md` using the format in [`conductor-v2.1-design-brief.md`](../conductor-v2.1-design-brief.md) → D3:

   ```markdown
   # {Short title of the decision}

   > **Recorded:** {YYYY-MM-DD HH:MM}
   > **Status:** accepted

   {1–3 sentences: what's the context, what did we decide, and why.}
   ```

   Optional sections (only when they add value): `## Considered Options`, `## Consequences`, `## Superseded by`.

3. If this is the **first ADR** for the project, queue an index-sync append for the `adr/` directory (applied in Step 7).

### Step 3d: Pulse-bucket decisions

For decisions classified as **Pulse**, append them to the Session Memory section of `pulse.md` using a compact format:

```markdown
- *{date}* — {one-line decision summary} _(operational)_
```

These will be retained for the standard 200-line / 2-session archiving guardrail from Step 2.

### Step 3e: Drop-bucket decisions

For decisions classified as **Drop**, do nothing. They remain only in the conversation transcript.

---

## Step 4: Track Status Check

For any tracks worked on during this session:

1. Update the track's `plan.md` — mark completed tasks with `[x]`.
2. Update `metadata.json` — set `status` and `updated_at` fields.
3. If a track is fully completed:
   - Update `metadata.json` status to `completed`.
   - Move the track entry from "Active Tracks" to "Completed Tracks" in `conductor/tracks.md`.

---

## Step 5: Relay Handoff Entry

Append a timestamped entry to `conductor/relay.md`:

```markdown
## {YYYY-MM-DD HH:MM}
- **Session:** {focus summary}
- **Tracks touched:** {list of track IDs}
- **Status:** {brief status}
- **Decisions:** {ADR titles recorded this checkpoint, or "None"}
- **Next:** {suggested next actions}
```

> The `Decisions:` field lists only the ADRs written in Step 3c, not the pulse-bucket items. Pulse-bucket items stay scoped to `pulse.md` for the session.

---

## Step 6: Git Commit

Stage and commit all conductor changes:

```bash
git add conductor/
git commit -m "checkpoint: {brief summary} ({M ADRs})"
```

Simplify to `checkpoint: {brief summary}` when no ADRs were recorded.

---

## Step 7: Index Sync

If a first ADR was written this checkpoint (queued in Step 3c), apply [`protocols/index-sync.md`](../protocols/index-sync.md) to create the `## Decisions` section and append `- [ADR Directory](./adr/)` to `conductor/index.md`. Idempotent — no-op when the link already exists.

---

## Step 8: Confirm

Tell the user:

> "✅ Checkpoint saved.
> - Decisions classified: **{N total}** → **{ADR_count}** ADR, **{Pulse_count}** Pulse, **{Drop_count}** dropped.
> - Tracks touched: **{list}**.
> - Session state captured in `pulse.md`{; archived to pulse-archive/{date}.md if guardrail tripped}.
>
> **Options:**
> - Continue working on current track
> - Switch to a different track
> - End session"
