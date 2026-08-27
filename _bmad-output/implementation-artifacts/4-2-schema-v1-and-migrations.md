# Story 4.2: Schema v1 and migrations

Status: ready-for-dev

## Story

As a developer,
I want versioned checksummed migrations and WAL,
So that data survives upgrades.

**Requirements:** DAT-001, G-03
**ADRs:** ADR-008; ADR-009 Proposed so locator is an interface
**Dependencies:** Story 4.1
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** entities exist
**When** schema v1 is applied
**Then** tables match docs/08 including command_receipts and diagnostic_events
**And** backup-before-migration hook exists even if backup story is next
**And** WebView has no SQL

**Failure / recovery:**
Failed migration keeps original DB and enters read-only.

**Security / privacy / diagnostics / a11y / i18n / data:**
0600/0700 modes where possible. No iCloud default path.

**Automated verification:**
- `cargo test -p bronze-storage migrate`

**Signed-build / manual evidence:**
migration fixtures

**Non-goals:**
App Group container decision


## Dev Agent Record

### Agent Model Used

### File List
