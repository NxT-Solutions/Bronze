---
title: 'Story 1.7: shadcn React Aria foundation'
type: 'feature'
created: '2026-08-27'
status: 'done'
review_loop_iteration: 0
followup_review_recommended: true
context:
  - _bmad-output/implementation-artifacts/epic-1-context.md
  - docs/18-adrs.md
  - docs/06-system-architecture.md
  - docs/03-prd.md
  - AGENTS.md
  - _bmad-output/implementation-artifacts/1-2-scaffold-pnpm-turbo-workspace.md
  - _bmad-output/implementation-artifacts/1-6-i18n-catalogs-and-pseudo-locales.md
  - docs/11-i18n-localization.md
  - docs/10-accessibility-conformance-plan.md
  - _bmad-output/planning-artifacts/ux-designs/ux-bronze-app-2026-08-27/DESIGN.md
  - _bmad-output/planning-artifacts/ux-designs/ux-bronze-app-2026-08-27/EXPERIENCE.md
  - docs/14-agentic-implementation-plan.md
warnings: []
deferred: []
baseline_revision: '7a0ba915955d0cc86e643e540bdc74603e2ba3f0'
baseline_commit: '7a0ba915955d0cc86e643e540bdc74603e2ba3f0'
operator_actions: []
---

# Story 1.7: shadcn React Aria foundation

## Story

As a developer,
I want owned shadcn components on React Aria with tokens and visible focus,
So that UI stories do not mix primitive bases.

**Requirements:** A11Y-001, A11Y-002, A11Y-003
**ADRs:** ADR-012, ADR-013
**Dependencies:** Stories 1.2 and 1.6
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** packages/ui exists
**When** shadcn init uses React Aria base
**Then** Button/TextField/Dialog primitives have role/name/focus tests
**And** no clickable div
**And** focus-visible not removed
**And** Reduce Motion / Increase Contrast CSS hooks exist

**Failure / recovery:**
If React Aria base cannot initialize, stop for DG-06 rather than mixing Radix.

**Security / privacy / diagnostics / a11y / i18n / data:**
axe on default/empty/focus states. No color-only status.

**Automated verification:**
- `pnpm --filter @bronze/ui test`
- `pnpm --filter @bronze/ui lint`

**Signed-build / manual evidence:**
component tests; not VoiceOver

**Non-goals:**
queue cards, virtualized lists, contenteditable


## Code Map

- `packages/ui/package.json` -- React Aria + shadcn base deps (react-aria-components, cva, lucide, tailwind-merge, tailwindcss 4.3.3), vitest+axe+rtl test stack, test/lint/typecheck scripts, exports for css/components/utils
- `packages/ui/components.json` -- aria-nova style (React Aria base), rtl true, aliases
- `packages/ui/src/lib/utils.ts` -- cn() twMerge+clsx
- `packages/ui/src/styles/globals.css` -- Bronze tokens (DESIGN.md), system fonts only, :focus-visible rule, @media reduce-motion + contrast hooks with !important, @theme inline bridge
- `packages/ui/src/components/button.tsx` -- Button using RAC ButtonPrimitive + cva variants + cn + data-slot; focus ring preserved
- `packages/ui/src/components/text-field.tsx` -- TextField using RAC TextFieldPrimitive/Input/Label; error with id/aria-describedby
- `packages/ui/src/components/dialog.tsx` -- Dialog/DialogTrigger/DialogContent using RAC Dialog/ModalOverlay/Modal + tokens
- `packages/ui/src/index.ts` -- barrel re-exports
- `packages/ui/src/components/*.test.tsx` -- role/name/focus/axe/no-div/description assertions
- `packages/ui/src/styles/globals.test.ts` -- Reduce Motion / Increase Contrast CSS hook pins
- `packages/ui/tsconfig.json`, `vitest.config.ts`, `test/setup.ts` -- strict bundler TS, jsdom vitest + jest-dom, alias @/*
- `biome.json` (root) -- noImportantStyles:warn + tailwindDirectives + formatWithErrors (for required hooks)
- `pnpm-workspace.yaml` -- allowBuilds esbuild:true (side effect of dep add)

## Tasks & Acceptance

**Execution:**
- `packages/ui/package.json` -- declared full React Aria foundation stack + test/lint/typecheck
- `packages/ui/src/components/{button,text-field,dialog}.tsx` + tests -- implemented RAC primitives, semantic roles/names, focus-visible, no div clicks, CSS hooks
- `packages/ui/src/styles/globals.css` -- tokens + reduce-motion + increase-contrast hooks
- `packages/ui/*` config files -- vitest/TS/biome shadcn-compatible setup
- root `biome.json` + `package.json` + `pnpm-workspace.yaml` -- minimal support for new lint/build
- verification commands executed and passed (direct + pnpm filter after allowBuilds)

**Acceptance Criteria:**
- Given packages/ui exists
- When shadcn init uses React Aria base (replicated via structure/components.json + RAC)
- Then Button/TextField/Dialog primitives have role/name/focus tests
- And no clickable div
- And focus-visible not removed
- And Reduce Motion / Increase Contrast CSS hooks exist

## Spec Change Log

## Review Triage Log

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 3 (low: 2, medium: 1)
- defer: 9
- reject: 12
- addressed_findings:
  - `[low] [patch] added root "." export to packages/ui/package.json for consumers`
  - `[medium] [patch] added useId + aria-describedby + id on error div in TextField (association)`
  - `[low] [patch] normalized quotes to double in vitest.config.ts + test/setup.ts to satisfy biome rule`

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 8: (high 2, medium 3, low 3)
- defer: 0
- reject: 21
- addressed_findings:
  - `[high] [patch] added tailwindcss 4.3.3 so @import "tailwindcss" resolves`
  - `[high] [patch] removed focus-visible:outline-none; TextField uses ring-2 so global outline remains`
  - `[medium] [patch] role/name/focus/axe tests: ring assertion, dialog focus, axe on focus and open dialog`
  - `[medium] [patch] TextField error alert + isInvalid (no color-only status)`
  - `[medium] [patch] ModalOverlay isDismissable`
  - `[low] [patch] secondary tokens + DESIGN.md radii 8/10/12`
  - `[low] [patch] cn(buttonVariants(...), className)`
  - `[low] [patch] removed unused animate-in classes`

### 2026-08-27 — Review pass
- intent_gap: 0
- bad_spec: 0
- patch: 4: (high 0, medium 2, low 2)
- defer: 0
- reject: 27
- addressed_findings:
  - `[medium] [patch] open-dialog axe now runs on the dialog node (portal), not the render container`
  - `[medium] [patch] Dialog open/focus test asserts className does not include outline-none`
  - `[low] [patch] TextField error test asserts accessible description association`
  - `[low] [patch] globals.css test pins Reduce Motion / Increase Contrast media hooks`

## Design Notes

Replicated shadcn `init --base aria` output by hand (no net fetch per product policy; produced identical components.json + RAC usage + owned files). Used system fonts + Bronze tokens from DESIGN.md. !important retained (and warned) only to guarantee OS media query overrides per A11Y-003/ADR-013. Dialog uses working RAC overlay+modal composition that satisfies role+name+axe in tests; future stories may refine.

## Verification

**Commands:**
- `pnpm --filter @bronze/ui test` -- exit 0; 4 test files, 13 passed (roles/names/focus/axe default+focus+open-on-dialog-node + no-div + error alert/description + CSS hooks)
- `pnpm --filter @bronze/ui lint` -- exit 0; 4 warnings only (noImportantStyles on the 4 required !important lines for media hooks)
- `pnpm --filter @bronze/ui typecheck` -- exit 0
- `python3 tooling/planning-checks.py` -- PASS (requirement-parity, enum/schema, local-links, traceability; 56 IDs)
- `npx --yes markdownlint-cli2 'docs/**/*.md' README.md AGENTS.md` -- 0 issues

**Manual checks (if no CLI):**
- rg -n 'div\[role="button"\|onclick' packages/ui/src -- no matches in components
- grep -E 'focus-visible|ring' packages/ui/src/styles/globals.css packages/ui/src/components/button.tsx -- present
- grep -E 'prefers-reduced-motion|prefers-contrast' packages/ui/src/styles/globals.css -- hooks present with !important
- no user-facing sentence literals in src/components (only test data and prop labels)
- axe called on default/empty/focus states in tests
- packages/ui exists with RAC only (no radix)

## Dev Agent Record

### Agent Model Used
xai/grok-build-0.1 (via bmad-build-auto + subagent general)

### File List
- packages/ui/package.json
- packages/ui/components.json
- packages/ui/src/lib/utils.ts
- packages/ui/src/styles/globals.css
- packages/ui/src/styles/globals.test.ts
- packages/ui/src/components/button.tsx
- packages/ui/src/components/button.test.tsx
- packages/ui/src/components/text-field.tsx
- packages/ui/src/components/text-field.test.tsx
- packages/ui/src/components/dialog.tsx
- packages/ui/src/components/dialog.test.tsx
- packages/ui/src/index.ts
- packages/ui/tsconfig.json
- packages/ui/vitest.config.ts
- packages/ui/test/setup.ts
- biome.json
- package.json (root)
- pnpm-workspace.yaml
- _bmad-output/implementation-artifacts/1-7-shadcn-react-aria-foundation.md

## Auto Run Result

Status: done

Summary of implemented change: Follow-up review of the owned shadcn/React Aria `packages/ui` foundation (Button, TextField, Dialog, tokens, focus-visible, Reduce Motion / Increase Contrast hooks). This pass patched verification gaps only.

Files changed with one-line descriptions:
- `packages/ui/src/components/dialog.test.tsx` -- axe open dialog on the dialog node; assert no `outline-none`
- `packages/ui/src/components/text-field.test.tsx` -- assert error accessible description
- `packages/ui/src/styles/globals.test.ts` -- pin Reduce Motion / Increase Contrast CSS hooks
- `_bmad-output/implementation-artifacts/1-7-shadcn-react-aria-foundation.md` -- triage log and result

Review findings breakdown: patches applied 4 (medium 2, low 2); deferred 0; rejected 27.

Follow-up review recommendation: true (patched high 0, medium 2, low 2; score `3 × 2 + 2 = 8` ≥ 5).

Verification performed:
- `pnpm --filter @bronze/ui test` -- exit 0; 4 files, 13 passed
- `pnpm --filter @bronze/ui lint` -- exit 0; 4 noImportantStyles warnings on required media-hook `!important` lines
- `pnpm --filter @bronze/ui typecheck` -- exit 0

Residual risks: jsdom axe still logs HTMLCanvas `getContext` not-implemented noise; open-dialog axe covers the dialog node not the overlay; RAC Dialog still names via consumer `aria-label`; CSS hooks are file-existence pins not computed-style / matchMedia tests. Not VoiceOver / WCAG evidence.

