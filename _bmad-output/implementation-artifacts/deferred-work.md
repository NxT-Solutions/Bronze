### DW-1: planning-checks.py and current verify aggregate do not execute the cargo fmt/clippy/test workspace gates (only doc parity)
origin: spec-deferred 6b8f7fbf7741
location: tooling/planning-checks.py
source_spec: `1-3-scaffold-cargo-workspace.md`
severity: low
reason: verification-gap reviewer and emit_epics.py show cargo commands are story-specific; aggregate in 1.8; current run exercised manually
status: open

### DW-2: cross-references to new crates missing from planning docs and apps/desktop package metadata
origin: spec-deferred cf822f742eeb
location: docs/06-system-architecture.md, docs/14-agentic-implementation-plan.md
source_spec: `1-3-scaffold-cargo-workspace.md`
severity: low
reason: architecture spine and later stories reference them; 1.3 scope was only crates + verify
status: open

### DW-3: Follow-up review still recommended for 1-5-empty-tauri-app-with-csp-deny after the damping cap was spent
origin: review-budget-followup
location: n/a
source_spec: `1-5-empty-tauri-app-with-csp-deny.md`
severity: low
reason: The follow-up-review damping cap (limits.max_followup_reviews = 1) was spent with the story finalized (status: done, verify green) while the review pass still recommended an independent follow-up. The work was committed by bmad-loop run 20260827-110451-2bb6; this entry preserves the lingering recommendation for a deliberate later review.
status: open
