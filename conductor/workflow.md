<!-- Template: TheOracle v2.1 | Mode: light -->
# Project Workflow — Light

> Streamlined workflow for prototypes, websites, and non-product projects.
> Plan → Execute → Verify. Testing encouraged, not mandated.

---

## Guiding Principles

1. **The Plan is the Source of Truth:** All work must be tracked in `tracks.md`
2. **The Tech Stack is Deliberate:** Changes to the tech stack must be documented in the **Tech Stack** section of `project-context.md` *before* implementation. Architecturally significant changes (those satisfying the three-criteria ADR test: hard to reverse, surprising without context, real trade-off) additionally warrant an ADR in `conductor/adr/` — surface as a candidate at `/grill`, `/new-track`, or `/checkpoint` time.
3. **Ship Early, Iterate Fast:** Prioritize working software over ceremony
4. **Test Where It Matters:** Write tests for complex logic, critical paths, and fragile code — skip boilerplate coverage
5. **User Experience First:** Every decision should prioritize user experience
6. **Non-Interactive & CI-Aware:** Prefer non-interactive commands. Use `CI=true` for watch-mode tools

---

## Task Workflow

### Standard Task Lifecycle

1. **Select Task:** Choose the next available task from `tracks.md` in sequential order

2. **Mark In Progress:** Edit `tracks.md` and change the task from `[ ]` to `[~]`

3. **Plan Approach:**
   - Review the task requirements and acceptance criteria
   - Identify any dependencies or blockers
   - Decide if tests are valuable for this task (complex logic, regressions, integrations)

4. **Execute:**
   - Implement the feature or fix
   - Write tests if planned in step 3 (not mandatory but recommended for non-trivial logic)
   - Verify the implementation works as expected

5. **Verify:**
   - Run any existing tests to ensure nothing is broken: `CI=true <test command>`
   - Manual verification — confirm the feature works as intended
   - Check for obvious regressions

6. **Document Deviations:** If implementation differs from tech stack:
   - **STOP** implementation
   - Update the **Tech Stack** section of `project-context.md` with the new design
   - Add a dated note explaining the change (user-edited; no command writes to `project-context.md` post-init per S3)
   - If the change is architecturally significant (three-criteria ADR test), surface it as an ADR candidate at the next `/grill`, `/new-track`, or `/checkpoint`
   - Resume implementation

7. **Commit Code Changes:**
   - Stage all code changes related to the task
   - Commit with a clear, concise message following conventional commits format
   - Example: `feat(landing): Add hero section with CTA`

8. **Attach Task Summary with Git Notes:**
   - **8.1:** Get commit hash: `git log -1 --format="%H"`
   - **8.2:** Draft note content — task name, summary of changes, list of created/modified files
   - **8.3:** Attach note:
     ```bash
     git notes add -m "<note content>" <commit_hash>
     ```

9. **Record Task Completion:**
   - In `tracks.md`, update the completed task from `[~]` to `[x]` and append the first 7 characters of the commit hash

10. **Commit Plan Update:**
    - Stage `tracks.md`
    - Commit: `conductor(tracks): Mark task '<task name>' as complete`

---

## Phase Completion — Checkpointing Protocol

**Trigger:** Executed immediately after a task is completed that also concludes a phase in `tracks.md`.

1. **Announce Protocol Start:** Inform the user that the phase is complete and checkpointing has begun

2. **Run Existing Tests (if any):**
   - Announce the exact shell command before running
   - Execute the test command
   - If tests fail: inform user, attempt fix (max 2 attempts). If still failing, **stop and ask for guidance**

3. **Propose Manual Verification Plan:**
   - Analyze `project-context.md` (Product Definition section), `prd.md` (when present, for current scope), and `tracks.md` to determine the phase's user-facing goals
   - Generate step-by-step verification instructions with specific commands and expected outcomes

4. **Await User Feedback:**
   - Ask: "Does this meet your expectations? Please confirm with yes or provide feedback."
   - **PAUSE.** Do not proceed without explicit confirmation

5. **Create Checkpoint Commit:**
   - Stage all changes (or create empty commit if no changes)
   - Commit: `conductor(checkpoint): Checkpoint end of Phase X`

6. **Attach Verification Report via Git Notes:**
   - Draft report including test results (if any), manual verification steps, and user confirmation
   - Attach to checkpoint commit via `git notes add`

7. **Record Phase Checkpoint SHA:**
   - In `tracks.md`, append `[checkpoint: <7-char-sha>]` to the completed phase heading

8. **Commit Plan Update:**
   - Stage `tracks.md`
   - Commit: `conductor(tracks): Mark phase '<PHASE NAME>' as complete`

9. **Announce Completion:** Inform user that the phase is complete with checkpoint created

---

## Commit Guidelines

### Message Format
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding missing tests
- `chore`: Maintenance tasks
- `conductor`: Conductor file updates (plan, checkpoint, tracks)

### Examples
```bash
git commit -m "feat(auth): Add remember me functionality"
git commit -m "fix(layout): Correct mobile nav overflow"
git commit -m "chore(deps): Update dependencies"
git commit -m "conductor(tracks): Mark task 'Build landing page' as complete"
```

---

## Definition of Done

A task is complete when:

1. Feature implemented and working as intended
2. Existing tests still pass (no regressions)
3. Tests written for complex/critical logic (where valuable)
4. Code follows project style guidelines
5. Works on mobile (if applicable)
6. Implementation recorded in `tracks.md`
7. Changes committed with proper message
8. Git note with task summary attached

---

## Development Commands

> **Customize this section per project.** Replace examples with actual project commands.

### Setup
```bash
# Install dependencies and configure environment
# e.g., npm install / go mod tidy / pip install -r requirements.txt
```

### Daily Development
```bash
# Start dev server, run tests, lint
# e.g., npm run dev / go run main.go
```

### Before Committing
```bash
# Run pre-commit checks: format, lint, test
# e.g., npm run check / make check
```

---

## When to Add Tests

Tests are not mandatory in light mode but are strongly recommended for:

- **Complex business logic** — calculations, state machines, parsers
- **Data transformations** — serialization, API response mapping
- **Integration points** — API clients, database queries
- **Regression-prone areas** — code that has broken before
- **Security-sensitive paths** — auth, input validation, permissions

Skip tests for:
- Static content pages
- Simple CRUD with no logic
- One-off scripts and prototypes
- Pure UI layout (use visual verification instead)

---

## Emergency Procedures

### Critical Bug in Production
1. Create hotfix branch from main
2. Implement minimal fix
3. Verify fix works
4. Deploy immediately
5. Document in tracks.md

### Data Loss
1. Stop all write operations
2. Restore from latest backup
3. Verify data integrity
4. Document incident

### Security Breach
1. Rotate all secrets immediately
2. Review access logs
3. Patch vulnerability
4. Document and update security procedures
