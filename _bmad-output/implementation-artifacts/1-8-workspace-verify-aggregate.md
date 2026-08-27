---
title: 'Story 1.8: Workspace verify aggregate'
type: 'feature'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - _bmad-output/implementation-artifacts/epic-1-context.md
  - docs/18-adrs.md
  - docs/03-prd.md
  - AGENTS.md
  - _bmad-output/implementation-artifacts/1-2-scaffold-pnpm-turbo-workspace.md
  - _bmad-output/implementation-artifacts/1-3-scaffold-cargo-workspace.md
  - _bmad-output/implementation-artifacts/1-7-shadcn-react-aria-foundation.md
warnings: []
deferred:
  - summary: >-
      packages/ui globals.test.ts only regex-matches media-query strings, not
      !important or computed override of Tailwind utilities (A11Y-003).
    evidence: |-
      Pre-existing Story 1.7 test; still passed after hygiene stripped
      !important. This review restored !important but did not extend the test.
    location: >-
      packages/ui/src/styles/globals.test.ts:12
    severity: medium
baseline_revision: 'a9412c0165acbd7f81aa7309039e1a2b31ef188b'
baseline_commit: 'a9412c0165acbd7f81aa7309039e1a2b31ef188b'
operator_actions: []
---

# Story 1.8: Workspace verify aggregate

## Story

As a developer,
I want one local verify command covering format, lint, types, unit, i18n, planning-checks,
So that later stories have a gate.

**Requirements:** G-01 process, SEC-001 config
**ADRs:** ADR-003
**Dependencies:** Stories 1.2–1.7
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** workspaces exist
**When** root verify script is added
**Then** `pnpm verify` runs Biome/tsc/Vitest/i18n validate/cargo test/planning-checks
**And** signing/TCC tasks are not Turbo-cached
**And** verify fails if any required step fails

**Failure / recovery:**
Do not weaken a failing check to go green.

**Security / privacy / diagnostics / a11y / i18n / data:**
planning-checks remain required.

**Automated verification:**
- `pnpm verify`

**Signed-build / manual evidence:**
command output

**Non-goals:**
GitHub Actions beyond local script


<intent-contract>

## Intent

**Problem:** No single `pnpm verify` gate exists. Developers and later stories must manually invoke Biome, tsc, Vitest, i18n validate, cargo test, and planning-checks; there is no enforced aggregate that fails closed. Signing/TCC work (future) must never be Turbo-cached.

**Approach:** Add a root `verify` script in package.json that chains the required checks with `&&`; update turbo.json so build (and future signing/TCC) tasks declare `"cache": false`; keep the implementation as a thin aggregator only.

## Boundaries & Constraints

**Always:**
- The command is exactly `pnpm verify` (per story and AGENTS.md).
- Runs exactly: Biome (covers format+lint on root + packages), turbo typecheck, turbo test, turbo validate (i18n), `cargo test --workspace`, `python3 tooling/planning-checks.py`.
- Fails closed on first failure; no `|| true`, no weakening.
- signing/TCC paths use no Turbo cache (set on build task now).
- Follow AGENTS.md: run formatter/static/unit etc and report evidence; never claim from compile alone.
- Preserve requirement IDs; no sprint-status.yaml writes.

**Block If:**
- (none; all changes are local config + script)

**Never:**
- GitHub Actions or CI.
- Mutate sources during verify (checks only).
- Depend on network at runtime.
- Weaken checks or cache signing work.
- Touch sprint-status.yaml.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| HAPPY_PATH | clean post 1.2-1.7 tree | pnpm verify exits 0; each gate runs and succeeds | n/a |
| FAIL_BIOME | unformatted file | exits non-0 at Biome step; later steps not reached | user fixes format |
| FAIL_CARGO | compile break in crate | exits non-0 at cargo; reports the failure | fix code |

</intent-contract>

## Code Map

- `package.json` (root) -- add "verify" script under "scripts"; biome already present as devDep
- `turbo.json` -- add "build": {"cache": false} (and keep validate:false) so signing/TCC tasks will not be cached
- `_bmad-output/implementation-artifacts/1-8-workspace-verify-aggregate.md` -- this spec (plan + impl record)

## Tasks & Acceptance

**Execution:**
- `package.json` -- insert "verify" script under scripts with the && chain -- implements the aggregate per AC
- `turbo.json` -- declare build task with cache:false -- satisfies "signing/TCC tasks are not Turbo-cached"
- execute `pnpm verify` -- must exit 0 with all steps visible in output
- break one check temporarily, re-run, confirm non-zero exit and stops early -- verifies fail-closed

**Acceptance Criteria:**
- Given workspaces exist
- When root verify script is added
- Then `pnpm verify` runs Biome/tsc/Vitest/i18n validate/cargo test/planning-checks
- And signing/TCC tasks are not Turbo-cached
- And verify fails if any required step fails

## Spec Change Log

## Review Triage Log

## Design Notes

Use `&&` shell chain rather than a turbo "verify" task so that cargo (outside turbo) and planning-checks integrate cleanly and early-exit is guaranteed without extra runner.

Biome at root covers format+lint surface for configs + all package sources (package lints cover their src explicitly via their scripts if run separately).

cargo test --workspace covers the Rust side per AC (fmt/clippy deferred per prior notes; not listed in 1.8 AC).

No new deps; all invoked tools already reachable (biome via root, turbo, cargo, python).

## Verification

**Commands:**
- `pnpm verify` -- expected: exit 0; prints output from Biome, turbo typecheck/test/validate, cargo test, planning-checks.py all succeeding
- `pnpm verify` after temporarily breaking a file (e.g. add space to biome-formatted) -- expected: non-zero exit, stops before later gates

**Manual checks (if no CLI):**
- grep -A1 '"verify"' package.json -- contains the full chain
- grep -A2 '"build"' turbo.json -- has "cache": false
- python3 tooling/planning-checks.py -- PASS
- cargo test --workspace -- exit 0

## Spec Change Log

- Added root "verify" script per intent-contract and AC.
- Declared "build" task cache:false in turbo.json (for future signing/TCC).
- Made `biome check .` pass on product sources by:
  - Running biome --write --unsafe (applied format/organize/safe fixes to sources that were inconsistent post 1.2-1.7).
  - Added vcs + useIgnoreFile to biome.json and corresponding entries to .gitignore so meta agent dirs (.claude, .agents, _bmad*, .opencode) are not scanned by root biome (keeps check . focused on root+packages+apps without a11y noise from skill assets).
- These support files updated only to satisfy the "Biome at root" gate without weakening; product sources under packages/apps now conform.
- All AC verifications executed; fail-closed demonstrated.

## Review Triage Log

### 2026-08-27 — Implementation pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 0
- reject: 0
- addressed_findings: n/a (thin aggregator)

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 2 (medium: 2)
- reject: 30
- addressed_findings:
  - none

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 2: (high 1, medium 1)
- defer: 1: (medium 1)
- reject: 24
- addressed_findings:
  - `[high]` `[patch]` Restored `!important` on A11Y-003 Reduce Motion / Increase Contrast hooks in `packages/ui/src/styles/globals.css` (hygiene `--write --unsafe` had stripped them).
  - `[medium]` `[patch]` Removed 1.8 `.gitignore` of `.claude`/`.agents`/`.opencode`/`_bmad`/`_bmad-output`; Biome `files.includes` force-ignores those dirs so planning artifacts stay trackable.

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 0
- reject: 16
- addressed_findings:
  - none

## Design Notes (continued)

All per spec "Always", "Never", "Boundaries".

Used `biome check .` exactly (with supporting config so it runs cleanly on given workspaces).

## Additional Evidence

- Full pnpm verify output captured in /tmp/verify.out (exit 0)
- Fail demo in /tmp/verify-fail.out (exit 1 at biome)

## Auto Run Result

Status: done

Summary of implemented change: Root `pnpm verify` aggregator (`biome check . && turbo run typecheck && turbo run test && turbo run validate && cargo test --workspace && python3 tooling/planning-checks.py`) plus `turbo.json` `build.cache: false`. Hygiene made the Biome gate green; follow-up restored A11Y-003 `!important` and moved meta-dir skips into Biome `files.includes`.

Files changed:
- `package.json` — add `verify` script with fail-closed `&&` chain
- `turbo.json` — `build`: `{ cache: false }`
- `biome.json` — vcs + `files.includes` force-ignore of agent/meta dirs
- `packages/ui/src/styles/globals.css` — restore `!important` on reduced-motion and high-contrast overrides
- format hygiene on i18n/ui/capabilities (quotes/imports/whitespace)

This follow-up review pass: no patches, no new defers, 16 rejects (aggregator tests, turbo cache on typecheck/test, signing/TCC task name, lint-graph split, missing package scripts, rustfmt/clippy/Swift, loop policy docs, README, I/O matrix, `--unsafe` authoring mutations, already-recorded `globals.test.ts` gap). Existing spec `deferred` item preserved; deferred-work ledger and `sprint-status.yaml` not modified.

Review findings breakdown: patches applied 0; items deferred 0; items rejected 16.

Follow-up review recommendation: false (patched high 0, medium 0, low 0; score 3×0 + 1×0 = 0).

Verification performed: this pass `pnpm verify` exit 0 (Biome 4 `noImportantStyles` warnings only; typecheck/test cache hits; validate; cargo test; planning-checks PASS). Prior fail-closed demo at Biome in `/tmp/verify-fail.out`.

Residual risks: `globals.test.ts` still does not assert `!important` (already deferred); `noImportantStyles` remains warn; turbo typecheck/test still cacheable; packages without those scripts are skipped.

