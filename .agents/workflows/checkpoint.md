---
description: Save session state and classify accumulated decisions into ADR, pulse, or drop. The ADR gate.
---

# Checkpoint — Save Session State

When the user invokes `/checkpoint`, execute this sequence to persist the current session state. **Step 3 is the ADR gate** — it classifies accumulated decisions into architectural (→ ADR), operational (→ pulse Session Memory), and ephemeral (→ drop). Each decision ends up in exactly one place.

Supports `--quick` flag: `/checkpoint --quick` skips Step 3 (the decision classifier).

---

## Step 0: Pre-flight

Verify that `conductor/` exists in the project root. If it does not, halt with:
> "Conductor is not initialized. Run `/conductor-init` to set up the project."

Verify that `conductor/project-context.md` exists. If it does not, halt with:
> "`conductor/project-context.md` is missing — the conductor is broken. Run `/conductor-init` to repair."

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

> **Note on Session Memory:** Append the session's work summary here. Leave room for Step 3d to append classified operational decisions below — do not overwrite the section, only append.

### 200-Line Archiving Guardrail

After updating, check if `pulse.md` exceeds 200 lines. If it does:

1. **Session Memory:** Keep only the **last 2 sessions**. Move older entries to `conductor/pulse-archive/{YYYY-MM-DD}.md`.
2. **Recently Completed:** Keep only the **last 5 entries**. Move older rows to the same archive file.
3. Add a reference below each trimmed section:
   > 📦 Full history: `conductor/pulse-archive/{YYYY-MM-DD}.md`
4. Archive files are **append-only** — add new archived content at the bottom of existing files.

---

## Step 3: Decision Classifier

> **Skipped with `--quick` flag.**

### Step 3a: Scope (no double-processing)

`/checkpoint` only classifies decisions that were **not already handled by a command-end batch**. Source these from:

- The Session Memory entries that were appended during free-form work (no `/grill` or `/new-track` active at the time).
- Decisions inferred from conversation context that surfaced outside any batching command's session.

Decisions **already approved** by a prior `/grill` or `/new-track` ADR batch are already in `conductor/adr/` — do NOT re-record them.

Decisions **explicitly rejected** by a prior batch are settled-as-dropped — do NOT re-surface them. Re-surfacing a deliberate rejection erodes trust in the batching UX.

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
| **ADR** (architectural) | Hard to reverse, surprising without context, real trade-off | "Chose token-bucket over leaky-bucket", "Adopted event sourcing for orders" |
| **Pulse** (operational) | Workflow/process notes, session-scoped reminders, future-self breadcrumbs | "Sprinted on the navbar — finished header, hamburger pending" |
| **Drop** (ephemeral) | Thinking-out-loud, exploratory tangents that didn't pan out, transient debugging notes | "Tried `npx clear-cache` first, didn't help" |

### Step 3c: Write ADR-bucket decisions

For each decision the user classified as **ADR**:

1. Number sequentially from the highest existing `conductor/adr/NNNN-*.md`.
2. Write `conductor/adr/{NNNN}-{short-title-kebab}.md` using this format:

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

If a first ADR was written this checkpoint (queued in Step 3c), update `conductor/index.md`:

1. Create a `## Decisions` section (if it doesn't exist).
2. Append `- [ADR Directory](./adr/)` under that section.

Idempotency: if the link already exists, skip (no-op). Never write a dead link.

---

## Step 8: Confirm

Tell the user:

> "✅ Checkpoint saved.
>
> - Decisions classified: **{N total}** → **{ADR_count}** ADR, **{Pulse_count}** Pulse, **{Drop_count}** dropped.
> - Tracks touched: **{list}**.
> - Session state captured in `pulse.md`{; archived to pulse-archive/{date}.md if guardrail tripped}.
>
> **Options:**
>
> - Continue working on current track
> - Switch to a different track
> - End session"
