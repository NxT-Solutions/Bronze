---
title: 'DW-9: Extend globals.test.ts to assert !important overrides for A11Y-003 hooks'
type: 'chore'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: false
baseline_revision: '2bd4b85e10ead8eb506c0691b61c219b595cad39'
baseline_commit: '2bd4b85e10ead8eb506c0691b61c219b595cad39'
context:
  - _bmad-output/implementation-artifacts/epic-1-context.md
  - _bmad-output/implementation-artifacts/1-8-workspace-verify-aggregate.md
  - _bmad-output/implementation-artifacts/1-7-shadcn-react-aria-foundation.md
  - AGENTS.md
  - docs/03-prd.md
  - docs/18-adrs.md
  - packages/ui/src/styles/globals.css
  - packages/ui/src/styles/globals.test.ts
  - biome.json
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** packages/ui/src/styles/globals.test.ts only regex-matches media-query strings (`prefers-reduced-motion: reduce`, `prefers-contrast: more`). The actual A11Y-003 hooks ship `!important` overrides on `transition`/`animation`/`scroll-behavior` and `border-width` (to strengthen over Tailwind utilities after `@import "tailwindcss";`). Hygiene once stripped the `!important` and the test still passed; the DW-9 gap was recorded in 1.8 review.

**Approach:** Extend the existing source-content pin test (the only file that needs change) with explicit regex assertions for the four `!important` declarations. Keep the fs-read + vitest pattern; no DOM, no new deps, no CSS changes.

## Boundaries & Constraints

**Always:**
- Preserve A11Y-003 in test description and comments.
- The test must fail (red) if any `!important` declaration for the hooks is removed from globals.css.
- Use exactly the existing `pnpm --filter @bronze/ui test` (vitest).
- Read AGENTS.md, docs/03-prd.md, relevant specs, docs/18-adrs.md before editing.
- Report full verify command output as evidence before handoff.

**Block If:**
- (none; narrow, pre-existing files only)

**Never:**
- Do not edit the deferred-work ledger (`deferred-work.md`) or mutate deferred entries in 1-8 spec; orchestrator records resolution.
- Do not edit globals.css.
- Do not introduce computed-style / matchMedia simulation (jsdom does not apply `prefers-*` media rules to computed styles without brittle external helpers; source pin is the established and sufficient technique here).
- Do not add user-facing strings or locale work (CSS only).
- Do not accept or reference ADR-002/009/018.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| Hooks present with !important | globals.css as shipped | both media queries + 4 !important rules matched; test passes | n/a |
| Any !important removed (e.g. transition line) | css source read by test lacks `transition: none !important` | expect.toMatch fails; test fails closed | surfaces regression in verify |
| Media query string only present, !important stripped | hygiene-like edit | test now fails (unlike pre-fix) | prevents silent loss of override strength |
| Test run in filter | `pnpm --filter @bronze/ui test` | only ui package tests execute; globals.test included | n/a |

</intent-contract>

## Code Map

- `packages/ui/src/styles/globals.test.ts:12` -- the a11y hooks test (lines 12-16); sole file to edit; extend describe title, it name, and add four !important matchers after the existing two
- `packages/ui/src/styles/globals.css:95-104` -- @media (prefers-reduced-motion: reduce) { *,*::before,*::after { transition:none!important; animation:none!important; scroll-behavior:auto!important; } } -- source of 3 rules to pin
- `packages/ui/src/styles/globals.css:106-127` -- @media (prefers-contrast: more) { ... border-width:2px!important; on button/input/etc } -- source of 4th rule to pin
- `packages/ui/package.json:18` -- `"test": "vitest run"` -- how the test is executed (read-only)
- `biome.json:14` -- `"complexity": { "noImportantStyles": "warn" }` -- intentional for these 4 lines per 1.7 design note; test change does not affect
- `_bmad-output/implementation-artifacts/1-8-workspace-verify-aggregate.md:17` -- origin of DW-9 record (read-only; do not edit deferred section)
- `AGENTS.md:1,11,25,27` -- must read before code change; accessibility paths require tests; run formatter/static/unit/verify and report evidence; never claim conformance from compile alone
- `docs/03-prd.md:108` -- A11Y-003 authority (read-only)
- `docs/18-adrs.md:524` -- ADR-013 (A11Y evidence model) (read-only)

## Tasks & Acceptance

**Execution:**
- `packages/ui/src/styles/globals.test.ts` -- extend the it() and add toMatch assertions for the four !important declarations (plus tighten describe/it names to reference A11Y-003 and overrides) -- directly resolves DW-9

**Acceptance Criteria:**
- Given A11Y-003 hooks exist in globals.css with their !important rules
- When `pnpm --filter @bronze/ui test` (or root verify) executes the globals.test
- Then the test asserts the media query strings AND the !important overrides
- And the test fails if any !important is absent
- And full `pnpm verify` exits 0 with ui test passing

## Spec Change Log

## Review Triage Log

## Design Notes

The !important is not styling abuse: it is the minimal way to ensure the media-query preference rules win over Tailwind's utility classes (which are generated after the @import and can set transition/animation/border-width on the universal and control selectors). See 1.7 design note and A11Y-003/ADR-013.

Test stays a pure content pin (readFileSync + regex) exactly as 1.7 established. This is proportional to the risk (a11y regression path) and requires zero new surface.

The "computed override of Tailwind utilities" in the ledger entry describes the runtime effect the !important achieves; asserting the declarations in source is the practical, stable way to guard it without fragile runtime media simulation in jsdom.

## Verification

**Commands:**
- `pnpm --filter @bronze/ui test` -- expected: exit 0;  "globals.css a11y hooks (A11Y-003)" test passes with 6 matchers
- `pnpm verify` -- expected: exit 0 (includes the ui test gate; 4 noImportantStyles warnings remain on the intentional lines)
- `pnpm --filter @bronze/ui lint` -- expected: exit 0 (warnings only, unchanged)

**Manual checks (if no CLI):**
- cat packages/ui/src/styles/globals.test.ts -- contains the new !important regexes and A11Y-003 in title
- grep -E 'transition:\s*none\s*!important|animation:\s*none\s*!important|scroll-behavior:\s*auto\s*!important|border-width:\s*2px\s*!important' packages/ui/src/styles/globals.css -- still present in the media blocks
- No other files changed for this bundle

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 0
- reject: 22
- addressed_findings:
  - none

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 0
- defer: 0
- reject: 15
- addressed_findings:
  - none

## Auto Run Result

Status: done

Summary of implemented change: Extended the pre-existing globals.css a11y test (from 1.7) to also assert the four `!important` declarations inside the Reduce Motion and Increase Contrast media hooks. This ensures hygiene, future edits, or Tailwind interactions cannot silently drop the A11Y-003 override strength. Purely additive source regexes per the chosen reading of DW-9 intent (static pin; computed simulation out of scope per boundaries).

Files changed:
- `packages/ui/src/styles/globals.test.ts` — extended describe/it titles with A11Y-003 + "and !important overrides"; added four `toMatch( /... !important/ )` plus explanatory comment

Review findings breakdown: patches applied 0; items deferred 0; items rejected 15 (this follow-up pass). Prior pass rejected 22. Ledger mutation in `deferred-work.md` is orchestrator-owned and was not treated as this story's defect.

Follow-up review recommendation: false (patched high 0, medium 0, low 0; score 3×0 + 1×0 = 0).

Verification performed:
- `pnpm --filter @bronze/ui test` -- exit 0; Test Files 4 passed, Tests 13 passed (includes `src/styles/globals.test.ts` 1 test)
- `pnpm --filter @bronze/ui lint` -- exit 0; 4 noImportantStyles warnings (unchanged, intentional)
- `pnpm verify` -- exit 0 (biome + turbo typecheck/test/validate + cargo fmt/clippy/test + planning-checks)

Residual risks: source pin is file-wide substring presence, not media-block nesting or computed Tailwind override; that is the contract's chosen technique. Orchestrator-owned `deferred-work.md` left unstaged.

