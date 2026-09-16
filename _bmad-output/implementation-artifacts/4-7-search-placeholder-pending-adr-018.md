# Story 4.7: Search placeholder pending ADR-018

Status: done

## Story

As a developer,
I want FTS hook that does not claim locale semantics,
So that QUE-007 is not falsely completed.

**Requirements:** QUE-007, G-06, DG-10
**ADRs:** ADR-018 Proposed
**Dependencies:** Story 4.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items persist
**When** a naive FTS or application filter exists
**Then** story documents ADR-018 as blocking complete locale search
**And** raw body unchanged
**And** query never enters diagnostics

**Failure / recovery:**
Do not mark QUE-007 done.

**Security / privacy / diagnostics / a11y / i18n / data:**
FTS is sensitive data.

**Automated verification:**
- `cargo test -p bronze-storage search_placeholder`

**Signed-build / manual evidence:**
unit tests; tokenizer gate open

**Non-goals:**
final tokenizer


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok). Docs layer records rust-layer evidence only.

### File List

- `bronze-storage/src/search.rs` — substring placeholder; `QUE_007_COMPLETE=false`; locale-aware path errors; query never written to `diagnostic_events`

### Notes

- UI layer was a no-op at 4.7 (local search UI is story 6.5). **Debt #3 blocked:** ADR-018 remains **Proposed** in `docs/18-adrs.md`. Do not implement locale FTS, do not silently accept ADR-018, and do not mark QUE-007 done (`QUE_007_COMPLETE=false`). Query must never enter diagnostics.
