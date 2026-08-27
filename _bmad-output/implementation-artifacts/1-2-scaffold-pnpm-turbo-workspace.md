---
title: 'Story 1.2: Scaffold pnpm Turbo workspace'
type: 'feature'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context: []
warnings: []
deferred: []
baseline_revision: '6f89e377a67c61baf6fa595ae8a5728aa9df4a5e'
operator_actions: []
---

# Story 1.2: Scaffold pnpm Turbo workspace

## Story

As a developer,
I want pnpm-workspace and Turbo with empty packages,
So that JS work has a graph before product code.

**Requirements:** SEC-001, I18N-001 (package slots)
**ADRs:** ADR-003, ADR-017
**Dependencies:** Story 1.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** pinned package manager
**When** workspace is scaffolded
**Then** `pnpm-workspace.yaml`, `turbo.json`, `packages/ui`, `packages/contracts`, `packages/i18n`, `packages/test-support`, `apps/desktop` exist
**And** no product features
**And** Turbo cache disabled is not required yet because no signing tasks exist

**Failure / recovery:**
If pnpm install needs network, use lockfile creation once then keep lockfile; do not add remote fonts or CDN.

**Security / privacy / diagnostics / a11y / i18n / data:**
packages/ui must not own product state. No raw user strings in TSX.

**Automated verification:**
- `python3 tooling/planning-checks.py`
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md`
- `pnpm install`
- `pnpm exec turbo run lint --force || true`

**Signed-build / manual evidence:**
local install log

**Non-goals:**
shadcn components, Tauri commands, capture


<intent-contract>

## Intent

**Problem:** The root package.json from Story 1.1 has no workspace graph. JS/TS work for UI, contracts, i18n and the desktop shell cannot be expressed or built until pnpm workspaces + Turborepo declare the package boundaries.

**Approach:** Author the two root config files, update root package.json for turbo, create the five empty package directories each with a minimal package.json (no sources, no product state), then run pnpm install to produce the lockfile. All changes are surface-level package scaffolding only.

## Boundaries & Constraints

**Always:**
- packages/ui must not own product state or IPC calls (see architecture ownership table).
- No raw user strings or sentence concatenation anywhere in TS/TSX (I18N-001).
- Zero remote content: no fonts, no CDN, no scripts from network (SEC-001, ADR-017).
- Package manager remains exactly the pinned pnpm@11.9.0 from Story 1.1; lockfile kept once created.
- Follow existing AGENTS.md, README, docs/03-prd, docs/18-adrs, epic-1-context before edits.

**Block If:**
- Any step would require network fetches beyond a single initial lockfile generation.
- Decisions about actual component contents, Tauri config, or contracts content (those belong to later stories).

**Never:**
- Introduce shadcn components, React Aria usage, Tauri commands, capture logic, or any feature source.
- Enable Turbo remote cache or any signing/TCC tasks.
- Modify sprint-status.yaml or any orchestrator bookkeeping.
- Adopt Proposed ADR-002/009/018.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| HAPPY_PATH | clean tree after 1.1 pins | pnpm-workspace.yaml + turbo.json + 5 package dirs exist with valid package.jsons; pnpm install succeeds producing lockfile; turbo lint runs | none |
| NO_NET | pnpm needs registry for turbo dep | use `pnpm install --lockfile-only` (or keep generated lock); do not pull other packages | story failure recovery path documented |
| EXTRA_FILES | stray source under packages/ui | must not exist | verification and absence checks in plan |

</intent-contract>

## Code Map

- `pnpm-workspace.yaml` -- declares the packages/* and apps/* globs for pnpm
- `turbo.json` -- declares task graph (lint, build) for the workspace; cache policy left at default for non-signing tasks
- `package.json` -- root; adds "turbo" devDep and basic scripts so `pnpm exec turbo ...` works
- `packages/ui/package.json` -- empty UI package slot; name @bronze/ui; must stay free of product state
- `packages/contracts/package.json` -- shared contracts slot; name @bronze/contracts
- `packages/i18n/package.json` -- i18n catalog slot (I18N-001); name @bronze/i18n
- `packages/test-support/package.json` -- test fixtures slot; name @bronze/test-support
- `apps/desktop/package.json` -- Tauri app shell slot; filterable as "desktop"
- `packages/*/tsconfig.json` (optional skeleton) -- only if needed for later typecheck; keep empty for this story
- `.gitignore` -- ensure node_modules/, .turbo/, dist/ etc. are ignored (workspace hygiene)

## Tasks & Acceptance

**Execution:**
- `pnpm-workspace.yaml` -- create with packages and apps globs -- declares workspace shape per AC
- `turbo.json` -- create minimal v2 config with lint task -- enables `turbo run lint`
- `package.json` -- edit to add turbo under devDependencies and a lint script delegating to turbo -- makes turbo invocable from root
- `packages/ui/` -- mkdir -p ; write minimal package.json -- creates required dir
- `packages/contracts/` -- mkdir -p ; write minimal package.json -- creates required dir
- `packages/i18n/` -- mkdir -p ; write minimal package.json -- creates required dir
- `packages/test-support/` -- mkdir -p ; write minimal package.json -- creates required dir
- `apps/desktop/` -- mkdir -p ; write minimal package.json -- creates required dir
- run `pnpm install` -- produces pnpm-lock.yaml; verify no extraneous product files appear
- run `python3 tooling/planning-checks.py` and markdownlint and `pnpm exec turbo run lint --force || true` -- per automated verification

**Acceptance Criteria:**
- Given pinned package manager from 1.1
- When workspace is scaffolded as described
- Then the seven paths listed in AC exist
- And no product feature sources, no shadcn, no Tauri commands exist under packages/ or apps/
- And turbo lint task is executable (even if it reports no packages to lint)

## Spec Change Log

## Design Notes

Package naming: packages use @bronze/ scope to match later filter examples and ownership clarity; apps/desktop uses bare "desktop" name to match the `--filter desktop` invocations used in downstream story verification lists.

No src/ trees or index files are created; empty directories + package.json satisfy "exist" and "no product features".

## Verification

**Commands:**
- `python3 tooling/planning-checks.py` -- expected: PASS requirement-parity, enum/schema, local-links, traceability
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` -- expected: 0 issues
- `pnpm install` -- expected: success, pnpm-lock.yaml produced, only intended files added
- `pnpm exec turbo run lint --force || true` -- expected: runs without fatal error (empty workspace is acceptable)

**Manual checks (if no CLI):**
- `ls pnpm-workspace.yaml turbo.json packages/ui packages/contracts packages/i18n packages/test-support apps/desktop` -- all exist
- `git ls-files --others --exclude-standard | grep -E '(node_modules|\.turbo|dist)' || true` -- nothing leaked
- `rg -n 'react|tauri|capture|shadcn' packages/ apps/ || true` -- no product code

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 1 (high 0, medium 0, low 1)
- defer: 0
- reject: 16
- addressed_findings:
  - `[low] [patch] pinned turbo to exact 2.10.12 to match lockfile and bootstrap observed version (style parity with 1.1 pins)`

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 1: (high 0, medium 0, low 1)
- defer: 0
- reject: 18
- addressed_findings:
  - `[low] [patch] removed duplicate empty ## Review Triage Log heading left from the prior auto-run template`

## Auto Run Result

Status: done

Summary of implemented change: Follow-up review of Story 1.2 workspace scaffold. Product files were already committed (`80e95ff`). This pass confirmed the empty pnpm + Turbo graph, removed a duplicate spec heading, and left orchestrator `sprint-status.yaml` untouched.

Files changed with one-line descriptions:
- `pnpm-workspace.yaml` — workspace globs for `packages/*` and `apps/*`
- `turbo.json` — Turbo v2 lint task only
- `package.json` — pinned `turbo@2.10.12` and root `lint` script
- `packages/ui/package.json` — empty `@bronze/ui` slot
- `packages/contracts/package.json` — empty `@bronze/contracts` slot
- `packages/i18n/package.json` — empty `@bronze/i18n` slot
- `packages/test-support/package.json` — empty `@bronze/test-support` slot
- `apps/desktop/package.json` — empty `desktop` app slot
- `pnpm-lock.yaml` — lockfile for turbo 2.10.12
- `.gitignore` — node_modules, .turbo, dist, logs
- `_bmad-output/implementation-artifacts/1-2-scaffold-pnpm-turbo-workspace.md` — review triage and this result

Review findings breakdown: patches applied 1 (low: duplicate Review Triage Log heading); items deferred 0; items rejected 18 (later-story turbo tasks, lint scripts, `|| true` gate, remote cache disable, telemetry, `$schema` URL, `.npmrc`, pnpm/biome pins, workspace deps, package.json extras, gitignore extras, missing process evidence, README, test-support ownership, orchestrator sprint-status, EXTRA_FILES executable tests).

Follow-up review recommendation: false. Patched this pass: high 0, medium 0, low 1. Score `3 × 0 + 1 × 1 = 1` (threshold 5).

Verification performed:
- `python3 tooling/planning-checks.py` — PASS requirement-parity, enum/schema, local-links, traceability (56 IDs)
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` — 0 issues
- `pnpm install` — already up to date, pnpm v11.9.0
- `pnpm exec turbo run lint --force || true` — turbo 2.10.12, 5 packages in scope, no tasks executed, remote caching disabled, exit via `|| true` as specified
- `ls` of seven AC paths — all exist
- `rg` over `packages/` and `apps/` for react/tauri/capture/shadcn — no matches

Residual risks: `turbo run lint` has an empty task graph until later stories add package lint scripts; story verification still uses `|| true` as specified in the original AC. Orchestrator-owned `sprint-status.yaml` remains dirty in this worktree and was not written or reverted.

