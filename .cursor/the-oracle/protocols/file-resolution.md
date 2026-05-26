# File Resolution Protocol

<!-- Source: TheOracle v2.1 @ 2026-05-25 -->

**PROTOCOL: How to locate conductor files within any project.**

To find a file (e.g., "**Product & Operational Context**") within a specific context (Project Root or a specific Track):

## Step 1: Identify Index

Determine the relevant index file:

- **Project Context:** `conductor/index.md`
- **Track Context:**
  1. Resolve and read the **Tracks Registry** (via Project Context)
  2. Find the entry for the specific `<track_id>`
  3. Follow the link provided in the registry to locate the track's folder. The index file is `<track_folder>/index.md`
  4. **Fallback:** If the track is not yet registered (e.g., during creation) or the link is broken:
     1. Resolve the **Tracks Directory** (via Project Context)
     2. The index file is `<Tracks Directory>/<track_id>/index.md`

## Step 2: Check Index

Read the index file and look for a link with a matching or semantically similar label.

> **Note (v2.1):** The project `index.md` is **dynamic** — links are appended lazily as files come into existence (see [`index-sync.md`](./index-sync.md)). A missing link does NOT mean the file is missing; it may mean the file hasn't been created yet. Fall through to Step 4 to check the default path.

## Step 3: Resolve Path

If a link is found, resolve its path **relative to the directory containing the `index.md` file**.

*Example:* If `conductor/index.md` links to `./workflow.md`, the full path is `conductor/workflow.md`.

## Step 4: Fallback

If the index file is missing or the link is absent, use the **Default Path** keys below.

## Step 5: Verify

You MUST verify the resolved file actually exists on the disk. Several v2.1 files are lazy — they only exist once something has been written to them (e.g., `context.md`, `prd.md`, anything in `adr/`). Absence of the file is a valid state; do not auto-create empty stubs.

## Default Paths (Project) — v2.1

| Document                          | Default Path                  | Lifecycle |
|-----------------------------------|-------------------------------|-----------|
| **Product & Operational Context** | `conductor/project-context.md` | Created at init, user-edited thereafter (no command writes here) |
| **Domain Glossary**               | `conductor/context.md`         | Lazy — created by `/grill` or brownfield `/conductor-init` |
| **Context Map**                   | `conductor/context-map.md`     | Optional — multi-bounded-context projects only |
| **Product Requirements**          | `conductor/prd.md`             | Lazy — created by `/grill` when product scope crystallizes |
| **Workflow**                      | `conductor/workflow.md`        | Created at init from strict or light template |
| **ADR Directory**                 | `conductor/adr/`               | Scaffolded with `.gitkeep` at init; files appended by `/grill`, `/new-track`, `/checkpoint` (batched) |
| **Documentation**                 | `conductor/docs/`              | Scaffolded with `.gitkeep` at init; human-authored only (no command writes) |
| **Agent Rules**                   | `conductor/agent-rules/`       | Optional — installed by the agent-rules plugin |
| **Tracks Registry**               | `conductor/tracks.md`          | Created at init |
| **Tracks Directory**              | `conductor/tracks/`            | Created at init |
| **Pulse**                         | `conductor/pulse.md`           | Created at init, updated by `/checkpoint` and `/conductor` |
| **Relay**                         | `conductor/relay.md`           | Created at init, updated by `/checkpoint` |

**Removed in v2.1** (consolidated into `project-context.md` per design brief D7):

- ~~`conductor/product.md`~~
- ~~`conductor/tech-stack.md`~~
- ~~`conductor/product-guidelines.md`~~

If any of these are encountered on a pre-v2.1 project, treat their content as authoritative for the corresponding `project-context.md` section until `/conductor-init` migration consolidates them (see brief D9).

## Default Paths (Track)

| Document                | Default Path                                                |
|-------------------------|-------------------------------------------------------------|
| **Specification**       | `conductor/tracks/<domain>/<track_id>/spec.md`              |
| **Implementation Plan** | `conductor/tracks/<domain>/<track_id>/plan.md`              |
| **Metadata**            | `conductor/tracks/<domain>/<track_id>/metadata.json`        |
