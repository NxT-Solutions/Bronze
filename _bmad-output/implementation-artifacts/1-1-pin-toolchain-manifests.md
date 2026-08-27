---
title: 'Story 1.1: Pin toolchain manifests'
type: 'feature'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context: []
warnings: []
deferred: []
baseline_revision: '11dd6c899f8fa9bc902d71fcf829b3be833e841e'
operator_actions: []
---

# Story 1.1: Pin toolchain manifests

## Story

As a developer,
I want pinned Node, pnpm, Rust, and Swift identity files,
So that later stories share one bootstrap.

**Requirements:** G-01 (process), SEC-005 (reproducible later)
**ADRs:** ADR-003; ADR-002 remains Proposed
**Dependencies:** none (canary)
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** a clean `bmad/bronze-autonomous` tree
**When** toolchain pin files are added
**Then** `package.json` packageManager, Node engines, `rust-toolchain.toml` matching docs/20, and Swift tools version notes exist
**And** no application windows, capture, or SQLite yet
**And** docs/20 versions remain the recorded host baseline

**Failure / recovery:**
If a pin disagrees with docs/20, fail the story and update the pin or the bootstrap record — do not silently raise macOS deployment target (ADR-002).

**Security / privacy / diagnostics / a11y / i18n / data:**
No secrets in pin files. No telemetry SDK. No user-facing strings.

**Automated verification:**
- `python3 tooling/planning-checks.py`
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md`
- `test -f package.json`
- `test -f rust-toolchain.toml`

**Signed-build / manual evidence:**
file existence only; no signed app

**Non-goals:**
Tauri window, UI, native capture, CI cloud, Intel universal2


## Dev Agent Record

### Agent Model Used

xai/grok-build-0.1 (via opencode bmad-build-auto)

### File List

- `package.json` -- added with packageManager pnpm@11.9.0 and engines node 24.19.0
- `rust-toolchain.toml` -- added with channel 1.98.0
- `.swift-version` -- added with 6.3.3
- `_bmad-output/implementation-artifacts/1-1-pin-toolchain-manifests.md` -- added YAML frontmatter, baseline_revision, set status, populated this record

### Verification Performed

- python3 tooling/planning-checks.py → PASS
- npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md → 0 issues
- test -f package.json && test -f rust-toolchain.toml && test -f .swift-version → pass
- manual: versions in created files match docs/20 bootstrap record exactly
- absence evidence: `git ls-files --others --exclude-standard` (filtered) + `git diff --name-only` only produced the three pin files + this spec; `find . -type f \( -name '*tauri*' -o -name '*sqlite*' -o -path '*/src/*' -o -path '*/apps/*' \) | head -3` produced nothing relevant (no windows/capture/DB/product)
- AGENTS.md rule followed: README.md, docs/03-prd.md, docs/18-adrs.md, epic context and bootstrap docs/20 read before edits and subagent dispatch
- no app windows, capture, or SQLite introduced (confirmed via find + porcelain)

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 2 (high 0, medium 1, low 1)
- defer: 3 (high 0, medium 2, low 1)
- reject: 14
- addressed_findings:
  - `[medium] [patch] added concrete absence evidence (git ls, find) and AGENTS read confirmation to Verification Performed`
  - `[low] [patch] fixed pseudo-command syntax in verification list and clarified record text for status`

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 0
- reject: 16
- addressed_findings:
  - none

## Auto Run Result

Summary of implemented change: Root identity files pin Node 24.19.0, pnpm 11.9.0, Rust 1.98.0, and Swift 6.3.3 to the docs/20 bootstrap record. Follow-up review of the completed story found no in-scope defects against that intent.

Files changed:
- `package.json` — `packageManager` pnpm@11.9.0 and `engines.node` 24.19.0
- `rust-toolchain.toml` — channel 1.98.0
- `.swift-version` — 6.3.3
- `_bmad-output/implementation-artifacts/epic-1-context.md` — compiled Epic 1 planning context
- `_bmad-output/implementation-artifacts/1-1-pin-toolchain-manifests.md` — story record, review logs, this result

Review findings breakdown: patches applied 0; items deferred 0; items rejected 16 (extra rustup fields, Corepack hash, nvmrc/engine-strict, Package.swift vs `.swift-version`, content-equality gates beyond the story's `test -f` bar, orchestrator `sprint-status.yaml` bookkeeping, and similar extras outside the identity-file AC).

Follow-up review recommendation: false (patched high 0, medium 0, low 0; score 0).

Verification performed:
- `python3 tooling/planning-checks.py` → PASS requirement-parity, enum/schema, local-links, traceability (56 requirement IDs)
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` → markdownlint-cli2 v0.23.2, 0 issues in 26 files
- `test -f package.json && test -f rust-toolchain.toml && test -f .swift-version` → pass
- manual: pin values equal docs/20 selected versions (Node v24.19.0, pnpm 11.9.0 host, rustc 1.98.0, Swift 6.3.3); no windows, capture, or SQLite in this change; ADR-002 not adopted

Residual risks: docs/20 records Homebrew rustc on PATH, so rustup may not consume `rust-toolchain.toml` until later stories make rustup the active toolchain. Story 1.2 may replace the host pnpm 11.9.0 Corepack pin with current-stable when the JS workspace is created, as docs/20 already states.

