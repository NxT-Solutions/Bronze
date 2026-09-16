# Story 8.1: Tokenized bronze identity

Status: done

## Story

As a user,
I want warm bronze accents with measured contrast,
So that the app looks like Bronze.

**Requirements:** A11Y-003
**ADRs:** ADR-012
**Dependencies:** Stories 1.7 and 6.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** UI primitives exist
**When** tokens from DESIGN.md are applied
**Then** normal text contrast ≥4.5:1 in light/dark in automated sampling
**And** concept PNG is inspiration not pixel spec
**And** no Copper trade dress

**Failure / recovery:**
Color-only status fails.

**Security / privacy / diagnostics / a11y / i18n / data:**
Increase Contrast / Differentiate Without Color variants.

**Automated verification:**
- `pnpm --filter @bronze/ui test -- contrast`

**Signed-build / manual evidence:**
token tests; not a public AA claim

**Non-goals:**
marketing site


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok).

### File List

- `bronze-domain/src/tokens.rs` — light/dark pairs, WCAG contrast ≥4.5
- `packages/ui/src/lib/contrast.ts` — CSS token sampling
- `packages/ui/src/styles/globals.css` — DESIGN.md-inspired bronze tokens
- `apps/desktop/src/index.html` — warm parchment surfaces

### Notes

- Concept PNG is inspiration, not a pixel spec. Sampling is not a public AA claim. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
