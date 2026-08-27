---
title: 'DW-1: Add cargo fmt --check and clippy to pnpm verify'
type: 'chore'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - _bmad-output/implementation-artifacts/epic-1-context.md
  - _bmad-output/implementation-artifacts/1-8-workspace-verify-aggregate.md
  - _bmad-output/implementation-artifacts/1-3-scaffold-cargo-workspace.md
  - AGENTS.md
  - package.json
  - tooling/planning-checks.py
warnings: []
deferred:
  - summary: >-
      The root verify script addition has no automated regression guard; presence of the fmt/clippy sub-commands (and their fail-closed ordering) is asserted only by manual grep inside this spec and ad-hoc runs.
    evidence: |-
      Removing the two subcommands from package.json:7 lets `pnpm verify` still exit 0 (biome+turbo+cargo-test+planning pass); only manual inspection or deliberate violation demos (which are not part of recurring gates) would catch omission. Matches the verification style used for cargo test in 1.8.
    location: >-
      package.json:7
    severity: medium
  - summary: >-
      Story 1.8 ACs and related planning docs continue to list the verify aggregate as containing only cargo test for the Rust side (fmt/clippy still appear deferred or omitted).
    evidence: |-
      1-8 spec AC, design note, and epics.md describe "cargo test --workspace" without the new gates; no later story owns the aggregate per DW-1.
    location: >-
      _bmad-output/implementation-artifacts/1-8-workspace-verify-aggregate.md , epics.md
    severity: low
baseline_revision: '1d25fbfab2f3f7c4b464ecda32a4f2aac628dabf'
baseline_commit: '1d25fbfab2f3f7c4b464ecda32a4f2aac628dabf'
operator_actions: []
---

<intent-contract>

## Intent

**Problem:** The root `pnpm verify` (from Story 1.8) only runs `cargo test --workspace`; `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` (introduced and manually verified in Story 1.3) are absent from the aggregate gate. planning-checks.py performs only doc parity. This leaves the fmt-rust/clippy stages referenced in docs/13-testing-quality-release.md unexercised by the standard local verify, creating a persistent verification gap (DW-1).

**Approach:** Edit only the "verify" script in root package.json to insert the two cargo static-analysis commands (fmt then clippy) before the existing cargo test. Leave planning-checks.py, turbo.json, Cargo.toml, and all docs unchanged. Ensure the chain remains fail-closed via &&.

## Boundaries & Constraints

**Always:**
- Exact addition to the verify script: `cargo fmt --check && cargo clippy --workspace -- -D warnings &&` immediately before `cargo test --workspace`
- The full verify remains a single-line && chain in package.json scripts.verify
- planning-checks.py must stay a pure doc-parity tool with no cargo, subprocess, or rust calls
- Commands use the exact flags from 1.3 AC and clippy -D warnings; no config changes or relaxations
- Run pnpm verify (and the cargo commands) as part of verification; report full command outputs and exit codes as evidence
- Preserve all existing story IDs, requirement refs (G-01 etc), and do not touch sprint-status.yaml or any deferred-work ledger

**Block If:** none

**Never:**
- Modify planning-checks.py or add cargo logic to it
- Weaken any gate (e.g. drop -D warnings, add || true, use --allow-dirty)
- Mutate source during verify (fmt only --check)
- Update CI definitions, docs/13, or any markdown as part of this change
- Introduce new files, rustfmt.toml, or clippy lints config

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| HAPPY_PATH | clean post-scaffold tree | pnpm verify runs all steps including fmt --check (clean), clippy (no warnings), test; exits 0 | n/a |
| FAIL_FMT | .rs file with formatting diff | `cargo fmt --check` exits 1; subsequent steps (clippy, test, ...) not executed | user runs `cargo fmt` to fix |
| FAIL_CLIPPY | code triggering any warning | clippy exits non-0 due to -D warnings; stops chain | fix the warning |
| FAIL_LATER | fmt/clippy pass but cargo test fails | reaches and fails at cargo test step | fix test |

</intent-contract>

## Code Map

- `package.json` (root) -- root scripts; line 7 holds the verify chain: "biome check . && turbo run typecheck && turbo run test && turbo run validate && cargo test --workspace && python3 tooling/planning-checks.py" ; edit target to insert fmt+clippy after validate && before cargo test
- `Cargo.toml` (root) -- [workspace] members; cargo --workspace resolves from here
- `tooling/planning-checks.py` -- read-only confirmation target: contains only doc checks (no cargo, no subprocess calls to rust tools); must not be edited
- `docs/13-testing-quality-release.md:27` -- CI graph references `fmt-rust ─ clippy` stages; this change makes the underlying commands part of local verify that CI will mirror
- `_bmad-output/implementation-artifacts/1-3-scaffold-cargo-workspace.md:134` -- original definition of `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` as required gates
- `_bmad-output/implementation-artifacts/1-8-workspace-verify-aggregate.md:138` -- notes "cargo test --workspace covers the Rust side per AC (fmt/clippy deferred per prior notes; not listed in 1.8 AC)"; review logs mention rustfmt/clippy as rejected from 1.8
- `_bmad-output/implementation-artifacts/epic-1-context.md` -- constraints: "Planning checks, markdown lint, and gate scripts must pass"; "Verify gate required before downstream stories"
- all `bronze-*/src/lib.rs` and `apps/desktop/src-tauri/src/**/*.rs` etc -- subject to formatting and -D warnings (currently clean)

## Tasks & Acceptance

**Execution:**
- `package.json` -- modify the verify script string to: "biome check . && turbo run typecheck && turbo run test && turbo run validate && cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace && python3 tooling/planning-checks.py" -- adds the missing static gates per intent
- `pnpm verify` -- execute and capture output; must exit 0 with visible execution of the new cargo fmt and clippy steps
- temporarily introduce a fmt violation (e.g. whitespace edit in a .rs), re-execute `pnpm verify`, confirm non-zero exit at fmt step -- proves fail-closed
- revert, introduce a clippy-detectable issue (e.g. unused var), rerun verify, confirm fails at clippy -- proves clippy gate
- revert and confirm `pnpm verify` returns to exit 0

**Acceptance Criteria:**
- Given the Epic 1 workspaces from 1.1-1.8 and current clean state
- When `pnpm verify` is invoked
- Then the script executes (in sequence) `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` (plus prior steps)
- And the verify exits 0 only when all (including new) cargo gates pass
- And planning-checks.py is unmodified

## Spec Change Log

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 2: (medium 1, low 1)
- reject: 15
- addressed_findings:
  - none

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 0
- reject: 22
- addressed_findings:
  - none

## Design Notes

The placement before `cargo test` follows logical order (format → lint → test) and mirrors the CI subgraph `fmt-rust ─ clippy ─ unit-rust`.

Direct cargo invocations (not wrapped in turbo) match the existing `cargo test` pattern in 1.8 and avoid turbo caching concerns for native tools.

No changes to any Rust source, manifests, or .github; the gap was only the missing wiring in the documented aggregate.

Per AGENTS.md and 1.8 verification: after edit, full `pnpm verify` + targeted failure demos are required evidence; compilation alone is insufficient.

## Verification

**Commands:**
- `pnpm verify` -- expected: exit 0; output includes successful "cargo fmt --check", "cargo clippy --workspace -- -D warnings", "cargo test --workspace", and "PASS ..." from planning-checks
- `cargo fmt --check` -- expected: exits 0, no diff output
- `cargo clippy --workspace -- -D warnings` -- expected: exits 0, no warnings (Finished ...)

**Manual checks (if no CLI):**
- grep '"verify"' package.json -- the string contains both `cargo fmt --check` and `cargo clippy --workspace -- -D warnings`
- head -20 tooling/planning-checks.py -- no import subprocess, no "cargo", no os.system/popen calls

## Auto Run Result

Summary of implemented change: Added `cargo fmt --check && cargo clippy --workspace -- -D warnings &&` (using exact commands and -D warnings from Story 1.3) into the root `pnpm verify` &&-chain immediately before the pre-existing `cargo test --workspace`. This closes DW-1 so that the 1.3 workspace gates are now exercised by the standard aggregate (and by extension the fmt-rust/clippy stages in docs/13).

Files changed:
- `package.json:7` — inserted the two cargo static-analysis commands into the verify script (only product change)

Review findings breakdown: patches applied 0; items deferred 0 this pass (2 remain from prior pass); items rejected 22.

Follow-up review recommendation: false (patched high 0, medium 0, low 0; score 3×0 + 1×0 = 0).

Verification performed:
- Prior implementation pass: Pre-change `cargo fmt --check` (0), `cargo clippy --workspace -- -D warnings` (0), `pnpm verify` (0). Post-edit `pnpm verify` (EXIT 0). FAIL_FMT EXIT=1 at bronze-domain/src/lib.rs. FAIL_CLIPPY EXIT=101 unused variable. FAIL_LATER sim EXIT=1. Final clean `pnpm verify` (0).
- Follow-up review pass (2026-08-27): `package.json:7` still has exact `cargo fmt --check && cargo clippy --workspace -- -D warnings &&` before `cargo test --workspace`. `cargo fmt --check` EXIT 0. `cargo clippy --workspace -- -D warnings` EXIT 0 (Finished dev). `python3 tooling/planning-checks.py` EXIT 0 (PASS requirement-parity, enum/schema, local-links, traceability). planning-checks.py has no cargo/subprocess. Deferred-work ledger left untouched (orchestrator-owned).

Residual risks:
- No automated assertion (beyond manual grep + ad-hoc demos) that the fmt/clippy subcommands remain in the verify string (pre-existing verification style for the aggregator; see deferred item).
- 1.8-era AC text and epics still describe the old cargo-only aggregate (deferred; edits to md forbidden by Never).
- Toolchain components (rustfmt/clippy) assumed present in dev env; no guards added (out of scope per intent).
- Pre-existing Biome deprecation warnings and A11Y !important test gap from 1.8 remain untouched.
- CI yaml (if any) and docs/13 graph description not updated (per Never in contract).

