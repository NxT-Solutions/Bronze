# Story 7.3: Permission health center

Status: ready-for-dev

## Story

As a user,
I want distinct permission rows with why, retest, and alternatives,
So that denial is recoverable.

**Requirements:** SET-003, SET-004, CAP-003
**ADRs:** ADR-005
**Dependencies:** Stories 3.1 and 7.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** permission enum exists
**When** health UI is implemented
**Then** each capability is independent
**And** Screen Recording shown as Not used
**And** denial leaves manual composer

**Failure / recovery:**
Launch-loop prompting forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
No content in self-test.

**Automated verification:**
- `pnpm --filter desktop test -- permissions`

**Signed-build / manual evidence:**
component tests; real TCC human

**Non-goals:**
completing Story 3.9


## Dev Agent Record

### Agent Model Used

### File List
