# Story 4.5: Online backup restore

Status: done

## Story

As a developer,
I want SQLite online backup API and verified restore,
So that DAT-002 holds.

**Requirements:** DAT-002
**ADRs:** ADR-008
**Dependencies:** Story 4.2
**Blocking gates:** none beyond listed dependencies; Proposed ADR-002/009/018 remain unresolved

**Acceptance Criteria:**

**Given** schema exists
**When** backup/restore are implemented
**Then** schedule enum is daily|weekly only
**And** restore snapshots current DB first
**And** integrity failure does not swap

**Failure / recovery:**
File-copy of live WAL is forbidden.

**Security / privacy / diagnostics / a11y / i18n / data:**
Backups inherit 0600. No network.

**Automated verification:**
- `cargo test -p bronze-storage backup`

**Signed-build / manual evidence:**
fault tests

**Non-goals:**
backup UI


## Dev Agent Record

### Agent Model Used

Hedgehog authored loop / layer-eng (cursor-grok). Docs layer records rust-layer evidence only.

### File List

- `bronze-storage/src/backup.rs` — online `Backup` API; daily|weekly schedule; restore snapshots first; corrupt source does not swap

### Notes

- UI layer was a no-op. File-copy of live WAL is forbidden. Proposed ADR-002 / ADR-009 / ADR-018 remain unresolved.
