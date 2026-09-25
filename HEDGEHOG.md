# bronze-app

Bronze is a local-first macOS selection-to-action work queue. The operator selects text, captures it, and acts from a deliberate queue. It is not a clipboard recorder, task manager, note vault, or hosted AI client.

Stack is already decided: Tauri 2, React, Rust, in-process Swift. Planning authority is `docs/03-prd.md` and `docs/18-adrs.md`. This repo is a Hedgehog **adopted** project — do not scaffold a new workspace or switch to a shipped web core.

This project is built with **Hedgehog**: one gated step at a time. Follow `AGENTS.md` for Bronze product rules, then the Hedgehog loop for execution. Chrome and UX follow `hig/SKILL.md` (Apple HIG) and `wcag/SKILL.md` (WCAG 2.2) on every agent.

## How to work here

- Query `.hedgehog/hedgehog.db` via `hedgehog status` / `hedgehog next`. Do not re-derive the queue from sprint-status prose.
- `hedgehog verify <task-id>` is the only thing that completes a task.
- ADR-002 is Accepted (operator asked for a separate Intel build). Keep ADR-009 and ADR-018 Proposed unless the user accepts them.
- Do not resume `bmad-loop` run `6a79`.
