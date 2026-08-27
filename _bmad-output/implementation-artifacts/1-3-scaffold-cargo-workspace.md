---
title: 'Story 1.3: Scaffold Cargo workspace'
type: 'feature'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context: []
warnings: []
deferred:
  - summary: >-
      planning-checks.py and current verify aggregate do not execute the cargo fmt/clippy/test workspace gates (only doc parity)
    evidence: |-
      verification-gap reviewer and emit_epics.py show cargo commands are story-specific; aggregate in 1.8; current run exercised manually
    location: >-
      tooling/planning-checks.py
    severity: low
  - summary: >-
      cross-references to new crates missing from planning docs and apps/desktop package metadata
    evidence: |-
      architecture spine and later stories reference them; 1.3 scope was only crates + verify
    location: >-
      docs/06-system-architecture.md, docs/14-agentic-implementation-plan.md
    severity: low
baseline_revision: 'd45f154e019d3176484b4ce1662a260dce467132'
operator_actions: []
---

# Story 1.3: Scaffold Cargo workspace

## Story

As a developer,
I want Cargo workspace crates with rustfmt and Clippy deny-warnings,
So that Rust domain has a home.

**Requirements:** DAT-001 (slot), SEC-002 (slot)
**ADRs:** ADR-003, ADR-008, ADR-010
**Dependencies:** Story 1.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** toolchain pin
**When** crates are created
**Then** `bronze-domain`, `bronze-capture`, `bronze-storage`, `bronze-settings`, `bronze-diagnostics`, `bronze-platform`, `bronze-platform-macos` compile empty
**And** Clippy `-D warnings` on workspace
**And** no SQL, AX, or Tauri commands yet

**Failure / recovery:**
Compilation failure blocks. Do not allow unused-mut warnings.

**Security / privacy / diagnostics / a11y / i18n / data:**
No macOS framework imports outside bronze-platform-macos.

**Automated verification:**
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

**Signed-build / manual evidence:**
local cargo logs

**Non-goals:**
schema, ABI, event tap


<intent-contract>

## Intent

**Problem:** Toolchain is pinned (Story 1.1) and the pnpm/Turbo JS workspace exists (Story 1.2), but there is no Rust workspace. The domain, capture, storage, settings, diagnostics, and platform crates have no Cargo definitions or build graph; subsequent stories cannot compile Rust units or invoke `cargo test -p bronze-*`.

**Approach:** Author a root workspace `Cargo.toml`, create the seven listed crate directories at repo root, each with a minimal `Cargo.toml` + `src/lib.rs`, ensure rustfmt and `clippy -D warnings` are clean, run all listed verifications, generate `Cargo.lock`, and extend `.gitignore`. Crates are dependency-free libs with zero implementation.

## Boundaries & Constraints

**Always:**
- Crate names, count, and purposes exactly match the AC, epic-1-context, and architecture spine references.
- Workspace uses resolver = "2"; all packages use edition = "2021".
- `cargo clippy --workspace -- -D warnings` produces zero diagnostics (no unused-mut, no dead_code surprises).
- No [dependencies] in any manifest for this story.
- bronze-platform-macos may declare macOS-only structure but src/lib.rs compiles without any macOS framework `use` or `extern crate`.
- Carry forward continuity from Story 1.2 (workspace hygiene, no extraneous files, lockfile committed once).
- Never write or revert sprint-status.yaml.

**Block If:**
- Any step would require a registry fetch or external crate.
- Decisions about schema, ABI layout, or Tauri command registration (later stories).

**Never:**
- SQL crates (rusqlite), AX / AppKit imports, Tauri, objc, or any business logic.
- Product code, tests exercising features, or non-scaffold files under the crates.
- Adopt Proposed ADR-002, ADR-009 or ADR-018.
- Introduce src/bin, examples, or benches.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| HAPPY_PATH | clean tree post 1.1+1.2 | root Cargo.toml + exactly the 7 crate dirs + Cargo.lock + updated .gitignore; fmt --check, clippy -D, test --workspace all succeed with 0 warnings/failures | none |
| MISSING_MEMBER | one of the 7 crates omitted from workspace | cargo commands fail to see the crate or AC paths missing | verification fails per plan |
| WARNING_INTRODUCED | code or manifest that emits any warning | clippy -D fails the gate | blocks; must be removed |
| STRAY_FILE | extra .rs, Cargo.toml, or source under a bronze-* dir | ls / rg checks catch it; crates must stay minimal | verification fails |

</intent-contract>

## Code Map

- `Cargo.toml` -- root workspace manifest; members list in AC order; resolver = "2"
- `bronze-domain/Cargo.toml` -- lib package (entities, commands, lifecycle per DAT-001)
- `bronze-domain/src/lib.rs` -- minimal scaffold (compiles clean)
- `bronze-capture/Cargo.toml` -- capture request/state machine slot
- `bronze-capture/src/lib.rs`
- `bronze-storage/Cargo.toml` -- storage slot (DAT)
- `bronze-storage/src/lib.rs`
- `bronze-settings/Cargo.toml` -- settings slot (SEC-002)
- `bronze-settings/src/lib.rs`
- `bronze-diagnostics/Cargo.toml` -- diagnostics slot
- `bronze-diagnostics/src/lib.rs`
- `bronze-platform/Cargo.toml` -- neutral traits (no macOS)
- `bronze-platform/src/lib.rs`
- `bronze-platform-macos/Cargo.toml` -- macOS façade slot (imports forbidden outside this crate)
- `bronze-platform-macos/src/lib.rs`
- `.gitignore` -- append target/ and **/*.rs.bk (hygiene parity with 1.2)
- `Cargo.lock` -- generated by cargo for the workspace (committed once, like pnpm-lock)

## Tasks & Acceptance

**Execution:**
- `Cargo.toml` -- write workspace root manifest declaring the seven members
- for each crate in [bronze-domain, bronze-capture, bronze-storage, bronze-settings, bronze-diagnostics, bronze-platform, bronze-platform-macos]: mkdir -p <crate>/src ; write <crate>/Cargo.toml ; write <crate>/src/lib.rs (minimal, warning-free)
- `.gitignore` -- append Rust artifact ignores
- `cargo fmt --check` -- must pass
- `cargo clippy --workspace -- -D warnings` -- must pass with zero output
- `cargo test --workspace` -- must pass
- `python3 tooling/planning-checks.py` and markdownlint -- per standard gates
- produce and stage Cargo.lock

**Acceptance Criteria:**
- Given toolchain pin (1.1) and JS workspace (1.2)
- When crates are created as described
- Then the seven named crates exist and compile empty
- And `cargo clippy --workspace -- -D warnings` succeeds with no warnings
- And no SQL/AX/Tauri/deps or product logic present
- And verification commands listed in AC all succeed

## Spec Change Log

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 1 (high 0, medium 0, low 1)
- defer: 2
- reject: 20
- addressed_findings:
  - `[low] [patch] extended .gitignore with .cargo/, Cargo.lock.bak, *.pdb for completeness (hygiene)`

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 0
- reject: 18: (high 0, medium 0, low 18)
- addressed_findings:
  - none

## Design Notes

Crate directories live at repository root (not nested under crates/ or rust/) to match every cross-reference in epic context, architecture spine, and future `cargo test -p <name>` commands.

All seven are library crates; the final Tauri binary will consume selected members via path deps (later stories).

lib.rs files contain only a module-level comment to satisfy "compile" while producing a clean -D warnings report. A single inert `pub(crate) fn __scaffold() {}` would also work but comment-only keeps the surface truly empty.

Lockfile is produced exactly once (no-deps case) and committed, matching the pnpm-lock pattern from 1.2.

No rustfmt.toml or .cargo/config.toml is required for this story; the pinned toolchain + default rustfmt + the clippy flag on the command line satisfy the AC.

## Verification

**Commands:**
- `cargo fmt --check` -- expected: exits 0, no diff
- `cargo clippy --workspace -- -D warnings` -- expected: exits 0, no warnings emitted
- `cargo test --workspace` -- expected: exits 0 (0 tests is acceptable for scaffold)
- `python3 tooling/planning-checks.py` -- expected: PASS requirement-parity, enum/schema, local-links, traceability
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` -- expected: 0 issues
- `ls -1 Cargo.toml bronze-domain/src/lib.rs bronze-capture/src/lib.rs bronze-storage/src/lib.rs bronze-settings/src/lib.rs bronze-diagnostics/src/lib.rs bronze-platform/src/lib.rs bronze-platform-macos/src/lib.rs Cargo.lock` -- all exist
- `rg -n 'rusqlite|objc|core-graphics|tauri|event|AX|accessibility' bronze-*/src || true` -- expected: no matches
- `cargo tree --workspace --depth 0` -- expected: only the 7 workspace members, no external deps

**Manual checks (if no CLI):**
- `ls bronze-*/src` contains only lib.rs in each
- open each lib.rs: only a comment or empty, no logic
- `git status --porcelain | grep -E '(^ M |^A |^?? )' | grep -E '(sprint-status|target/)' || true` -- orchestrator file untouched; no target/ leaked

## Auto Run Result

Status: done

Summary of implemented change: Per bmad-build-auto + user guardrails on sprint-status ownership. Created root Cargo workspace and 7 empty compile-only crates exactly as named in AC (no deps, no mac imports, no product logic). Filled spec from stub using template (intent-contract, code map, tasks, verification). All direct AC verifications executed and passed. sprint-status.yaml left completely untouched (row remains ready-for-dev as orchestrator bookkeeping). No human-only external actions in ACs, so normal done (no awaiting-operator). Committed changes.

Files changed with one-line descriptions:
- `Cargo.toml` — root workspace declaring exactly the 7 members
- `bronze-domain/{Cargo.toml,src/lib.rs}` — empty lib scaffold
- `bronze-capture/{Cargo.toml,src/lib.rs}` — empty lib scaffold
- `bronze-storage/{Cargo.toml,src/lib.rs}` — empty lib scaffold
- `bronze-settings/{Cargo.toml,src/lib.rs}` — empty lib scaffold
- `bronze-diagnostics/{Cargo.toml,src/lib.rs}` — empty lib scaffold
- `bronze-platform/{Cargo.toml,src/lib.rs}` — empty lib scaffold
- `bronze-platform-macos/{Cargo.toml,src/lib.rs}` — empty lib scaffold (no framework imports)
- `Cargo.lock` — generated (no external deps)
- `.gitignore` — added rust target/ + extras (one low patch during review)
- `_bmad-output/implementation-artifacts/1-3-scaffold-cargo-workspace.md` — full template expansion + triage + auto result (status draft -> ready -> in-progress -> in-review -> done)

Review findings breakdown: patches applied 1 (low: 1); items deferred 2; items rejected 20. No high/medium. Score `3 × 0 + 1 × 1 = 1` (threshold 5) → followup false.

Verification performed:
- `cargo fmt --check` — PASSED (0)
- `cargo clippy --workspace -- -D warnings` — PASSED (0 warnings, finished dev profile)
- `cargo test --workspace` — PASSED (all 7 crates: 0 tests ok)
- `python3 tooling/planning-checks.py` — PASS requirement-parity, enum/schema, local-links, traceability (56 IDs)
- `npx --yes markdownlint-cli2 ...` — 0 issues (26 files)
- ls + rg + cargo tree + find — all AC paths and negative checks passed
- full manual listed checks passed
- baseline rev captured; no sprint-status or git revert of orchestrator row

Residual risks: versions are 0.0.0 (not inherited); no rustfmt.toml (per design note, command flag suffices); planning-checks does not yet invoke cargo gates (deferred to 1.8 aggregate); docs cross-refs and desktop linkage come in later stories. All per explicit non-goals and contract. target/ produced (gitignored).

Follow-up review recommendation: false

### 2026-08-27 follow-up review

Status: done

Review findings (this pass):
- patches applied: 0
- items deferred: 0 new (2 pre-existing preserved)
- items rejected: 18

Follow-up review recommendation: false (patched high=0, medium=0, low=0; score 0)

Verification:
- `cargo fmt --check` — exit 0
- `cargo clippy --workspace -- -D warnings` — exit 0
- `cargo test --workspace` — exit 0 (0 tests per crate)
- `python3 tooling/planning-checks.py` — PASS requirement-parity, enum/schema, local-links, traceability (56 IDs)
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` — 0 issues in 26 files
- manifests/`lib.rs`/`Cargo.lock` exist; each `bronze-*/src` contains only `lib.rs`
- `rg` forbidden tokens in `bronze-*/src` — no matches
- `cargo tree --workspace --depth 0` — seven members, no external deps

Residual risks:
- cargo fmt/clippy/test are not in `planning-checks.py` / verify aggregate (deferred; story 1.8)
- planning docs still lack crate path cross-refs (deferred)

