---
title: 'DW-2: cross-references to new crates missing from planning docs'
type: 'chore'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - AGENTS.md
  - Cargo.toml
  - _bmad-output/implementation-artifacts/epic-1-context.md
  - _bmad-output/implementation-artifacts/1-3-scaffold-cargo-workspace.md
  - _bmad-output/planning-artifacts/SOURCE-MAP.md
  - docs/06-system-architecture.md
  - docs/14-agentic-implementation-plan.md
warnings: []
deferred:
  - summary: >-
      Stale `crates/` references and missing bronze-settings cross-reference remain in other planning artifacts (ARCHITECTURE-SPINE.md, 1-3 spec, epic context) after the targeted update.
    evidence: |-
      DW-2 bundle intent and spec explicitly limited the scope to exactly docs/06-system-architecture.md and docs/14-agentic-implementation-plan.md; other files that duplicated the old diagrams were intentionally untouched.
    location: >-
      _bmad-output/planning-artifacts/architecture/architecture-bronze-app-2026-08-27/ARCHITECTURE-SPINE.md:204
    severity: low
  - summary: >-
      Updated tree diagrams omit root-level peers (Cargo.toml, package.json, tooling/, docs/) and do not document cargo vs pnpm/turbo manager differences for the sibling entries.
    evidence: |-
      Pre-existing abbreviated diagram style; narrow intent asked only to remove the fictional crates/ prefix and add the one table row.
    location: >-
      docs/06-system-architecture.md:181 ; docs/14-agentic-implementation-plan.md:36
    severity: low
  - summary: >-
      Ownership table now lists bronze-settings but still omits apps/desktop/src-tauri (a workspace member) and the new row is the only one using (SET-*) requirement tag.
    evidence: |-
      Table historically listed selective packages; the inserted row text was taken verbatim from the spec's prescribed value for this DW.
    location: >-
      docs/06-system-architecture.md:207
    severity: low
  - summary: >-
      06 and 14 diagrams remain asymmetric (no JS/Rust grouping, different fence styles ~~~ vs ```, asymmetric coverage of tooling/docs/test-support, src vs src-tauri naming).
    evidence: |-
      Pre-existing inconsistencies between the two documents' views of the layout; this change only performed the crate flattening and single table addition.
    severity: low
  - summary: >-
      bronze-settings "Must not own" column lists "capture decisions" which overlaps the responsibility stated for bronze-capture.
    evidence: |-
      Exact row text (including the must-not phrase) was dictated by the spec's "using:" instruction derived from the bundle intent.
    location: >-
      docs/06-system-architecture.md:207
    severity: low
  - summary: >-
      docs/14 bronze-domain comment still says "capture state/result", which contradicts the 06 domain ownership row (entities/lifecycle/undo) and belongs with bronze-capture.
    evidence: |-
      Pre-existing 14 comment text was preserved on flatten; 06 domain row is "Entities, lifecycle, order, undo invariants" while 14 labels bronze-domain with capture state/result at root indent.
    location: >-
      docs/14-agentic-implementation-plan.md:36
    severity: low
baseline_revision: '1fa0aa9561a43d1fa795d78ee1879a680a60af53'
baseline_commit: '1fa0aa9561a43d1fa795d78ee1879a680a60af53'
operator_actions: []
---

<intent-contract>

## Intent

**Problem:** The seven Rust crates live at repository root (per root Cargo.toml workspace.members) but docs/06-system-architecture.md and docs/14-agentic-implementation-plan.md still diagram them under a fictional `crates/` prefix. bronze-settings appears in the 06 tree listing (line 185) but is absent from the Ownership table (205-210). This breaks cross-references from the 1.3 scaffold and architecture spine.

**Approach:** Update only the two named docs: flatten the crate listings to repository root (remove crates/ wrapper, dedent bronze-* entries), and insert the missing bronze-settings row into 06's ownership table. Do not edit any Cargo.toml (including apps/desktop/src-tauri), do not add dependencies, do not touch source, deferred-work ledger, sprint-status, or any other file.

## Boundaries & Constraints

**Always:**
- Edit exactly two files: docs/06-system-architecture.md and docs/14-agentic-implementation-plan.md
- In both "tree" / blueprint diagrams, remove the `crates/` line and list the seven bronze-* crates directly under the repo root (same indent level as packages/ and native/)
- Insert one new row for bronze-settings into 06 ownership table, immediately after the bronze-storage row and before bronze-diagnostics, using: `| bronze-settings | Typed settings schema, shortcut registry and policy (SET-*) | Rendering, Tauri/AppKit, capture decisions |`
- Keep all other table rows, text, IDs, and whitespace identical outside the changed sections
- The changes must make the docs match the actual on-disk layout from Cargo.toml and the 1.3 crate list
- Leave apps/desktop/src-tauri/Cargo.toml and all bronze-*/Cargo.toml untouched

**Block If:** none

**Never:**
- Modify any .toml, .rs, .json, .py, ledger, or sprint files
- Add bronze-* entries to any package dependencies
- Change crate order or descriptions in ways that contradict 1-3-scaffold-cargo-workspace.md or epic-1-context
- Update or reference the deferred-work ledger (orchestrator records resolution)
- Touch docs other than the two explicitly named

</intent-contract>

## Code Map

- `docs/06-system-architecture.md:181-188` -- Target tree diagram; currently wrapped under `crates/`; bronze-settings present here
- `docs/06-system-architecture.md:205-210` -- Ownership table; bronze-settings missing (storage then jumps to diagnostics)
- `docs/14-agentic-implementation-plan.md:36-43` -- "3. Repository blueprint" diagram; lists crates/ subdir with the seven members + comments
- `Cargo.toml:1-11` -- authoritative workspace members at root (read-only reference; no edits)
- `_bmad-output/implementation-artifacts/1-3-scaffold-cargo-workspace.md:46,110-124,132` -- lists exactly the seven crates, their slots, and AC requiring them at root
- `_bmad-output/implementation-artifacts/epic-1-context.md:13,40` -- 1.3 and Rust crate list for Epic 1
- `docs/14-agentic-implementation-plan.md:40` -- inline comment "# typed settings/shortcut policy" for bronze-settings
- `/Users/noah/fdev/projects/bronze-app/.bmad-loop/runs/20260827-174416-675e/bundles/crate-path-doc-cross-refs/intent.md` -- bundle intent (verbatim source of this task; do not edit)
- `docs/06-system-architecture.md:185` -- location evidence for bronze-settings in tree

## Tasks & Acceptance

**Execution:**
- `docs/06-system-architecture.md` -- edit the Target tree block (remove `crates/` wrapper and dedent the seven bronze-* lines so they sit at the same level as `packages/` and `native/`) -- matches actual FS and Cargo.toml
- `docs/06-system-architecture.md` -- insert bronze-settings ownership row after bronze-storage row in the Ownership table -- closes the omission noted in DW-2 and 1.3
- `docs/14-agentic-implementation-plan.md` -- edit the repository blueprint code block (remove `crates/` line and dedent bronze-* crates) -- so diagram matches Cargo workspace members
- `git diff --stat` -- confirm exactly two files touched, no others
- `grep -n "crates/" docs/06-system-architecture.md docs/14-agentic-implementation-plan.md` -- must return zero matches after edits
- `python3 tooling/planning-checks.py` -- must pass (doc-only change must not break parity)

**Acceptance Criteria:**
- Given the docs as captured in 1.3 and current tree (crates at root)
- When the two targeted doc updates are performed per the boundaries
- Then `docs/06-system-architecture.md` target tree shows the bronze crates directly (no `crates/` prefix) and ownership table lists all seven crates in sequence including bronze-settings between storage and diagnostics
- And `docs/14-agentic-implementation-plan.md` repository blueprint shows bronze crates at root level
- And `git status` / diff reports no other files modified
- And no bronze-* dependencies were added to apps/desktop/src-tauri/Cargo.toml or anywhere
- And planning-checks.py reports clean on the changed docs

## Spec Change Log

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 5 (low:5)
- reject: 9 (low:9)
    - addressed_findings:
      - none

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 1: (high 0, medium 0, low 1)
- reject: 14: (high 0, medium 0, low 14)
- addressed_findings:
  - none

## Verification

**Commands:**
- `git diff --stat` -- expected: only the two .md files listed
- `grep -c "crates/" docs/06-system-architecture.md docs/14-agentic-implementation-plan.md || true` -- expected: 0
- `grep -A 2 -E "bronze-storage|bronze-settings|bronze-diagnostics" docs/06-system-architecture.md` -- expected: settings row present between the other two
- `python3 tooling/planning-checks.py` -- expected: exit 0, no errors on architecture or agentic plan docs
- `git diff docs/06-system-architecture.md docs/14-agentic-implementation-plan.md | cat` -- expected: shows the path flattening + the added table row only

**Manual checks (if no CLI):**
- Open the two docs; visually confirm crate trees are flat at root and table contains bronze-settings with sensible owns/must-not text.

## Auto Run Result

Status: done

Summary: Follow-up review of the completed DW-2 doc flatten. `docs/06-system-architecture.md` and `docs/14-agentic-implementation-plan.md` list the seven `bronze-*` crates at repository root (no `crates/` wrapper) and 06's ownership table includes the prescribed bronze-settings row between storage and diagnostics. Diff implements the literal tree+table reading of the intent. No patches applied.

Files changed:
- `docs/06-system-architecture.md` — flattened crate tree to repo root; inserted bronze-settings ownership row
- `docs/14-agentic-implementation-plan.md` — flattened repository blueprint crate listing to repo root

Review findings breakdown:
- patches applied: 0
- items deferred: 1 (low) — 14 bronze-domain comment still says "capture state/result"
- items rejected: 14 (low) — restyle/grouping asks, already-deferred other-doc/table gaps, prescribed-row wording, comment-column restyle

Follow-up review recommendation: false (patched high=0, medium=0, low=0; score `3×0+1×0=0`)

Verification performed:
- `git diff --stat -- docs/06-system-architecture.md docs/14-agentic-implementation-plan.md` — 2 files, 15 insertions, 16 deletions
- `grep -n "crates/" docs/06-system-architecture.md docs/14-agentic-implementation-plan.md` — zero matches
- ownership grep — bronze-settings row at line 207 between bronze-storage and bronze-diagnostics
- `python3 tooling/planning-checks.py` — PASS requirement-parity, enum/schema, local-links, traceability (56 IDs)
- Cargo.toml workspace members remain root-level; no .toml/.rs/.json/.py edits in this story

Residual risks:
- Other planning artifacts still diagram `crates/` (already deferred)
- Orchestrator-owned `_bmad-output/implementation-artifacts/deferred-work.md` is dirty in this worktree and was not modified, reverted, or committed by this run

