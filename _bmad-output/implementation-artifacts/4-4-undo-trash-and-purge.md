# Story 4.4: Undo trash and purge

Status: done

## Story

As a developer,
I want tombstones, undo inverses, dependency-safe purge,
So that deletes are recoverable.

**Requirements:** QUE-006, DAT-004
**ADRs:** ADR-008
**Dependencies:** Story 4.3
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** items persist
**When** trash/undo/purge are implemented
**Then** purge never cascade-deletes non-trashed descendants
**And** default trash 30 days
**And** forensic erasure is not claimed

**Failure / recovery:**
Empty Trash without confirmation API is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
FTS rows removed on purge when FTS exists.

**Automated verification:**
- `cargo test -p bronze-storage undo`
- `cargo test -p bronze-domain purge`

**Signed-build / manual evidence:**
unit tests

**Non-goals:**
Empty Trash UI


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok). Docs layer records rust-layer evidence only.

### File List

- `bronze-domain/src/purge.rs` — 30-day retention; no live-descendant purge; no forensic-erasure claim
- `bronze-storage/src/undo.rs` — trash/undo/purge; empty-trash requires confirmation

### Notes

- UI layer was a no-op. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
