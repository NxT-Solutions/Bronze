# Story 1.7: shadcn React Aria foundation

Status: ready-for-dev

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


## Dev Agent Record

### Agent Model Used

### File List
