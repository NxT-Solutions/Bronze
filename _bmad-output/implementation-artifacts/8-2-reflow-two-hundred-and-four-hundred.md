# Story 8.2: Reflow two hundred and four hundred

Status: done

## Story

As a user,
I want 200% text resize and 400%/320 CSS px reflow,
So that A11Y-003 is evidenced in WebView.

**Requirements:** A11Y-003
**ADRs:** ADR-013
**Dependencies:** Stories 6.2, 6.3, 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** core screens exist
**When** layout tests run at 320 CSS px and 200% text
**Then** no control requires two-axis scroll
**And** toolbar overflow labeled
**And** composer reachable

**Failure / recovery:**
Lost functionality fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
en-XA long strings included.

**Automated verification:**
- `pnpm --filter desktop test -- reflow`

**Signed-build / manual evidence:**
layout tests; manual zoom human

**Non-goals:**
WCAG conformance claim


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `bronze-domain/src/reflow.rs` — 320 CSS px / 200% text, no two-axis scroll
- `packages/ui/src/lib/reflow.ts` — layout helpers
- `packages/ui/src/components/copy-toolbar.tsx` — labeled overflow
- `apps/desktop/src/index.html` — overflow-x hidden, composer in flow

### Notes

- Layout tests are not a public WCAG claim. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
