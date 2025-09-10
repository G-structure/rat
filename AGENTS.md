# AI Agent Instructions

## Primary References
Always consult and maintain these files throughout every task:
- CLAUDE.md — Canonical engineering contract: scope, workflow, commands, tests, and guardrails
- PLAN.md — Living plan (rat/PLAN.md): update early and often during work; it is the project heartbeat recording progress, decisions, risks, and next steps
- README.md — Project overview, quickstart, and user-facing context
- NOTES.md — Private scratchpad (rat/NOTES.md): personal working memory for continuity across runs; never commit contents unless explicitly requested

## Instructions for AI Agents (gpt-5)
1. At the beginning of each task, read CLAUDE.md, PLAN.md (rat/PLAN.md), and README.md in full to align on scope, workflow, and current status. Open NOTES.md (rat/NOTES.md) for context from prior runs.
2. Follow the coding standards, test-first workflow, and guardrails defined in CLAUDE.md without deviation.
3. Use only the commands and tools specified in CLAUDE.md for building, testing, linting, snapshots, and mutation testing.
4. Respect the project structure documented in CLAUDE.md and README.md; keep changes minimal and localized.
5. Keep PLAN.md (rat/PLAN.md) up to date continuously:
   - Record what was accomplished, and how it was accomplished (tests-first, minimal diffs, verification steps).
   - List remaining work, risks, and next actions.
   - Update status checkboxes and phase progress as appropriate.
6. Maintain a private NOTES.md (rat/NOTES.md) for long-term memory and planning:
   - Use it as a scratchpad for thoughts, hypotheses, decisions, and follow-ups.
   - Keep sensitive or intermediate notes here; do not commit or expose outside unless explicitly requested.
   - Treat NOTES.md as personal state: update during and after runs to preserve context for your future self.
7. Cadence: Read PLAN.md and NOTES.md at the start of work; append to both during implementation; summarize updates to PLAN.md before yielding.

## Updating PLAN.md
For each task, append or update entries in PLAN.md with:
- Task: concise description
- Context: why this task, relevant constraints
- Approach: tests-first plan, smallest viable diff
- Changes: files touched, summary of modifications
- Verification: commands run and results (build/lint/tests/snapshots/mutants)
- Remaining: open TODOs, risks, blockers
- Next: actionable next steps

Keep PLAN.md synchronized with reality after each commit or verified change.

## NOTES.md (Private Working Memory)
- File: rat/NOTES.md
- Purpose: personal, persistent notes to your future self for continuity across sessions and runs.
- Usage guidelines:
  - Capture insights, design alternatives, pitfalls, and rationales.
  - Track breadcrumbs for ongoing investigations and experiments.
  - Store links, references, and scratch calculations.
- Privacy: treat NOTES.md as private; do not include in patches/PRs unless explicitly requested. Keep it uncommitted locally.

## Getting Started
Before making any changes:
1. Read CLAUDE.md, PLAN.md, and README.md thoroughly.
2. Confirm your plan aligns with CLAUDE.md’s “Plan → test → code → verify → summarize” workflow.
3. Review existing code conventions and project structure.
4. Check available commands and tools in CLAUDE.md and prepare the test environment.
5. Open NOTES.md to capture session notes, and prepare PLAN.md for status updates.
