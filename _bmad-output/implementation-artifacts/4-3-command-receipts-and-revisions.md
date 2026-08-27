# Story 4.3: Command receipts and revisions

Status: ready-for-dev

## Story

As a developer,
I want idempotent command IDs and optimistic revisions,
So that retries do not duplicate work.

**Requirements:** DAT-001, QUE-002
**ADRs:** ADR-008
**Dependencies:** Story 4.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** schema exists
**When** command API is implemented
**Then** duplicate ID inside window reconstructs prior result
**And** expired ID returns idempotency_expired and does not execute
**And** receipts contain no body

**Failure / recovery:**
Stale revision returns typed conflict.

**Security / privacy / diagnostics / a11y / i18n / data:**
Content-free receipts.

**Automated verification:**
- `cargo test -p bronze-storage receipts`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
UI conflict dialog


## Dev Agent Record

### Agent Model Used

### File List
