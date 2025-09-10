# AI Agent Instructions

## Primary References
Always consult these three documents at the start of every task:
- CLAUDE.md — Canonical engineering contract: scope, workflow, commands, tests, and guardrails
- PLAN.md — Current roadmap: what’s done, how it was done, what remains, next steps
- README.md — Project overview, quickstart, and user-facing context

## Instructions for AI Agents (gpt-5)
1. At the beginning of each task, read CLAUDE.md, PLAN.md, and README.md in full to align on scope, workflow, and current status.
2. Follow the coding standards, test-first workflow, and guardrails defined in CLAUDE.md without deviation.
3. Use only the commands and tools specified in CLAUDE.md for building, testing, linting, snapshots, and mutation testing.
4. Respect the project structure documented in CLAUDE.md and README.md; keep changes minimal and localized.
5. Keep PLAN.md up to date:
   - Record what was accomplished, and how it was accomplished (tests-first, minimal diffs, verification steps).
   - List remaining work, risks, and next actions.
   - Update status checkboxes and phase progress as appropriate.
6. Maintain a private NOTES.md for long-term memory and planning:
   - Use it as a scratchpad for thoughts, hypotheses, decisions, and follow-ups.
   - Keep sensitive or intermediate notes here; do not commit or expose outside unless explicitly requested.

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
- Purpose: personal, persistent notes to your future self for continuity across sessions.
- Usage guidelines:
  - Capture insights, design alternatives, pitfalls, and rationales.
  - Track breadcrumbs for ongoing investigations and experiments.
  - Store links, references, and scratch calculations.
- Privacy: treat NOTES.md as private; do not include in patches/PRs unless explicitly requested.

## Getting Started
Before making any changes:
1. Read CLAUDE.md, PLAN.md, and README.md thoroughly.
2. Confirm your plan aligns with CLAUDE.md’s “Plan → test → code → verify → summarize” workflow.
3. Review existing code conventions and project structure.
4. Check available commands and tools in CLAUDE.md and prepare the test environment.
5. Open NOTES.md to capture session notes, and prepare PLAN.md for status updates.
