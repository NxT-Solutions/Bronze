# Story 6.5: Local search UI

Status: ready-for-dev

## Story

As a user,
I want to search items locally,
So that I can find parked text.

**Requirements:** QUE-007, A11Y-002
**ADRs:** ADR-018 Proposed
**Dependencies:** Stories 4.7 and 6.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** library exists
**When** search UI is added
**Then** results announce count without putting query in diagnostics
**And** QUE-007 locale tokenizer remains partial until ADR-018

**Failure / recovery:**
Do not claim G-06 search complete.

**Security / privacy / diagnostics / a11y / i18n / data:**
Query stays on device.

**Automated verification:**
- `pnpm --filter desktop test -- search`

**Signed-build / manual evidence:**
component tests

**Non-goals:**
final FTS semantics


## Dev Agent Record

### Agent Model Used

### File List
