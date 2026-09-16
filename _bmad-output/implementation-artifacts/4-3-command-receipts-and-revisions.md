# Story 4.3: Command receipts and revisions

Status: done

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

Hedgehog authored loop / layer-eng (cursor-grok). Docs layer records rust-layer evidence only.

### File List

- `bronze-storage/src/receipts.rs` — idempotent command IDs; 7-day window; content-free receipts; revision conflict

### Notes

- UI layer was a no-op. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
